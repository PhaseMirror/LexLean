//! The hand-constructed GNAF requests of SPEC.md §17.15, and the generated
//! `compiler` project files that state their answers to Lean.
//!
//! Every request is written here, by hand; its expected answer is computed
//! by the host transcription `lexlean::gnaf::answer`. The committed
//! `compiler/gnaf/<name>.json` files, the `compiler/src/GnafFixtures`
//! module, and the `Gnaf` model it imports ([`crate::gnaf_model`]) are
//! exactly what [`files`] renders, which `cargo xtask check-calculus`
//! enforces. In the module, each request is a `Gnaf.Request` definition and
//! a theorem that the kernel reduces `Gnaf.answer` of it to the expected
//! answer, so the model, not the transcription, is the oracle.

use std::collections::BTreeMap;

use lexlean::calculus::{term, Arm, Expr, Function, Prim, Program, Shape, Ty, Value, PROGRAM_SPEC};
use lexlean::gnaf::{
    self, Action, ActionKind, Answer, Boundary, Carrier, Charge, ClaimClass, Completeness, Fixture,
    Machine, Objective, Request, Scope, FIXTURE_SPEC, MODEL, REQUEST_SPEC,
};
use serde_json::{json, Value as Json};

fn nat_t() -> Ty {
    Ty::Nat
}
fn list_t() -> Ty {
    Ty::List {
        element: Box::new(nat_t()),
    }
}
fn option_t() -> Ty {
    Ty::Option {
        value: Box::new(nat_t()),
    }
}
fn natv(number: u64) -> Value {
    Value::Nat {
        value: number.to_string(),
    }
}
fn listv(items: &[u64]) -> Value {
    Value::List {
        items: items.iter().copied().map(natv).collect(),
    }
}
fn var(name: u64) -> Expr {
    Expr::Var { name }
}
fn prim(operation: Prim, operands: Vec<Expr>) -> Expr {
    Expr::Prim {
        operation,
        operands,
    }
}
fn build(shape: Shape, operands: Vec<Expr>) -> Expr {
    Expr::Build {
        shape,
        ty: option_t(),
        operands,
    }
}
fn arm(shape: Shape, binders: Vec<u64>, body: Expr) -> Arm {
    Arm {
        shape,
        binders,
        body,
    }
}
fn on_list(scrutinee: Expr, empty: Expr, binders: Vec<u64>, nonempty: Expr) -> Expr {
    Expr::Match {
        ty: option_t(),
        scrutinee: Box::new(scrutinee),
        arms: vec![
            arm(Shape::Nil, Vec::new(), empty),
            arm(Shape::Cons, binders, nonempty),
        ],
    }
}
fn plan(body: Expr) -> Function {
    Function {
        parameters: vec![0],
        types: vec![list_t()],
        result: option_t(),
        body,
    }
}

/// The last element by structural recursion: cheap on short lists, one
/// round of matching per element on long ones.
fn last_by_recursion(at: u64) -> Function {
    plan(on_list(
        var(0),
        build(Shape::None, Vec::new()),
        vec![1, 2],
        on_list(
            var(2),
            build(Shape::Some, vec![var(1)]),
            vec![3, 4],
            Expr::Call {
                function: at,
                operands: vec![var(2)],
            },
        ),
    ))
}

/// The last element by slicing at the end: a fixed cost on every list,
/// above recursion's on short lists and far below it on long ones.
fn last_by_slice() -> Function {
    let one = || Expr::Value {
        ty: nat_t(),
        value: natv(1),
    };
    let length = || prim(Prim::Length, vec![var(0)]);
    plan(Expr::Cond {
        condition: Box::new(prim(
            Prim::NatLt,
            vec![
                Expr::Value {
                    ty: nat_t(),
                    value: natv(0),
                },
                length(),
            ],
        )),
        then_branch: Box::new(Expr::Match {
            ty: option_t(),
            scrutinee: Box::new(prim(
                Prim::Slice,
                vec![var(0), prim(Prim::NatSub, vec![length(), one()]), one()],
            )),
            arms: vec![
                arm(Shape::None, Vec::new(), build(Shape::None, Vec::new())),
                arm(
                    Shape::Some,
                    vec![1],
                    on_list(
                        var(1),
                        build(Shape::None, Vec::new()),
                        vec![2, 3],
                        build(Shape::Some, vec![var(2)]),
                    ),
                ),
            ],
        }),
        else_branch: Box::new(build(Shape::None, Vec::new())),
    })
}

/// The first element: a wrong plan, inadmissible on every list of two or
/// more elements.
fn first_element() -> Function {
    plan(on_list(
        var(0),
        build(Shape::None, Vec::new()),
        vec![1, 2],
        build(Shape::Some, vec![var(1)]),
    ))
}

/// A plan that never returns, so its cost is unknown at any fuel.
fn diverging(at: u64) -> Function {
    plan(Expr::Call {
        function: at,
        operands: vec![var(0)],
    })
}

/// The problem: the last element of a list of naturals.
fn reference() -> Program {
    Program {
        spec: PROGRAM_SPEC.to_owned(),
        adts: Vec::new(),
        functions: vec![last_by_recursion(0)],
    }
}

fn short_lists() -> Vec<Value> {
    vec![listv(&[7]), listv(&[3]), listv(&[9])]
}

fn long_lists() -> Vec<Value> {
    vec![listv(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12])]
}

/// Short and long lists, on which neither plan alone is best.
fn mixed_domain() -> Vec<Value> {
    let mut out = short_lists();
    out.extend(long_lists());
    out
}

fn accounted(kind: ActionKind, charge: Charge) -> Action {
    Action { kind, charge }
}

/// The machine every valid request starts from: the four actions systems
/// perform, each charged by steps, compared on the complete boundary.
fn machine() -> Machine {
    Machine {
        fuel: 256,
        actions: [
            ActionKind::Observation,
            ActionKind::Dispatch,
            ActionKind::Fallback,
            ActionKind::Execution,
        ]
        .into_iter()
        .map(|kind| accounted(kind, Charge::Steps))
        .collect(),
        boundary: Boundary::Complete,
        preparation_common: false,
    }
}

/// A grammar over the plans at indices 1.. of every realization, with one
/// dispatch threshold.
fn grammar(plans: Vec<Function>, thresholds: Vec<u64>) -> Carrier {
    Carrier::Grammar {
        argument: list_t(),
        result: option_t(),
        plans,
        thresholds,
    }
}

fn two_plans() -> Carrier {
    grammar(vec![last_by_recursion(1), last_by_slice()], vec![3])
}

fn request(
    carrier: Carrier,
    domain: Vec<Value>,
    objective: Objective,
    claim: ClaimClass,
) -> Request {
    Request {
        spec: REQUEST_SPEC.to_owned(),
        reference: reference(),
        domain,
        machine: machine(),
        carrier,
        completeness: Completeness::GrammarEquality,
        objective,
        claim,
        scope: Scope::GrammarUniverse,
    }
}

fn scalar() -> Request {
    request(
        two_plans(),
        mixed_domain(),
        Objective::Scalar,
        ClaimClass::GlobalOptimal,
    )
}

fn vector() -> Request {
    request(
        two_plans(),
        mixed_domain(),
        Objective::Vector,
        ClaimClass::FrontierComplete,
    )
}

fn with_machine(edit: impl FnOnce(&mut Machine)) -> Request {
    let mut out = scalar();
    edit(&mut out.machine);
    out
}

fn set_charge(machine: &mut Machine, kind: ActionKind, charge: Charge) {
    machine.actions.retain(|action| action.kind != kind);
    machine.actions.push(accounted(kind, charge));
}

/// Every request, by name, before its answer is computed.
fn requests() -> Vec<(&'static str, Request)> {
    let mut out = vec![
        ("argmin-over-systems", scalar()),
        ("frontier-incomparable", vector()),
        (
            "frontier-pareto-optimal",
            Request {
                claim: ClaimClass::ParetoOptimal,
                ..vector()
            },
        ),
        (
            "internal-best-on-short-lists",
            Request {
                domain: short_lists(),
                ..scalar()
            },
        ),
        (
            "internal-best-on-long-lists",
            Request {
                domain: long_lists(),
                ..scalar()
            },
        ),
        (
            "restricted-universe-argmin",
            Request {
                claim: ClaimClass::RestrictedUniverseOptimal,
                ..scalar()
            },
        ),
        (
            "argmin-complete",
            Request {
                claim: ClaimClass::ArgminComplete,
                ..scalar()
            },
        ),
        (
            "inadmissible-plan-excluded",
            Request {
                carrier: grammar(vec![last_by_recursion(1), first_element()], vec![2]),
                ..scalar()
            },
        ),
        (
            "infeasible-universe",
            Request {
                carrier: grammar(vec![first_element()], Vec::new()),
                ..scalar()
            },
        ),
        (
            "unknown-cost-incomplete",
            Request {
                carrier: grammar(
                    vec![last_by_recursion(1), last_by_slice(), diverging(3)],
                    Vec::new(),
                ),
                ..scalar()
            },
        ),
        (
            "preparation-charged",
            with_machine(|machine| {
                set_charge(
                    machine,
                    ActionKind::Preprocessing,
                    Charge::Constant { cost: 3 },
                );
                set_charge(machine, ActionKind::Advice, Charge::Constant { cost: 2 });
            }),
        ),
        (
            "preparation-common-state",
            with_machine(|machine| {
                set_charge(machine, ActionKind::RetainedState, Charge::Free);
                machine.boundary = Boundary::PreparedState;
                machine.preparation_common = true;
            }),
        ),
        (
            "preparation-common-plan",
            with_machine(|machine| {
                set_charge(machine, ActionKind::Preprocessing, Charge::Free);
                machine.boundary = Boundary::PreparedPlan;
                machine.preparation_common = true;
            }),
        ),
        // Fail-closed requests: each violates exactly one rule.
        (
            "reject-empty-domain",
            Request {
                domain: Vec::new(),
                ..scalar()
            },
        ),
        (
            "reject-internal-plan-universe",
            Request {
                carrier: Carrier::InternalPlans,
                ..scalar()
            },
        ),
        (
            "reject-optimizer-defined-universe",
            Request {
                carrier: Carrier::OptimizerOutput,
                ..scalar()
            },
        ),
        (
            "reject-discovered-universe",
            Request {
                carrier: Carrier::Discovered {
                    members: vec![0, 1],
                },
                ..scalar()
            },
        ),
        (
            "reject-cached-universe",
            Request {
                carrier: Carrier::Cached,
                ..scalar()
            },
        ),
        (
            "reject-missing-completeness",
            Request {
                completeness: Completeness::Missing,
                ..scalar()
            },
        ),
        (
            "reject-self-referential-completeness",
            Request {
                completeness: Completeness::CitesUniverseId,
                ..scalar()
            },
        ),
        (
            "reject-optimizer-completeness",
            Request {
                completeness: Completeness::CitesOptimizer,
                ..scalar()
            },
        ),
        (
            "reject-duplicate-action",
            with_machine(|machine| {
                machine
                    .actions
                    .push(accounted(ActionKind::Dispatch, Charge::Steps));
            }),
        ),
        (
            "reject-unaccounted-dispatch",
            with_machine(|machine| {
                machine
                    .actions
                    .retain(|action| action.kind != ActionKind::Dispatch);
            }),
        ),
        (
            "reject-zero-cost-dispatch",
            with_machine(|machine| {
                set_charge(machine, ActionKind::Dispatch, Charge::Constant { cost: 0 });
            }),
        ),
        (
            "reject-constant-cost-fallback",
            with_machine(|machine| {
                set_charge(machine, ActionKind::Fallback, Charge::Constant { cost: 1 });
            }),
        ),
        (
            "reject-free-observation",
            with_machine(|machine| set_charge(machine, ActionKind::Observation, Charge::Free)),
        ),
        (
            "reject-undeclared-execution",
            with_machine(|machine| {
                set_charge(machine, ActionKind::Execution, Charge::Undeclared);
            }),
        ),
        (
            "reject-zero-cost-preprocessing",
            with_machine(|machine| {
                set_charge(
                    machine,
                    ActionKind::Preprocessing,
                    Charge::Constant { cost: 0 },
                );
            }),
        ),
        (
            "reject-step-charged-advice",
            with_machine(|machine| set_charge(machine, ActionKind::Advice, Charge::Steps)),
        ),
        (
            "reject-undeclared-retained-state",
            with_machine(|machine| {
                set_charge(machine, ActionKind::RetainedState, Charge::Undeclared);
            }),
        ),
        (
            "reject-free-preprocessing-complete-boundary",
            with_machine(|machine| set_charge(machine, ActionKind::Preprocessing, Charge::Free)),
        ),
        (
            "reject-uncommon-prepared-state",
            with_machine(|machine| {
                set_charge(machine, ActionKind::Preprocessing, Charge::Free);
                machine.boundary = Boundary::PreparedState;
            }),
        ),
        (
            "reject-scalar-claim-partial-order",
            Request {
                claim: ClaimClass::GlobalOptimal,
                ..vector()
            },
        ),
        (
            "reject-vector-claim-total-order",
            Request {
                claim: ClaimClass::FrontierComplete,
                ..scalar()
            },
        ),
        (
            "reject-unsupported-claim",
            Request {
                claim: ClaimClass::BestKnown,
                ..scalar()
            },
        ),
        (
            "reject-instance-optimal-claim",
            Request {
                claim: ClaimClass::InstanceOptimal { alpha: 2, beta: 1 },
                ..scalar()
            },
        ),
        (
            "reject-calculus-program-scope",
            Request {
                scope: Scope::CalculusPrograms,
                ..scalar()
            },
        ),
        (
            "reject-rust-program-scope",
            Request {
                scope: Scope::RustPrograms,
                ..scalar()
            },
        ),
    ];
    for kind in [
        ActionKind::Communication,
        ActionKind::Randomness,
        ActionKind::Scheduling,
    ] {
        let name = match kind {
            ActionKind::Communication => "reject-communication",
            ActionKind::Randomness => "reject-randomness",
            _ => "reject-scheduling",
        };
        out.push((
            name,
            with_machine(|machine| set_charge(machine, kind, Charge::Steps)),
        ));
    }
    out
}

/// Every fixture, sorted by name, its answer computed by the host.
///
/// # Panics
///
/// Panics on a request the host does not load, which no generated request
/// is.
#[must_use]
pub fn fixtures() -> Vec<Fixture> {
    let mut out: Vec<Fixture> = requests()
        .into_iter()
        .map(|(name, request)| {
            let bytes = gnaf::to_file_bytes(&request);
            let request =
                gnaf::load(&bytes).unwrap_or_else(|error| panic!("request {name}: {error:?}"));
            Fixture {
                spec: FIXTURE_SPEC.to_owned(),
                name: name.to_owned(),
                expected: gnaf::answer(&request)
                    .unwrap_or_else(|error| panic!("request {name}: {error:?}")),
                request,
            }
        })
        .collect();
    out.sort_by(|left, right| left.name.cmp(&right.name));
    out
}

/// The axioms Lean reports for a theorem that reduces the denotation.
const RUN_AXIOMS: [&str; 3] = ["Classical.choice", "Quot.sound", "propext"];

/// The declarations stating one fixture: its request, and that the model
/// answers it with the expected answer.
#[must_use]
pub fn fixture_declarations(fixture: &Fixture) -> Vec<Json> {
    let id = crate::calculus::identifier(&fixture.name);
    let request_name = format!("{id}Request");
    vec![
        json!({"kind": "definition", "name": request_name, "parameters": [],
               "result": term::named(MODEL, "Request"), "body": gnaf::request_term(&fixture.request)}),
        json!({"kind": "theorem", "name": format!("{id}Answer"), "parameters": [], "axioms": RUN_AXIOMS,
               "statement": {"kind": "eq",
                   "left": {"kind": "call", "function": term::member(MODEL, "answer"),
                            "arguments": [{"kind": "call", "function": {"name": request_name}, "arguments": []}]},
                   "right": gnaf::answer_term(&fixture.expected)},
               "proof": {"kind": "reflexivity"}}),
    ]
}

/// The `GnafFixtures` module.
///
/// # Panics
///
/// Panics only if `serde_json` cannot serialize a JSON value.
#[must_use]
pub fn fixtures_module(fixtures: &[Fixture]) -> String {
    let declarations: Vec<Json> = fixtures.iter().flat_map(fixture_declarations).collect();
    crate::lx::module_tex(
        "GnafFixtures",
        &[term::SYNTAX, term::SEMANTICS, MODEL],
        declarations,
    )
}

/// Every generated file, by path relative to the repository root.
#[must_use]
pub fn files() -> BTreeMap<String, Vec<u8>> {
    let fixtures = fixtures();
    let mut out = BTreeMap::new();
    for fixture in &fixtures {
        out.insert(
            format!("compiler/gnaf/{}.json", fixture.name),
            gnaf::to_file_bytes(fixture),
        );
    }
    out.insert(
        "compiler/src/GnafFixtures.lex.tex".to_owned(),
        fixtures_module(&fixtures).into_bytes(),
    );
    out.insert(
        crate::gnaf_model::PATH.to_owned(),
        crate::gnaf_model::module().into_bytes(),
    );
    out
}

/// The answer a fixture is expected to have, for the conformance cases that
/// read one by name.
///
/// # Panics
///
/// Panics on a name no fixture has.
#[must_use]
pub fn expected(name: &str) -> Answer {
    fixtures()
        .into_iter()
        .find(|fixture| fixture.name == name)
        .unwrap_or_else(|| panic!("no GNAF fixture {name}"))
        .expected
}
