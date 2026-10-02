//! The `extraction` suite: NE-01..NE-06, named-root extraction through
//! Lean's compiler front end (SPEC.md §22.10).
//!
//! Lean is the oracle twice over: it produces the closure, and the closure
//! must equal the one the production-eligibility analysis computed
//! independently from the semantic IR. The rejection classes are exercised
//! against the committed extraction record of the production example, so
//! every planted record differs from a real answer of pinned Lean in exactly
//! the defect under test.

use std::collections::BTreeSet;

use crate::support::{self, P};
use lexlean::production::lcnf::{self, Rejection};
use lexlean::production::ModuleReport;

/// The committed raw extraction record of the production example: the
/// normalized stdout of the extraction process.
fn committed_record() -> String {
    let root =
        support::repo_root().join("examples/production/expected/verify/extract/process.json");
    let record: serde_json::Value = serde_json::from_slice(
        &std::fs::read(root.as_std_path()).expect("the committed extraction record"),
    )
    .expect("process record JSON");
    record["stdout"].as_str().expect("stdout").to_owned()
}

/// The roots, modules, and eligibility reports of the production example.
fn production_inputs() -> (Vec<String>, Vec<String>, Vec<ModuleReport>) {
    let project = P::copy_example("production");
    let checked = support::checked_project(&project);
    let reports: Vec<ModuleReport> = checked
        .modules
        .values()
        .filter_map(|module| module.production.clone())
        .filter(|report| !report.roots.is_empty())
        .collect();
    let roots = reports
        .iter()
        .flat_map(|report| report.roots.iter().map(|root| root.root.clone()))
        .collect();
    let modules = ["Production.Kernel", "Production.Main"]
        .map(str::to_owned)
        .to_vec();
    (roots, modules, reports)
}

fn extract(record: &str) -> Result<lcnf::CompilerInput, Rejection> {
    let (roots, modules, reports) = production_inputs();
    let reports: Vec<&ModuleReport> = reports.iter().collect();
    lcnf::compiler_input(record, &roots, &modules, &reports)
}

fn rejected(record: &str, fragment: &str) {
    match extract(record) {
        Err(Rejection::Rejected(reason)) => {
            assert!(
                reason.contains(fragment),
                "expected {fragment:?}, got {reason}"
            );
        }
        other => panic!("expected an LLV7011 rejection containing {fragment:?}, got {other:?}"),
    }
}

fn mutate(record: &str, edit: impl FnOnce(&mut serde_json::Value)) -> String {
    let mut value: serde_json::Value =
        serde_json::from_str(record.trim_end()).expect("record JSON");
    edit(&mut value);
    serde_json::to_string(&value).expect("record serializes")
}

fn declaration_mut<'a>(value: &'a mut serde_json::Value, name: &str) -> &'a mut serde_json::Value {
    value["declarations"]
        .as_array_mut()
        .expect("declarations")
        .iter_mut()
        .find(|declaration| declaration["name"] == name)
        .expect("the declaration")
}

/// Run pinned `lean` on a standalone extraction module: no project module is
/// imported, so only the probes, the adapter, and the root lookup run.
fn run_lean(text: &str) -> (i32, String) {
    let _guard = support::env_lock();
    let project = P::copy_example("production");
    let loaded = lexlean::project::Project::load(&project.root.join("lexlean.toml")).expect("load");
    let toolchain =
        lexlean::verify::toolchain::preflight(&loaded.config.limits).expect("the pinned toolchain");
    let source = project.root.join("Extract.lean");
    std::fs::write(source.as_std_path(), text).expect("write the extraction module");
    let output = std::process::Command::new(toolchain.lean.path.as_std_path())
        .arg(source.as_std_path())
        .current_dir(project.root.as_std_path())
        .output()
        .expect("run lean");
    (
        output.status.code().unwrap_or(-1),
        format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        ),
    )
}

#[allow(clippy::too_many_lines)]
pub fn run(id: &str) {
    match id {
        // §22.10: verification publishes one canonical compiler input whose
        // bytes do not depend on the project root.
        "NE-01" => {
            let record = committed_record();
            let input = extract(&record).expect("the committed record extracts");
            let bytes = input.to_file_bytes();
            let value: serde_json::Value = serde_json::from_slice(&bytes).expect("input JSON");
            support::assert_schema("compiler-input", "the production compiler input", &value);
            let committed = std::fs::read(
                support::repo_root()
                    .join("examples/production/expected/verify/production/compiler-input.json")
                    .as_std_path(),
            )
            .expect("the committed compiler input");
            assert_eq!(
                bytes, committed,
                "the committed input is the canonical form of the record"
            );
            assert_eq!(
                input.id(),
                lexlean::artifact::content_id::Sha256Digest::of(&committed)
            );
            // The input is a function of the program, not of Lean's unique
            // counter: renumbering every variable leaves it unchanged.
            let renumbered = record.replace("_uniq.", "_uniq.9");
            assert_ne!(renumbered, record);
            assert_eq!(
                extract(&renumbered).expect("renumbered").to_file_bytes(),
                bytes
            );
            if support::lean_backed("NE-01") {
                let first = P::copy_example("production");
                let second = P::copy_example("production");
                assert_ne!(first.root, second.root);
                let read = |project: &P| {
                    let verified = support::verify_ok(project);
                    let input = std::fs::read(
                        verified
                            .root
                            .join("production/compiler-input.json")
                            .as_std_path(),
                    )
                    .expect("published compiler input");
                    let attestation: serde_json::Value = serde_json::from_slice(
                        &std::fs::read(verified.root.join("attestation.json").as_std_path())
                            .expect("attestation"),
                    )
                    .expect("attestation JSON");
                    assert_eq!(
                        attestation["compiler_input"]["sha256"],
                        lexlean::artifact::content_id::Sha256Digest::of(&input).to_hex(),
                        "the attestation records the compiler input"
                    );
                    input
                };
                let left = read(&first);
                assert_eq!(
                    left,
                    read(&second),
                    "the compiler input is root independent"
                );
                assert_eq!(
                    left, committed,
                    "pinned Lean reproduces the committed input"
                );
            }
            // A project without a production root publishes no input.
            if support::lean_backed("NE-01") {
                let formal = P::copy_example("recursion");
                let verified = support::verify_ok(&formal);
                assert!(!verified.root.join("production").as_std_path().exists());
                assert!(!verified.root.join("extract").as_std_path().exists());
            }
        }
        // §22.10: each root's closure is exactly its computational
        // dependencies, equal to the eligibility closure, complete, and
        // free of proofs.
        "NE-02" => {
            let input = extract(&committed_record()).expect("extracts");
            let closure = |root: &str| {
                input
                    .closures
                    .iter()
                    .find(|closure| closure.root == format!("Production.Main.{root}"))
                    .expect("the root's closure")
                    .clone()
            };
            let halvings = closure("halvings");
            assert_eq!(
                halvings.declarations,
                ["Production.Kernel.countdown", "Production.Main.halvings"]
            );
            assert_eq!(halvings.erased, ["Production.Kernel.countdown_decreases"]);
            assert_eq!(
                closure("shapeArea").declarations,
                ["Production.Kernel.area", "Production.Main.shapeArea"]
            );
            assert_eq!(
                closure("quadruple").declarations,
                ["Production.Kernel.applyTwice", "Production.Main.quadruple"]
            );
            assert_eq!(
                closure("checkedSum").declarations,
                ["Production.Main.checkedSum"]
            );
            let names: BTreeSet<&str> = input
                .declarations
                .iter()
                .map(|declaration| declaration.name.as_str())
                .collect();
            assert_eq!(names.len(), 11, "{names:?}");
            assert!(!names.contains("Production.Kernel.countdown_decreases"));
            assert_eq!(input.erased, ["Production.Kernel.countdown_decreases"]);
            // Every closure declaration is the eligibility closure of its
            // root, computed independently from the semantic IR.
            let (_, _, reports) = production_inputs();
            for report in &reports {
                for root in &report.roots {
                    let lean = input
                        .closures
                        .iter()
                        .find(|closure| closure.root == root.root)
                        .expect("closure");
                    let analysis: BTreeSet<&str> = root
                        .runtime
                        .iter()
                        .map(|member| member.declaration.as_str())
                        .collect();
                    let extracted: BTreeSet<&str> =
                        lean.declarations.iter().map(String::as_str).collect();
                    assert_eq!(analysis, extracted, "{}", root.root);
                }
            }
            // Externals are Lean core constants or runtime members only.
            for external in &input.externals {
                assert!(
                    external.module.starts_with("Init")
                        || external.name.contains(".LexLeanRuntime.")
                        || external.name.contains(".LexLeanCollections."),
                    "{external:?}"
                );
            }
            // A proof-only dependency the eligibility analysis did not erase
            // is a disagreement, not a silent addition.
            rejected(
                &mutate(&committed_record(), |value| {
                    for closure in value["closures"].as_array_mut().expect("closures") {
                        if closure["root"] == "Production.Main.halvings" {
                            closure["erased"] = serde_json::json!([]);
                        }
                    }
                }),
                "the proof-only dependencies of `Production.Main.halvings` differ",
            );
        }
        // §22.10: every rejection class fails closed with LLV7011.
        "NE-03" => {
            let record = committed_record();
            let set_kind = |kind: &'static str| {
                mutate(&record, move |value| {
                    declaration_mut(value, "Production.Kernel.area")["kind"] = kind.into();
                })
            };
            rejected(&set_kind("opaque"), "`Production.Kernel.area` is an opaque");
            rejected(&set_kind("axiom"), "`Production.Kernel.area` is an axiom");
            rejected(
                &set_kind("partial-definition"),
                "`Production.Kernel.area` is a partial-definition",
            );
            rejected(
                &set_kind("unsafe-definition"),
                "`Production.Kernel.area` is an unsafe-definition",
            );
            rejected(
                &mutate(&record, |value| {
                    declaration_mut(value, "Production.Kernel.area")["safe"] = false.into();
                }),
                "`Production.Kernel.area` is unsafe",
            );
            rejected(
                &mutate(&record, |value| {
                    declaration_mut(value, "Production.Kernel.area")["computable"] = false.into();
                }),
                "`Production.Kernel.area` is noncomputable",
            );
            rejected(
                &mutate(&record, |value| {
                    declaration_mut(value, "Production.Kernel.area")["value"] =
                        serde_json::json!({"kind": "extern"});
                }),
                "unsupported compiler form: `Production.Kernel.area` is implemented externally",
            );
            rejected(
                &mutate(&record, |value| {
                    declaration_mut(value, "Production.Kernel.area")["type"] =
                        serde_json::json!({"kind": "unsupported", "expression": "fun x => x"});
                }),
                "unsupported compiler form: the LCNF type `fun x => x`",
            );
            rejected(
                &mutate(&record, |value| {
                    value["externals"].as_array_mut().expect("externals").push(serde_json::json!({
                        "name": "Std.HashMap.insert", "kind": "definition", "module": "Std.Data.HashMap.Basic",
                        "computable": true, "generates_code": true
                    }));
                }),
                "unresolved dependency: `Std.HashMap.insert`",
            );
            rejected(
                &mutate(&record, |value| {
                    for external in value["externals"].as_array_mut().expect("externals") {
                        if external["name"] == "Nat.blt" {
                            external["kind"] = "axiom".into();
                        }
                    }
                }),
                "`Nat.blt` is an axiom",
            );
            rejected(
                &mutate(&record, |value| {
                    for closure in value["closures"].as_array_mut().expect("closures") {
                        if closure["root"] == "Production.Main.sumAll" {
                            closure["root"] = "Production.Main.sumEverything".into();
                        }
                    }
                }),
                "unknown root: `Production.Main.sumEverything`",
            );
            rejected(
                &format!("{record}\nnoise\n"),
                "more than its single JSON record",
            );
            rejected(
                &mutate(&record, |value| value["surplus"] = true.into()),
                "the extraction record is malformed",
            );
            rejected(
                &mutate(&record, |value| {
                    value["roots"] = serde_json::json!(["Production.Main.sumAll"])
                }),
                "the extraction answered roots",
            );
            // Pinned Lean itself refuses a root that does not exist.
            if support::lean_backed("NE-03") {
                let driver = lcnf::driver(
                    "0".repeat(32).as_str(),
                    &["LexLeanNoSuch.root".to_owned()],
                    &[],
                )
                .expect("driver");
                let (exit, output) = run_lean(&driver.text);
                assert_ne!(exit, 0, "{output}");
                assert_eq!(
                    lcnf::classify_failure(&driver, &output),
                    Rejection::Rejected("unknown constant `LexLeanNoSuch.root`".to_owned()),
                    "{output}"
                );
            }
        }
        // §22.10: the authority interface is closed, pinned, and probed.
        "NE-04" => {
            let authority = lcnf::authority().expect("the authority registry");
            assert_eq!(authority.lean_version, "4.32.1");
            assert_eq!(
                authority.lean_githash,
                "f054605aea4b840552cca2e725580bffd1e1b704"
            );
            let model =
                repo_model::Model::load(&support::repo_root().join("model").into_std_path_buf())
                    .expect("model");
            assert!(
                model
                    .authorities
                    .authority
                    .iter()
                    .any(|row| row.id == authority.authority),
                "the registry cites a model authority row"
            );
            let adapter = lcnf::adapter().expect("adapter");
            // Every Lean constant the adapter calls has a registry row.
            for call in &authority.call {
                let short = call.name.rsplit('.').next().expect("segment");
                assert!(
                    adapter.contains(short),
                    "`{}` is registered but unused",
                    call.name
                );
            }
            for used in [
                "toDecl",
                "CompilerM.run",
                "toLCNFType",
                "shouldGenerateCode",
                "isNoncomputable",
                "getModuleIdxFor?",
                "getUsedConstants",
                "value?",
                "isTheorem",
                "isInternal",
                "versionString",
                "githash",
                "MetaM.run'",
                "liftCoreM",
                "moduleNames",
                "find?",
            ] {
                assert!(
                    adapter.contains(used),
                    "the adapter no longer uses `{used}`"
                );
                assert!(
                    authority.call.iter().any(|call| call.name.ends_with(used)),
                    "`{used}` has no registry row"
                );
            }
            assert!(
                !adapter.contains("--") && !adapter.contains("/-"),
                "the adapter has no comment"
            );
            assert!(
                !adapter.contains(".simp") && !adapter.contains("PassManager"),
                "no LCNF pass runs"
            );
            // A Lean identity other than the pin is drift.
            let foreign = mutate(&committed_record(), |value| {
                value["lean"]["githash"] = "0000000000000000000000000000000000000000".into();
            });
            assert!(
                matches!(extract(&foreign), Err(Rejection::Drift(reason)) if reason.contains("pinned to 4.32.1"))
            );
            if support::lean_backed("NE-04") {
                // Every source identity matches the pinned toolchain.
                let project = P::copy_example("production");
                let loaded = lexlean::project::Project::load(&project.root.join("lexlean.toml"))
                    .expect("load");
                let toolchain = lexlean::verify::toolchain::preflight(&loaded.config.limits)
                    .expect("toolchain");
                for (source, sha256) in authority
                    .call
                    .iter()
                    .map(|call| (&call.source, &call.source_sha256))
                    .chain(
                        authority
                            .types
                            .iter()
                            .map(|row| (&row.source, &row.source_sha256)),
                    )
                {
                    let bytes = std::fs::read(toolchain.root.join(source).as_std_path())
                        .expect("pinned source");
                    assert_eq!(
                        &lexlean::artifact::content_id::Sha256Digest::of(&bytes).to_hex(),
                        sha256,
                        "{source}"
                    );
                }
                // Every probe elaborates under pinned Lean.
                let driver = lcnf::driver("0".repeat(32).as_str(), &[], &[]).expect("driver");
                let (exit, output) = run_lean(&driver.text);
                assert_eq!(exit, 0, "{output}");
                assert!(
                    output.contains("\"spec\":\"lexlean/lcnf-extraction/1\""),
                    "{output}"
                );
                // A drifted signature fails on its probe line.
                let drifted = lcnf::Driver {
                    text: driver.text.replacen(
                        "Lean.Name → Lean.CoreM Bool",
                        "Lean.Name → Lean.CoreM Nat",
                        1,
                    ),
                    ..driver.clone()
                };
                assert_ne!(drifted.text, driver.text);
                let (exit, output) = run_lean(&drifted.text);
                assert_ne!(exit, 0);
                match lcnf::classify_failure(&drifted, &output) {
                    Rejection::Drift(reason) => assert!(
                        reason.contains("`Lean.Compiler.LCNF.shouldGenerateCode`"),
                        "{reason}"
                    ),
                    other => panic!("expected drift, got {other:?}"),
                }
            }
        }
        // §22.10: a dependency dropped by either side is caught.
        "NE-05" => {
            let record = committed_record();
            // Lean's side drops `Production.Kernel.area` from the closure and
            // the declarations: the eligibility closure still realizes it.
            let dropped = mutate(&record, |value| {
                value["declarations"]
                    .as_array_mut()
                    .expect("declarations")
                    .retain(|declaration| declaration["name"] != "Production.Kernel.area");
                for closure in value["closures"].as_array_mut().expect("closures") {
                    closure["declarations"]
                        .as_array_mut()
                        .expect("declarations")
                        .retain(|name| name != "Production.Kernel.area");
                }
            });
            rejected(
                &dropped,
                "realizes `Production.Kernel.area`, which Lean's compiler does not reach",
            );
            // Lean drops it from the declarations only: `shapeArea` still
            // calls it, so the input would be incomplete.
            let incomplete = mutate(&record, |value| {
                value["declarations"]
                    .as_array_mut()
                    .expect("declarations")
                    .retain(|declaration| declaration["name"] != "Production.Kernel.area");
            });
            rejected(&incomplete, "Production.Kernel.area");
            // The eligibility side drops it: Lean's compiler still reaches it.
            let (roots, modules, mut reports) = production_inputs();
            for report in &mut reports {
                for root in &mut report.roots {
                    root.runtime
                        .retain(|member| member.declaration != "Production.Kernel.area");
                }
            }
            let reports: Vec<&ModuleReport> = reports.iter().collect();
            match lcnf::compiler_input(&record, &roots, &modules, &reports) {
                Err(Rejection::Rejected(reason)) => assert!(
                    reason.contains("dropped dependency: Lean's compiler reaches `Production.Kernel.area` from `Production.Main.shapeArea`"),
                    "{reason}"
                ),
                other => panic!("expected the dropped dependency, got {other:?}"),
            }
        }
        // §22.10: a proof presented as a runtime member is rejected.
        "NE-06" => {
            let record = committed_record();
            let proof_as_runtime = mutate(&record, |value| {
                value["declarations"]
                    .as_array_mut()
                    .expect("declarations")
                    .push(serde_json::json!({
                        "name": "Production.Kernel.countdown_decreases", "kind": "theorem",
                        "computable": true, "generates_code": false
                    }));
                for closure in value["closures"].as_array_mut().expect("closures") {
                    if closure["root"] == "Production.Main.halvings" {
                        closure["declarations"] = serde_json::json!([
                            "Production.Kernel.countdown",
                            "Production.Kernel.countdown_decreases",
                            "Production.Main.halvings"
                        ]);
                    }
                }
            });
            rejected(
                &proof_as_runtime,
                "proof-as-runtime dependency: the theorem `Production.Kernel.countdown_decreases` is in the runtime closure",
            );
            // Erased and realized at once is the same defect.
            let both = mutate(&record, |value| {
                for closure in value["closures"].as_array_mut().expect("closures") {
                    if closure["root"] == "Production.Main.shapeArea" {
                        closure["erased"] = serde_json::json!(["Production.Kernel.area"]);
                    }
                }
            });
            match extract(&both) {
                Err(Rejection::Rejected(_)) => {}
                other => panic!("expected a rejection, got {other:?}"),
            }
        }
        other => panic!("no extraction case is wired for {other}"),
    }
}
