//! VR-20..VR-22 (SPEC.md §22.11, §22.12): the verifier's operational width,
//! its invariance, and its telemetry.
//!
//! These cases are about how verification is run, so they check that running
//! it differently changes nothing that is kept: the same project at two widths
//! publishes the same bytes, a verification with telemetry on publishes what
//! one with it off does, and a result never depends on the order in which the
//! processes finish. Each half that needs the pinned Lean runs through the
//! host gate, and the rest runs everywhere.

use std::collections::BTreeSet;
use std::sync::mpsc;
use std::sync::Mutex;
use std::time::Duration;

use lexlean::verify::operational::{Operational, DEFAULT_WIDTH, MAX_WIDTH};
use lexlean::verify::telemetry::{LABELS, VOCABULARY};
use lexlean::{Selection, VerifyRequest};

use crate::support::{self, P};

pub(crate) fn run(id: &str) {
    match id {
        "VR-20" => width_is_operational(),
        "VR-21" => width_invariance(),
        "VR-22" => telemetry_is_noncanonical(),
        other => panic!("no operational case for {other}"),
    }
}

fn verify_at(
    project: &P,
    operational: Operational,
) -> Result<lexlean::VerifiedProject, lexlean::LexLeanError> {
    // Shared with every other verification of the suite and exclusive of the
    // cases that replace `ELAN_HOME`: a verification that straddled one would
    // run its toolchain from two homes.
    let _environment = support::env_shared();
    operational.scoped(|| {
        project.engine().verify(VerifyRequest {
            selection: Selection::Entrypoints,
        })
    })
}

/// Every file under a verified directory, with its bytes.
fn tree(root: &camino::Utf8Path) -> std::collections::BTreeMap<String, Vec<u8>> {
    support::file_set(root)
        .into_iter()
        .map(|name| {
            let bytes =
                std::fs::read(root.join(&name).as_std_path()).expect("a verified file reads");
            (name, bytes)
        })
        .collect()
}

fn assert_trees_equal(
    left: &std::collections::BTreeMap<String, Vec<u8>>,
    right: &std::collections::BTreeMap<String, Vec<u8>>,
    what: &str,
) {
    assert_eq!(
        left.keys().collect::<Vec<_>>(),
        right.keys().collect::<Vec<_>>(),
        "{what}: the same files are published"
    );
    for (name, bytes) in left {
        assert!(
            bytes == &right[name],
            "{what}: `{name}` differs byte for byte between the two verifications"
        );
    }
}

/// VR-20: the width is an operational profile.
fn width_is_operational() {
    assert_eq!(DEFAULT_WIDTH, 1, "the conservative profile is the default");
    assert_eq!(MAX_WIDTH, 64);
    for (text, expected) in [
        (None, Some(1)),
        (Some(""), Some(1)),
        (Some("1"), Some(1)),
        (Some("2"), Some(2)),
        (Some("64"), Some(64)),
        (Some("0"), None),
        (Some("65"), None),
        (Some("+2"), None),
        (Some("2.0"), None),
        (Some("two"), None),
        (Some(" 2"), None),
    ] {
        match (Operational::parse_width(text), expected) {
            (Ok(width), Some(want)) => assert_eq!(width, want, "{text:?}"),
            (Err(error), None) => assert_eq!(error.code.as_str(), "LLV7017", "{text:?}"),
            (got, want) => panic!("{text:?}: expected {want:?}, got {got:?}"),
        }
    }

    // A malformed width is refused by the command line before anything is
    // built or published, as a registered code, on every host.
    let root = support::repo_root();
    let fixture = root.join("tests/negative/invalid-verify-width");
    let observed = crate::fixtures::check_shared(&fixture).expect("the committed fixture holds");
    assert_eq!(observed.codes, ["LLV7017"]);
    assert_eq!(observed.exit, 3, "an environment-class failure");
    assert!(
        !observed.project.root.join(".lexlean").exists(),
        "a malformed width is refused before the build is published"
    );
    for bad in ["2.5", "65", "-1"] {
        let project = P::example();
        let (exit, stdout, _) = support::with_env(&[("LEXLEAN_VERIFY_WIDTH", Some(bad))], || {
            project.cli(&["--diagnostic-format", "json", "verify"])
        });
        assert_eq!(exit, 3, "{bad}");
        assert!(stdout.contains("LLV7017"), "{bad}: {stdout}");
    }

    // The width is nowhere in the project: a project that states one is
    // refused as an unknown field, so no file of a project can carry it.
    let project = P::example();
    project.edit(
        "lexlean.toml",
        "language = \"1.0\"\n",
        "language = \"1.0\"\nprocess_width = 2\n",
    );
    let error = lexlean::Engine::load(&project.root.join("lexlean.toml"))
        .err()
        .expect("a project that states a width does not load");
    assert!(
        error
            .diagnostics
            .iter()
            .any(|d| d.code.as_str().starts_with("LLC")),
        "a width in lexlean.toml is a configuration error: {error}"
    );

    if !support::lean_backed("VR-20") {
        return;
    }
    // And the width reaches no artifact: the project verified at width 2 has
    // the attestation the default run of the same project has, and neither
    // the attestation nor any other file says what width produced it.
    let baseline = support::verified();
    let project = P::example();
    let wide = verify_at(&project, Operational::new(2, None).expect("width 2")).expect("verifies");
    assert_eq!(wide.attestation_id, baseline.outcome.attestation_id);
    assert_trees_equal(&tree(&wide.root), &tree(&baseline.outcome.root), "width 2");
    for (name, bytes) in tree(&wide.root) {
        let text = String::from_utf8_lossy(&bytes);
        assert!(
            !text.contains("process_width") && !text.contains("LEXLEAN_VERIFY_WIDTH"),
            "{name} records the width"
        );
    }
    assert!(crate::evidence::telemetry_in(wide.root.as_std_path())
        .expect("scans")
        .is_empty());
}

/// The imports of a project's modules, as `module -> imports`, from source.
fn imports_of(project: &P) -> std::collections::BTreeMap<String, Vec<String>> {
    let mut graph = std::collections::BTreeMap::new();
    for name in support::file_set(&project.root.join("src")) {
        let Some(module) = name.strip_suffix(".lex.tex") else {
            continue;
        };
        let text = project.read(&format!("src/{name}"));
        let imports = text
            .lines()
            .filter_map(|line| line.strip_prefix("\\importmodule{"))
            .filter_map(|rest| rest.split('}').next())
            .map(str::to_owned)
            .collect();
        graph.insert(module.to_owned(), imports);
    }
    graph
}

/// The sizes of the topological frontiers of an import graph.
fn frontiers(graph: &std::collections::BTreeMap<String, Vec<String>>) -> Vec<usize> {
    let mut placed: BTreeSet<&str> = BTreeSet::new();
    let mut sizes = Vec::new();
    while placed.len() < graph.len() {
        let ready: Vec<&str> = graph
            .iter()
            .filter(|(module, imports)| {
                !placed.contains(module.as_str())
                    && imports.iter().all(|i| placed.contains(i.as_str()))
            })
            .map(|(module, _)| module.as_str())
            .collect();
        assert!(!ready.is_empty(), "the import graph is acyclic");
        sizes.push(ready.len());
        placed.extend(ready);
    }
    sizes
}

/// VR-21: the same project at two widths, and a completion order that changes
/// nothing.
fn width_invariance() {
    // The scheduler, on every host: items that finish in reverse order are
    // consumed in item order, and of two failures the one first in item order
    // is the one reported, though the later one finished first.
    let items: Vec<usize> = (0..4).collect();
    let (senders, receivers): (Vec<_>, Vec<_>) =
        (0..items.len()).map(|_| mpsc::channel::<()>()).unzip();
    let receivers: Vec<Mutex<mpsc::Receiver<()>>> = receivers.into_iter().map(Mutex::new).collect();
    let finished = Mutex::new(Vec::new());
    let mut consumed = Vec::new();
    lexlean::verify::pool::run_ordered(
        4,
        &items,
        &|item: usize| {
            if item + 1 < items.len() {
                receivers[item]
                    .lock()
                    .expect("lock")
                    .recv_timeout(Duration::from_secs(10))
                    .expect("the next item finishes first");
            }
            finished.lock().expect("lock").push(item);
            if item > 0 {
                senders[item - 1].send(()).expect("wake the previous item");
            }
            item
        },
        &|_: &usize| false,
        |item, value| {
            assert_eq!(item, value);
            consumed.push(item);
            Ok(())
        },
    )
    .expect("four independent jobs");
    assert_eq!(
        *finished.lock().expect("lock"),
        [3, 2, 1, 0],
        "the jobs finish in reverse order"
    );
    assert_eq!(consumed, [0, 1, 2, 3], "and are consumed in item order");

    let (wake, asleep) = mpsc::channel::<()>();
    let asleep = Mutex::new(asleep);
    let error = lexlean::verify::pool::run_ordered(
        4,
        &items,
        &|item: usize| {
            match item {
                1 => asleep
                    .lock()
                    .expect("lock")
                    .recv_timeout(Duration::from_secs(10))
                    .expect("item 3 fails first"),
                3 => wake.send(()).expect("wake item 1"),
                _ => {}
            }
            item
        },
        &|item: &usize| *item == 1 || *item == 3,
        |item, value| {
            if value == 1 || value == 3 {
                Err(lexlean::LexLeanError::from_diagnostic(
                    lexlean::Diagnostic::new(
                        lexlean::code!("LLI9001"),
                        format!("item {item} failed"),
                    ),
                ))
            } else {
                Ok(())
            }
        },
    )
    .expect_err("two items fail");
    assert!(
        error.to_string().contains("item 1 failed"),
        "the failure first in item order is reported: {error}"
    );

    if !support::lean_backed("VR-21") {
        return;
    }
    // The committed peano-arithmetic project: `Addition` is imported by
    // `Divisibility`, `Doubling` and `Order`, which `Main` imports, so its
    // frontiers are one, three, and one modules wide.
    let narrow = P::copy_example("peano-arithmetic");
    let wide = P::copy_example("peano-arithmetic");
    assert_eq!(frontiers(&imports_of(&narrow)), [1, 3, 1]);

    let at_one = verify_at(&narrow, Operational::new(1, None).expect("width 1")).expect("width 1");
    let telemetry = tempfile::tempdir().expect("tempdir");
    let at_three = verify_at(
        &wide,
        Operational::new(3, Some(telemetry.path().to_path_buf())).expect("width 3"),
    )
    .expect("width 3");

    assert_eq!(at_one.attestation_id, at_three.attestation_id);
    let (one, three) = (tree(&at_one.root), tree(&at_three.root));
    assert_trees_equal(&one, &three, "peano-arithmetic at widths 1 and 3");
    let attestation = |files: &std::collections::BTreeMap<String, Vec<u8>>| -> serde_json::Value {
        serde_json::from_slice(&files["attestation.json"]).expect("attestation parses")
    };
    let (a1, a3) = (attestation(&one), attestation(&three));
    assert_eq!(
        a1["declarations"], a3["declarations"],
        "observed axiom sets and policy results"
    );
    assert_eq!(
        a1["processes"], a3["processes"],
        "process records, in order"
    );
    assert_eq!(
        a1["compiler_input"], a3["compiler_input"],
        "compiler input (absent for language 1.0)"
    );
    assert_eq!(a1["attestation_id"], a3["attestation_id"]);

    // The wide run really ran side by side: the frontier of three was one
    // frontier of three members at width 3.
    let events: Vec<serde_json::Value> = std::fs::read_to_string(
        telemetry
            .path()
            .join(lexlean::verify::telemetry::VERIFY_FILE),
    )
    .expect("telemetry was written")
    .lines()
    .map(|line| serde_json::from_str(line).expect("a telemetry line is JSON"))
    .collect();
    assert_eq!(events[0]["process_width"], 3);
    let members: Vec<u64> = events
        .iter()
        .filter(|event| event["event"] == "frontier")
        .filter_map(|event| event["members"].as_u64())
        .collect();
    assert_eq!(
        members,
        [1, 3, 1],
        "the frontiers of the project at width 3"
    );

    // A prior verification is never taken in place of the work: a published
    // directory that no longer equals what this run staged is refused, at the
    // wide width as at the narrow one.
    let process_record = at_three
        .root
        .join("process/lean/PeanoArithmetic.Addition.json");
    let mut tampered = std::fs::read(process_record.as_std_path()).expect("reads");
    tampered.push(b' ');
    std::fs::write(process_record.as_std_path(), tampered).expect("tampers");
    let error = verify_at(&wide, Operational::new(3, None).expect("width 3"))
        .err()
        .expect("a tampered published directory is not reused");
    assert!(
        error.to_string().contains("LLB6003")
            && error.to_string().contains("refusing to overwrite"),
        "{error}"
    );
}

/// The numbers of a spec sentence: the backticked words between the first `(`
/// after `marker` and its `)`.
fn quoted_words(text: &str, marker: &str) -> Vec<String> {
    let after = &text[text.find(marker).expect("the marker is in the text") + marker.len()..];
    let inside = &after[after.find('(').expect("an opening parenthesis") + 1..];
    let inside = &inside[..inside.find(')').expect("a closing parenthesis")];
    inside
        .split('`')
        .skip(1)
        .step_by(2)
        .map(str::to_owned)
        .collect()
}

/// VR-22: telemetry is evidence, never artifact.
fn telemetry_is_noncanonical() {
    // The vocabulary the gate refuses is the vocabulary §22.12 states.
    let stated = quoted_words(
        support::spec_text(),
        "Every key it uses is in a closed vocabulary",
    );
    assert_eq!(
        stated, VOCABULARY,
        "§22.12 and the verifier name the same keys"
    );
    assert!(
        LABELS.iter().all(|label| !VOCABULARY.contains(label)),
        "a label of an event is not a measurement"
    );

    // A scan finds a measurement planted anywhere in a verified directory, and
    // passes a directory that has none.
    let directory = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(directory.path().join("process/lean")).expect("mkdir");
    std::fs::write(
        directory.path().join("process/lean/M.json"),
        br#"{"tool":"lean","exit_code":0}"#,
    )
    .expect("write");
    assert!(crate::evidence::telemetry_in(directory.path())
        .expect("scans")
        .is_empty());
    for key in VOCABULARY {
        std::fs::write(
            directory.path().join("process/lean/M.json"),
            format!(r#"{{"tool":"lean","nested":[{{"{key}":1}}]}}"#),
        )
        .expect("plants");
        let found = crate::evidence::telemetry_in(directory.path()).expect("scans");
        assert_eq!(found.len(), 1, "{key} is found: {found:?}");
        assert!(found[0].contains(key));
    }

    if !support::lean_backed("VR-22") {
        return;
    }
    let project = P::example();
    let without =
        verify_at(&project, Operational::new(1, None).expect("width 1")).expect("verifies");
    let sink = tempfile::tempdir().expect("tempdir");
    assert!(
        !sink.path().starts_with(project.root.as_std_path()),
        "the telemetry directory is not inside the project"
    );
    let other = P::example();
    let with = verify_at(
        &other,
        Operational::new(1, Some(sink.path().to_path_buf())).expect("width 1"),
    )
    .expect("verifies with telemetry");
    assert_eq!(with.attestation_id, without.attestation_id);
    assert_trees_equal(
        &tree(&with.root),
        &tree(&without.root),
        "telemetry on and off",
    );
    assert!(crate::evidence::telemetry_in(with.root.as_std_path())
        .expect("scans")
        .is_empty());
    assert!(
        !other
            .root
            .join(".lexlean")
            .join("verified")
            .join("verify-telemetry.jsonl")
            .exists(),
        "no telemetry is written beside the artifacts"
    );

    let text = std::fs::read_to_string(sink.path().join(lexlean::verify::telemetry::VERIFY_FILE))
        .expect("telemetry was written");
    let events: Vec<serde_json::Value> = text
        .lines()
        .map(|line| serde_json::from_str(line).expect("a line is JSON"))
        .collect();
    let allowed: BTreeSet<&str> = VOCABULARY.iter().chain(LABELS.iter()).copied().collect();
    for event in &events {
        for key in event.as_object().expect("an object").keys() {
            assert!(
                allowed.contains(key.as_str()),
                "`{key}` is in no vocabulary: {event}"
            );
        }
    }
    let kinds: BTreeSet<&str> = events
        .iter()
        .filter_map(|event| event["event"].as_str())
        .collect();
    for kind in [
        "verify_begin",
        "stage",
        "frontier",
        "process",
        "completion",
        "verify_end",
    ] {
        assert!(kinds.contains(kind), "no `{kind}` event: {kinds:?}");
    }
    let stages: BTreeSet<&str> = events
        .iter()
        .filter(|event| event["event"] == "stage")
        .filter_map(|event| event["stage"].as_str())
        .collect();
    for stage in [
        "probe",
        "elaboration",
        "replay",
        "audit",
        "policy",
        "attestation",
    ] {
        assert!(stages.contains(stage), "no `{stage}` stage: {stages:?}");
    }
    assert_eq!(events.last().expect("events")["ok"], true);
    if cfg!(target_os = "linux") {
        assert!(
            events
                .iter()
                .filter(|event| event["event"] == "process" && event["tool"] == "lean")
                .all(|event| event["peak_rss_kib"].as_u64().is_some_and(|peak| peak > 0)),
            "Lean processes are measured where the host has /proc"
        );
    }

    // A planted measurement in a real verified directory is found: the
    // attestation of the run, with a wall time added to its first process.
    let planted = tempfile::tempdir().expect("tempdir");
    for (name, bytes) in tree(&with.root) {
        let destination = planted.path().join(&name);
        std::fs::create_dir_all(destination.parent().expect("parent")).expect("mkdir");
        std::fs::write(destination, bytes).expect("copy");
    }
    let path = planted.path().join("process/lean").join(
        support::file_set(&with.root.join("process/lean"))
            .into_iter()
            .next()
            .expect("a Lean process record"),
    );
    let mut record: serde_json::Value =
        serde_json::from_slice(&std::fs::read(&path).expect("reads")).expect("parses");
    record["wall_ms"] = serde_json::json!(1234);
    std::fs::write(&path, serde_json::to_vec(&record).expect("serializes")).expect("plants");
    let found = crate::evidence::telemetry_in(planted.path()).expect("scans");
    assert_eq!(found.len(), 1, "{found:?}");

    // A telemetry directory that cannot be written changes no result: the
    // sink is a path under a regular file.
    let blocked = tempfile::tempdir().expect("tempdir");
    std::fs::write(blocked.path().join("file"), b"x").expect("write");
    let third = P::example();
    let unwritable = verify_at(
        &third,
        Operational::new(1, Some(blocked.path().join("file").join("sink"))).expect("width 1"),
    )
    .expect("an unwritable sink does not fail verification");
    assert_eq!(unwritable.attestation_id, without.attestation_id);
}
