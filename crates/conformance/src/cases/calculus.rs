//! The `calculus` suite: TC-01..TC-07, the production realization calculus
//! (SPEC.md §17.14).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use lexlean::calculus::library::Template;
use lexlean::calculus::{
    self as target, check, interp, realization, rust, Expr, Fixture, Outcome, Program, Value,
};
use serde_json::{json, Value as Json};

use crate::calculus::{self as fixtures, Case};
use crate::support::{self, repo_root, P};

/// The committed fixtures, read from disk exactly as published.
fn committed() -> Vec<(PathBuf, Vec<u8>, Fixture)> {
    let dir = repo_root().join("compiler/fixtures");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir.as_std_path()).expect("compiler/fixtures exists") {
        let path = entry.expect("entry").path();
        let bytes = std::fs::read(&path).expect("fixture bytes");
        let fixture: Fixture = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        out.push((path, bytes, fixture));
    }
    out.sort_by(|left, right| left.0.cmp(&right.0));
    assert!(
        !out.is_empty(),
        "the calculus has hand-constructed fixtures"
    );
    out
}

fn schema(name: &str) -> Json {
    serde_json::from_slice(
        &std::fs::read(repo_root().join("schemas").join(name).as_std_path()).expect("schema"),
    )
    .expect("schema parses")
}

/// Rename every local of every function by an injective map that differs
/// from first-binding order, keeping the program alpha-equivalent.
fn alpha_variant(program: &Program) -> Program {
    fn rename(expr: &mut Expr) {
        let fresh = |name: &mut u64| *name = *name * 7 + 1000;
        match expr {
            Expr::Value { .. } => {}
            Expr::Var { name } => fresh(name),
            Expr::Let {
                name, bound, body, ..
            } => {
                fresh(name);
                rename(bound);
                rename(body);
            }
            Expr::Cond {
                condition,
                then_branch,
                else_branch,
            } => {
                rename(condition);
                rename(then_branch);
                rename(else_branch);
            }
            Expr::Match {
                scrutinee, arms, ..
            } => {
                rename(scrutinee);
                for arm in arms {
                    arm.binders.iter_mut().for_each(fresh);
                    rename(&mut arm.body);
                }
            }
            Expr::Build { operands, .. }
            | Expr::Call { operands, .. }
            | Expr::Prim { operands, .. }
            | Expr::Closure {
                captures: operands, ..
            } => {
                operands.iter_mut().for_each(rename);
            }
            Expr::Apply { target, operands } => {
                rename(target);
                operands.iter_mut().for_each(rename);
            }
            Expr::First { value } | Expr::Second { value } | Expr::Field { value, .. } => {
                rename(value)
            }
        }
    }
    let mut out = program.clone();
    for function in &mut out.functions {
        function
            .parameters
            .iter_mut()
            .for_each(|name| *name = *name * 7 + 1000);
        rename(&mut function.body);
    }
    out
}

/// A load expected to fail closed with `LLB6005` naming `message`.
fn rejects(bytes: &[u8], message: &str) {
    let error = target::load(bytes)
        .err()
        .unwrap_or_else(|| panic!("an invalid program loads: expected `{message}`"));
    support::expect_code(&error, "LLB6005");
    assert_eq!(error.class.exit_code(), 1, "{error}");
    assert!(
        error.to_string().contains(message),
        "expected `{message}` in: {error}"
    );
}

fn program_json(name: &str) -> Json {
    let fixture = fixtures::cases()
        .into_iter()
        .find(|case| case.fixture.name == name)
        .expect("fixture")
        .fixture;
    serde_json::to_value(&fixture.program).expect("program serializes")
}

/// The Lean evaluator's outcome for every fixture, read from the verified
/// compiler's oleans: `#eval` runs the generated definitions as compiled by
/// pinned Lean.
fn lean_evaluations(verified: &camino::Utf8Path, cases: &[Case]) -> BTreeMap<String, Json> {
    let toolchain = support::real_elan_home()
        .join("toolchains")
        .join(support::mangled_toolchain_name());
    let lean = toolchain.join("bin").join("lean");
    let scratch = tempfile::Builder::new()
        .prefix("lexlean-calculus-eval-")
        .tempdir()
        .expect("tempdir");
    let mut source = String::from(EVALUATOR);
    for case in cases {
        source.push_str(&format!(
            "#eval IO.println (\"FIXTURE {} \" ++ showOutcome Compiler.TargetFixtures.{}Run)\n",
            case.fixture.name,
            fixtures::identifier(&case.fixture.name)
        ));
    }
    let file = scratch.path().join("Evaluate.lean");
    std::fs::write(&file, source).expect("evaluator source");
    let library = toolchain.join("lib").join("lean");
    // The verified tree publishes oleans without compiled IR, which `#eval`
    // needs, so the published generated sources are compiled again here by
    // the same pinned Lean, in import order.
    let compiled = scratch.path().join("out");
    std::fs::create_dir_all(compiled.join("Compiler")).expect("output directory");
    let search = std::env::join_paths([compiled.clone(), library]).expect("LEAN_PATH");
    for module in [
        "TargetSyntax",
        "TargetSemantics",
        "TargetOracle",
        "TargetFixtures",
    ] {
        let built = std::process::Command::new(&lean)
            .arg("-o")
            .arg(compiled.join("Compiler").join(format!("{module}.olean")))
            .arg(format!("Compiler/{module}.lean"))
            // Lean names a module by its path relative to the working
            // directory.
            .current_dir(verified.join("modules").as_std_path())
            .env("LEAN_PATH", &search)
            .output()
            .expect("pinned lean runs");
        assert!(
            built.status.success(),
            "{module} does not compile:\n{}\n{}",
            String::from_utf8_lossy(&built.stdout),
            String::from_utf8_lossy(&built.stderr)
        );
    }
    let output = std::process::Command::new(&lean)
        .arg(&file)
        .env("LEAN_PATH", search)
        .output()
        .expect("pinned lean runs");
    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        output.status.success(),
        "the evaluator fails:\n{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let mut out = BTreeMap::new();
    for line in stdout.lines() {
        let rest = line
            .strip_prefix("FIXTURE ")
            .unwrap_or_else(|| panic!("unexpected evaluator output: {line}"));
        let (name, json) = rest.split_once(' ').expect("name and outcome");
        out.insert(
            name.to_owned(),
            serde_json::from_str(json).unwrap_or_else(|error| panic!("{name}: {error}: {json}")),
        );
    }
    out
}

/// Prints a denotation outcome in the fixtures' exact JSON form.
const EVALUATOR: &str = r#"import Compiler.TargetFixtures
open Compiler.TargetSyntax Compiler.TargetSemantics

def hexDigit (n : Nat) : Char := if n < 10 then Char.ofNat (48 + n) else Char.ofNat (87 + n)
def byteHex (b : UInt8) : String := String.ofList [hexDigit (b.toNat / 16), hexDigit (b.toNat % 16)]
def quote (s : String) : String :=
  "\"" ++ String.join (s.toList.map fun c =>
    if c = '"' then "\\\"" else if c = '\\' then "\\\\"
    else if c.toNat < 32 then "\\u00" ++ byteHex c.toNat.toUInt8 else c.toString) ++ "\""
def tagged (kind : String) (rest : String) : String := "{\"kind\":\"" ++ kind ++ "\"" ++ rest ++ "}"
def number (kind : String) (text : String) : String := tagged kind (",\"value\":\"" ++ text ++ "\"")
partial def showValue : Value → String
  | .unit => tagged "unit" ""
  | .bool b => tagged "bool" (",\"value\":" ++ toString b)
  | .nat n => number "nat" (toString n)
  | .int n => number "int" (toString n)
  | .u8 n => number "u8" (toString n)
  | .u16 n => number "u16" (toString n)
  | .u32 n => number "u32" (toString n)
  | .u64 n => number "u64" (toString n)
  | .i8 n => number "i8" (toString n)
  | .i16 n => number "i16" (toString n)
  | .i32 n => number "i32" (toString n)
  | .i64 n => number "i64" (toString n)
  | .string s => tagged "string" (",\"value\":" ++ quote s)
  | .bytes b => tagged "bytes" (",\"hex\":\"" ++ String.join (b.toList.map byteHex) ++ "\"")
  | .ordering o => tagged "ordering" (",\"value\":\"" ++ (match o with | .less => "lt" | .same => "eq" | .more => "gt") ++ "\"")
  | .none => tagged "none" ""
  | .some v => tagged "some" (",\"value\":" ++ showValue v)
  | .ok v => tagged "ok" (",\"value\":" ++ showValue v)
  | .error v => tagged "error" (",\"value\":" ++ showValue v)
  | .list items => tagged "list" (",\"items\":[" ++ ",".intercalate (items.map showValue) ++ "]")
  | .pair l r => tagged "pair" (",\"left\":" ++ showValue l ++ ",\"right\":" ++ showValue r)
  | .adt c fields => tagged "adt" (",\"constructor\":" ++ toString c ++ ",\"fields\":[" ++ ",".intercalate (fields.map showValue) ++ "]")
  | .closure f captures => tagged "closure" (",\"function\":" ++ toString f ++ ",\"captures\":[" ++ ",".intercalate (captures.map showValue) ++ "]")
def showOutcome : Outcome → String
  | .value v steps => tagged "value" (",\"value\":" ++ showValue v ++ ",\"steps\":" ++ toString steps)
  | .overflow steps => tagged "overflow" (",\"steps\":" ++ toString steps)
  | .stuck => tagged "stuck" ""
  | .exhausted => tagged "exhausted" ""
"#;

/// The planted verification: a copy of the compiler whose fixture module
/// states one wrong expected outcome and runs one mutated library instance
/// against LexLean's own primitive. Pinned Lean must reject both.
fn planted_verification() -> &'static Vec<String> {
    static ERROR: OnceLock<Vec<String>> = OnceLock::new();
    ERROR.get_or_init(|| {
        let named = |name: &str| {
            fixtures::cases()
                .into_iter()
                .find(|case| case.fixture.name == name)
                .expect("fixture")
        };
        // The interpreter and the denotation disagree: 56 is not the sum.
        let mut wrong_sum = named("sum-to");
        let Outcome::Value { steps, .. } = wrong_sum.fixture.expected.clone() else {
            panic!("sum-to is a value")
        };
        wrong_sum.fixture.expected = Outcome::Value {
            value: Value::Nat {
                value: "56".to_owned(),
            },
            steps,
        };
        // The realization inserts a smaller key after the element it
        // precedes; the interpreter follows the mutation, LexLean does not.
        let mut insert = named("set-insert-nat");
        let entry =
            usize::try_from(insert.library.as_ref().expect("a library fixture").at).expect("index");
        insert.fixture.program = mutate_insert(&insert.fixture.program, entry);
        insert.fixture.expected = interp::run(
            &insert.fixture.program,
            insert.fixture.fuel,
            insert.fixture.entry,
            &insert.fixture.arguments,
        );
        let project = P::compiler();
        project.write(
            "src/TargetFixtures.lex.tex",
            &fixtures::fixtures_module(&[wrong_sum, insert]),
        );
        let _guard = support::env_lock();
        let error = project.verify_fails_with("LLV7002");
        error
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect()
    })
}

/// Insert a smaller key one position too late: `lt => head :: key :: tail`
/// instead of `key :: head :: tail`.
fn mutate_insert(program: &Program, entry: usize) -> Program {
    let mut out = program.clone();
    let Expr::Match { arms, .. } = &mut out.functions[entry].body else {
        panic!("set_insert is a match")
    };
    let Expr::Match { arms: orders, .. } = &mut arms[1].body else {
        panic!("the cons arm compares")
    };
    let Expr::Build { operands, .. } = &mut orders[0].body else {
        panic!("the lt arm builds")
    };
    let ty = target::Ty::List {
        element: Box::new(target::Ty::Nat),
    };
    *operands = vec![
        Expr::Var { name: 2 },
        Expr::Build {
            shape: target::Shape::Cons,
            ty,
            operands: vec![Expr::Var { name: 1 }, Expr::Var { name: 3 }],
        },
    ];
    check::check(&out).expect("the mutated realization is still well typed");
    out
}

fn rustc() -> String {
    std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned())
}

/// Compile a rendered harness with rustc and return what it prints.
fn run_rust(source: &str, dir: &Path, name: &str) -> String {
    let file = dir.join(format!("{name}.rs"));
    let binary = dir.join(format!("{name}{}", std::env::consts::EXE_SUFFIX));
    std::fs::write(&file, source).expect("rendered source");
    let compiled = std::process::Command::new(rustc())
        .args([
            "--edition",
            "2021",
            "-D",
            "warnings",
            "-C",
            "opt-level=0",
            "--crate-name",
            "fixture",
            "-o",
        ])
        .arg(&binary)
        .arg(&file)
        .output()
        .expect("rustc runs");
    assert!(
        compiled.status.success(),
        "{name}: rustc rejects the rendering:\n{}",
        String::from_utf8_lossy(&compiled.stderr)
    );
    let ran = std::process::Command::new(&binary)
        .output()
        .expect("the rendered program runs");
    assert!(
        ran.status.success(),
        "{name}: the rendered program fails:\n{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    String::from_utf8(ran.stdout)
        .expect("utf8")
        .trim_end()
        .to_owned()
}

/// Whether a value holds a closure, which Rust does not observe.
fn observable(value: &Value) -> bool {
    !realization::value_elements(value).contains("value:closure")
}

pub fn run(id: &str) {
    match id {
        // §17.14: closed form, canonical bytes, content identity, alpha.
        "TC-01" => {
            let program_schema = schema("target-program.schema.json");
            let fixture_schema = schema("target-fixture.schema.json");
            let all = committed();
            let mut renamed = 0;
            for (path, bytes, fixture) in &all {
                let (bytes, fixture) = (bytes.clone(), fixture.clone());
                let name = path
                    .file_stem()
                    .expect("stem")
                    .to_string_lossy()
                    .into_owned();
                assert_eq!(fixture.name, name, "a fixture is named by its file");
                assert_eq!(fixture.spec, target::FIXTURE_SPEC);
                assert_eq!(
                    bytes,
                    fixture.to_file_bytes(),
                    "{name}: committed bytes are canonical"
                );
                let json: Json = serde_json::from_slice(&bytes).expect("json");
                let violations = crate::schema::validate(&fixture_schema, &json);
                assert!(violations.is_empty(), "{name}: {violations:?}");
                let violations = crate::schema::validate(&program_schema, &json["program"]);
                assert!(violations.is_empty(), "{name}: {violations:?}");
                let canonical = fixture.program.canonical().expect("bound");
                assert_eq!(
                    canonical, fixture.program,
                    "{name}: the committed program is in first-binding order"
                );
                let loaded = target::load(&fixture.program.to_file_bytes())
                    .expect("a fixture program loads");
                assert_eq!(loaded.to_file_bytes(), fixture.program.to_file_bytes());
                // Alpha-equivalent spellings canonicalize to the same bytes
                // and identity, and loading canonicalizes them.
                let variant = alpha_variant(&fixture.program);
                renamed += usize::from(variant != fixture.program);
                assert_eq!(
                    variant.canonical().expect("bound").to_file_bytes(),
                    fixture.program.to_file_bytes(),
                    "{name}"
                );
                assert_eq!(
                    variant.id().expect("bound"),
                    fixture.program.id().expect("bound"),
                    "{name}"
                );
                assert_eq!(
                    target::load(&variant.to_file_bytes())
                        .expect("loads")
                        .to_file_bytes(),
                    fixture.program.to_file_bytes()
                );
                // The denotation's reference transcription is deterministic.
                let first = interp::run(
                    &fixture.program,
                    fixture.fuel,
                    fixture.entry,
                    &fixture.arguments,
                );
                let second = interp::run(&variant, fixture.fuel, fixture.entry, &fixture.arguments);
                assert_eq!(
                    first, fixture.expected,
                    "{name}: the expected outcome is the interpreter's"
                );
                assert_eq!(
                    second, first,
                    "{name}: an alpha variant has the same outcome"
                );
            }
            assert!(
                renamed * 2 > all.len(),
                "most fixtures bind locals, so renaming is exercised ({renamed})"
            );
            // A change that is not a renaming changes the identity.
            let program: Program =
                serde_json::from_value(program_json("closures")).expect("program");
            let mut swapped = program.clone();
            swapped.functions[2].body = Expr::Prim {
                operation: target::Prim::NatAdd,
                operands: vec![Expr::Var { name: 0 }, Expr::Var { name: 1 }],
            };
            assert_ne!(swapped.id().expect("bound"), program.id().expect("bound"));
            // Identity is the SHA-256 of the canonical bytes.
            assert_eq!(
                program.id().expect("bound"),
                lexlean::artifact::content_id::Sha256Digest::of(&program.to_file_bytes())
            );
        }
        // §17.14: every invalid target construct fails closed.
        "TC-02" => {
            let base = program_json("adt-evaluation");
            let bytes = |json: &Json| serde_json::to_vec(json).expect("json");
            let mutate = |edit: &dyn Fn(&mut Json)| {
                let mut json = base.clone();
                edit(&mut json);
                bytes(&json)
            };
            rejects(b"{", "target program is malformed");
            rejects(
                &mutate(&|json| json["extra"] = json!(1)),
                "unknown field `extra`",
            );
            rejects(
                &mutate(&|json| json["spec"] = json!("lexlean/target-program/9")),
                "has spec `lexlean/target-program/9`",
            );
            rejects(
                &mutate(&|json| json["functions"][0]["body"] = json!({"kind": "var", "name": 99})),
                "function 0: local 99 is unbound",
            );
            rejects(
                &mutate(
                    &|json| json["functions"][1]["body"] = json!({"kind": "value", "type": {"kind": "bool"}, "value": {"kind": "bool", "value": true}}),
                ),
                "function 1: the body has type Bool, expected Nat",
            );
            rejects(
                &mutate(&|json| {
                    json["functions"][0]["body"]["arms"]
                        .as_array_mut()
                        .expect("arms")
                        .pop();
                }),
                "the match is not exhaustive: Adt { constructor: 2 } is not covered",
            );
            rejects(
                &mutate(&|json| {
                    let arm = json["functions"][0]["body"]["arms"][0].clone();
                    json["functions"][0]["body"]["arms"]
                        .as_array_mut()
                        .expect("arms")
                        .push(arm);
                }),
                "the arm for Adt { constructor: 0 } is unreachable",
            );
            rejects(
                &mutate(&|json| {
                    json["functions"][1]["body"]["operands"]
                        .as_array_mut()
                        .expect("operands")
                        .push(json!({"kind": "var", "name": 0}));
                }),
                "function 0 takes 1 operand(s), received 2",
            );
            rejects(
                &mutate(&|json| json["functions"][1]["body"]["function"] = json!(9)),
                "function 9 is not declared",
            );
            rejects(
                &mutate(&|json| {
                    json["functions"][0]["types"][0] = json!({"kind": "adt", "index": 9})
                }),
                "ADT 9 is not declared",
            );
            rejects(
                &mutate(&|json| {
                    json["adts"]
                        .as_array_mut()
                        .expect("adts")
                        .push(json!({"constructors": []}))
                }),
                "ADT 2 has no constructor",
            );
            rejects(
                &mutate(&|json| {
                    json["functions"][0]["parameters"] = json!([0, 0]);
                    json["functions"][0]["types"]
                        .as_array_mut()
                        .expect("types")
                        .push(json!({"kind": "nat"}));
                }),
                "a parameter name repeats",
            );
            rejects(
                &mutate(&|json| {
                    json["functions"][0]["body"]["arms"][2]["body"]["operands"][0]["operands"][0] =
                        json!({"kind": "field", "value": {"kind": "var", "name": 5}, "index": 0})
                }),
                "`field` requires an ADT with exactly one constructor; ADT 0 has 3",
            );
            // Literals are in range and spelled canonically.
            let literal = |ty: Json, value: Json| {
                bytes(
                    &json!({"spec": target::PROGRAM_SPEC, "adts": [], "functions": [
                        {"parameters": [], "types": [], "result": ty, "body": {"kind": "value", "type": ty, "value": value}}
                    ]}),
                )
            };
            rejects(
                &literal(
                    json!({"kind": "fixed", "width": "u8"}),
                    json!({"kind": "u8", "value": "256"}),
                ),
                "256 is outside u8",
            );
            rejects(
                &literal(
                    json!({"kind": "nat"}),
                    json!({"kind": "nat", "value": "007"}),
                ),
                "`007` is not a canonical decimal",
            );
            rejects(
                &literal(
                    json!({"kind": "int"}),
                    json!({"kind": "int", "value": "-0"}),
                ),
                "`-0` is not a canonical decimal",
            );
            rejects(
                &literal(
                    json!({"kind": "nat"}),
                    json!({"kind": "nat", "value": "18446744073709551616"}),
                ),
                "outside the 64-bit realization",
            );
            rejects(
                &literal(
                    json!({"kind": "bytes"}),
                    json!({"kind": "bytes", "hex": "abc"}),
                ),
                "byte literal `abc` is not even-length lowercase hexadecimal",
            );
            rejects(
                &literal(json!({"kind": "nat"}), json!({"kind": "int", "value": "1"})),
                "does not have type Nat",
            );
            rejects(
                &literal(
                    json!({"kind": "fn", "parameters": [], "result": {"kind": "nat"}}),
                    json!({"kind": "unit"}),
                ),
                "a function type takes at least one parameter",
            );
            rejects(
                &literal(
                    json!({"kind": "nat"}),
                    json!({"kind": "closure", "function": 0, "captures": []}),
                ),
                "a closure has no literal form",
            );
            // Closures leave a parameter; key order exists only on keys;
            // primitives apply only at their closed signatures.
            let closures = program_json("closures");
            let mut capture_all = closures.clone();
            capture_all["functions"][0]["body"]["operands"][0]["captures"]
                .as_array_mut()
                .expect("captures")
                .push(json!({"kind": "var", "name": 0}));
            rejects(
                &bytes(&capture_all),
                "closure of function 2 captures 2 of its 2 parameters; at least one must remain",
            );
            let unary = |operation: Json, operand: Json, operand_ty: Json, result: Json| {
                bytes(
                    &json!({"spec": target::PROGRAM_SPEC, "adts": [], "functions": [
                        {"parameters": [0], "types": [operand_ty], "result": result,
                         "body": {"kind": "prim", "operation": operation, "operands": [operand, {"kind": "var", "name": 0}]}}
                    ]}),
                )
            };
            let nats = json!({"kind": "list", "element": {"kind": "nat"}});
            rejects(
                &unary(
                    json!({"kind": "compare"}),
                    json!({"kind": "var", "name": 0}),
                    nats,
                    json!({"kind": "ordering"}),
                ),
                "primitive Compare does not apply to",
            );
            rejects(
                &unary(
                    json!({"kind": "equal"}),
                    json!({"kind": "var", "name": 0}),
                    json!({"kind": "int"}),
                    json!({"kind": "bool"}),
                ),
                "primitive Equal does not apply to [Int, Int]",
            );
            rejects(
                &unary(
                    json!({"kind": "nat_add"}),
                    json!({"kind": "var", "name": 0}),
                    json!({"kind": "int"}),
                    json!({"kind": "int"}),
                ),
                "primitive NatAdd takes [Nat, Nat], received [Int, Int]",
            );
            // A valid program loads and renders; an invalid one does neither.
            let valid = target::load(&bytes(&base)).expect("loads");
            assert!(target::render(&valid)
                .expect("renders")
                .starts_with("#![forbid(unsafe_code)]"));
            let invalid: Program = serde_json::from_value({
                let mut json = base.clone();
                json["functions"][0]["body"] = json!({"kind": "var", "name": 99});
                json
            })
            .expect("well formed");
            let error = target::render(&invalid).expect_err("an invalid program has no rendering");
            support::expect_code(&error, "LLB6005");
        }
        // §17.14: the denotation is a kernel-checked LexLean definition and
        // states every fixture's outcome.
        "TC-03" => {
            let cases = fixtures::cases();
            let module = fixtures::fixtures_module(&cases);
            let mut opaque = 0;
            for case in &cases {
                let id = fixtures::identifier(&case.fixture.name);
                assert!(module.contains(&format!("\"name\":\"{id}Run\"")), "{id}Run");
                let theorem = module.contains(&format!("\"name\":\"{id}Outcome\""));
                assert_eq!(
                    theorem,
                    fixtures::kernel_reducible(&case.fixture),
                    "{id}: a kernel theorem exactly where the kernel reduces"
                );
                if !theorem {
                    opaque += 1;
                }
                // A valid program never gets stuck: overflow and exhaustion
                // are its only failures.
                assert_ne!(case.fixture.expected, Outcome::Stuck, "{id}");
            }
            assert!(
                opaque > 0 && opaque < cases.len() / 4,
                "kernel-opaque fixtures are the exception ({opaque})"
            );
            assert!(cases
                .iter()
                .any(|case| matches!(case.fixture.expected, Outcome::Overflow { .. })));
            assert!(cases
                .iter()
                .any(|case| case.fixture.expected == Outcome::Exhausted));
            fixtures::check(repo_root().as_std_path(), false)
                .expect("the committed fixtures equal their generator");
            if let Some(verified) = support::lean_backed("TC-03").then(support::verified_compiler) {
                assert!(
                    verified.outcome.units.contains_key("TargetFixtures"),
                    "the fixture module is verified"
                );
                let planted = planted_verification();
                // Lean names the evaluation and the outcome it was told to
                // expect: the wrong sum is refused.
                assert!(
                    planted
                        .iter()
                        .any(|message| message.contains("sumToRun")
                            && message.contains("Value.nat 56")),
                    "a wrong expected outcome is rejected: {planted:#?}"
                );
            }
        }
        // §17.14: Lean's evaluator decides every fixture, the kernel-opaque
        // ones included.
        "TC-04" => {
            let cases = fixtures::cases();
            if !support::lean_backed("TC-04") {
                return;
            }
            let verified = support::verified_compiler();
            let evaluated = lean_evaluations(&verified.outcome.root, &cases);
            assert_eq!(evaluated.len(), cases.len());
            for case in &cases {
                let expected = serde_json::to_value(&case.fixture.expected).expect("outcome");
                assert_eq!(
                    evaluated[&case.fixture.name], expected,
                    "{}: Lean's evaluator and the interpreter disagree",
                    case.fixture.name
                );
            }
            // The comparison detects a planted discrepancy.
            let first = &cases[0];
            let mut wrong = serde_json::to_value(&first.fixture.expected).expect("outcome");
            wrong["steps"] = json!(9_999_999);
            assert_ne!(evaluated[&first.fixture.name], wrong);
        }
        // §17.14: every library template realizes LexLean's own primitive.
        "TC-05" => {
            let cases = fixtures::cases();
            let mut covered = BTreeSet::new();
            for case in cases.iter().filter(|case| case.library.is_some()) {
                let library = case.library.as_ref().expect("library");
                covered.insert(library.template);
                let instance = library
                    .template
                    .instantiate(&library.types, library.at)
                    .expect("instantiates");
                let at = usize::try_from(library.at).expect("index");
                let mut expected = case.fixture.program.clone();
                expected.functions.truncate(at);
                expected.functions.extend(instance);
                assert_eq!(
                    expected.canonical().expect("bound"),
                    case.fixture.program,
                    "{}: the committed instance is the template's",
                    case.fixture.name
                );
                assert!(
                    case.oracle.is_some(),
                    "{}: a library fixture states LexLean's own result",
                    case.fixture.name
                );
                assert!(
                    fixtures::kernel_reducible(&case.fixture),
                    "{}: the kernel decides it",
                    case.fixture.name
                );
                let id = fixtures::identifier(&case.fixture.name);
                assert!(
                    fixtures::fixtures_module(&cases).contains(&format!("\"name\":\"{id}Agrees\""))
                );
            }
            assert_eq!(
                covered.len(),
                Template::ALL.len(),
                "every template has a fixture"
            );
            if support::lean_backed("TC-05") {
                let _ = support::verified_compiler();
                let planted = planted_verification();
                // The mutated instance still matches its own interpreted
                // outcome, so only the statement against LexLean's
                // `set_insert` (through `TargetOracle`) fails.
                assert!(
                    planted
                        .iter()
                        .any(|message| message.contains("setInsertNatRun")
                            && message.contains("TargetOracle.encodeNats")),
                    "a mutated realization disagrees with LexLean: {planted:#?}"
                );
                assert!(
                    !planted
                        .iter()
                        .any(|message| message.contains("setInsertNatRun")
                            && !message.contains("TargetOracle")),
                    "the mutated instance agrees with its own interpreted outcome: {planted:#?}"
                );
            }
        }
        // §17.14, §17.13: the realization table covers the production
        // registry and the fixtures exercise every calculus element.
        "TC-06" => {
            let registry: toml::Value = toml::from_str(
                &std::fs::read_to_string(
                    repo_root()
                        .join("language/production-1.2.toml")
                        .as_std_path(),
                )
                .expect("registry"),
            )
            .expect("registry parses");
            let runtime: BTreeSet<String> = registry["construct"]
                .as_array()
                .expect("constructs")
                .iter()
                .filter(|row| row["disposition"].as_str() == Some("runtime"))
                .map(|row| row["key"].as_str().expect("key").to_owned())
                .collect();
            assert!(runtime.len() > 100, "the registry has runtime rows");
            realization::check_table(&runtime).expect("the realization table");
            // A dropped row and an unknown reference are both caught.
            let mut short = runtime.clone();
            short.insert("term.invented".to_owned());
            let error = realization::check_table(&short).expect_err("a missing row");
            assert!(
                error.contains("runtime construct `term.invented` has no realization row"),
                "{error}"
            );
            let mut fewer = runtime.clone();
            fewer.remove("primitive.map_insert");
            let error = realization::check_table(&fewer).expect_err("an extra row");
            assert!(
                error.contains("realization row `primitive.map_insert` is not a runtime construct"),
                "{error}"
            );
            let mut used = BTreeSet::new();
            for case in fixtures::cases() {
                used.extend(realization::program_elements(&case.fixture.program));
                for argument in &case.fixture.arguments {
                    used.extend(realization::value_elements(argument));
                }
                if let Outcome::Value { value, .. } = &case.fixture.expected {
                    used.extend(realization::value_elements(value));
                }
            }
            let templates: BTreeSet<Template> = fixtures::cases()
                .iter()
                .filter_map(|case| case.library.as_ref().map(|library| library.template))
                .collect();
            assert_eq!(
                templates.len(),
                Template::ALL.len(),
                "every library template is exercised"
            );
            let all = realization::all_elements().expect("complete samples");
            let missing: Vec<&String> = all.difference(&used).collect();
            assert!(
                missing.is_empty(),
                "calculus elements no fixture exercises: {missing:?}"
            );
            for kind in target::IntKind::ALL {
                assert!(
                    fixtures::cases().iter().any(|case| serde_json::to_string(
                        &case.fixture.program
                    )
                    .expect("json")
                    .contains(&format!("\"width\":\"{}\"", kind.name()))),
                    "fixed width {} is exercised",
                    kind.name()
                );
            }
        }
        // §17.14: the Rust profile reproduces every observable outcome, and
        // a planted renderer discrepancy is detected.
        "TC-07" => {
            let cases = fixtures::cases();
            let dir = tempfile::Builder::new()
                .prefix("lexlean-calculus-rust-")
                .tempdir()
                .expect("tempdir");
            let mut rendered = Vec::new();
            for case in &cases {
                let fixture = &case.fixture;
                let observed = match &fixture.expected {
                    Outcome::Value { value, .. } if observable(value) => {
                        rust::observable(&fixture.expected)
                    }
                    Outcome::Overflow { .. } => rust::observable(&fixture.expected),
                    _ => None,
                };
                let Some(observed) = observed else { continue };
                let source =
                    rust::render_harness(&fixture.program, fixture.entry, &fixture.arguments)
                        .unwrap_or_else(|reason| panic!("{}: {reason}", fixture.name));
                assert!(
                    source.starts_with("#![forbid(unsafe_code)]"),
                    "{}",
                    fixture.name
                );
                assert!(
                    !source.contains("unsafe {")
                        && !source.contains("unreachable!")
                        && !source.contains("panic!"),
                    "{}",
                    fixture.name
                );
                rendered.push((fixture.name.clone(), source, observed));
            }
            assert!(
                rendered.len() + 3 >= cases.len(),
                "only closure-valued and exhausted fixtures are unobservable in Rust"
            );
            let outputs: Vec<(String, String)> = std::thread::scope(|scope| {
                let handles: Vec<_> = rendered
                    .chunks(rendered.len().div_ceil(8))
                    .map(|chunk| {
                        let dir = dir.path();
                        scope.spawn(move || {
                            chunk
                                .iter()
                                .map(|(name, source, _)| {
                                    (
                                        name.clone(),
                                        run_rust(source, dir, &crate::calculus::identifier(name)),
                                    )
                                })
                                .collect::<Vec<_>>()
                        })
                    })
                    .collect();
                handles
                    .into_iter()
                    .flat_map(|handle| handle.join().expect("thread"))
                    .collect()
            });
            let by_name: BTreeMap<String, String> = outputs.into_iter().collect();
            for (name, _, observed) in &rendered {
                let printed: Json = serde_json::from_str(&by_name[name])
                    .unwrap_or_else(|error| panic!("{name}: {error}: {}", by_name[name]));
                assert_eq!(
                    &printed, observed,
                    "{name}: the Rust rendering and the denotation disagree"
                );
            }
            // Planted renderer discrepancies: a wrapping addition and a
            // Euclidean quotient each change an observable outcome.
            for (name, from, to) in [
                (
                    "nat-overflow-add",
                    "a.checked_add(b).ok_or(Overflow)",
                    "Ok(a.wrapping_add(b))",
                ),
                (
                    "int-arithmetic",
                    "a.checked_div(b).ok_or(Overflow)",
                    "a.checked_div_euclid(b).ok_or(Overflow)",
                ),
            ] {
                let (_, source, observed) = rendered
                    .iter()
                    .find(|(candidate, _, _)| candidate == name)
                    .expect("rendered");
                assert!(source.contains(from), "{name}: the plant site exists");
                let planted = source.replacen(from, to, 1);
                let printed: Json = serde_json::from_str(&run_rust(
                    &planted,
                    dir.path(),
                    &format!("planted_{}", crate::calculus::identifier(name)),
                ))
                .expect("json");
                assert_ne!(
                    &printed, observed,
                    "{name}: the planted discrepancy is detected"
                );
            }
        }
        other => panic!("no calculus case is wired for {other}"),
    }
}
