//! The `gnaf` suite: GN-01..GN-07, GNAF requests over the production
//! realization calculus (SPEC.md §17.15).

use std::path::PathBuf;
use std::sync::OnceLock;

use lexlean::gnaf::{
    self, Action, ActionKind, Answer, Boundary, Carrier, Charge, ClaimClass, Completeness, Fixture,
    Objective, Rejection, Request, Scope, Selector, Status, MAX_FUEL, MAX_SYSTEMS,
};
use serde_json::{json, Value as Json};

use crate::gnaf as fixtures;
use crate::support::{self, repo_root, P};

/// The committed fixtures, read from disk exactly as published.
fn committed() -> Vec<(PathBuf, Vec<u8>, Fixture)> {
    let dir = repo_root().join("compiler/gnaf");
    let mut out = Vec::new();
    for entry in std::fs::read_dir(dir.as_std_path()).expect("compiler/gnaf exists") {
        let path = entry.expect("entry").path();
        let bytes = std::fs::read(&path).expect("fixture bytes");
        let fixture: Fixture = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        out.push((path, bytes, fixture));
    }
    out.sort_by(|left, right| left.0.cmp(&right.0));
    assert!(!out.is_empty(), "GNAF has hand-constructed requests");
    out
}

fn schema(name: &str) -> Json {
    serde_json::from_slice(
        &std::fs::read(repo_root().join("schemas").join(name).as_std_path()).expect("schema"),
    )
    .expect("schema parses")
}

fn fixture(name: &str) -> Fixture {
    fixtures::fixtures()
        .into_iter()
        .find(|fixture| fixture.name == name)
        .unwrap_or_else(|| panic!("no GNAF fixture {name}"))
}

fn request(name: &str) -> Request {
    fixture(name).request
}

fn answer(request: &Request) -> Answer {
    gnaf::answer(request).expect("within capacity")
}

fn statuses(request: &Request) -> Vec<(u64, Selector, Status)> {
    gnaf::statuses(request).expect("within capacity")
}

fn rejected(rejection: Rejection) -> Answer {
    Answer::Rejected { rejection }
}

fn admitted_steps(status: Status) -> u64 {
    match status {
        Status::Admitted { steps, .. } => steps,
        other => panic!("not admitted: {other:?}"),
    }
}

/// A load expected to fail closed with `code` naming `message`.
fn refuses(bytes: &[u8], code: &str, message: &str) {
    let error = gnaf::load(bytes)
        .err()
        .unwrap_or_else(|| panic!("an invalid request loads: expected `{message}`"));
    support::expect_code(&error, code);
    assert!(
        error.to_string().contains(message),
        "expected `{message}` in: {error}"
    );
}

fn request_json(name: &str) -> Json {
    serde_json::to_value(request(name)).expect("request serializes")
}

fn bytes(json: &Json) -> Vec<u8> {
    serde_json::to_vec(json).expect("serializes")
}

/// The semantic data of a committed `compiler/src` module.
fn module_data(text: &str) -> Json {
    let start = text.find("\\semanticdata{").expect("semantic data") + "\\semanticdata{".len();
    let end = text
        .rfind("}\n\\end{semanticmodule}")
        .expect("end of semantic data");
    serde_json::from_str(&text[start..end]).expect("module data")
}

fn with_module_data(text: &str, data: &Json) -> String {
    let start = text.find("\\semanticdata{").expect("semantic data") + "\\semanticdata{".len();
    let end = text
        .rfind("}\n\\end{semanticmodule}")
        .expect("end of semantic data");
    format!(
        "{}{}{}",
        &text[..start],
        serde_json::to_string(data).expect("serializes"),
        &text[end..]
    )
}

/// The planted fixture verification: a copy of the compiler whose
/// `GnafFixtures` states one wrong answer and two answers computed from a
/// universe with the dispatching systems omitted. Pinned Lean must reject
/// all three.
fn planted_fixtures() -> &'static Vec<String> {
    static ERRORS: OnceLock<Vec<String>> = OnceLock::new();
    ERRORS.get_or_init(|| {
        let mut wrong = fixture("argmin-complete");
        let Answer::Argmin { members, steps } = wrong.expected.clone() else {
            panic!("argmin-complete has an argmin")
        };
        wrong.expected = Answer::Argmin {
            members,
            steps: steps + 1,
        };
        let omitted = |name: &str| {
            let mut planted = fixture(name);
            let mut narrowed = planted.request.clone();
            let Carrier::Grammar { thresholds, .. } = &mut narrowed.carrier else {
                panic!("{name} has a grammar")
            };
            thresholds.clear();
            planted.expected = answer(&narrowed);
            assert_ne!(
                planted.expected,
                fixture(name).expected,
                "{name}: the omission changes the answer"
            );
            planted
        };
        let planted = vec![
            wrong,
            omitted("argmin-over-systems"),
            omitted("frontier-incomparable"),
        ];
        let project = P::compiler();
        project.write(
            "src/GnafFixtures.lex.tex",
            &fixtures::fixtures_module(&planted),
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

/// The planted authority vector: a copy of the compiler whose `Gnaf`
/// states that the GNAF-VEC-02 frontier omits `rC`.
fn planted_vector() -> Vec<String> {
    let project = P::compiler();
    let text = std::fs::read_to_string(repo_root().join("compiler/src/Gnaf.lex.tex").as_std_path())
        .expect("Gnaf source");
    let mut data = module_data(&text);
    let theorem = data["declarations"]
        .as_array_mut()
        .expect("declarations")
        .iter_mut()
        .find(|declaration| declaration["name"] == "vec02Frontier")
        .expect("vec02Frontier");
    // [0, 1, 2] becomes [0, 1]: drop the last cons cell.
    let tail = &mut theorem["statement"]["right"]["tail"]["tail"];
    assert_eq!(
        tail["kind"], "cons",
        "the stated frontier has three members"
    );
    *tail = tail["tail"].clone();
    project.write("src/Gnaf.lex.tex", &with_module_data(&text, &data));
    let _guard = support::env_lock();
    let error = project.verify_fails_with("LLV7002");
    error
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.clone())
        .collect()
}

/// The machine of a request with one action's charge replaced.
fn charged(name: &str, kind: ActionKind, charge: Charge) -> Request {
    let mut out = request(name);
    out.machine.actions.retain(|action| action.kind != kind);
    out.machine.actions.push(Action { kind, charge });
    out
}

/// The claim classes of UOR-GNAF §12.4.
fn every_claim() -> Vec<ClaimClass> {
    use ClaimClass as C;
    vec![
        C::Exact,
        C::NormalForm,
        C::Canonical,
        C::RepresentationMinimal,
        C::ComparisonTheorem,
        C::ProfileDefinedComparison,
        C::InputTotal,
        C::GlobalOptimal,
        C::ArgminComplete,
        C::ParetoOptimal,
        C::FrontierComplete,
        C::PointwiseEnvelopeComplete,
        C::QueryFamilyAnswerComplete,
        C::UseCaseGlobalOptimal,
        C::WorkloadArgminComplete,
        C::WorkloadParetoOptimal,
        C::WorkloadFrontierComplete,
        C::FamilyOptimal,
        C::CompetitiveBound,
        C::CompetitiveOptimal,
        C::AsymptoticBound,
        C::AsymptoticOptimal,
        C::UseCaseClassComplete,
        C::UseCaseClassAnswerComplete,
        C::MaintainedUseCaseClass,
        C::RestrictedUniverseOptimal,
        C::RevisionPreserved,
        C::BestKnown,
        C::MeasuredBestAmongTested,
        C::HeuristicSelected,
        C::InstanceOptimal { alpha: 2, beta: 1 },
    ]
}

/// Run the case for one GN conformance ID.
///
/// # Panics
///
/// Panics when the capability does not hold.
#[allow(clippy::too_many_lines)]
pub fn run(id: &str) {
    match id {
        // §17.15: closed request form, canonical fixtures, fail-closed load,
        // and the host's capacities.
        "GN-01" => {
            let request_schema = schema("gnaf-request.schema.json");
            let fixture_schema = schema("gnaf-fixture.schema.json");
            for (path, bytes, fixture) in committed() {
                let name = path
                    .file_stem()
                    .expect("stem")
                    .to_string_lossy()
                    .into_owned();
                assert_eq!(fixture.name, name, "a fixture is named by its file");
                assert_eq!(fixture.spec, gnaf::FIXTURE_SPEC);
                assert_eq!(
                    bytes,
                    gnaf::to_file_bytes(&fixture),
                    "{name}: committed bytes are canonical"
                );
                let json: Json = serde_json::from_slice(&bytes).expect("json");
                let violations = crate::schema::validate(&fixture_schema, &json);
                assert!(violations.is_empty(), "{name}: {violations:?}");
                let violations = crate::schema::validate(&request_schema, &json["request"]);
                assert!(violations.is_empty(), "{name}: {violations:?}");
                let loaded = gnaf::load(&gnaf::to_file_bytes(&fixture.request))
                    .unwrap_or_else(|error| panic!("{name}: {error}"));
                assert_eq!(answer(&loaded), fixture.expected, "{name}");
            }
            crate::calculus::check(repo_root().as_std_path(), false)
                .expect("the committed requests equal their generator");

            let base = request_json("argmin-over-systems");
            refuses(b"{", "LLB6006", "malformed");
            let mut unknown = base.clone();
            unknown["optimizer"] = json!("fastest");
            refuses(&bytes(&unknown), "LLB6006", "unknown field");
            let mut spec = base.clone();
            spec["spec"] = json!("lexlean/gnaf-request/0");
            refuses(
                &bytes(&spec),
                "LLB6006",
                "expected `lexlean/gnaf-request/1`",
            );
            let mut reference = base.clone();
            reference["reference"]["functions"][0]["result"] = json!({"kind": "bool"});
            refuses(&bytes(&reference), "LLB6006", "reference:");
            let mut plan = base.clone();
            plan["carrier"]["plans"][1]["result"] = json!({"kind": "bool"});
            refuses(&bytes(&plan), "LLB6006", "system");
            let mut domain = base.clone();
            domain["domain"][0] = json!({"kind": "bool", "value": true});
            refuses(&bytes(&domain), "LLB6006", "domain argument 0:");
            let mut signature = base.clone();
            signature["carrier"]["result"] = json!({"kind": "nat"});
            refuses(
                &bytes(&signature),
                "LLB6006",
                "differ from the reference entry's",
            );
            for violations in [&unknown, &spec] {
                assert!(
                    !crate::schema::validate(&request_schema, violations).is_empty(),
                    "the schema refuses what the loader refuses"
                );
            }

            // The capacities: fuel beyond MAX_FUEL, a universe beyond
            // MAX_SYSTEMS, each before any evaluation.
            let mut fuel = base.clone();
            fuel["machine"]["fuel"] = json!(MAX_FUEL + 1);
            refuses(&bytes(&fuel), "LLS8002", "fuel");
            let mut universe = base.clone();
            let one_plan = base["carrier"]["plans"][1].clone();
            let plans = usize::try_from(MAX_SYSTEMS).expect("fits").isqrt() + 2;
            universe["carrier"]["plans"] = json!(vec![one_plan; plans]);
            refuses(&bytes(&universe), "LLS8002", "universe");
            let mut beyond = request("argmin-over-systems");
            beyond.machine.fuel = MAX_FUEL + 1;
            support::expect_code(
                &gnaf::answer(&beyond).expect_err("over capacity"),
                "LLS8002",
            );

            // At full capacity a diverging plan recurses MAX_FUEL deep and
            // its unknown cost leaves the answer incomplete.
            let mut deepest = request("unknown-cost-incomplete");
            deepest.machine.fuel = MAX_FUEL;
            let loaded = gnaf::load(&gnaf::to_file_bytes(&deepest)).expect("within capacity");
            assert_eq!(answer(&loaded), Answer::Incomplete);
            // A cost beyond the host's integers is unknown, never wrapped.
            let huge = charged(
                "argmin-over-systems",
                ActionKind::Preprocessing,
                Charge::Constant { cost: u64::MAX },
            );
            assert_eq!(gnaf::validate(&huge), None);
            assert_eq!(answer(&huge), Answer::Incomplete);
        }
        // §17.15: the kernel is the oracle for every committed answer.
        "GN-02" => {
            let fixtures = fixtures::fixtures();
            let module = fixtures::fixtures_module(&fixtures);
            for fixture in &fixtures {
                let id = crate::calculus::identifier(&fixture.name);
                assert!(
                    module.contains(&format!("\"name\":\"{id}Request\"")),
                    "{id}"
                );
                assert!(module.contains(&format!("\"name\":\"{id}Answer\"")), "{id}");
            }
            assert_eq!(
                std::fs::read_to_string(
                    repo_root()
                        .join("compiler/src/GnafFixtures.lex.tex")
                        .as_std_path()
                )
                .expect("module"),
                module,
                "the committed module equals its generator"
            );
            let main = std::fs::read_to_string(
                repo_root().join("compiler/src/Main.lex.tex").as_std_path(),
            )
            .expect("Main");
            assert!(
                main.contains("\\importmodule{GnafFixtures}"),
                "the entrypoint reaches the fixture module, so verify checks it"
            );
            if support::lean_backed("GN-02") {
                let verified = support::verified_compiler();
                assert!(
                    verified.outcome.units.contains_key("GnafFixtures"),
                    "the fixture module is verified"
                );
                // Lean names the request whose stated answer the kernel
                // refuses to reduce to.
                let planted = planted_fixtures();
                for request in [
                    "argminCompleteRequest",
                    "argminOverSystemsRequest",
                    "frontierIncomparableRequest",
                ] {
                    assert!(
                        planted.iter().any(|message| message.contains(&format!(
                            "Gnaf.answer {request}\nis not definitionally equal"
                        ))),
                        "{request}: Lean rejects the planted answer: {planted:#?}"
                    );
                }
            }
        }
        // §17.15: the universe is the grammar's expansion, independent of
        // evaluation, and no other carrier or evidence is admitted.
        "GN-03" => {
            let scalar = request("argmin-over-systems");
            assert_eq!(
                gnaf::expand(&scalar.carrier),
                vec![
                    Selector::Fixed { plan: 0 },
                    Selector::Fixed { plan: 1 },
                    Selector::Dispatch {
                        threshold: 3,
                        small: 0,
                        large: 1
                    },
                    Selector::Dispatch {
                        threshold: 3,
                        small: 1,
                        large: 0
                    },
                ]
            );
            // Membership does not depend on any evaluation: a fuel at which
            // nothing returns leaves every member in the universe.
            let mut starved = scalar.clone();
            starved.machine.fuel = 1;
            assert_eq!(statuses(&starved).len(), 4);
            assert!(statuses(&starved)
                .iter()
                .all(|(_, _, status)| *status == Status::Unresolved));
            assert_eq!(answer(&starved), Answer::Incomplete);
            for (name, rejection) in [
                (
                    "reject-internal-plan-universe",
                    Rejection::InternalPlanUniverse,
                ),
                (
                    "reject-optimizer-defined-universe",
                    Rejection::OptimizerDefinedUniverse,
                ),
                ("reject-discovered-universe", Rejection::DiscoveredUniverse),
                ("reject-cached-universe", Rejection::CachedUniverse),
                (
                    "reject-missing-completeness",
                    Rejection::MissingCompleteness,
                ),
                (
                    "reject-self-referential-completeness",
                    Rejection::SelfReferentialCompleteness,
                ),
                (
                    "reject-optimizer-completeness",
                    Rejection::OptimizerCompleteness,
                ),
                ("reject-empty-domain", Rejection::EmptyDomain),
            ] {
                assert_eq!(fixtures::expected(name), rejected(rejection), "{name}");
            }
            // A discovered carrier that happens to list every member is
            // still not a universe fixed before discovery.
            let everything = Request {
                carrier: Carrier::Discovered {
                    members: (0..4).collect(),
                },
                ..scalar.clone()
            };
            assert_eq!(answer(&everything), rejected(Rejection::DiscoveredUniverse));
            assert_eq!(
                answer(&Request {
                    completeness: Completeness::CitesOptimizer,
                    ..scalar
                }),
                rejected(Rejection::OptimizerCompleteness)
            );
        }
        // §17.15: the machine contract accounts every action.
        "GN-04" => {
            use ActionKind as K;
            for (name, rejection) in [
                (
                    "reject-duplicate-action",
                    Rejection::DuplicateAction {
                        action: K::Dispatch,
                    },
                ),
                (
                    "reject-unaccounted-dispatch",
                    Rejection::UnaccountedAction {
                        action: K::Dispatch,
                    },
                ),
                (
                    "reject-zero-cost-dispatch",
                    Rejection::HiddenCost {
                        action: K::Dispatch,
                    },
                ),
                (
                    "reject-constant-cost-fallback",
                    Rejection::HiddenCost {
                        action: K::Fallback,
                    },
                ),
                (
                    "reject-free-observation",
                    Rejection::HiddenCost {
                        action: K::Observation,
                    },
                ),
                (
                    "reject-undeclared-execution",
                    Rejection::HiddenCost {
                        action: K::Execution,
                    },
                ),
                (
                    "reject-zero-cost-preprocessing",
                    Rejection::HiddenCost {
                        action: K::Preprocessing,
                    },
                ),
                (
                    "reject-step-charged-advice",
                    Rejection::HiddenCost { action: K::Advice },
                ),
                (
                    "reject-undeclared-retained-state",
                    Rejection::HiddenCost {
                        action: K::RetainedState,
                    },
                ),
                (
                    "reject-free-preprocessing-complete-boundary",
                    Rejection::UncommonPreparation,
                ),
                (
                    "reject-uncommon-prepared-state",
                    Rejection::UncommonPreparation,
                ),
                (
                    "reject-communication",
                    Rejection::UnrealizableAction {
                        action: K::Communication,
                    },
                ),
                (
                    "reject-randomness",
                    Rejection::UnrealizableAction {
                        action: K::Randomness,
                    },
                ),
                (
                    "reject-scheduling",
                    Rejection::UnrealizableAction {
                        action: K::Scheduling,
                    },
                ),
            ] {
                assert_eq!(fixtures::expected(name), rejected(rejection), "{name}");
            }
            // Every performed kind, under every charge but steps, is hidden.
            for kind in ActionKind::ALL.into_iter().filter(|kind| kind.performed()) {
                for charge in [
                    Charge::Constant { cost: 0 },
                    Charge::Constant { cost: 5 },
                    Charge::Free,
                    Charge::Undeclared,
                ] {
                    assert_eq!(
                        gnaf::validate(&charged("argmin-over-systems", kind, charge)),
                        Some(Rejection::HiddenCost { action: kind }),
                        "{kind:?} charged {charge:?}"
                    );
                }
            }
            // Declared preparation is charged per invocation to every
            // system; common prepared state is excluded for every system.
            let Answer::Argmin { members, steps } = fixtures::expected("argmin-over-systems")
            else {
                panic!("an argmin")
            };
            let domain = request("argmin-over-systems").domain.len() as u64;
            assert_eq!(
                fixtures::expected("preparation-charged"),
                Answer::Argmin {
                    members: members.clone(),
                    steps: steps + domain * (3 + 2),
                }
            );
            for name in ["preparation-common-state", "preparation-common-plan"] {
                assert_eq!(
                    fixtures::expected(name),
                    Answer::Argmin {
                        members: members.clone(),
                        steps
                    },
                    "{name}"
                );
            }
            let plain = statuses(&request("argmin-over-systems"));
            let charged = statuses(&request("preparation-charged"));
            for (before, after) in plain.iter().zip(&charged) {
                assert_eq!(
                    admitted_steps(after.2),
                    admitted_steps(before.2) + domain * 5
                );
            }
            let common = Request {
                machine: gnaf::Machine {
                    boundary: Boundary::PreparedState,
                    preparation_common: false,
                    ..request("preparation-common-state").machine
                },
                ..request("preparation-common-state")
            };
            assert_eq!(
                gnaf::validate(&common),
                Some(Rejection::UncommonPreparation)
            );
        }
        // §17.15: claims and orders, and a frontier of incomparable systems.
        "GN-05" => {
            for (name, rejection) in [
                (
                    "reject-scalar-claim-partial-order",
                    Rejection::ScalarClaimOverPartialOrder,
                ),
                (
                    "reject-vector-claim-total-order",
                    Rejection::VectorClaimOverTotalOrder,
                ),
                ("reject-unsupported-claim", Rejection::UnsupportedClaim),
                ("reject-instance-optimal-claim", Rejection::UnsupportedClaim),
                ("reject-calculus-program-scope", Rejection::UncoveredScope),
                ("reject-rust-program-scope", Rejection::UncoveredScope),
            ] {
                assert_eq!(fixtures::expected(name), rejected(rejection), "{name}");
            }
            let scalar_claims = [
                ClaimClass::GlobalOptimal,
                ClaimClass::ArgminComplete,
                ClaimClass::RestrictedUniverseOptimal,
            ];
            let vector_claims = [ClaimClass::ParetoOptimal, ClaimClass::FrontierComplete];
            let claims = every_claim();
            assert_eq!(claims.len(), 31, "every §12.4 class");
            for claim in claims {
                for objective in [Objective::Scalar, Objective::Vector] {
                    let posed = Request {
                        claim,
                        objective,
                        ..request("argmin-over-systems")
                    };
                    let expected = if scalar_claims.contains(&claim) {
                        (objective == Objective::Vector)
                            .then_some(Rejection::ScalarClaimOverPartialOrder)
                    } else if vector_claims.contains(&claim) {
                        (objective == Objective::Scalar)
                            .then_some(Rejection::VectorClaimOverTotalOrder)
                    } else {
                        Some(Rejection::UnsupportedClaim)
                    };
                    assert_eq!(gnaf::validate(&posed), expected, "{claim:?} {objective:?}");
                    if expected.is_none() {
                        for scope in [Scope::CalculusPrograms, Scope::RustPrograms] {
                            assert_eq!(
                                gnaf::validate(&Request {
                                    scope,
                                    ..posed.clone()
                                }),
                                Some(Rejection::UncoveredScope)
                            );
                        }
                    }
                }
            }
            for name in ["frontier-incomparable", "frontier-pareto-optimal"] {
                assert_eq!(
                    fixtures::expected(name),
                    Answer::Frontier {
                        members: vec![1, 2]
                    },
                    "{name}"
                );
            }
            let entries = statuses(&request("frontier-incomparable"));
            let cost = |index: usize| match entries[index].2 {
                Status::Admitted { steps, size } => (steps, size),
                other => panic!("{index}: {other:?}"),
            };
            let (slower, faster) = (cost(1), cost(2));
            assert!(
                slower.0 > faster.0 && slower.1 < faster.1,
                "neither frontier member dominates the other: {slower:?} {faster:?}"
            );
            // No weighting turns the pair into a total order: the scalar
            // answer keeps only the faster one.
            assert_eq!(
                fixtures::expected("argmin-over-systems"),
                Answer::Argmin {
                    members: vec![2],
                    steps: faster.0
                }
            );
        }
        // §17.15: complete systems, not internal plans or envelopes.
        "GN-06" => {
            let whole = request("argmin-over-systems");
            let entries = statuses(&whole);
            let Answer::Argmin { members, steps } = fixtures::expected("argmin-over-systems")
            else {
                panic!("an argmin")
            };
            assert_eq!(members, vec![2]);
            assert!(
                matches!(entries[2].1, Selector::Dispatch { .. }),
                "the optimum is a dispatching system"
            );
            let best_plan = entries
                .iter()
                .filter(|(_, selector, _)| matches!(selector, Selector::Fixed { .. }))
                .map(|(_, _, status)| admitted_steps(*status))
                .min()
                .expect("fixed systems");
            assert!(
                best_plan > steps,
                "the best single plan ({best_plan}) is not the optimum ({steps})"
            );
            // Each restricted domain has its own best plan; together they
            // form the per-input envelope, which no system attains.
            let short = fixtures::expected("internal-best-on-short-lists");
            let long = fixtures::expected("internal-best-on-long-lists");
            let (
                Answer::Argmin {
                    members: short_best,
                    steps: short_steps,
                },
                Answer::Argmin {
                    members: long_best,
                    steps: long_steps,
                },
            ) = (short, long)
            else {
                panic!("both restricted requests have an argmin")
            };
            assert_eq!((short_best, long_best), (vec![0], vec![1]));
            let envelope = short_steps + long_steps;
            assert!(envelope < steps, "{envelope} < {steps}");
            assert!(entries
                .iter()
                .all(|(_, _, status)| admitted_steps(*status) > envelope));
            // Selection is charged: the dispatching system costs exactly
            // the envelope plus its observation and selection per argument.
            assert_eq!(
                steps - envelope,
                5 * whole.domain.len() as u64,
                "length, literal, comparison, conditional, and local per argument"
            );
            // An inadmissible plan is excluded, an unresolved one is never
            // removed, and nothing admitted is infeasible.
            let excluded = statuses(&request("inadmissible-plan-excluded"));
            assert_eq!(excluded[1].2, Status::Inadmissible);
            assert_eq!(
                fixtures::expected("inadmissible-plan-excluded"),
                Answer::Argmin {
                    members: vec![0],
                    steps: admitted_steps(excluded[0].2)
                }
            );
            let unknown = statuses(&request("unknown-cost-incomplete"));
            assert_eq!(unknown[2].2, Status::Unresolved);
            assert!(matches!(unknown[0].2, Status::Admitted { .. }));
            assert_eq!(
                fixtures::expected("unknown-cost-incomplete"),
                Answer::Incomplete
            );
            assert_eq!(
                fixtures::expected("infeasible-universe"),
                Answer::Infeasible
            );
        }
        // §17.15, §27.4: the authority's vectors are theorems, the authority
        // is pinned, and its claims are some-true.
        "GN-07" => {
            let text = std::fs::read_to_string(
                repo_root().join("compiler/src/Gnaf.lex.tex").as_std_path(),
            )
            .expect("Gnaf source");
            let data = module_data(&text);
            let theorems: Vec<&str> = data["declarations"]
                .as_array()
                .expect("declarations")
                .iter()
                .filter(|declaration| declaration["kind"] == "theorem")
                .filter_map(|declaration| declaration["name"].as_str())
                .collect();
            for vector in [
                "vec01Optimum",
                "vec01Extension",
                "vec02Frontier",
                "vec04NormalFormIsB",
                "vec04GlobalMinimumIsC",
                "vec17EnvelopeAtZero",
                "vec17EnvelopeAtOne",
                "vec17UniformWorstCase",
                "rej14ComponentwiseMinimaUnattained",
                "rej29FrontierOmission",
            ] {
                assert!(theorems.contains(&vector), "{vector} is stated");
            }
            let model = repo_model::Model::load_from_repo_root().expect("model");
            let authority = model
                .authorities
                .authority
                .iter()
                .find(|authority| authority.id == "UOR-GNAF-1-DRAFT-2")
                .expect("the GNAF authority row");
            let revision = authority.revision.as_deref().expect("revision");
            assert_eq!(revision.len(), 40, "a full commit");
            assert!(authority
                .immutable_url
                .as_deref()
                .expect("immutable URL")
                .contains(revision));
            assert!(authority.citation.contains(revision));
            let digest = authority.acquired_sha256.as_deref().expect("SHA-256");
            assert!(
                digest.len() == 64 && digest.bytes().all(|byte| byte.is_ascii_hexdigit()),
                "{digest}"
            );
            assert_eq!(
                authority.canonical_identifier.as_deref(),
                Some("uor-gnaf/1-draft.2")
            );
            assert_eq!(authority.source_role.as_deref(), Some("normative"));
            let claim = model
                .ledger
                .claim
                .iter()
                .find(|claim| claim.authority.as_deref() == Some("UOR-GNAF-1-DRAFT-2"))
                .expect("a ledger claim");
            assert_eq!(claim.level, repo_model::Level::SomeTrue);
            assert!(
                !model
                    .authorities
                    .authority
                    .iter()
                    .any(|authority| authority.statement.contains("UOR-NAF")),
                "UOR-NAF is informative and cited as no authority"
            );
            if support::lean_backed("GN-07") {
                let verified = support::verified_compiler();
                assert!(
                    verified.outcome.units.contains_key("Gnaf"),
                    "the model is verified"
                );
                let planted = planted_vector();
                assert!(
                    planted.iter().any(|message| message.contains(
                        "tableFrontier [(0, [1, 3]), (1, [2, 2]), (2, [3, 1]), (3, [3, 3])] = [0, 1]\nis false"
                    )),
                    "Lean rejects a frontier omitting rC: {planted:#?}"
                );
            }
        }
        _ => panic!("no GNAF case {id}"),
    }
}
