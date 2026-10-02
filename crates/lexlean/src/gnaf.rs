//! GNAF requests over the production realization calculus (SPEC.md §17.15).
//!
//! The normative model is the LexLean module `compiler/src/Gnaf`, which Lean
//! elaborates and the kernel checks: it fixes the request components that
//! UOR-GNAF (`uor-gnaf/1-draft.2`) requires before any optimization (the
//! machine's accounting boundary, the complete-system universe and its
//! completeness evidence, the objective and its order, and the claim with
//! its scope), the fail-closed validation of a request, and the answer the
//! grammar universe has under the calculus denotation. This module is the
//! host side of that definition: the closed JSON form
//! `lexlean/gnaf-request/1`, a step-for-step transcription of validation and
//! evaluation, and the emission of a request as a LexLean term, so the
//! kernel confirms every committed request's answer against the model.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value as Json};

use crate::calculus::{
    interp, term, Expr, Function, Outcome, Prim, Program, Ty, Value, PROGRAM_SPEC,
};
use crate::code;
use crate::diagnostic::Diagnostic;
use crate::error::LexLeanError;

/// The schema tag of a request.
pub const REQUEST_SPEC: &str = "lexlean/gnaf-request/1";

/// The schema tag of a fixture.
pub const FIXTURE_SPEC: &str = "lexlean/gnaf-fixture/1";

/// The largest fuel the host evaluator admits. Fuel bounds evaluation
/// depth, and the evaluator recurses once per level, so the capacity fixes
/// the stack an evaluation can need (§17.15).
pub const MAX_FUEL: u64 = 4096;

/// The largest universe the host expands.
pub const MAX_SYSTEMS: u64 = 65_536;

/// The evaluation stack per level of fuel: about seven times what an
/// unoptimized build measures for the deepest per-level frame chain.
const STACK_PER_LEVEL: usize = 16 * 1024;

/// The module defining the model.
pub const MODEL: &str = "Gnaf";

/// A claim class of UOR-GNAF §12.4.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "class", rename_all = "snake_case", deny_unknown_fields)]
pub enum ClaimClass {
    Exact,
    NormalForm,
    Canonical,
    RepresentationMinimal,
    ComparisonTheorem,
    ProfileDefinedComparison,
    InputTotal,
    GlobalOptimal,
    ArgminComplete,
    ParetoOptimal,
    FrontierComplete,
    PointwiseEnvelopeComplete,
    QueryFamilyAnswerComplete,
    UseCaseGlobalOptimal,
    WorkloadArgminComplete,
    WorkloadParetoOptimal,
    WorkloadFrontierComplete,
    FamilyOptimal,
    CompetitiveBound,
    CompetitiveOptimal,
    AsymptoticBound,
    AsymptoticOptimal,
    UseCaseClassComplete,
    UseCaseClassAnswerComplete,
    MaintainedUseCaseClass,
    RestrictedUniverseOptimal,
    RevisionPreserved,
    BestKnown,
    MeasuredBestAmongTested,
    HeuristicSelected,
    InstanceOptimal { alpha: u64, beta: u64 },
}

impl ClaimClass {
    /// The constructor name in the LexLean model.
    #[must_use]
    pub const fn constructor(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::NormalForm => "normalForm",
            Self::Canonical => "canonical",
            Self::RepresentationMinimal => "representationMinimal",
            Self::ComparisonTheorem => "comparisonTheorem",
            Self::ProfileDefinedComparison => "profileDefinedComparison",
            Self::InputTotal => "inputTotal",
            Self::GlobalOptimal => "globalOptimal",
            Self::ArgminComplete => "argminComplete",
            Self::ParetoOptimal => "paretoOptimal",
            Self::FrontierComplete => "frontierComplete",
            Self::PointwiseEnvelopeComplete => "pointwiseEnvelopeComplete",
            Self::QueryFamilyAnswerComplete => "queryFamilyAnswerComplete",
            Self::UseCaseGlobalOptimal => "useCaseGlobalOptimal",
            Self::WorkloadArgminComplete => "workloadArgminComplete",
            Self::WorkloadParetoOptimal => "workloadParetoOptimal",
            Self::WorkloadFrontierComplete => "workloadFrontierComplete",
            Self::FamilyOptimal => "familyOptimal",
            Self::CompetitiveBound => "competitiveBound",
            Self::CompetitiveOptimal => "competitiveOptimal",
            Self::AsymptoticBound => "asymptoticBound",
            Self::AsymptoticOptimal => "asymptoticOptimal",
            Self::UseCaseClassComplete => "useCaseClassComplete",
            Self::UseCaseClassAnswerComplete => "useCaseClassAnswerComplete",
            Self::MaintainedUseCaseClass => "maintainedUseCaseClass",
            Self::RestrictedUniverseOptimal => "restrictedUniverseOptimal",
            Self::RevisionPreserved => "revisionPreserved",
            Self::BestKnown => "bestKnown",
            Self::MeasuredBestAmongTested => "measuredBestAmongTested",
            Self::HeuristicSelected => "heuristicSelected",
            Self::InstanceOptimal { .. } => "instanceOptimal",
        }
    }
}

/// A kind of action a system may take inside the machine boundary
/// (UOR-GNAF §8.2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActionKind {
    Observation,
    Preprocessing,
    Advice,
    RetainedState,
    Dispatch,
    Fallback,
    Communication,
    Randomness,
    Scheduling,
    Execution,
}

impl ActionKind {
    /// Every kind, in the model's order.
    pub const ALL: [Self; 10] = [
        Self::Observation,
        Self::Preprocessing,
        Self::Advice,
        Self::RetainedState,
        Self::Dispatch,
        Self::Fallback,
        Self::Communication,
        Self::Randomness,
        Self::Scheduling,
        Self::Execution,
    ];

    /// The constructor name in the LexLean model.
    #[must_use]
    pub const fn constructor(self) -> &'static str {
        match self {
            Self::Observation => "observation",
            Self::Preprocessing => "preprocessing",
            Self::Advice => "advice",
            Self::RetainedState => "retainedState",
            Self::Dispatch => "dispatch",
            Self::Fallback => "fallback",
            Self::Communication => "communication",
            Self::Randomness => "randomness",
            Self::Scheduling => "scheduling",
            Self::Execution => "execution",
        }
    }

    /// `Gnaf.performed`: the calculus machine itself does this work, so the
    /// step count is its only faithful charge.
    #[must_use]
    pub const fn performed(self) -> bool {
        matches!(
            self,
            Self::Observation | Self::Dispatch | Self::Fallback | Self::Execution
        )
    }

    /// `Gnaf.preparation`: work before the invocation, which the calculus
    /// has no phase for.
    #[must_use]
    pub const fn preparation(self) -> bool {
        matches!(
            self,
            Self::Preprocessing | Self::Advice | Self::RetainedState
        )
    }
}

/// How one in-boundary action is accounted (UOR-GNAF §8.2, §9.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Charge {
    /// Charged by the calculus step count.
    Steps,
    /// A fixed charge per invocation.
    Constant { cost: u64 },
    /// Explicitly declared free.
    Free,
    /// Present in the boundary with no declared accounting.
    Undeclared,
}

/// One in-boundary action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Action {
    pub kind: ActionKind,
    pub charge: Charge,
}

/// The comparison boundary (UOR-GNAF §10.8).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Boundary {
    Complete,
    PreparedState,
    PreparedPlan,
}

/// The machine contract: the calculus machine, its fuel, and its
/// accounting boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Machine {
    pub fuel: u64,
    pub actions: Vec<Action>,
    pub boundary: Boundary,
    pub preparation_common: bool,
}

/// One complete system of the grammar.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Selector {
    /// Always run one plan.
    Fixed { plan: u64 },
    /// Run `small` when the argument's length is below `threshold`, else
    /// `large`.
    Dispatch {
        threshold: u64,
        small: u64,
        large: u64,
    },
}

/// The universe carrier (UOR-GNAF §8.3).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Carrier {
    /// Every system the grammar generates over the shared plans.
    Grammar {
        argument: Ty,
        result: Ty,
        plans: Vec<Function>,
        thresholds: Vec<u64>,
    },
    /// One system's internal plans taken as the universe of systems.
    InternalPlans,
    /// The candidates an optimizer returned.
    OptimizerOutput,
    /// The candidates one search encountered.
    Discovered { members: Vec<u64> },
    /// The candidates currently cached.
    Cached,
}

/// The universe's completeness evidence (UOR-GNAF §8.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Completeness {
    GrammarEquality,
    Missing,
    CitesUniverseId,
    CitesOptimizer,
}

/// The universe a claim ranges over.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Scope {
    GrammarUniverse,
    CalculusPrograms,
    RustPrograms,
}

/// The objective: total steps, or the vector (steps, size).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Objective {
    Scalar,
    Vector,
}

/// A request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub spec: String,
    pub reference: Program,
    pub domain: Vec<Value>,
    pub machine: Machine,
    pub carrier: Carrier,
    pub completeness: Completeness,
    pub objective: Objective,
    pub claim: ClaimClass,
    pub scope: Scope,
}

/// Why a request is refused.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Rejection {
    EmptyDomain,
    InternalPlanUniverse,
    OptimizerDefinedUniverse,
    DiscoveredUniverse,
    CachedUniverse,
    MissingCompleteness,
    SelfReferentialCompleteness,
    OptimizerCompleteness,
    DuplicateAction { action: ActionKind },
    UnaccountedAction { action: ActionKind },
    HiddenCost { action: ActionKind },
    UnrealizableAction { action: ActionKind },
    UncommonPreparation,
    ScalarClaimOverPartialOrder,
    VectorClaimOverTotalOrder,
    UncoveredScope,
    UnsupportedClaim,
}

/// The answer of a request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Answer {
    Rejected {
        rejection: Rejection,
    },
    /// Every system attaining the minimum total steps, and that minimum.
    Argmin {
        members: Vec<u64>,
        steps: u64,
    },
    /// Every system no other admitted system strictly dominates.
    Frontier {
        members: Vec<u64>,
    },
    Infeasible,
    Incomplete,
}

/// A committed request with the answer the model gives it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fixture {
    pub spec: String,
    pub name: String,
    pub request: Request,
    pub expected: Answer,
}

/// A system's admission status (UOR-GNAF §8.4).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Admitted { steps: u64, size: u64 },
    Inadmissible,
    Unresolved,
}

fn malformed(reason: &str) -> LexLeanError {
    LexLeanError::from_diagnostic(Diagnostic::new(
        code!("LLB6006"),
        format!("GNAF request: {reason}"),
    ))
}

fn beyond_capacity(reason: &str) -> LexLeanError {
    LexLeanError::from_diagnostic(Diagnostic::new(
        code!("LLS8002"),
        format!("GNAF request: {reason}"),
    ))
}

/// The number of systems a carrier's grammar generates, if the host can
/// count them.
fn universe_size(carrier: &Carrier) -> Option<u64> {
    let Carrier::Grammar {
        plans, thresholds, ..
    } = carrier
    else {
        return Some(0);
    };
    let count = u64::try_from(plans.len()).ok()?;
    let pairs = count.checked_mul(count.saturating_sub(1))?;
    u64::try_from(thresholds.len())
        .ok()?
        .checked_mul(pairs)?
        .checked_add(count)
}

/// The host's capacities: the request's fuel and universe fit the
/// evaluator.
fn check_capacity(request: &Request) -> Result<(), LexLeanError> {
    if request.machine.fuel > MAX_FUEL {
        return Err(beyond_capacity(&format!(
            "fuel {} exceeds the evaluator's capacity {MAX_FUEL}",
            request.machine.fuel
        )));
    }
    match universe_size(&request.carrier) {
        Some(size) if size <= MAX_SYSTEMS => Ok(()),
        _ => Err(beyond_capacity(&format!(
            "the universe exceeds the evaluator's capacity of {MAX_SYSTEMS} systems"
        ))),
    }
}

/// Read a request and check that its programs and plans are valid calculus
/// code within the host's capacities. Validation of the GNAF components is
/// not an error: a refused request has the answer `rejected`.
///
/// # Errors
///
/// `LLB6006` for a malformed request, an invalid reference program, or a
/// grammar whose realizations are not valid programs; `LLS8002` for fuel or
/// a universe beyond [`MAX_FUEL`] or [`MAX_SYSTEMS`].
pub fn load(bytes: &[u8]) -> Result<Request, LexLeanError> {
    let request: Request =
        serde_json::from_slice(bytes).map_err(|error| malformed(&format!("malformed: {error}")))?;
    if request.spec != REQUEST_SPEC {
        return Err(malformed(&format!(
            "spec `{}`, expected `{REQUEST_SPEC}`",
            request.spec
        )));
    }
    check_capacity(&request)?;
    crate::calculus::check::check(&request.reference)
        .map_err(|reason| malformed(&format!("reference: {reason}")))?;
    // The problem is a unary function: every domain argument is an argument
    // of the reference's entry, and the grammar's systems have exactly the
    // entry's signature, so no argument can make a system stuck by type.
    for (position, argument) in request.domain.iter().enumerate() {
        crate::calculus::check::check_arguments(
            &request.reference,
            0,
            std::slice::from_ref(argument),
        )
        .map_err(|reason| malformed(&format!("domain argument {position}: {reason}")))?;
    }
    if let Carrier::Grammar {
        argument, result, ..
    } = &request.carrier
    {
        let entry = request
            .reference
            .functions
            .first()
            .ok_or_else(|| malformed("the reference has no entry function"))?;
        if entry.types != std::slice::from_ref(argument) || &entry.result != result {
            return Err(malformed(
                "the grammar's argument and result types differ from the reference entry's",
            ));
        }
        for (index, selector) in expand(&request.carrier).into_iter().enumerate() {
            let system = realize(&request.carrier, selector).expect("a grammar carrier");
            crate::calculus::check::check(&system)
                .map_err(|reason| malformed(&format!("system {index}: {reason}")))?;
        }
    }
    Ok(request)
}

fn find_action(actions: &[Action], wanted: ActionKind) -> Option<Charge> {
    actions
        .iter()
        .find(|action| action.kind == wanted)
        .map(|action| action.charge)
}

/// `Gnaf.checkDistinct`.
fn check_distinct(actions: &[Action]) -> Option<Rejection> {
    actions
        .iter()
        .find(|action| {
            actions
                .iter()
                .filter(|other| other.kind == action.kind)
                .count()
                > 1
        })
        .map(|action| Rejection::DuplicateAction {
            action: action.kind,
        })
}

/// `Gnaf.checkPerformed`.
fn check_performed(machine: &Machine, wanted: ActionKind) -> Option<Rejection> {
    match find_action(&machine.actions, wanted) {
        None => Some(Rejection::UnaccountedAction { action: wanted }),
        Some(Charge::Steps) => None,
        Some(Charge::Constant { .. } | Charge::Free | Charge::Undeclared) => {
            Some(Rejection::HiddenCost { action: wanted })
        }
    }
}

/// `Gnaf.checkDeclared`.
fn check_declared(machine: &Machine, action: &Action) -> Option<Rejection> {
    let kind = action.kind;
    if kind.performed() {
        return None;
    }
    if !kind.preparation() {
        return Some(Rejection::UnrealizableAction { action: kind });
    }
    match action.charge {
        Charge::Constant { cost } if cost > 0 => None,
        Charge::Free => match machine.boundary {
            Boundary::Complete => Some(Rejection::UncommonPreparation),
            Boundary::PreparedState | Boundary::PreparedPlan => {
                (!machine.preparation_common).then_some(Rejection::UncommonPreparation)
            }
        },
        Charge::Steps | Charge::Constant { .. } | Charge::Undeclared => {
            Some(Rejection::HiddenCost { action: kind })
        }
    }
}

/// `Gnaf.preparationCharge`: the per-invocation charge of the admitted
/// preparation actions, or `None` beyond `u64`.
#[must_use]
pub fn preparation_charge(actions: &[Action]) -> Option<u64> {
    actions
        .iter()
        .filter(|action| action.kind.preparation())
        .try_fold(0u64, |total, action| match action.charge {
            Charge::Constant { cost } => total.checked_add(cost),
            Charge::Steps | Charge::Free | Charge::Undeclared => Some(total),
        })
}

fn check_claim(request: &Request) -> Option<Rejection> {
    match request.claim {
        ClaimClass::GlobalOptimal
        | ClaimClass::ArgminComplete
        | ClaimClass::RestrictedUniverseOptimal => (request.objective == Objective::Vector)
            .then_some(Rejection::ScalarClaimOverPartialOrder),
        ClaimClass::ParetoOptimal | ClaimClass::FrontierComplete => {
            (request.objective == Objective::Scalar).then_some(Rejection::VectorClaimOverTotalOrder)
        }
        ClaimClass::Exact
        | ClaimClass::NormalForm
        | ClaimClass::Canonical
        | ClaimClass::RepresentationMinimal
        | ClaimClass::ComparisonTheorem
        | ClaimClass::ProfileDefinedComparison
        | ClaimClass::InputTotal
        | ClaimClass::PointwiseEnvelopeComplete
        | ClaimClass::QueryFamilyAnswerComplete
        | ClaimClass::UseCaseGlobalOptimal
        | ClaimClass::WorkloadArgminComplete
        | ClaimClass::WorkloadParetoOptimal
        | ClaimClass::WorkloadFrontierComplete
        | ClaimClass::FamilyOptimal
        | ClaimClass::CompetitiveBound
        | ClaimClass::CompetitiveOptimal
        | ClaimClass::AsymptoticBound
        | ClaimClass::AsymptoticOptimal
        | ClaimClass::UseCaseClassComplete
        | ClaimClass::UseCaseClassAnswerComplete
        | ClaimClass::MaintainedUseCaseClass
        | ClaimClass::RevisionPreserved
        | ClaimClass::BestKnown
        | ClaimClass::MeasuredBestAmongTested
        | ClaimClass::HeuristicSelected
        | ClaimClass::InstanceOptimal { .. } => Some(Rejection::UnsupportedClaim),
    }
}

/// `Gnaf.validate`: the first violated rule, in the model's order.
#[must_use]
pub fn validate(request: &Request) -> Option<Rejection> {
    if request.domain.is_empty() {
        return Some(Rejection::EmptyDomain);
    }
    match &request.carrier {
        Carrier::Grammar { .. } => {}
        Carrier::InternalPlans => return Some(Rejection::InternalPlanUniverse),
        Carrier::OptimizerOutput => return Some(Rejection::OptimizerDefinedUniverse),
        Carrier::Discovered { .. } => return Some(Rejection::DiscoveredUniverse),
        Carrier::Cached => return Some(Rejection::CachedUniverse),
    }
    match request.completeness {
        Completeness::GrammarEquality => {}
        Completeness::Missing => return Some(Rejection::MissingCompleteness),
        Completeness::CitesUniverseId => return Some(Rejection::SelfReferentialCompleteness),
        Completeness::CitesOptimizer => return Some(Rejection::OptimizerCompleteness),
    }
    if let Some(rejection) = check_distinct(&request.machine.actions) {
        return Some(rejection);
    }
    let performed = ActionKind::ALL.into_iter().filter(|kind| kind.performed());
    if let Some(rejection) = performed
        .filter_map(|kind| check_performed(&request.machine, kind))
        .next()
    {
        return Some(rejection);
    }
    if let Some(rejection) = request
        .machine
        .actions
        .iter()
        .find_map(|action| check_declared(&request.machine, action))
    {
        return Some(rejection);
    }
    if let Some(rejection) = check_claim(request) {
        return Some(rejection);
    }
    match request.scope {
        Scope::GrammarUniverse => None,
        Scope::CalculusPrograms | Scope::RustPrograms => Some(Rejection::UncoveredScope),
    }
}

/// `Gnaf.expand`: every fixed plan, then every dispatch at every threshold
/// between two distinct plans, in that order. Empty for a carrier that is
/// not a grammar.
#[must_use]
pub fn expand(carrier: &Carrier) -> Vec<Selector> {
    let Carrier::Grammar {
        plans, thresholds, ..
    } = carrier
    else {
        return Vec::new();
    };
    let count = plans.len() as u64;
    let mut out: Vec<Selector> = (0..count).map(|plan| Selector::Fixed { plan }).collect();
    for threshold in thresholds {
        for small in 0..count {
            for large in (0..count).filter(|large| *large != small) {
                out.push(Selector::Dispatch {
                    threshold: *threshold,
                    small,
                    large,
                });
            }
        }
    }
    out
}

/// `Gnaf.entryBody`.
#[must_use]
pub fn entry_body(selector: Selector) -> Expr {
    let argument = || Expr::Var { name: 0 };
    let run = |plan: u64| Expr::Call {
        function: plan + 1,
        operands: vec![argument()],
    };
    match selector {
        Selector::Fixed { plan } => run(plan),
        Selector::Dispatch {
            threshold,
            small,
            large,
        } => Expr::Cond {
            condition: Box::new(Expr::Prim {
                operation: Prim::NatLt,
                operands: vec![
                    Expr::Prim {
                        operation: Prim::Length,
                        operands: vec![argument()],
                    },
                    Expr::Value {
                        ty: Ty::Nat,
                        value: Value::Nat {
                            value: threshold.to_string(),
                        },
                    },
                ],
            }),
            then_branch: Box::new(run(small)),
            else_branch: Box::new(run(large)),
        },
    }
}

/// `Gnaf.realize`: the selector over the shared plans at indices 1...
#[must_use]
pub fn realize(carrier: &Carrier, selector: Selector) -> Option<Program> {
    let Carrier::Grammar {
        argument,
        result,
        plans,
        ..
    } = carrier
    else {
        return None;
    };
    let mut functions = vec![Function {
        parameters: vec![0],
        types: vec![argument.clone()],
        result: result.clone(),
        body: entry_body(selector),
    }];
    functions.extend(plans.iter().cloned());
    Some(Program {
        spec: PROGRAM_SPEC.to_owned(),
        adts: Vec::new(),
        functions,
    })
}

fn expr_size(expr: &Expr) -> u64 {
    let all = |exprs: &[Expr]| exprs.iter().map(expr_size).sum::<u64>();
    1 + match expr {
        Expr::Value { .. } | Expr::Var { .. } => 0,
        Expr::Let { bound, body, .. } => expr_size(bound) + expr_size(body),
        Expr::Cond {
            condition,
            then_branch,
            else_branch,
        } => expr_size(condition) + expr_size(then_branch) + expr_size(else_branch),
        Expr::Match {
            scrutinee, arms, ..
        } => expr_size(scrutinee) + arms.iter().map(|arm| 1 + expr_size(&arm.body)).sum::<u64>(),
        Expr::Build { operands, .. }
        | Expr::Call { operands, .. }
        | Expr::Closure {
            captures: operands, ..
        }
        | Expr::Prim { operands, .. } => all(operands),
        Expr::Apply { target, operands } => expr_size(target) + all(operands),
        Expr::First { value } | Expr::Second { value } | Expr::Field { value, .. } => {
            expr_size(value)
        }
    }
}

/// `Gnaf.functionsSize`: one per expression node and per arm.
#[must_use]
pub fn program_size(program: &Program) -> u64 {
    program
        .functions
        .iter()
        .map(|function| expr_size(&function.body))
        .sum()
}

/// `Gnaf.statusOn` with `Gnaf.status`'s size: the first invocation, in domain order, that does not
/// produce the reference value decides; an exhausted evaluation leaves
/// admission unresolved.
#[must_use]
fn status(system: &Program, reference: &Program, fuel: u64, domain: &[Value]) -> Status {
    let mut steps = 0u64;
    for argument in domain {
        let arguments = std::slice::from_ref(argument);
        let produced = match interp::run(system, fuel, 0, arguments) {
            Outcome::Value { value, steps } => (value, steps),
            Outcome::Exhausted => return Status::Unresolved,
            Outcome::Overflow { .. } | Outcome::Stuck => return Status::Inadmissible,
        };
        let Outcome::Value {
            value: expected, ..
        } = interp::run(reference, fuel, 0, arguments)
        else {
            return Status::Unresolved;
        };
        if produced.0 != expected {
            return Status::Inadmissible;
        }
        // A cost this host cannot represent is unknown to it, and an unknown
        // cost leaves the answer incomplete rather than removing the system.
        let Some(total) = steps.checked_add(produced.1) else {
            return Status::Unresolved;
        };
        steps = total;
    }
    Status::Admitted {
        steps,
        size: program_size(system),
    }
}

/// Run an evaluation of `request` on a thread whose stack its fuel cannot
/// exhaust.
fn on_evaluation_stack<T: Send>(
    request: &Request,
    evaluation: impl FnOnce(&Request) -> T + Send,
) -> Result<T, LexLeanError> {
    check_capacity(request)?;
    let levels = usize::try_from(request.machine.fuel).unwrap_or(usize::MAX);
    let stack = levels.saturating_add(64).saturating_mul(STACK_PER_LEVEL);
    let platform = |reason: String| {
        LexLeanError::from_diagnostic(Diagnostic::new(
            code!("LLV7010"),
            format!("GNAF evaluation thread: {reason}"),
        ))
    };
    std::thread::scope(|scope| {
        std::thread::Builder::new()
            .name("gnaf-evaluation".to_owned())
            .stack_size(stack)
            .spawn_scoped(scope, || evaluation(request))
            .map_err(|error| platform(error.to_string()))?
            .join()
            .map_err(|_| platform("the evaluation did not complete".to_owned()))
    })
}

/// `Gnaf.statuses`: every system of a grammar request with its status,
/// numbered in expansion order, each admitted system charged its
/// preparation per invocation.
///
/// # Errors
///
/// `LLS8002` for a request beyond the host's capacities, `LLV7010` when the
/// platform cannot provide the evaluation thread.
pub fn statuses(request: &Request) -> Result<Vec<(u64, Selector, Status)>, LexLeanError> {
    on_evaluation_stack(request, evaluate_statuses)
}

fn evaluate_statuses(request: &Request) -> Vec<(u64, Selector, Status)> {
    expand(&request.carrier)
        .into_iter()
        .enumerate()
        .map(|(index, selector)| {
            let system = realize(&request.carrier, selector).expect("a grammar carrier");
            let charged = match status(
                &system,
                &request.reference,
                request.machine.fuel,
                &request.domain,
            ) {
                Status::Admitted { steps, size } => preparation_charge(&request.machine.actions)
                    .and_then(|charge| charge.checked_mul(request.domain.len() as u64))
                    .and_then(|charged| charged.checked_add(steps))
                    .map_or(Status::Unresolved, |steps| Status::Admitted { steps, size }),
                other => other,
            };
            (index as u64, selector, charged)
        })
        .collect()
}

fn dominates(left: (u64, u64), right: (u64, u64)) -> bool {
    left.0 <= right.0 && left.1 <= right.1 && (left.0 < right.0 || left.1 < right.1)
}

/// `Gnaf.answer`.
///
/// # Errors
///
/// `LLS8002` for a request beyond the host's capacities, `LLV7010` when the
/// platform cannot provide the evaluation thread.
pub fn answer(request: &Request) -> Result<Answer, LexLeanError> {
    on_evaluation_stack(request, evaluate_answer)
}

fn evaluate_answer(request: &Request) -> Answer {
    if let Some(rejection) = validate(request) {
        return Answer::Rejected { rejection };
    }
    let entries = evaluate_statuses(request);
    if entries
        .iter()
        .any(|(_, _, status)| *status == Status::Unresolved)
    {
        return Answer::Incomplete;
    }
    let costed: Vec<(u64, (u64, u64))> = entries
        .iter()
        .filter_map(|(index, _, status)| match status {
            Status::Admitted { steps, size } => Some((*index, (*steps, *size))),
            Status::Inadmissible | Status::Unresolved => None,
        })
        .collect();
    let Some(best) = costed.iter().map(|(_, (steps, _))| *steps).min() else {
        return Answer::Infeasible;
    };
    match request.objective {
        Objective::Scalar => Answer::Argmin {
            members: costed
                .iter()
                .filter(|(_, (steps, _))| *steps == best)
                .map(|(index, _)| *index)
                .collect(),
            steps: best,
        },
        Objective::Vector => Answer::Frontier {
            members: costed
                .iter()
                .filter(|(_, cost)| !costed.iter().any(|(_, other)| dominates(*other, *cost)))
                .map(|(index, _)| *index)
                .collect(),
        },
    }
}

// --- LexLean terms -------------------------------------------------------------

fn model(name: &str, arguments: Vec<Json>) -> Json {
    json!({"kind": "constructor", "constructor": term::member(MODEL, name), "arguments": arguments})
}

fn boolean(value: bool) -> Json {
    json!({"kind": "bool", "value": value})
}

fn record(name: &str, fields: Vec<(&str, Json)>) -> Json {
    let fields: Vec<Json> = fields
        .into_iter()
        .map(|(field, value)| json!({"field": field, "value": value}))
        .collect();
    json!({"kind": "record", "type": term::member(MODEL, name), "fields": fields})
}

fn action_kind(kind: ActionKind) -> Json {
    model(&format!("ActionKind.{}", kind.constructor()), Vec::new())
}

fn charge(charge: &Charge) -> Json {
    match charge {
        Charge::Steps => model("Charge.steps", Vec::new()),
        Charge::Constant { cost } => model("Charge.constant", vec![term::nat(*cost)]),
        Charge::Free => model("Charge.free", Vec::new()),
        Charge::Undeclared => model("Charge.undeclared", Vec::new()),
    }
}

fn machine(machine: &Machine) -> Json {
    let boundary = match machine.boundary {
        Boundary::Complete => "Boundary.complete",
        Boundary::PreparedState => "Boundary.preparedState",
        Boundary::PreparedPlan => "Boundary.preparedPlan",
    };
    let actions = machine
        .actions
        .iter()
        .map(|action| {
            record(
                "Action",
                vec![
                    ("kind", action_kind(action.kind)),
                    ("charge", charge(&action.charge)),
                ],
            )
        })
        .collect();
    record(
        "Machine",
        vec![
            ("fuel", term::nat(machine.fuel)),
            (
                "actions",
                term::list(&term::named(MODEL, "Action"), actions),
            ),
            ("boundary", model(boundary, Vec::new())),
            ("preparationCommon", boolean(machine.preparation_common)),
        ],
    )
}

fn carrier(carrier: &Carrier) -> Json {
    let nat_type = json!({"kind": "nat"});
    match carrier {
        Carrier::Grammar {
            argument,
            result,
            plans,
            thresholds,
        } => model(
            "Carrier.grammar",
            vec![record(
                "Grammar",
                vec![
                    ("argument", term::ty(argument)),
                    ("result", term::ty(result)),
                    (
                        "plans",
                        term::list(
                            &term::named(term::SYNTAX, "Function"),
                            plans.iter().map(term::function).collect(),
                        ),
                    ),
                    (
                        "thresholds",
                        term::list(
                            &nat_type,
                            thresholds
                                .iter()
                                .map(|threshold| term::nat(*threshold))
                                .collect(),
                        ),
                    ),
                ],
            )],
        ),
        Carrier::InternalPlans => model("Carrier.internalPlans", Vec::new()),
        Carrier::OptimizerOutput => model("Carrier.optimizerOutput", Vec::new()),
        Carrier::Discovered { members } => model(
            "Carrier.discovered",
            vec![term::list(
                &nat_type,
                members.iter().map(|member| term::nat(*member)).collect(),
            )],
        ),
        Carrier::Cached => model("Carrier.cached", Vec::new()),
    }
}

fn claim(claim: ClaimClass) -> Json {
    let name = format!("ClaimClass.{}", claim.constructor());
    match claim {
        ClaimClass::InstanceOptimal { alpha, beta } => {
            model(&name, vec![term::nat(alpha), term::nat(beta)])
        }
        _ => model(&name, Vec::new()),
    }
}

/// A request as a `Gnaf.Request` term.
#[must_use]
pub fn request_term(request: &Request) -> Json {
    let completeness = match request.completeness {
        Completeness::GrammarEquality => "Completeness.grammarEquality",
        Completeness::Missing => "Completeness.missing",
        Completeness::CitesUniverseId => "Completeness.citesUniverseId",
        Completeness::CitesOptimizer => "Completeness.citesOptimizer",
    };
    let objective = match request.objective {
        Objective::Scalar => "Objective.scalar",
        Objective::Vector => "Objective.vector",
    };
    let scope = match request.scope {
        Scope::GrammarUniverse => "Scope.grammarUniverse",
        Scope::CalculusPrograms => "Scope.calculusPrograms",
        Scope::RustPrograms => "Scope.rustPrograms",
    };
    record(
        "Request",
        vec![
            ("reference", term::program(&request.reference)),
            (
                "domain",
                term::list(
                    &term::named(term::SYNTAX, "Value"),
                    request.domain.iter().map(term::value).collect(),
                ),
            ),
            ("machine", machine(&request.machine)),
            ("carrier", carrier(&request.carrier)),
            ("completeness", model(completeness, Vec::new())),
            ("objective", model(objective, Vec::new())),
            ("claim", claim(request.claim)),
            ("scope", model(scope, Vec::new())),
        ],
    )
}

fn rejection_term(rejection: &Rejection) -> Json {
    let simple = |name: &str| model(&format!("Rejection.{name}"), Vec::new());
    match rejection {
        Rejection::EmptyDomain => simple("emptyDomain"),
        Rejection::InternalPlanUniverse => simple("internalPlanUniverse"),
        Rejection::OptimizerDefinedUniverse => simple("optimizerDefinedUniverse"),
        Rejection::DiscoveredUniverse => simple("discoveredUniverse"),
        Rejection::CachedUniverse => simple("cachedUniverse"),
        Rejection::MissingCompleteness => simple("missingCompleteness"),
        Rejection::SelfReferentialCompleteness => simple("selfReferentialCompleteness"),
        Rejection::OptimizerCompleteness => simple("optimizerCompleteness"),
        Rejection::DuplicateAction { action } => {
            model("Rejection.duplicateAction", vec![action_kind(*action)])
        }
        Rejection::UnaccountedAction { action } => {
            model("Rejection.unaccountedAction", vec![action_kind(*action)])
        }
        Rejection::HiddenCost { action } => {
            model("Rejection.hiddenCost", vec![action_kind(*action)])
        }
        Rejection::UnrealizableAction { action } => {
            model("Rejection.unrealizableAction", vec![action_kind(*action)])
        }
        Rejection::UncommonPreparation => simple("uncommonPreparation"),
        Rejection::ScalarClaimOverPartialOrder => simple("scalarClaimOverPartialOrder"),
        Rejection::VectorClaimOverTotalOrder => simple("vectorClaimOverTotalOrder"),
        Rejection::UncoveredScope => simple("uncoveredScope"),
        Rejection::UnsupportedClaim => simple("unsupportedClaim"),
    }
}

/// An answer as a `Gnaf.Answer` term.
#[must_use]
pub fn answer_term(answer: &Answer) -> Json {
    let nat_type = json!({"kind": "nat"});
    let ids = |members: &[u64]| {
        term::list(
            &nat_type,
            members.iter().map(|member| term::nat(*member)).collect(),
        )
    };
    match answer {
        Answer::Rejected { rejection } => model("Answer.rejected", vec![rejection_term(rejection)]),
        Answer::Argmin { members, steps } => {
            model("Answer.argmin", vec![ids(members), term::nat(*steps)])
        }
        Answer::Frontier { members } => model("Answer.frontier", vec![ids(members)]),
        Answer::Infeasible => model("Answer.infeasible", Vec::new()),
        Answer::Incomplete => model("Answer.incomplete", Vec::new()),
    }
}

/// The canonical file bytes of a request.
///
/// # Panics
///
/// Panics only if `serde_json` produces text the canonical JSON parser
/// rejects, which would be an internal invariant failure.
#[must_use]
pub fn to_file_bytes<T: Serialize>(value: &T) -> Vec<u8> {
    let text = serde_json::to_string(value).expect("serializes");
    crate::artifact::canonical_json::Json::parse(text.as_bytes())
        .expect("is JSON")
        .to_file_bytes()
}
