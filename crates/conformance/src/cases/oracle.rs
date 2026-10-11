//! Conformance cases for the differential validation oracle (SPEC.md §17.19).
//!
//! Each case has a host-independent half, which runs everywhere and decides
//! what the pin, the tables, the comparison, and the isolation audit are,
//! and an oracle-backed half, which needs the pinned toolchain and the
//! acquired oracle and which says that it did not run where it could not.

use std::collections::BTreeSet;
use std::path::Path;

use crate::oracle::{self, Pin, Unavailable};
use crate::oracle_differential as differential;
use crate::support::{self, repo_root};

fn read(relative: &str) -> String {
    std::fs::read_to_string(repo_root().join(relative).as_std_path())
        .unwrap_or_else(|error| panic!("{relative}: {error}"))
}

fn write(root: &Path, relative: &str, text: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().expect("parent")).expect("directory");
    std::fs::write(path, text).expect("file");
}

/// A pin that selects the files under `lean/` and `rust/`, for trees the
/// cases build themselves.
fn synthetic_pin() -> Pin {
    let text = read("oracle/lean4-prod.toml");
    Pin::parse(&text).expect("the committed pin")
}

fn pin_tree(dir: &Path) {
    write(dir, "lean/a.txt", "alpha");
    write(dir, "rust/b/c.rs", "beta");
    write(dir, "other/x", "gamma");
}

/// The digest of the tree [`pin_tree`] builds, computed independently of
/// the implementation (a Python transcription of the frame the pin states).
const SYNTHETIC_DIGEST: &str = "f85d0b3ee2fe54cb7fa069126812bd692d665b5c4e27a7dd40437edf7e5f1382";

/// The pin and the tree digest it names (OR-01), host-independent.
fn pin_and_digest() {
    let pin = Pin::committed();
    assert_eq!(pin.role, oracle::ROLE, "the oracle's role is validation evidence");
    assert_eq!(pin.name, "lean4-prod");
    assert_eq!(pin.remote, "https://github.com/auser/lean4-prod");
    assert!(!pin.licence.vendored && !pin.licence.text_shipped);
    assert_eq!(pin.used, ["lean/", "rust/", "rust-toolchain.toml"]);
    // Pin validation: each of these edits is a pin that is refused.
    let text = read("oracle/lean4-prod.toml");
    let refused = |from: &str, to: &str| -> String {
        let edited = text.replacen(from, to, 1);
        assert_ne!(edited, text, "the edit `{from}` applied");
        Pin::parse(&edited).expect_err(&format!("`{from}` -> `{to}` is refused"))
    };
    assert!(refused("role = \"validation-evidence\"", "role = \"authority\"")
        .contains("validation-evidence"));
    assert!(refused("role = \"validation-evidence\"", "role = \"proof\"")
        .contains("validation-evidence"));
    assert!(!refused(&format!("commit = \"{}\"\n", pin.commit), "").is_empty());
    assert!(!refused("name = \"lean4-prod\"\n", "").is_empty());
    assert!(!refused("timeout_seconds = 300", "timeout_seconds = 300\nextra = 1").is_empty());
    assert!(refused("vendored = false", "vendored = true").contains("vendored"));
    assert!(!refused(&pin.commit, &pin.commit[..39]).is_empty());
    assert!(!refused(&pin.tree_sha256, &pin.tree_sha256.to_uppercase()).is_empty());
    assert!(!refused("used = [\"lean/\", \"rust/\", \"rust-toolchain.toml\"]", "used = []").is_empty());
    assert!(!refused("used = [\"lean/\", \"rust/\", \"rust-toolchain.toml\"]", "used = [\"/etc/\"]").is_empty());
    // The digest: the frame is the one the pin states.
    let dir = tempfile::tempdir().expect("tempdir");
    pin_tree(dir.path());
    let (digest, files) = oracle::tree_digest(dir.path(), &synthetic_pin()).expect("digest");
    assert_eq!((digest.as_str(), files), (SYNTHETIC_DIGEST, 2));
    // A file outside the used prefixes, a `.git` or `.lake` or `target`
    // directory, and an empty directory are not part of it ...
    write(dir.path(), "docs/readme", "ignored");
    write(dir.path(), "lean/.lake/build/x", "ignored");
    write(dir.path(), "rust/target/debug/y", "ignored");
    write(dir.path(), ".git/HEAD", "ignored");
    std::fs::create_dir_all(dir.path().join("lean/empty")).expect("dir");
    assert_eq!(
        oracle::tree_digest(dir.path(), &synthetic_pin()).expect("digest").0,
        SYNTHETIC_DIGEST
    );
    // ... and a byte changed, a file added, a file removed, and a file
    // renamed each change it.
    let changed = |edit: &dyn Fn(&Path)| -> String {
        let copy = tempfile::tempdir().expect("tempdir");
        pin_tree(copy.path());
        edit(copy.path());
        oracle::tree_digest(copy.path(), &synthetic_pin()).expect("digest").0
    };
    let mut seen = BTreeSet::from([SYNTHETIC_DIGEST.to_owned()]);
    for (name, digest) in [
        ("a byte changed", changed(&|d| write(d, "lean/a.txt", "alphb"))),
        ("a file added", changed(&|d| write(d, "lean/new.lean", ""))),
        ("a file removed", changed(&|d| std::fs::remove_file(d.join("rust/b/c.rs")).expect("rm"))),
        (
            "a file renamed",
            changed(&|d| {
                std::fs::rename(d.join("lean/a.txt"), d.join("lean/z.txt")).expect("mv");
            }),
        ),
    ] {
        assert!(seen.insert(digest), "{name} changes the digest");
    }
    // The oracle is no authority and no ledger claim.
    let identifiers = repo_model::isolation::identifiers(repo_root().as_std_path()).expect("pin");
    for file in ["model/authorities.toml", "model/ledger.toml"] {
        assert_eq!(
            repo_model::isolation::mention(&read(file), &identifiers),
            None,
            "{file} must not cite the oracle: it is validation evidence, not an authority (§27.4)"
        );
    }
}

// ---- OR-07: the oracle's silence is never agreement --------------------------

/// A fake tool: a POSIX shell script with the given body.
#[cfg(unix)]
fn fake_tool(dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
    use std::os::unix::fs::PermissionsExt as _;
    let path = dir.join(name);
    std::fs::write(&path, format!("#!/bin/sh\n{body}\n")).expect("script");
    let mut mode = std::fs::metadata(&path).expect("metadata").permissions();
    mode.set_mode(0o755);
    std::fs::set_permissions(&path, mode).expect("chmod");
    path
}

/// The exporter that writes an IR defining `body` to its `--out`.
#[cfg(unix)]
const WRITES_IR: &str = "out=''\nwhile [ $# -gt 0 ]; do if [ \"$1\" = '--out' ]; then out=\"$2\"; fi; shift; done\nmkdir -p \"$out\"\nprintf '(module X\\n  (def f () Nat 1)\\n)\\n' > \"$out/kernel.ir\"";

/// What each failure of the oracle's tools is reported as.
#[cfg(unix)]
fn failures_are_unavailable() {
    use oracle::{Oracle, Stage, Verdict};
    let dir = tempfile::tempdir().expect("tempdir");
    let scratch = dir.path().join("scratch");
    std::fs::create_dir_all(&scratch).expect("scratch");
    let library = dir.path().join("lib");
    std::fs::create_dir_all(&library).expect("lib");
    let mut pin = Pin::committed();
    pin.timeout_seconds = 1;
    let exporter_ok = fake_tool(dir.path(), "export-ok", WRITES_IR);
    let prod_ok_body = "out=''\nwhile [ $# -gt 0 ]; do if [ \"$1\" = '--output' ]; then out=\"$2\"; fi; shift; done\nmkdir -p \"$out/src\"\nprintf '[package]\\nname = \"x\"\\n' > \"$out/Cargo.toml\"\nprintf 'pub fn f() -> u64 { 1 }\\n' > \"$out/src/lib.rs\"";
    let prod_ok = fake_tool(dir.path(), "prod-ok", prod_ok_body);
    let ask = |exporter: &Path, prod: &Path| {
        Oracle::with_tools(&pin, exporter.to_path_buf(), prod.to_path_buf(), scratch.clone())
            .verdict(&library, "M", "M.f", "crate-x")
    };
    // The reference: working tools are an acceptance.
    match ask(&exporter_ok, &prod_ok) {
        Ok(Verdict::Accepted { package, .. }) => assert!(package.contains_key("src/lib.rs")),
        other => panic!("working tools accept: {other:?}"),
    }
    // The oracle's own refusals are answers ...
    let refuses_export = fake_tool(
        dir.path(),
        "export-refuses",
        "echo 'uncaught exception: prod-export failed: unsupported local function in M.f: escaping' >&2\nexit 1",
    );
    match ask(&refuses_export, &prod_ok) {
        Ok(Verdict::Refused { stage: Stage::Export, message }) => {
            assert!(message.starts_with("unsupported local function"), "{message}");
        }
        other => panic!("a structured export refusal is a refusal: {other:?}"),
    }
    let refuses_gen = fake_tool(
        dir.path(),
        "prod-refuses",
        "echo 'Code generation refused: UnresolvedCall(\"Nat.succ\"): no' >&2\nexit 1",
    );
    match ask(&exporter_ok, &refuses_gen) {
        Ok(Verdict::Refused { stage: Stage::Codegen, message }) => {
            assert!(message.starts_with("UnresolvedCall"), "{message}");
        }
        other => panic!("a structured code generation refusal is a refusal: {other:?}"),
    }
    // ... and everything else is not: a crash, a signal, a missing output,
    // an output defining nothing, a timeout, and a failure of the generator
    // that does not state a refusal.
    let crashes = fake_tool(dir.path(), "crashes", "echo \"thread 'main' panicked at x\" >&2\nexit 101");
    let killed = fake_tool(dir.path(), "killed", "kill -9 $$");
    let silent = fake_tool(dir.path(), "silent", "exit 0");
    let empty = fake_tool(
        dir.path(),
        "empty",
        "out=''\nwhile [ $# -gt 0 ]; do if [ \"$1\" = '--out' ]; then out=\"$2\"; fi; shift; done\nmkdir -p \"$out\"\nprintf '(module X\\n)\\n' > \"$out/kernel.ir\"",
    );
    let slow = fake_tool(dir.path(), "slow", "sleep 30");
    let missing = dir.path().join("does-not-exist");
    for (name, result) in [
        ("an exporter that crashes", ask(&crashes, &prod_ok)),
        ("an exporter killed by a signal", ask(&killed, &prod_ok)),
        ("an exporter that writes nothing", ask(&silent, &prod_ok)),
        ("an exporter whose IR defines nothing", ask(&empty, &prod_ok)),
        ("an exporter that overruns its time", ask(&slow, &prod_ok)),
        ("an exporter that is not there", ask(&missing, &prod_ok)),
        ("a generator that crashes", ask(&exporter_ok, &crashes)),
        ("a generator that writes no crate", ask(&exporter_ok, &silent)),
        ("a generator that is not there", ask(&exporter_ok, &missing)),
    ] {
        match result {
            Err(Unavailable::ToolFailed { .. }) => {}
            other => panic!("{name} must be unavailable, not an answer: {other:?}"),
        }
    }
}

#[cfg(not(unix))]
fn failures_are_unavailable() {
    support::unix_only("OR-07", "POSIX shell for the oracle's failing stand-ins");
}

/// Acquisition fails closed: a checkout that is missing, that is another
/// oracle, or that does not build is unavailable.
fn acquisition_fails_closed() {
    let pin = Pin::committed();
    let cache = tempfile::tempdir().expect("tempdir");
    // A source that is not there.
    match oracle::Oracle::acquire_in(&pin, cache.path(), Some(cache.path().join("nowhere"))) {
        Err(Unavailable::NotAcquired(_)) => {}
        other => panic!("a missing checkout is not acquired: {:?}", other.map(|_| ())),
    }
    // A source that holds nothing of the oracle.
    let empty = tempfile::tempdir().expect("tempdir");
    match oracle::verify_checkout(&pin, empty.path()) {
        Err(Unavailable::PinMismatch(_)) => {}
        other => panic!("an empty checkout is not the pin: {other:?}"),
    }
}

/// A run with nothing to say says nothing: validated only when cases were
/// compared against an oracle that generated Rust.
fn vacuous_runs_are_unavailable() {
    let table = differential::KnownDifferences::committed();
    let empty = differential::Run {
        roots: Vec::new(),
        executed: std::collections::BTreeMap::new(),
        pin: Pin::committed(),
        timings: std::collections::BTreeMap::new(),
    };
    match differential::validate(&empty, &table) {
        differential::Validation::Unavailable(Unavailable::Vacuous(_)) => {}
        other => panic!("a run over no root is unavailable, not validated: {}", other.report()),
    }
    assert!(!differential::validate(&empty, &table).validated());
}

/// What a gate does with an oracle that did not run.
fn gates_decide() {
    let unavailable: Result<differential::Run, Unavailable> =
        Err(Unavailable::NotAcquired("planted".into()));
    match differential::decide(true, &unavailable) {
        differential::Gate::Fail(reason) => assert!(reason.contains("not a validation")),
        other => panic!("the normative host fails on an unavailable oracle: {other:?}"),
    }
    match differential::decide(false, &unavailable) {
        differential::Gate::NotValidated(reason) => assert!(reason.contains("planted")),
        other => panic!("another host reports not validated: {other:?}"),
    }
}

// ---- OR-08: evidence only ---------------------------------------------------

/// A tree with one small file under each root the isolation audit reads, and
/// the pin.
fn isolation_tree() -> tempfile::TempDir {
    let dir = tempfile::tempdir().expect("tempdir");
    write(dir.path(), "oracle/lean4-prod.toml", &read("oracle/lean4-prod.toml"));
    write(dir.path(), "crates/lexlean/src/lib.rs", "pub fn compile() {}\n");
    write(dir.path(), "language/semantics-1.2.toml", "version = \"1.2\"\n");
    write(dir.path(), "schemas/attestation.schema.json", "{}\n");
    write(dir.path(), "compiler/lexlean.lock", "spec = \"lexlean/lock/1\"\n");
    write(dir.path(), "examples/demo/expected/build/manifest.json", "{}\n");
    write(dir.path(), "model/authorities.toml", "spec = \"lexlean/authorities/1\"\n");
    write(dir.path(), "model/ledger.toml", "spec = \"lexlean/ledger/1\"\n");
    dir
}

fn isolation_audit() {
    // The real tree: nothing the identities are computed from names the
    // oracle, and the audit read every root it is meant to.
    let scanned = repo_model::isolation::scan(repo_root().as_std_path()).expect("the tree is isolated");
    assert!(scanned.files > 200, "{scanned:?}");
    assert_eq!(scanned.per_root.len(), repo_model::isolation::SCANNED.len());
    assert!(scanned.per_root.iter().all(|(_, count)| *count > 0), "{scanned:?}");
    // The compiler-semantics ID is the digest of exactly the embedded files,
    // and none of them names the oracle.
    let identifiers = repo_model::isolation::identifiers(repo_root().as_std_path()).expect("pin");
    assert!(lexlean::embedded::FILES.len() > 100);
    for (path, bytes) in lexlean::embedded::FILES {
        let text = String::from_utf8_lossy(bytes);
        assert_eq!(
            repo_model::isolation::mention(&text, &identifiers),
            None,
            "{path} is an input of the compiler-semantics ID and names the oracle"
        );
    }
    // A synthetic tree is clean, and the audit fails on each way of planting
    // the oracle in each place an identity or a record is read from.
    let pin = Pin::committed();
    let plants = [
        ("its name", "lean4-prod".to_owned()),
        ("its Lake target", "prod-export".to_owned()),
        ("its pinned commit", pin.commit.clone()),
        ("the digest of its files", pin.tree_sha256.clone()),
        ("its git tree", pin.git_tree.clone()),
        ("its pin's path", "oracle/lean4-prod.toml".to_owned()),
        ("its pin's schema", "lexlean/oracle-pin/1".to_owned()),
    ];
    let places = [
        "crates/lexlean/src/lib.rs",
        "language/semantics-1.2.toml",
        "schemas/attestation.schema.json",
        "compiler/lexlean.lock",
        "examples/demo/expected/build/manifest.json",
        "model/authorities.toml",
        "model/ledger.toml",
    ];
    let clean = isolation_tree();
    repo_model::isolation::scan(clean.path()).expect("a clean tree passes");
    for place in places {
        for (what, plant) in &plants {
            let tree = isolation_tree();
            let path = tree.path().join(place);
            let mut text = std::fs::read_to_string(&path).expect("file");
            text.push_str(&format!("\n{plant}\n"));
            std::fs::write(&path, text).expect("plant");
            let error = repo_model::isolation::scan(tree.path())
                .expect_err(&format!("planting {what} in {place} must fail the audit"));
            assert!(error.contains(place), "{error}");
        }
    }
    // An audit that read nothing has not passed.
    let hollow = isolation_tree();
    std::fs::remove_file(hollow.path().join("model/ledger.toml")).expect("rm");
    assert!(repo_model::isolation::scan(hollow.path())
        .expect_err("a scanned root with no file")
        .contains("holds no file"));
}

/// A planted `oracle` field in each record an identity or an attestation is
/// read back from fails the record's schema, which is closed.
fn records_are_closed() {
    use serde_json::{json, Value};
    let load = |name: &str| -> Value {
        serde_json::from_str(&read(&format!("schemas/{name}"))).expect("schema")
    };
    let plant = |instance: &Value| -> Value {
        let mut planted = instance.clone();
        planted["oracle"] = json!({"commit": Pin::committed().commit, "verdict": "validated"});
        planted
    };
    let found = |schema: &Value, instance: &Value| -> bool {
        crate::schema::validate(schema, instance)
            .iter()
            .any(|violation| violation.to_string().starts_with("/oracle"))
    };
    // Committed records: valid as they are, refused with the field.
    let manifest: Value = serde_json::from_str(&read("examples/production/expected/build/manifest.json"))
        .expect("manifest");
    let preservation: Value =
        serde_json::from_str(&read("examples/production/expected/verify/preserve/preservation.json"))
            .expect("preservation");
    let fixture: Value =
        serde_json::from_str(&read("compiler/certificates/bridge-attained.json")).expect("fixture");
    let certificate = fixture["certificate"].clone();
    for (schema_name, instance) in [
        ("build-manifest-v2.schema.json", &manifest),
        ("preservation.schema.json", &preservation),
        ("gnaf-certificate.schema.json", &certificate),
    ] {
        let schema = load(schema_name);
        assert!(
            crate::schema::validate(&schema, instance).is_empty(),
            "{schema_name}: the committed record is valid"
        );
        assert!(
            found(&schema, &plant(instance)),
            "{schema_name}: an oracle field in the record is refused"
        );
    }
    // Records generated only by a verification run: their schemas admit no
    // oracle field either.
    for schema_name in [
        "attestation-v2.schema.json",
        "attestation.schema.json",
        "semantic-snapshot-v2.schema.json",
        "compiler-input.schema.json",
    ] {
        let schema = load(schema_name);
        assert_eq!(schema["additionalProperties"], json!(false), "{schema_name} is closed");
        assert!(
            found(&schema, &json!({"oracle": {"verdict": "validated"}})),
            "{schema_name}: an oracle field is refused"
        );
    }
}

/// Nothing outside the oracle's own modules names the oracle's module in the
/// conformance crate: the other cases are what they are with the oracle gone.
fn oracle_is_only_its_own() {
    let mut users = Vec::new();
    for entry in std::fs::read_dir(repo_root().join("crates/conformance/src").as_std_path())
        .expect("sources")
        .flatten()
    {
        let path = entry.path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let name = path.file_name().map(|n| n.to_string_lossy().into_owned()).unwrap_or_default();
        let text = std::fs::read_to_string(&path).expect("source");
        if text.contains("crate::oracle") && !name.starts_with("oracle") {
            users.push(name);
        }
    }
    assert!(
        users.is_empty(),
        "only the oracle's modules and its cases use the oracle: {users:?}"
    );
}
