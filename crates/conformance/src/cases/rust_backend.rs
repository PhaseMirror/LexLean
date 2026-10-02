//! Conformance cases for the Rust backend (SPEC.md §17.16).

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use lexlean::calculus::rust::ast::{Block, Callee, Crate, Expr, ItemDef, Let, Pat, Type};
use lexlean::calculus::rust::package::{self, Manifest};
use lexlean::calculus::rust::runtime::Item;
use lexlean::calculus::rust::{self, validate, Caller, Profile};
use lexlean::calculus::{self as target, realization, Outcome};
use serde_json::Value as Json;

use crate::calculus::cases;
use crate::rust_packages::{self, Committed};
use crate::support::{self, repo_root};

fn schema(name: &str) -> Json {
    serde_json::from_slice(
        &std::fs::read(repo_root().join("schemas").join(name).as_std_path()).expect("schema"),
    )
    .expect("schema JSON")
}

/// Apply `edit` to the first expression, depth first, it accepts.
fn edit_first(krate: &mut Crate, edit: &mut dyn FnMut(&mut Expr) -> bool) -> bool {
    fn block(block: &mut Block, edit: &mut dyn FnMut(&mut Expr) -> bool) -> bool {
        block
            .lets
            .iter_mut()
            .any(|binding| expr(&mut binding.value, edit))
            || expr(&mut block.tail, edit)
    }
    fn expr(expr_: &mut Expr, edit: &mut dyn FnMut(&mut Expr) -> bool) -> bool {
        if edit(expr_) {
            return true;
        }
        match expr_ {
            Expr::Box(inner) | Expr::Widen(inner) | Expr::Succeed(inner) | Expr::Not(inner) => {
                expr(inner, edit)
            }
            Expr::Call { args, .. } | Expr::Apply { args, .. } | Expr::Construct { args, .. } => {
                args.iter_mut().any(|arg| expr(arg, edit))
            }
            Expr::Pair(left, right) => expr(left, edit) || expr(right, edit),
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => expr(condition, edit) || block(then_branch, edit) || block(else_branch, edit),
            Expr::Match { scrutinee, arms } => {
                expr(scrutinee, edit) || arms.iter_mut().any(|(_, body)| block(body, edit))
            }
            Expr::Block(inner) => block(inner, edit),
            _ => false,
        }
    }
    krate.items.iter_mut().any(|item| match item {
        ItemDef::Function { body, .. } => {
            let mut whole = Expr::Block(Box::new(body.clone()));
            let edited = expr(&mut whole, edit);
            if let Expr::Block(edited_body) = whole {
                *body = *edited_body;
            }
            edited
        }
        _ => false,
    })
}

fn lowered(name: &str, profile: Profile) -> Crate {
    let case = cases()
        .into_iter()
        .find(|case| case.fixture.name == name)
        .unwrap_or_else(|| panic!("fixture {name}"));
    rust::lower(&case.fixture.program, profile).expect("lowers")
}

fn refused(krate: &Crate, message: &str) {
    let error = validate::validate(krate).expect_err("the planted crate is refused");
    assert!(error.contains(message), "{error}");
}

/// Every committed negative manifest whose name starts with `prefix` fails
/// with `LLB6005` and exactly its stated error.
fn negatives(prefix: &str) -> usize {
    let dir = repo_root().join("compiler/rust/negative");
    let mut count = 0;
    for (name, (bytes, error)) in rust_packages::negatives() {
        if !name.starts_with(prefix) {
            continue;
        }
        let committed = std::fs::read(dir.join(format!("{name}.json")).as_std_path())
            .expect("negative manifest");
        assert_eq!(
            committed, bytes,
            "{name}: the committed manifest is generated"
        );
        let stated = std::fs::read_to_string(dir.join(format!("{name}.error")).as_std_path())
            .expect("stated error");
        assert_eq!(stated.trim_end(), error, "{name}");
        let failure = target::package(&committed).expect_err("the manifest is refused");
        support::expect_code(&failure, "LLB6005");
        assert!(
            failure
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.message.contains(&error)),
            "{name}: {failure:?}"
        );
        count += 1;
    }
    assert!(count > 0, "a negative manifest exercises `{prefix}`");
    count
}

fn expected_outcome(fixture: &str) -> (Outcome, Vec<target::Value>, u64) {
    let case = cases()
        .into_iter()
        .find(|case| case.fixture.name == fixture)
        .unwrap_or_else(|| panic!("fixture {fixture}"));
    (
        case.fixture.expected,
        case.fixture.arguments,
        case.fixture.entry,
    )
}

fn cargo() -> String {
    std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned())
}

/// A package to build: its crate name, its files, and the harness that
/// calls its export.
struct Built {
    name: String,
    files: BTreeMap<String, Vec<u8>>,
    harness: String,
}

/// Write `packages` and one harness crate each into a workspace at `dir`.
fn workspace(dir: &Path, packages: &[Built]) {
    let mut members = Vec::new();
    for Built {
        name,
        files,
        harness,
    } in packages
    {
        let package = dir.join("p").join(name);
        for (path, bytes) in files {
            let file = package.join(path);
            std::fs::create_dir_all(file.parent().expect("parent")).expect("package directory");
            std::fs::write(file, bytes).expect("package file");
        }
        let runner = dir.join("h").join(format!("run_{name}"));
        std::fs::create_dir_all(runner.join("src")).expect("harness directory");
        std::fs::write(
            runner.join("Cargo.toml"),
            format!(
                "[package]\nname = \"run_{name}\"\nversion = \"0.0.0\"\nedition = \"2021\"\npublish = false\n\n[dependencies]\n{name} = {{ path = \"../../p/{name}\" }}\n"
            ),
        )
        .expect("harness manifest");
        std::fs::write(runner.join("src/main.rs"), harness).expect("harness source");
        members.push(format!("\"p/{name}\""));
        members.push(format!("\"h/run_{name}\""));
    }
    std::fs::write(
        dir.join("Cargo.toml"),
        format!(
            "[workspace]\nresolver = \"2\"\nmembers = [{}]\n",
            members.join(", ")
        ),
    )
    .expect("workspace manifest");
}

fn cargo_in(dir: &Path, arguments: &[&str]) -> std::process::Output {
    std::process::Command::new(cargo())
        .args(arguments)
        .current_dir(dir)
        .env("CARGO_TARGET_DIR", dir.join("target"))
        .env_remove("RUSTFLAGS")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .output()
        .expect("cargo runs")
}

fn harness_for(committed: &Committed) -> String {
    let (_, arguments, entry) = expected_outcome(&committed.fixture);
    let case = cases()
        .into_iter()
        .find(|case| case.fixture.name == committed.fixture)
        .expect("fixture");
    rust::render_caller(
        &case.fixture.program,
        committed.profile,
        entry,
        &arguments,
        &Caller {
            library: &committed.manifest.name,
            function: "run",
            passing: &committed.manifest.exports[0].parameters,
        },
    )
    .expect("harness")
}

fn committed_files(committed: &Committed) -> BTreeMap<String, Vec<u8>> {
    let dir = repo_root().join(format!(
        "compiler/rust/{}/{}",
        committed.profile.target(),
        committed.directory
    ));
    ["Cargo.toml", "src/lib.rs"]
        .into_iter()
        .map(|path| {
            (
                path.to_owned(),
                std::fs::read(dir.join(path).as_std_path()).expect("committed package file"),
            )
        })
        .collect()
}

#[allow(clippy::too_many_lines)]
pub fn run(id: &str) {
    match id {
        // §17.16: every emitted construct corresponds to an element of the
        // program it realizes.
        "RB-01" => {
            let mut emitted = BTreeSet::new();
            let mut lowered_count = 0;
            for case in cases() {
                for profile in Profile::ALL {
                    match rust::lower(&case.fixture.program, profile) {
                        Ok(krate) => {
                            emitted.extend(validate::constructs(&krate));
                            lowered_count += 1;
                        }
                        // Only rust-core refuses, and only for the heap.
                        Err(reason) => assert!(
                            profile == Profile::Core && reason.contains("requires heap allocation"),
                            "{} ({}): {reason}",
                            case.fixture.name,
                            profile.target()
                        ),
                    }
                }
            }
            for committed in rust_packages::packages() {
                let (krate, _, _) = package::lower(&committed.manifest).expect("package lowers");
                emitted.extend(validate::constructs(&krate));
            }
            assert!(lowered_count > 100, "{lowered_count} renderings");
            let table: BTreeSet<String> = validate::CORRESPONDENCE
                .iter()
                .map(|(name, _)| (*name).to_owned())
                .collect();
            let unexercised: Vec<&String> = table.difference(&emitted).collect();
            assert!(
                unexercised.is_empty(),
                "correspondence rows no rendering exercises: {unexercised:?}"
            );
            // A construct the program does not justify is refused.
            let case = cases()
                .into_iter()
                .find(|case| case.fixture.name == "sum-to")
                .expect("sum-to");
            let krate = rust::lower(&case.fixture.program, Profile::Std).expect("lowers");
            let mut elements = realization::program_elements(&case.fixture.program);
            elements.insert("function".to_owned());
            elements.insert("overflow".to_owned());
            validate::correspond(&krate, &elements).expect("the full element set justifies it");
            elements.remove("prim:nat_add");
            let error = validate::correspond(&krate, &elements).expect_err("unjustified");
            assert!(
                error.contains("the construct `call:runtime:nat_add` realizes none of"),
                "{error}"
            );
            // A construct whose element the program lacks is refused even
            // when every other construct is justified.
            elements.insert("prim:nat_add".to_owned());
            let mut extended = krate.clone();
            extended.items.push(ItemDef::Function {
                name: lexlean::calculus::rust::ast::Ident::Function(99),
                parameters: Vec::new(),
                result: Type::Ref(Box::new(Type::Nat)),
                body: Block::of(Expr::Lit(lexlean::calculus::rust::ast::Lit::Nat(0))),
            });
            let error = validate::correspond(&extended, &elements).expect_err("no export");
            assert!(error.contains("`type:ref`"), "{error}");
        }
        // §17.16: identifiers are hygienic and an exported name never
        // collides.
        "RB-02" => {
            assert!(negatives("identifier-") >= 7);
            let mut krate = lowered("sum-to", Profile::Std);
            let duplicated = edit_first(&mut krate, &mut |expr| {
                if let Expr::Block(block) = expr {
                    if let Some(binding) = block
                        .lets
                        .iter()
                        .find(|binding| matches!(binding.pat, Pat::Bind(_)))
                        .cloned()
                    {
                        block.lets.push(binding);
                        return true;
                    }
                }
                false
            });
            assert!(duplicated, "the plant site exists");
            refused(&krate, "hygiene: ");
        }
        // §17.16: every value is moved at most once and never read after.
        "RB-03" => {
            assert!(negatives("ownership-") >= 1);
            for (fixture, message) in [
                ("adt-evaluation", "is moved twice"),
                ("sum-to", "is read after it was moved"),
            ] {
                let mut krate = lowered(fixture, Profile::Std);
                let planted = edit_first(&mut krate, &mut |expr| {
                    if let Expr::Block(block) = expr {
                        if let Some(Pat::Bind(holder)) = block
                            .lets
                            .first()
                            .map(|binding| binding.pat.clone())
                            .filter(|pattern| {
                                matches!(
                                    pattern,
                                    Pat::Bind(lexlean::calculus::rust::ast::Ident::Holder(_))
                                )
                            })
                        {
                            block.lets.push(Let {
                                pat: Pat::Wild,
                                ty: None,
                                value: Expr::Move(holder),
                            });
                            return true;
                        }
                    }
                    false
                });
                assert!(planted, "{fixture}: the plant site exists");
                refused(&krate, message);
            }
        }
        // §17.16: no unsupported type crosses a package boundary and
        // rust-core never allocates.
        "RB-04" => {
            assert!(negatives("unsupported-") >= 1);
            assert!(negatives("hidden-allocation-") >= 2);
            let mut krate = lowered("sum-to", Profile::Core);
            krate.items.push(ItemDef::Enum {
                name: Type::Adt(0),
                variants: vec![(0, vec![Type::Str])],
            });
            refused(&krate, "hidden allocation: rust-core renders the heap type");
            let mut krate = lowered("nat-overflow-add", Profile::Core);
            let planted = edit_first(&mut krate, &mut |expr| {
                if let Expr::Call {
                    callee: Callee::Runtime(item),
                    ..
                } = expr
                {
                    *item = Item::LengthBytes;
                    return true;
                }
                false
            });
            assert!(planted, "the plant site exists");
            refused(
                &krate,
                "hidden allocation: rust-core renders the runtime function `length_bytes`",
            );
        }
        // §17.16: failure is typed exactly: every fallible call propagates,
        // nothing else does, and an export states its function's failure.
        "RB-05" => {
            assert!(negatives("arithmetic-") >= 2);
            for case in cases() {
                let fallible = rust::fallible_functions(&case.fixture.program).expect("valid");
                if matches!(case.fixture.expected, Outcome::Overflow { .. }) {
                    assert!(
                        fallible[usize::try_from(case.fixture.entry).expect("entry")],
                        "{}: an entry that overflows is typed fallible",
                        case.fixture.name
                    );
                }
            }
            let mut krate = lowered("sum-to", Profile::Std);
            let planted = edit_first(&mut krate, &mut |expr| {
                if let Expr::Call {
                    propagate: propagate @ true,
                    ..
                } = expr
                {
                    *propagate = false;
                    return true;
                }
                false
            });
            assert!(planted, "the plant site exists");
            refused(&krate, "arithmetic: the fallible call of");
            // A fallible function's tail must be a fallible value.
            let mut krate = lowered("sum-to", Profile::Std);
            let planted = edit_first(&mut krate, &mut |expr| {
                if let Expr::Call {
                    callee: Callee::Runtime(Item::NatAdd),
                    propagate: propagate @ false,
                    ..
                } = expr
                {
                    *propagate = true;
                    return true;
                }
                false
            });
            assert!(planted, "the plant site exists");
            refused(&krate, "is not a fallible value");
            let mut krate = lowered("booleans", Profile::Std);
            let planted = edit_first(&mut krate, &mut |expr| {
                if let Expr::Call { propagate, .. } = expr {
                    *propagate = true;
                    return true;
                }
                false
            });
            assert!(planted, "the plant site exists");
            refused(&krate, "arithmetic: the infallible call of");
            let mut krate = lowered("sum-to", Profile::Std);
            for item in &mut krate.items {
                if let ItemDef::Function { result, .. } = item {
                    if let Type::Fallible(inner) = result.clone() {
                        *result = *inner;
                    }
                }
            }
            refused(&krate, "arithmetic: ");
        }
        // §17.16: every package builds under its declared gates and its
        // export prints the denotation's observable outcome.
        "RB-06" => {
            let dir = tempfile::Builder::new()
                .prefix("lexlean-rust-packages-")
                .tempdir()
                .expect("tempdir");
            let mut built = Vec::new();
            let mut expected = BTreeMap::new();
            for committed in rust_packages::packages() {
                let (outcome, _, _) = expected_outcome(&committed.fixture);
                let Some(observed) = rust::observable(&outcome) else {
                    continue;
                };
                if !matches!(&outcome, Outcome::Value { value, .. } if !realization::value_elements(value).contains("value:closure"))
                    && !matches!(outcome, Outcome::Overflow { .. })
                {
                    continue;
                }
                built.push(Built {
                    name: committed.manifest.name.clone(),
                    files: committed_files(&committed),
                    harness: harness_for(&committed),
                });
                expected.insert(committed.manifest.name.clone(), observed);
            }
            assert!(built.len() > 100, "{} packages run", built.len());
            workspace(dir.path(), &built);
            let build = cargo_in(
                dir.path(),
                &["build", "--offline", "--workspace", "--quiet"],
            );
            assert!(
                build.status.success(),
                "a package does not build:\n{}",
                String::from_utf8_lossy(&build.stderr)
            );
            let lint = cargo_in(
                dir.path(),
                &[
                    "clippy",
                    "--offline",
                    "--workspace",
                    "--quiet",
                    "--exclude",
                    "run_*",
                ],
            );
            assert!(
                lint.status.success(),
                "a package fails its lint gate:\n{}",
                String::from_utf8_lossy(&lint.stderr)
            );
            for (name, observed) in &expected {
                let binary = dir
                    .path()
                    .join("target/debug")
                    .join(format!("run_{name}{}", std::env::consts::EXE_SUFFIX));
                let ran = std::process::Command::new(&binary)
                    .output()
                    .expect("the harness runs");
                assert!(
                    ran.status.success(),
                    "{name}: {}",
                    String::from_utf8_lossy(&ran.stderr)
                );
                let stdout = String::from_utf8(ran.stdout).expect("utf8");
                let printed: Json =
                    serde_json::from_str(stdout.lines().next().expect("outcome line"))
                        .unwrap_or_else(|error| panic!("{name}: {error}: {stdout}"));
                assert_eq!(
                    &printed, observed,
                    "{name}: the export and the denotation disagree"
                );
            }
            // The lint gate and the differential are real: a clone of a
            // `Copy` value fails Clippy, and a wrapping subtraction changes
            // the printed value.
            let planted = tempfile::Builder::new()
                .prefix("lexlean-rust-planted-")
                .tempdir()
                .expect("tempdir");
            let committed = rust_packages::packages()
                .into_iter()
                .find(|committed| {
                    committed.fixture == "nat-arithmetic" && committed.profile == Profile::Std
                })
                .expect("nat-arithmetic package");
            let mut lint_files = committed_files(&committed);
            lint_files
                .get_mut("src/lib.rs")
                .expect("library")
                .extend_from_slice(b"\npub fn planted(a: u64) -> u64 {\n    a.clone()\n}\n");
            let mut wrong_files = committed_files(&committed);
            let library = String::from_utf8(wrong_files["src/lib.rs"].clone()).expect("utf8");
            let from = "pub fn nat_sub(a: u64, b: u64) -> u64 { a.saturating_sub(b) }";
            assert!(library.contains(from), "the plant site exists");
            wrong_files.insert(
                "src/lib.rs".to_owned(),
                library
                    .replacen(
                        from,
                        "pub fn nat_sub(a: u64, b: u64) -> u64 { a.wrapping_sub(b) }",
                        1,
                    )
                    .into_bytes(),
            );
            let harness = harness_for(&committed);
            let mut lint_package = committed.manifest.name.clone();
            lint_package.push_str("_lint");
            let lint_harness = harness.replace(
                &format!("use {}::*;", committed.manifest.name),
                &format!("use {lint_package}::*;"),
            );
            let lint_cargo = String::from_utf8(lint_files["Cargo.toml"].clone())
                .expect("utf8")
                .replace(
                    &format!("name = \"{}\"", committed.manifest.name),
                    &format!("name = \"{lint_package}\""),
                );
            lint_files.insert("Cargo.toml".to_owned(), lint_cargo.into_bytes());
            workspace(
                planted.path(),
                &[
                    Built {
                        name: committed.manifest.name.clone(),
                        files: wrong_files,
                        harness,
                    },
                    Built {
                        name: lint_package.clone(),
                        files: lint_files,
                        harness: lint_harness,
                    },
                ],
            );
            let lint = cargo_in(
                planted.path(),
                &["clippy", "--offline", "--quiet", "-p", &lint_package],
            );
            assert!(!lint.status.success(), "the planted lint is refused");
            assert!(
                String::from_utf8_lossy(&lint.stderr).contains("clippy::clone_on_copy"),
                "{}",
                String::from_utf8_lossy(&lint.stderr)
            );
            let build = cargo_in(
                planted.path(),
                &[
                    "build",
                    "--offline",
                    "--quiet",
                    "-p",
                    &format!("run_{}", committed.manifest.name),
                ],
            );
            assert!(
                build.status.success(),
                "{}",
                String::from_utf8_lossy(&build.stderr)
            );
            let ran =
                std::process::Command::new(planted.path().join("target/debug").join(format!(
                    "run_{}{}",
                    committed.manifest.name,
                    std::env::consts::EXE_SUFFIX
                )))
                .output()
                .expect("the planted harness runs");
            let printed: Json = serde_json::from_str(
                String::from_utf8(ran.stdout)
                    .expect("utf8")
                    .lines()
                    .next()
                    .expect("outcome line"),
            )
            .expect("JSON");
            assert_ne!(
                &printed, &expected[&committed.manifest.name],
                "the planted subtraction is detected"
            );
        }
        // §17.16: packages are deterministic, schema-valid, and bound to
        // their program, runtime, and LexLean's compiler semantics.
        "RB-07" => {
            let manifest_schema = schema("rust-package.schema.json");
            let provenance_schema = schema("rust-provenance.schema.json");
            let program_schema = schema("target-program.schema.json");
            let semantics = lexlean::compiler_semantics_id_for(lexlean::LANGUAGE_1_2).to_hex();
            let roots = [
                tempfile::Builder::new()
                    .prefix("lexlean-rust-a-")
                    .tempdir()
                    .expect("tempdir"),
                tempfile::Builder::new()
                    .prefix("lexlean-rust-b-")
                    .tempdir()
                    .expect("tempdir"),
            ];
            let mut count = 0;
            for committed in rust_packages::packages() {
                let directory = repo_root().join(format!(
                    "compiler/rust/{}/{}",
                    committed.profile.target(),
                    committed.directory
                ));
                let manifest_bytes =
                    std::fs::read(directory.join("package.json").as_std_path()).expect("manifest");
                let manifest_json: Json = serde_json::from_slice(&manifest_bytes).expect("JSON");
                let violations = crate::schema::validate(&manifest_schema, &manifest_json);
                assert!(violations.is_empty(), "{directory}: {violations:?}");
                let violations =
                    crate::schema::validate(&program_schema, &manifest_json["program"]);
                assert!(violations.is_empty(), "{directory}: {violations:?}");
                let manifest = Manifest::parse(&manifest_bytes).expect("manifest parses");
                // Two renderings, written under two roots, are byte-identical
                // to each other and to the committed package.
                let first = target::package(&manifest_bytes).expect("packages");
                let second = target::package(&manifest_bytes).expect("packages");
                assert_eq!(first, second, "{directory}");
                for root in &roots {
                    for (path, bytes) in &first.files {
                        let file = root
                            .path()
                            .join(committed.directory.clone())
                            .join(committed.profile.target())
                            .join(path);
                        std::fs::create_dir_all(file.parent().expect("parent")).expect("directory");
                        std::fs::write(&file, bytes).expect("file");
                    }
                }
                for (path, bytes) in &first.files {
                    let committed_bytes =
                        std::fs::read(directory.join(path).as_std_path()).expect("committed file");
                    assert_eq!(&committed_bytes, bytes, "{directory}/{path}");
                    let read = |root: &tempfile::TempDir| {
                        std::fs::read(
                            root.path()
                                .join(committed.directory.clone())
                                .join(committed.profile.target())
                                .join(path),
                        )
                        .expect("written file")
                    };
                    assert_eq!(read(&roots[0]), read(&roots[1]), "{directory}/{path}");
                }
                let provenance: Json =
                    serde_json::from_slice(&first.files["provenance.json"]).expect("provenance");
                let violations = crate::schema::validate(&provenance_schema, &provenance);
                assert!(violations.is_empty(), "{directory}: {violations:?}");
                for path in ["Cargo.toml", "src/lib.rs"] {
                    assert_eq!(
                        provenance["files"][path],
                        lexlean::artifact::content_id::Sha256Digest::of(&first.files[path])
                            .to_hex(),
                        "{directory}/{path}"
                    );
                }
                assert_eq!(
                    provenance["program"],
                    manifest.program.id().expect("valid").to_hex(),
                    "{directory}"
                );
                assert_eq!(provenance["compiler_semantics"], semantics.as_str());
                let runtime = match committed.profile {
                    Profile::Core => rust::runtime::CORE.to_owned(),
                    Profile::Std => format!("{}{}", rust::runtime::CORE, rust::runtime::STD),
                };
                assert_eq!(
                    provenance["runtime"],
                    lexlean::artifact::content_id::Sha256Digest::of(runtime.as_bytes()).to_hex()
                );
                count += 1;
            }
            assert!(count > 100, "{count} packages");
            // A provenance or package file that drifts from its generator is
            // refused by the generated-file gate.
            crate::calculus::check(repo_root().as_std_path(), false)
                .expect("the committed packages equal their generator");
            let mut sourced: Manifest = rust_packages::packages()[0].manifest.clone();
            sourced.sources = vec!["0".repeat(64), "f".repeat(64)];
            let package = package::package(&sourced).expect("sources are admitted");
            let provenance: Json =
                serde_json::from_slice(&package.files["provenance.json"]).expect("provenance");
            assert_eq!(provenance["sources"], serde_json::json!(sourced.sources));
        }
        other => panic!("no Rust backend case is wired for {other}"),
    }
}
