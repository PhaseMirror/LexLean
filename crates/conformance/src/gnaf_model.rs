//! The `Gnaf` module of the `compiler` project: the GNAF request model of
//! SPEC.md §17.15, written here as LexLean semantic terms.
//!
//! The committed `compiler/src/Gnaf.lex.tex` is exactly what [`module`]
//! renders, which `cargo xtask check-calculus` enforces, so the model a
//! reviewer reads here is the model Lean checks. The host transcription
//! `lexlean::gnaf` and the `GnafFixtures` theorems are both measured
//! against it.
//!
//! Binders a branch never reads are named `ignored<n>` (and the binders of
//! `first_rejection` `rejection<n>`) from one counter, in the order the
//! terms are built. The committed names depend on that order: declarations
//! are built in module order, and within one, Rust evaluates arguments left
//! to right, inner calls before the call that receives them. Reordering the
//! construction of two terms that both draw a name renames binders, which
//! the byte comparison reports.

use std::cell::Cell;

use lexlean::calculus::term::{self, SEMANTICS, SYNTAX};
use lexlean::gnaf::MODEL;
use serde_json::Value as Json;

use crate::lx::{
    self, add, and, axioms, beq, ble, blt, boolean, call, cons, decide, definition, eq, first,
    inductive, ite, let_in, list, list_t, local_t, mutual, nat, nat_t, nil, none, not, option_t,
    or, pair, parameter, prim, product_t, project, recursive, second, some, structure, theorem,
    var,
};

/// The path of the generated module, relative to the repository root.
pub const PATH: &str = "compiler/src/Gnaf.lex.tex";

/// The axioms Lean reports for a declaration that reaches
/// `TargetSemantics.run`: the evaluator's well-founded and quotient
/// machinery. Exactly the declarations that evaluate a realization carry
/// them; the authority vectors are decided over plain tables and carry
/// none.
const RUN_AXIOMS: [&str; 3] = ["Classical.choice", "Quot.sound", "propext"];

/// Names for binders the branch body never reads; see the module comment.
struct Unused {
    count: Cell<u32>,
}

impl Unused {
    fn new() -> Self {
        Self {
            count: Cell::new(0),
        }
    }
    fn named(&self, base: &str) -> String {
        self.count.set(self.count.get() + 1);
        format!("{base}{}", self.count.get())
    }
    fn next(&self) -> String {
        self.named("ignored")
    }
}

/// A binder list mixing fixed names with names drawn from [`Unused`],
/// evaluated left to right.
macro_rules! binders {
    ($($binder:expr),* $(,)?) => {
        vec![$(($binder).to_string()),*]
    };
}

// --- vocabulary ------------------------------------------------------------

/// §12.4 claim classes without constants; `instanceOptimal`, which carries
/// two, closes the inductive.
const CLAIMS: [&str; 30] = [
    "exact",
    "normalForm",
    "canonical",
    "representationMinimal",
    "comparisonTheorem",
    "profileDefinedComparison",
    "inputTotal",
    "globalOptimal",
    "argminComplete",
    "paretoOptimal",
    "frontierComplete",
    "pointwiseEnvelopeComplete",
    "queryFamilyAnswerComplete",
    "useCaseGlobalOptimal",
    "workloadArgminComplete",
    "workloadParetoOptimal",
    "workloadFrontierComplete",
    "familyOptimal",
    "competitiveBound",
    "competitiveOptimal",
    "asymptoticBound",
    "asymptoticOptimal",
    "useCaseClassComplete",
    "useCaseClassAnswerComplete",
    "maintainedUseCaseClass",
    "restrictedUniverseOptimal",
    "revisionPreserved",
    "bestKnown",
    "measuredBestAmongTested",
    "heuristicSelected",
];
/// The claims the scalar (total) order answers.
const SCALAR_CLAIMS: [&str; 3] = [
    "globalOptimal",
    "argminComplete",
    "restrictedUniverseOptimal",
];
/// The claims the componentwise (partial) order answers.
const VECTOR_CLAIMS: [&str; 2] = ["paretoOptimal", "frontierComplete"];

/// §8.2, §9.1: the kinds of action a system may take inside the machine
/// boundary.
const ACTION_KINDS: [&str; 10] = [
    "observation",
    "preprocessing",
    "advice",
    "retainedState",
    "dispatch",
    "fallback",
    "communication",
    "randomness",
    "scheduling",
    "execution",
];
/// Actions the calculus machine itself performs.
const PERFORMED: [&str; 4] = ["observation", "dispatch", "fallback", "execution"];
/// Actions before the invocation.
const PREPARATION: [&str; 3] = ["preprocessing", "advice", "retainedState"];

/// The constructors of `TargetSyntax.Value`, in declaration order, with
/// their field counts.
const VALUE_CONSTRUCTORS: [(&str, usize); 23] = [
    ("unit", 0),
    ("bool", 1),
    ("nat", 1),
    ("int", 1),
    ("u8", 1),
    ("u16", 1),
    ("u32", 1),
    ("u64", 1),
    ("i8", 1),
    ("i16", 1),
    ("i32", 1),
    ("i64", 1),
    ("string", 1),
    ("bytes", 1),
    ("ordering", 1),
    ("none", 0),
    ("some", 1),
    ("ok", 1),
    ("error", 1),
    ("list", 1),
    ("pair", 2),
    ("adt", 2),
    ("closure", 2),
];

// --- terms of this model ---------------------------------------------------

fn syntax_t(name: &str) -> Json {
    term::named(SYNTAX, name)
}
fn value_t() -> Json {
    syntax_t("Value")
}
fn expr_t() -> Json {
    syntax_t("Expr")
}
fn rejection_t() -> Json {
    local_t("Rejection")
}
fn kind_t() -> Json {
    local_t("ActionKind")
}
fn selector_t() -> Json {
    local_t("Selector")
}
fn status_t() -> Json {
    local_t("Status")
}

/// A constructor of this module.
fn local(constructor: &str, arguments: Vec<Json>) -> Json {
    lx::construct(lx::member(constructor), arguments)
}
/// A constructor of `TargetSyntax`.
fn syntax(constructor: &str, arguments: Vec<Json>) -> Json {
    lx::construct(term::member(SYNTAX, constructor), arguments)
}
/// A call of a definition of this module.
fn local_call(function: &str, arguments: Vec<Json>) -> Json {
    call(lx::member(function), arguments)
}
/// A call of a definition of `TargetSemantics`.
fn semantics_call(function: &str, arguments: Vec<Json>) -> Json {
    call(term::member(SEMANTICS, function), arguments)
}

/// A branch on a constructor of this module or a builtin.
fn arm(constructor: &str, binders: Vec<String>, body: Json) -> Json {
    lx::branch(lx::member(constructor), binders, body)
}
fn syntax_arm(constructor: &str, binders: Vec<String>, body: Json) -> Json {
    lx::branch(term::member(SYNTAX, constructor), binders, body)
}
fn semantics_arm(constructor: &str, binders: Vec<String>, body: Json) -> Json {
    lx::branch(term::member(SEMANTICS, constructor), binders, body)
}
fn matching(scrutinee: Json, branches: Vec<Json>) -> Json {
    lx::matching(scrutinee, branches)
}

fn on_list(
    scrutinee: Json,
    empty: Json,
    head: impl Into<String>,
    tail: impl Into<String>,
    nonempty: Json,
) -> Json {
    matching(
        scrutinee,
        vec![
            arm("List.nil", Vec::new(), empty),
            arm("List.cons", vec![head.into(), tail.into()], nonempty),
        ],
    )
}
fn on_option(scrutinee: Json, absent: Json, found: impl Into<String>, present: Json) -> Json {
    matching(
        scrutinee,
        vec![
            arm("Option.none", Vec::new(), absent),
            arm("Option.some", vec![found.into()], present),
        ],
    )
}
/// A match on `nat` by `zero` and `succ remaining`.
fn on_nat(scrutinee: Json, zero: Json, successor: Json) -> Json {
    matching(
        scrutinee,
        vec![
            arm("Nat.zero", Vec::new(), zero),
            arm("Nat.succ", binders!["remaining"], successor),
        ],
    )
}
/// A total match on a `TargetSyntax.Value`: the listed constructors take
/// their bodies, every other one `default`.
fn on_value(
    unused: &Unused,
    scrutinee: Json,
    cases: Vec<(&str, Vec<String>, Json)>,
    default: &Json,
) -> Json {
    let branches = VALUE_CONSTRUCTORS
        .iter()
        .map(|(constructor, fields)| {
            let name = format!("Value.{constructor}");
            match cases.iter().find(|(case, _, _)| case == constructor) {
                Some((_, binders, body)) => syntax_arm(&name, binders.clone(), body.clone()),
                None => syntax_arm(
                    &name,
                    (0..*fields).map(|_| unused.next()).collect(),
                    default.clone(),
                ),
            }
        })
        .collect();
    matching(scrutinee, branches)
}

fn yes() -> Json {
    boolean(true)
}
fn no() -> Json {
    boolean(false)
}

/// `none`: the request passes this check.
fn pass() -> Json {
    none(rejection_t())
}
/// `some` rejection `constructor arguments`.
fn reject(constructor: &str, arguments: Vec<Json>) -> Json {
    some(
        rejection_t(),
        local(&format!("Rejection.{constructor}"), arguments),
    )
}
/// The first rejection among `checks`, each an `Option Rejection`, or
/// `none`. Written as nested matches rather than a fold so every binder is
/// named in the generated Lean.
fn first_rejection(unused: &Unused, checks: Vec<Json>) -> Json {
    let mut checks = checks.into_iter();
    let Some(head) = checks.next() else {
        return pass();
    };
    let found = unused.named("rejection");
    let rest = first_rejection(unused, checks.collect());
    on_option(head, rest, found.clone(), some(rejection_t(), var(&found)))
}

fn status(constructor: &str, arguments: Vec<Json>) -> Json {
    local(&format!("Status.{constructor}"), arguments)
}

// --- the request -----------------------------------------------------------

/// Constructors without fields.
fn none_of(names: &[&'static str]) -> Vec<(&'static str, Vec<Json>)> {
    names.iter().map(|name| (*name, Vec::new())).collect()
}

fn request_types() -> Vec<Json> {
    let mut claims = none_of(&CLAIMS);
    claims.push(("instanceOptimal", vec![nat_t(), nat_t()]));
    vec![
        inductive("ClaimClass", claims),
        // §8.2, §9.1: how each admitted action is accounted.
        inductive("ActionKind", none_of(&ACTION_KINDS)),
        inductive(
            "Charge",
            vec![
                ("steps", Vec::new()),
                ("constant", vec![nat_t()]),
                ("free", Vec::new()),
                ("undeclared", Vec::new()),
            ],
        ),
        structure(
            "Action",
            vec![("kind", kind_t()), ("charge", local_t("Charge"))],
        ),
        // §10.8: the comparison boundary.
        inductive(
            "Boundary",
            none_of(&["complete", "preparedState", "preparedPlan"]),
        ),
        structure(
            "Machine",
            vec![
                ("fuel", nat_t()),
                ("actions", list_t(local_t("Action"))),
                ("boundary", local_t("Boundary")),
                ("preparationCommon", lx::bool_t()),
            ],
        ),
        // The complete-system grammar: every system is a selector over the
        // shared plan functions, which sit at indices 1.. of every
        // realization.
        inductive(
            "Selector",
            vec![
                ("fixed", vec![nat_t()]),
                ("dispatch", vec![nat_t(), nat_t(), nat_t()]),
            ],
        ),
        structure(
            "Grammar",
            vec![
                ("argument", syntax_t("Ty")),
                ("result", syntax_t("Ty")),
                ("plans", list_t(syntax_t("Function"))),
                ("thresholds", list_t(nat_t())),
            ],
        ),
        // §8.3: the universe carrier and its completeness evidence.
        inductive(
            "Carrier",
            vec![
                ("grammar", vec![local_t("Grammar")]),
                ("internalPlans", Vec::new()),
                ("optimizerOutput", Vec::new()),
                ("discovered", vec![list_t(nat_t())]),
                ("cached", Vec::new()),
            ],
        ),
        inductive(
            "Completeness",
            none_of(&[
                "grammarEquality",
                "missing",
                "citesUniverseId",
                "citesOptimizer",
            ]),
        ),
        inductive(
            "Scope",
            none_of(&["grammarUniverse", "calculusPrograms", "rustPrograms"]),
        ),
        // §9.2, §9.3: a scalar objective is total steps; a vector objective
        // is (steps, size), ordered componentwise.
        inductive("Objective", none_of(&["scalar", "vector"])),
        structure(
            "Request",
            vec![
                ("reference", syntax_t("Program")),
                ("domain", list_t(value_t())),
                ("machine", local_t("Machine")),
                ("carrier", local_t("Carrier")),
                ("completeness", local_t("Completeness")),
                ("objective", local_t("Objective")),
                ("claim", local_t("ClaimClass")),
                ("scope", local_t("Scope")),
            ],
        ),
        inductive(
            "Rejection",
            vec![
                ("emptyDomain", Vec::new()),
                ("internalPlanUniverse", Vec::new()),
                ("optimizerDefinedUniverse", Vec::new()),
                ("discoveredUniverse", Vec::new()),
                ("cachedUniverse", Vec::new()),
                ("missingCompleteness", Vec::new()),
                ("selfReferentialCompleteness", Vec::new()),
                ("optimizerCompleteness", Vec::new()),
                ("duplicateAction", vec![kind_t()]),
                ("unaccountedAction", vec![kind_t()]),
                ("hiddenCost", vec![kind_t()]),
                ("unrealizableAction", vec![kind_t()]),
                ("uncommonPreparation", Vec::new()),
                ("scalarClaimOverPartialOrder", Vec::new()),
                ("vectorClaimOverTotalOrder", Vec::new()),
                ("uncoveredScope", Vec::new()),
                ("unsupportedClaim", Vec::new()),
            ],
        ),
        inductive(
            "Status",
            vec![
                ("admitted", vec![nat_t(), nat_t()]),
                ("inadmissible", Vec::new()),
                ("unresolved", Vec::new()),
            ],
        ),
        inductive(
            "Answer",
            vec![
                ("rejected", vec![rejection_t()]),
                ("argmin", vec![list_t(nat_t()), nat_t()]),
                ("frontier", vec![list_t(nat_t())]),
                ("infeasible", Vec::new()),
                ("incomplete", Vec::new()),
            ],
        ),
    ]
}

// --- the machine contract --------------------------------------------------

/// A match on `kind` with one branch per action kind.
fn on_kind(body: impl Fn(usize, &str) -> Json) -> Json {
    matching(
        var("kind"),
        ACTION_KINDS
            .iter()
            .enumerate()
            .map(|(index, kind)| arm(&format!("ActionKind.{kind}"), Vec::new(), body(index, kind)))
            .collect(),
    )
}
/// Whether `kind` is one of `kinds`.
fn kind_in(kinds: &[&str]) -> Json {
    on_kind(|_, kind| boolean(kinds.contains(&kind)))
}
/// A free preparation action is admitted only as state every competitor
/// shares at a prepared boundary (§10.8).
fn prepared_common() -> Json {
    let common = || {
        ite(
            project(var("machine"), "preparationCommon"),
            pass(),
            reject("uncommonPreparation", Vec::new()),
        )
    };
    matching(
        project(var("machine"), "boundary"),
        vec![
            arm(
                "Boundary.complete",
                Vec::new(),
                reject("uncommonPreparation", Vec::new()),
            ),
            arm("Boundary.preparedState", Vec::new(), common()),
            arm("Boundary.preparedPlan", Vec::new(), common()),
        ],
    )
}
fn hidden_cost() -> Json {
    reject("hiddenCost", vec![var("kind")])
}

fn machine_contract(unused: &Unused) -> Vec<Json> {
    let actions_t = || list_t(local_t("Action"));
    let action_kind = || project(var("action"), "kind");
    let same_kind_as_wanted = || local_call("sameKind", vec![action_kind(), var("wanted")]);
    vec![
        definition(
            "kindIndex",
            vec![parameter("kind", kind_t())],
            nat_t(),
            on_kind(|index, _| nat(index as u64)),
        ),
        definition(
            "sameKind",
            vec![parameter("left", kind_t()), parameter("right", kind_t())],
            lx::bool_t(),
            beq(
                local_call("kindIndex", vec![var("left")]),
                local_call("kindIndex", vec![var("right")]),
            ),
        ),
        // Actions the calculus machine itself performs: their work is
        // calculus steps, so the step count is their only faithful charge.
        definition(
            "performed",
            vec![parameter("kind", kind_t())],
            lx::bool_t(),
            kind_in(&PERFORMED),
        ),
        // Actions before the invocation; the calculus has no such phase, so
        // their cost is a declared constant or a common prepared boundary.
        definition(
            "preparation",
            vec![parameter("kind", kind_t())],
            lx::bool_t(),
            kind_in(&PREPARATION),
        ),
        recursive(
            "actions",
            definition(
                "findAction",
                vec![
                    parameter("actions", actions_t()),
                    parameter("wanted", kind_t()),
                ],
                option_t(local_t("Charge")),
                on_list(
                    var("actions"),
                    none(local_t("Charge")),
                    "action",
                    "rest",
                    ite(
                        same_kind_as_wanted(),
                        some(local_t("Charge"), project(var("action"), "charge")),
                        local_call("findAction", vec![var("rest"), var("wanted")]),
                    ),
                ),
            ),
        ),
        recursive(
            "actions",
            definition(
                "countKind",
                vec![
                    parameter("actions", actions_t()),
                    parameter("wanted", kind_t()),
                ],
                nat_t(),
                on_list(
                    var("actions"),
                    nat(0),
                    "action",
                    "rest",
                    add(
                        ite(same_kind_as_wanted(), nat(1), nat(0)),
                        local_call("countKind", vec![var("rest"), var("wanted")]),
                    ),
                ),
            ),
        ),
        // Each kind is accounted at most once: two charges for one kind
        // leave its cost ambiguous.
        recursive(
            "actions",
            definition(
                "checkDistinct",
                vec![
                    parameter("actions", actions_t()),
                    parameter("all", actions_t()),
                ],
                option_t(rejection_t()),
                on_list(
                    var("actions"),
                    pass(),
                    "action",
                    "rest",
                    ite(
                        blt(
                            nat(1),
                            local_call("countKind", vec![var("all"), action_kind()]),
                        ),
                        reject("duplicateAction", vec![action_kind()]),
                        local_call("checkDistinct", vec![var("rest"), var("all")]),
                    ),
                ),
            ),
        ),
        // Every action some system of the universe performs is declared and
        // charged by steps; a constant, free, or undeclared charge hides or
        // replaces the work the calculus actually counts.
        definition(
            "checkPerformed",
            vec![
                parameter("machine", local_t("Machine")),
                parameter("wanted", kind_t()),
            ],
            option_t(rejection_t()),
            on_option(
                local_call(
                    "findAction",
                    vec![project(var("machine"), "actions"), var("wanted")],
                ),
                reject("unaccountedAction", vec![var("wanted")]),
                "charge",
                matching(
                    var("charge"),
                    vec![
                        arm("Charge.steps", Vec::new(), pass()),
                        arm(
                            "Charge.constant",
                            binders![unused.next()],
                            reject("hiddenCost", vec![var("wanted")]),
                        ),
                        arm(
                            "Charge.free",
                            Vec::new(),
                            reject("hiddenCost", vec![var("wanted")]),
                        ),
                        arm(
                            "Charge.undeclared",
                            Vec::new(),
                            reject("hiddenCost", vec![var("wanted")]),
                        ),
                    ],
                ),
            ),
        ),
        // An admitted preparation action costs a positive constant per
        // invocation, or is free only as common prepared state of every
        // competitor (§10.8). Communication, randomness and scheduling are
        // not actions of the sequential deterministic calculus machine, so a
        // universe admitting them is not the one the grammar generates.
        definition(
            "checkDeclared",
            vec![
                parameter("machine", local_t("Machine")),
                parameter("action", local_t("Action")),
            ],
            option_t(rejection_t()),
            let_in(
                "kind",
                kind_t(),
                action_kind(),
                ite(
                    local_call("performed", vec![var("kind")]),
                    pass(),
                    ite(
                        local_call("preparation", vec![var("kind")]),
                        matching(
                            project(var("action"), "charge"),
                            vec![
                                arm("Charge.steps", Vec::new(), hidden_cost()),
                                arm(
                                    "Charge.constant",
                                    binders!["amount"],
                                    ite(beq(var("amount"), nat(0)), hidden_cost(), pass()),
                                ),
                                arm("Charge.free", Vec::new(), prepared_common()),
                                arm("Charge.undeclared", Vec::new(), hidden_cost()),
                            ],
                        ),
                        reject("unrealizableAction", vec![var("kind")]),
                    ),
                ),
            ),
        ),
        recursive(
            "actions",
            definition(
                "checkAllDeclared",
                vec![
                    parameter("machine", local_t("Machine")),
                    parameter("actions", actions_t()),
                ],
                option_t(rejection_t()),
                on_list(
                    var("actions"),
                    pass(),
                    "action",
                    "rest",
                    on_option(
                        local_call("checkDeclared", vec![var("machine"), var("action")]),
                        local_call("checkAllDeclared", vec![var("machine"), var("rest")]),
                        "rejection",
                        some(rejection_t(), var("rejection")),
                    ),
                ),
            ),
        ),
        // The per-invocation charge of the admitted preparation actions.
        recursive(
            "actions",
            definition(
                "preparationCharge",
                vec![parameter("actions", actions_t())],
                nat_t(),
                on_list(
                    var("actions"),
                    nat(0),
                    "action",
                    "rest",
                    add(
                        ite(
                            local_call("preparation", vec![action_kind()]),
                            matching(
                                project(var("action"), "charge"),
                                vec![
                                    arm("Charge.steps", Vec::new(), nat(0)),
                                    arm("Charge.constant", binders!["amount"], var("amount")),
                                    arm("Charge.free", Vec::new(), nat(0)),
                                    arm("Charge.undeclared", Vec::new(), nat(0)),
                                ],
                            ),
                            nat(0),
                        ),
                        local_call("preparationCharge", vec![var("rest")]),
                    ),
                ),
            ),
        ),
    ]
}

// --- validation ------------------------------------------------------------

/// The claim must be answerable by the request's order: a scalar claim
/// over the total order, a vector claim over the componentwise one, and no
/// other claim at all.
fn check_claim(unused: &Unused) -> Json {
    let objective = |scalar: Json, vector: Json| {
        matching(
            project(var("request"), "objective"),
            vec![
                arm("Objective.scalar", Vec::new(), scalar),
                arm("Objective.vector", Vec::new(), vector),
            ],
        )
    };
    let mut branches: Vec<Json> = CLAIMS
        .iter()
        .map(|claim| {
            let body = if SCALAR_CLAIMS.contains(claim) {
                objective(pass(), reject("scalarClaimOverPartialOrder", Vec::new()))
            } else if VECTOR_CLAIMS.contains(claim) {
                objective(reject("vectorClaimOverTotalOrder", Vec::new()), pass())
            } else {
                reject("unsupportedClaim", Vec::new())
            };
            arm(&format!("ClaimClass.{claim}"), Vec::new(), body)
        })
        .collect();
    branches.push(arm(
        "ClaimClass.instanceOptimal",
        binders![unused.next(), unused.next()],
        reject("unsupportedClaim", Vec::new()),
    ));
    matching(project(var("request"), "claim"), branches)
}

fn validation(unused: &Unused) -> Vec<Json> {
    let request_t = || local_t("Request");
    let field = |name: &str| project(var("request"), name);
    let actions = || project(field("machine"), "actions");
    // `checkClaim` precedes `validate` in the module, so it draws its
    // unused names first.
    let claim = check_claim(unused);
    let mut checks = vec![
        // §8.1: vacuous truth never establishes exactness or optimality.
        on_list(
            field("domain"),
            reject("emptyDomain", Vec::new()),
            unused.next(),
            unused.next(),
            pass(),
        ),
        matching(
            field("carrier"),
            vec![
                arm("Carrier.grammar", binders![unused.next()], pass()),
                arm(
                    "Carrier.internalPlans",
                    Vec::new(),
                    reject("internalPlanUniverse", Vec::new()),
                ),
                arm(
                    "Carrier.optimizerOutput",
                    Vec::new(),
                    reject("optimizerDefinedUniverse", Vec::new()),
                ),
                arm(
                    "Carrier.discovered",
                    binders![unused.next()],
                    reject("discoveredUniverse", Vec::new()),
                ),
                arm(
                    "Carrier.cached",
                    Vec::new(),
                    reject("cachedUniverse", Vec::new()),
                ),
            ],
        ),
        matching(
            field("completeness"),
            vec![
                arm("Completeness.grammarEquality", Vec::new(), pass()),
                arm(
                    "Completeness.missing",
                    Vec::new(),
                    reject("missingCompleteness", Vec::new()),
                ),
                arm(
                    "Completeness.citesUniverseId",
                    Vec::new(),
                    reject("selfReferentialCompleteness", Vec::new()),
                ),
                arm(
                    "Completeness.citesOptimizer",
                    Vec::new(),
                    reject("optimizerCompleteness", Vec::new()),
                ),
            ],
        ),
        local_call("checkDistinct", vec![actions(), actions()]),
    ];
    checks.extend(PERFORMED.iter().map(|kind| {
        local_call(
            "checkPerformed",
            vec![
                field("machine"),
                local(&format!("ActionKind.{kind}"), Vec::new()),
            ],
        )
    }));
    checks.extend([
        local_call("checkAllDeclared", vec![field("machine"), actions()]),
        local_call("checkClaim", vec![var("request")]),
        matching(
            field("scope"),
            vec![
                arm("Scope.grammarUniverse", Vec::new(), pass()),
                arm(
                    "Scope.calculusPrograms",
                    Vec::new(),
                    reject("uncoveredScope", Vec::new()),
                ),
                arm(
                    "Scope.rustPrograms",
                    Vec::new(),
                    reject("uncoveredScope", Vec::new()),
                ),
            ],
        ),
    ]);
    vec![
        definition(
            "checkClaim",
            vec![parameter("request", request_t())],
            option_t(rejection_t()),
            claim,
        ),
        // §17.15: the first violated rule, in this order.
        definition(
            "validate",
            vec![parameter("request", request_t())],
            option_t(rejection_t()),
            first_rejection(unused, checks),
        ),
    ]
}

// --- the universe ----------------------------------------------------------

/// The grammar's systems, independently of any optimizer: every fixed plan,
/// then every dispatch at every threshold between two distinct plans.
fn universe() -> Vec<Json> {
    let index_pair_t = || product_t(nat_t(), nat_t());
    let next_index = || add(var("next"), nat(1));
    vec![
        recursive(
            "count",
            definition(
                "fixedSelectors",
                vec![parameter("count", nat_t()), parameter("next", nat_t())],
                list_t(selector_t()),
                on_nat(
                    var("count"),
                    nil(selector_t()),
                    cons(
                        local("Selector.fixed", vec![var("next")]),
                        local_call("fixedSelectors", vec![var("remaining"), next_index()]),
                    ),
                ),
            ),
        ),
        recursive(
            "candidates",
            definition(
                "pairsFrom",
                vec![
                    parameter("small", nat_t()),
                    parameter("candidates", list_t(nat_t())),
                ],
                list_t(index_pair_t()),
                on_list(
                    var("candidates"),
                    nil(index_pair_t()),
                    "large",
                    "rest",
                    ite(
                        beq(var("small"), var("large")),
                        local_call("pairsFrom", vec![var("small"), var("rest")]),
                        cons(
                            pair(var("small"), var("large")),
                            local_call("pairsFrom", vec![var("small"), var("rest")]),
                        ),
                    ),
                ),
            ),
        ),
        recursive(
            "smalls",
            definition(
                "orderedPairs",
                vec![
                    parameter("smalls", list_t(nat_t())),
                    parameter("all", list_t(nat_t())),
                ],
                list_t(index_pair_t()),
                on_list(
                    var("smalls"),
                    nil(index_pair_t()),
                    "small",
                    "rest",
                    prim(
                        "append",
                        vec![
                            local_call("pairsFrom", vec![var("small"), var("all")]),
                            local_call("orderedPairs", vec![var("rest"), var("all")]),
                        ],
                        list_t(index_pair_t()),
                    ),
                ),
            ),
        ),
        recursive(
            "count",
            definition(
                "indices",
                vec![parameter("count", nat_t()), parameter("next", nat_t())],
                list_t(nat_t()),
                on_nat(
                    var("count"),
                    nil(nat_t()),
                    cons(
                        var("next"),
                        local_call("indices", vec![var("remaining"), next_index()]),
                    ),
                ),
            ),
        ),
        recursive(
            "pairs",
            definition(
                "dispatchFor",
                vec![
                    parameter("threshold", nat_t()),
                    parameter("pairs", list_t(index_pair_t())),
                ],
                list_t(selector_t()),
                on_list(
                    var("pairs"),
                    nil(selector_t()),
                    "chosen",
                    "rest",
                    cons(
                        local(
                            "Selector.dispatch",
                            vec![
                                var("threshold"),
                                first(var("chosen")),
                                second(var("chosen")),
                            ],
                        ),
                        local_call("dispatchFor", vec![var("threshold"), var("rest")]),
                    ),
                ),
            ),
        ),
        recursive(
            "thresholds",
            definition(
                "dispatchSelectors",
                vec![
                    parameter("thresholds", list_t(nat_t())),
                    parameter("pairs", list_t(index_pair_t())),
                ],
                list_t(selector_t()),
                on_list(
                    var("thresholds"),
                    nil(selector_t()),
                    "threshold",
                    "rest",
                    prim(
                        "append",
                        vec![
                            local_call("dispatchFor", vec![var("threshold"), var("pairs")]),
                            local_call("dispatchSelectors", vec![var("rest"), var("pairs")]),
                        ],
                        list_t(selector_t()),
                    ),
                ),
            ),
        ),
        definition(
            "expand",
            vec![parameter("grammar", local_t("Grammar"))],
            list_t(selector_t()),
            let_in(
                "count",
                nat_t(),
                prim("length", vec![project(var("grammar"), "plans")], nat_t()),
                let_in(
                    "all",
                    list_t(nat_t()),
                    local_call("indices", vec![var("count"), nat(0)]),
                    prim(
                        "append",
                        vec![
                            local_call("fixedSelectors", vec![var("count"), nat(0)]),
                            local_call(
                                "dispatchSelectors",
                                vec![
                                    project(var("grammar"), "thresholds"),
                                    local_call("orderedPairs", vec![var("all"), var("all")]),
                                ],
                            ),
                        ],
                        list_t(selector_t()),
                    ),
                ),
            ),
        ),
    ]
}

// --- realization -----------------------------------------------------------

/// `Expr.var 0`, the realization entry's one argument.
fn argument() -> Json {
    syntax("Expr.var", vec![nat(0)])
}
/// A call of the function at `index` on the entry's argument.
fn call_on_argument(index: Json) -> Json {
    syntax("Expr.call", vec![index, list(expr_t(), vec![argument()])])
}

fn realization() -> Vec<Json> {
    let length_below_threshold = syntax(
        "Expr.prim",
        vec![
            syntax("Prim.natLt", Vec::new()),
            list(
                expr_t(),
                vec![
                    syntax(
                        "Expr.prim",
                        vec![
                            syntax("Prim.length", Vec::new()),
                            list(expr_t(), vec![argument()]),
                        ],
                    ),
                    syntax(
                        "Expr.value",
                        vec![
                            syntax("Ty.nat", Vec::new()),
                            syntax("Value.nat", vec![var("threshold")]),
                        ],
                    ),
                ],
            ),
        ],
    );
    let entry = lx::record(
        term::member(SYNTAX, "Function"),
        vec![
            ("parameters", list(nat_t(), vec![nat(0)])),
            (
                "types",
                list(syntax_t("Ty"), vec![project(var("grammar"), "argument")]),
            ),
            ("result", project(var("grammar"), "result")),
            ("body", local_call("entryBody", vec![var("selector")])),
        ],
    );
    vec![
        // A system is the entry selector over the shared plans at indices
        // 1.., so every system of one grammar shares their code and differs
        // only in its entry.
        definition(
            "entryBody",
            vec![parameter("selector", selector_t())],
            expr_t(),
            matching(
                var("selector"),
                vec![
                    arm(
                        "Selector.fixed",
                        binders!["plan"],
                        call_on_argument(add(var("plan"), nat(1))),
                    ),
                    arm(
                        "Selector.dispatch",
                        binders!["threshold", "small", "large"],
                        syntax(
                            "Expr.cond",
                            vec![
                                length_below_threshold,
                                call_on_argument(add(var("small"), nat(1))),
                                call_on_argument(add(var("large"), nat(1))),
                            ],
                        ),
                    ),
                ],
            ),
        ),
        definition(
            "realize",
            vec![
                parameter("grammar", local_t("Grammar")),
                parameter("selector", selector_t()),
            ],
            syntax_t("Program"),
            lx::record(
                term::member(SYNTAX, "Program"),
                vec![
                    ("adts", nil(syntax_t("Adt"))),
                    ("functions", cons(entry, project(var("grammar"), "plans"))),
                ],
            ),
        ),
    ]
}

// --- value equality --------------------------------------------------------

/// Value equality, structurally over the nested `Value` family: equal
/// constructors with equal fields.
fn value_equality(unused: &Unused) -> Vec<Json> {
    let both_have = |constructor: &'static str,
                     left: &[&str],
                     right: &[&str],
                     equal: Json|
     -> (&'static str, Vec<String>, Json) {
        let right_side = on_value(
            unused,
            var("right"),
            vec![(constructor, binders_of(right), equal)],
            &no(),
        );
        (constructor, binders_of(left), right_side)
    };
    let same_item = |constructor: &'static str, equal: fn(Json, Json) -> Json| {
        both_have(
            constructor,
            &["leftItem"],
            &["rightItem"],
            equal(var("leftItem"), var("rightItem")),
        )
    };
    let decidable = |left: Json, right: Json| prim("equal", vec![left, right], lx::bool_t());
    let value_eq = |left: Json, right: Json| local_call("valueEq", vec![left, right]);
    let values_eq = |left: Json, right: Json| local_call("valuesEq", vec![left, right]);
    let same_order = |left: Json, right: Json| semantics_call("sameOrder", vec![left, right]);
    let same_int = |left: Json, right: Json| {
        semantics_call(
            "sameOrder",
            vec![
                semantics_call("orderInt", vec![left, right]),
                syntax("Order.same", Vec::new()),
            ],
        )
    };
    // Built in this order, which fixes the unused binder names; the match
    // itself lists constructors in declaration order.
    let mut cases = vec![
        both_have("unit", &[], &[], yes()),
        both_have("none", &[], &[], yes()),
        same_item("bool", decidable),
        same_item("nat", beq),
        same_item("string", decidable),
        same_item("bytes", decidable),
        same_item("int", same_int),
        same_item("ordering", same_order),
        same_item("some", value_eq),
        same_item("ok", value_eq),
        same_item("error", value_eq),
        same_item("list", values_eq),
        both_have(
            "pair",
            &["leftFirst", "leftSecond"],
            &["rightFirst", "rightSecond"],
            and(
                value_eq(var("leftFirst"), var("rightFirst")),
                value_eq(var("leftSecond"), var("rightSecond")),
            ),
        ),
        both_have(
            "adt",
            &["leftTag", "leftFields"],
            &["rightTag", "rightFields"],
            and(
                beq(var("leftTag"), var("rightTag")),
                values_eq(var("leftFields"), var("rightFields")),
            ),
        ),
        both_have(
            "closure",
            &["leftFunction", "leftCaptures"],
            &["rightFunction", "rightCaptures"],
            and(
                beq(var("leftFunction"), var("rightFunction")),
                values_eq(var("leftCaptures"), var("rightCaptures")),
            ),
        ),
    ];
    for fixed in ["u8", "u16", "u32", "u64", "i8", "i16", "i32", "i64"] {
        cases.push(same_item(fixed, decidable));
    }
    let value_eq_body = on_value(unused, var("left"), cases, &no());
    vec![
        mutual(
            "ValueEquality",
            recursive(
                "left",
                definition(
                    "valueEq",
                    vec![parameter("left", value_t()), parameter("right", value_t())],
                    lx::bool_t(),
                    value_eq_body,
                ),
            ),
        ),
        mutual(
            "ValueEquality",
            recursive(
                "left",
                definition(
                    "valuesEq",
                    vec![
                        parameter("left", list_t(value_t())),
                        parameter("right", list_t(value_t())),
                    ],
                    lx::bool_t(),
                    on_list(
                        var("left"),
                        on_list(var("right"), yes(), unused.next(), unused.next(), no()),
                        "leftHead",
                        "leftTail",
                        on_list(
                            var("right"),
                            no(),
                            "rightHead",
                            "rightTail",
                            and(
                                value_eq(var("leftHead"), var("rightHead")),
                                values_eq(var("leftTail"), var("rightTail")),
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ]
}

fn binders_of(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_owned()).collect()
}

// --- program size ----------------------------------------------------------

/// Program size: one per expression node and per arm, structurally.
fn program_size(unused: &Unused) -> Vec<Json> {
    let one = || nat(1);
    let size = |name: &str| local_call("exprSize", vec![var(name)]);
    let operands = || add(one(), local_call("exprsSize", vec![var("operands")]));
    let inner = || add(one(), size("inner"));
    let over_list = |function: &str, element: &str, list_name: &str, item_size: Json| {
        recursive(
            list_name,
            mutual(
                "ExpressionSize",
                definition(
                    function,
                    vec![parameter(list_name, list_t(syntax_t(element)))],
                    nat_t(),
                    on_list(
                        var(list_name),
                        nat(0),
                        "head",
                        "rest",
                        add(item_size, local_call(function, vec![var("rest")])),
                    ),
                ),
            ),
        )
    };
    let expr_size = matching(
        var("expression"),
        vec![
            syntax_arm("Expr.value", binders![unused.next(), unused.next()], one()),
            syntax_arm("Expr.var", binders![unused.next()], one()),
            syntax_arm(
                "Expr.let",
                binders![unused.next(), unused.next(), "bound", "body"],
                add(one(), add(size("bound"), size("body"))),
            ),
            syntax_arm(
                "Expr.cond",
                binders!["condition", "thenBranch", "elseBranch"],
                add(
                    one(),
                    add(
                        size("condition"),
                        add(size("thenBranch"), size("elseBranch")),
                    ),
                ),
            ),
            syntax_arm(
                "Expr.match",
                binders![unused.next(), "scrutinee", "arms"],
                add(
                    one(),
                    add(size("scrutinee"), local_call("armsSize", vec![var("arms")])),
                ),
            ),
            syntax_arm(
                "Expr.build",
                binders![unused.next(), unused.next(), "operands"],
                operands(),
            ),
            syntax_arm("Expr.call", binders![unused.next(), "operands"], operands()),
            syntax_arm(
                "Expr.closure",
                binders![unused.next(), "operands"],
                operands(),
            ),
            syntax_arm(
                "Expr.apply",
                binders!["target", "operands"],
                add(
                    one(),
                    add(
                        size("target"),
                        local_call("exprsSize", vec![var("operands")]),
                    ),
                ),
            ),
            syntax_arm("Expr.prim", binders![unused.next(), "operands"], operands()),
            syntax_arm("Expr.first", binders!["inner"], inner()),
            syntax_arm("Expr.second", binders!["inner"], inner()),
            syntax_arm("Expr.field", binders!["inner", unused.next()], inner()),
        ],
    );
    let arm_size = matching(
        var("arm"),
        vec![syntax_arm(
            "Arm.arm",
            binders![unused.next(), unused.next(), "body"],
            add(one(), size("body")),
        )],
    );
    vec![
        recursive(
            "expression",
            mutual(
                "ExpressionSize",
                definition(
                    "exprSize",
                    vec![parameter("expression", expr_t())],
                    nat_t(),
                    expr_size,
                ),
            ),
        ),
        over_list(
            "exprsSize",
            "Expr",
            "expressions",
            local_call("exprSize", vec![var("head")]),
        ),
        over_list(
            "armsSize",
            "Arm",
            "arms",
            local_call("armSize", vec![var("head")]),
        ),
        recursive(
            "arm",
            mutual(
                "ExpressionSize",
                definition(
                    "armSize",
                    vec![parameter("arm", syntax_t("Arm"))],
                    nat_t(),
                    arm_size,
                ),
            ),
        ),
        recursive(
            "functions",
            definition(
                "functionsSize",
                vec![parameter("functions", list_t(syntax_t("Function")))],
                nat_t(),
                on_list(
                    var("functions"),
                    nat(0),
                    "head",
                    "rest",
                    add(
                        local_call("exprSize", vec![project(var("head"), "body")]),
                        local_call("functionsSize", vec![var("rest")]),
                    ),
                ),
            ),
        ),
    ]
}

// --- admission and cost ----------------------------------------------------

/// `TargetSemantics.run fuel program 0 [argument]`: the program's entry on
/// one domain argument.
fn run(program: Json, argument: Json, fuel: Json) -> Json {
    semantics_call(
        "run",
        vec![fuel, program, nat(0), list(value_t(), vec![argument])],
    )
}
/// A match on an outcome: a value with its steps, or a failure whose kind
/// (`overflow`, `stuck`, `exhausted`) selects the body.
fn on_outcome(
    unused: &Unused,
    outcome: Json,
    value: &str,
    steps: impl Into<String>,
    produced: Json,
    failed: impl Fn(&str) -> Json,
) -> Json {
    matching(
        outcome,
        vec![
            semantics_arm(
                "Outcome.value",
                vec![value.to_owned(), steps.into()],
                produced,
            ),
            semantics_arm(
                "Outcome.overflow",
                binders![unused.next()],
                failed("overflow"),
            ),
            semantics_arm("Outcome.stuck", Vec::new(), failed("stuck")),
            semantics_arm("Outcome.exhausted", Vec::new(), failed("exhausted")),
        ],
    )
}
/// Carry a status through unchanged except for `admitted`.
fn on_status(scrutinee: Json, admitted_binders: Vec<String>, admitted: Json) -> Json {
    matching(
        scrutinee,
        vec![
            arm("Status.admitted", admitted_binders, admitted),
            arm(
                "Status.inadmissible",
                Vec::new(),
                status("inadmissible", Vec::new()),
            ),
            arm(
                "Status.unresolved",
                Vec::new(),
                status("unresolved", Vec::new()),
            ),
        ],
    )
}

fn admission(unused: &Unused) -> Vec<Json> {
    let field = |name: &str| project(var("request"), name);
    let machine = |name: &str| project(field("machine"), name);
    let unresolved = || status("unresolved", Vec::new());
    let checked = on_outcome(
        unused,
        run(var("system"), var("argument"), var("fuel")),
        "produced",
        "steps",
        on_outcome(
            unused,
            run(var("reference"), var("argument"), var("fuel")),
            "expected",
            unused.next(),
            ite(
                local_call("valueEq", vec![var("produced"), var("expected")]),
                on_status(
                    local_call(
                        "statusOn",
                        vec![var("system"), var("reference"), var("fuel"), var("rest")],
                    ),
                    binders!["restSteps", "size"],
                    status(
                        "admitted",
                        vec![add(var("steps"), var("restSteps")), var("size")],
                    ),
                ),
                status("inadmissible", Vec::new()),
            ),
            |_| unresolved(),
        ),
        // Only fuel running out leaves the system's correctness unknown;
        // overflowing or getting stuck where the reference does not is a
        // wrong answer.
        |failure| {
            if failure == "exhausted" {
                unresolved()
            } else {
                status("inadmissible", Vec::new())
            }
        },
    );
    let per_invocation = prim(
        "multiply",
        vec![
            prim("length", vec![field("domain")], nat_t()),
            local_call("preparationCharge", vec![machine("actions")]),
        ],
        nat_t(),
    );
    let costed = on_status(
        local_call(
            "statusOn",
            vec![
                var("system"),
                field("reference"),
                machine("fuel"),
                field("domain"),
            ],
        ),
        binders!["steps", unused.next()],
        status(
            "admitted",
            vec![
                add(var("steps"), per_invocation),
                local_call("functionsSize", vec![project(var("system"), "functions")]),
            ],
        ),
    );
    vec![
        // §8.4: admission is correctness on every domain invocation; an
        // exhausted evaluation leaves eligibility unknown, never false.
        axioms(
            &RUN_AXIOMS,
            recursive(
                "domain",
                definition(
                    "statusOn",
                    vec![
                        parameter("system", syntax_t("Program")),
                        parameter("reference", syntax_t("Program")),
                        parameter("fuel", nat_t()),
                        parameter("domain", list_t(value_t())),
                    ],
                    status_t(),
                    on_list(
                        var("domain"),
                        status("admitted", vec![nat(0), nat(0)]),
                        "argument",
                        "rest",
                        checked,
                    ),
                ),
            ),
        ),
        // The cost of an admitted system: its steps over the domain plus the
        // declared preparation charge per invocation, and its size.
        axioms(
            &RUN_AXIOMS,
            definition(
                "status",
                vec![
                    parameter("request", local_t("Request")),
                    parameter("grammar", local_t("Grammar")),
                    parameter("selector", selector_t()),
                ],
                status_t(),
                let_in(
                    "system",
                    syntax_t("Program"),
                    local_call("realize", vec![var("grammar"), var("selector")]),
                    costed,
                ),
            ),
        ),
        axioms(
            &RUN_AXIOMS,
            recursive(
                "selectors",
                definition(
                    "statuses",
                    vec![
                        parameter("request", local_t("Request")),
                        parameter("grammar", local_t("Grammar")),
                        parameter("selectors", list_t(selector_t())),
                        parameter("next", nat_t()),
                    ],
                    list_t(product_t(nat_t(), status_t())),
                    on_list(
                        var("selectors"),
                        nil(product_t(nat_t(), status_t())),
                        "selector",
                        "rest",
                        cons(
                            pair(
                                var("next"),
                                local_call(
                                    "status",
                                    vec![var("request"), var("grammar"), var("selector")],
                                ),
                            ),
                            local_call(
                                "statuses",
                                vec![
                                    var("request"),
                                    var("grammar"),
                                    var("rest"),
                                    add(var("next"), nat(1)),
                                ],
                            ),
                        ),
                    ),
                ),
            ),
        ),
    ]
}

// --- the answer ------------------------------------------------------------

/// A system's index paired with its status.
fn entry_t() -> Json {
    product_t(nat_t(), status_t())
}
/// An admitted system's index paired with its (steps, size).
fn costed_t() -> Json {
    product_t(nat_t(), product_t(nat_t(), nat_t()))
}

fn answer(unused: &Unused) -> Vec<Json> {
    let steps_of = |entry: &str| first(second(var(entry)));
    vec![
        recursive(
            "entries",
            definition(
                "anyUnresolved",
                vec![parameter("entries", list_t(entry_t()))],
                lx::bool_t(),
                on_list(
                    var("entries"),
                    no(),
                    "entry",
                    "rest",
                    matching(
                        second(var("entry")),
                        vec![
                            arm("Status.unresolved", Vec::new(), yes()),
                            arm(
                                "Status.admitted",
                                binders![unused.next(), unused.next()],
                                local_call("anyUnresolved", vec![var("rest")]),
                            ),
                            arm(
                                "Status.inadmissible",
                                Vec::new(),
                                local_call("anyUnresolved", vec![var("rest")]),
                            ),
                        ],
                    ),
                ),
            ),
        ),
        recursive(
            "entries",
            definition(
                "admitted",
                vec![parameter("entries", list_t(entry_t()))],
                list_t(costed_t()),
                on_list(
                    var("entries"),
                    nil(costed_t()),
                    "entry",
                    "rest",
                    matching(
                        second(var("entry")),
                        vec![
                            arm(
                                "Status.admitted",
                                binders!["steps", "size"],
                                cons(
                                    pair(first(var("entry")), pair(var("steps"), var("size"))),
                                    local_call("admitted", vec![var("rest")]),
                                ),
                            ),
                            arm(
                                "Status.inadmissible",
                                Vec::new(),
                                local_call("admitted", vec![var("rest")]),
                            ),
                            arm(
                                "Status.unresolved",
                                Vec::new(),
                                local_call("admitted", vec![var("rest")]),
                            ),
                        ],
                    ),
                ),
            ),
        ),
        recursive(
            "costed",
            definition(
                "minimumSteps",
                vec![
                    parameter("costed", list_t(costed_t())),
                    parameter("best", nat_t()),
                ],
                nat_t(),
                on_list(
                    var("costed"),
                    var("best"),
                    "entry",
                    "rest",
                    local_call(
                        "minimumSteps",
                        vec![
                            var("rest"),
                            ite(
                                blt(steps_of("entry"), var("best")),
                                steps_of("entry"),
                                var("best"),
                            ),
                        ],
                    ),
                ),
            ),
        ),
        recursive(
            "costed",
            definition(
                "withSteps",
                vec![
                    parameter("costed", list_t(costed_t())),
                    parameter("value", nat_t()),
                ],
                list_t(nat_t()),
                on_list(
                    var("costed"),
                    nil(nat_t()),
                    "entry",
                    "rest",
                    ite(
                        beq(steps_of("entry"), var("value")),
                        cons(
                            first(var("entry")),
                            local_call("withSteps", vec![var("rest"), var("value")]),
                        ),
                        local_call("withSteps", vec![var("rest"), var("value")]),
                    ),
                ),
            ),
        ),
        // §9.3: strict componentwise dominance over (steps, size).
        definition(
            "dominates",
            vec![
                parameter("left", product_t(nat_t(), nat_t())),
                parameter("right", product_t(nat_t(), nat_t())),
            ],
            lx::bool_t(),
            and(
                and(
                    ble(first(var("left")), first(var("right"))),
                    ble(second(var("left")), second(var("right"))),
                ),
                or(
                    blt(first(var("left")), first(var("right"))),
                    blt(second(var("left")), second(var("right"))),
                ),
            ),
        ),
        recursive(
            "others",
            definition(
                "dominated",
                vec![
                    parameter("cost", product_t(nat_t(), nat_t())),
                    parameter("others", list_t(costed_t())),
                ],
                lx::bool_t(),
                on_list(
                    var("others"),
                    no(),
                    "other",
                    "rest",
                    or(
                        local_call("dominates", vec![second(var("other")), var("cost")]),
                        local_call("dominated", vec![var("cost"), var("rest")]),
                    ),
                ),
            ),
        ),
        recursive(
            "candidates",
            definition(
                "nondominated",
                vec![
                    parameter("candidates", list_t(costed_t())),
                    parameter("all", list_t(costed_t())),
                ],
                list_t(nat_t()),
                on_list(
                    var("candidates"),
                    nil(nat_t()),
                    "entry",
                    "rest",
                    ite(
                        local_call("dominated", vec![second(var("entry")), var("all")]),
                        local_call("nondominated", vec![var("rest"), var("all")]),
                        cons(
                            first(var("entry")),
                            local_call("nondominated", vec![var("rest"), var("all")]),
                        ),
                    ),
                ),
            ),
        ),
        // An unknown cost anywhere leaves the answer incomplete: the
        // unresolved system might be the optimum.
        axioms(
            &RUN_AXIOMS,
            definition(
                "evaluate",
                vec![
                    parameter("request", local_t("Request")),
                    parameter("grammar", local_t("Grammar")),
                ],
                local_t("Answer"),
                let_in(
                    "entries",
                    list_t(entry_t()),
                    local_call(
                        "statuses",
                        vec![
                            var("request"),
                            var("grammar"),
                            local_call("expand", vec![var("grammar")]),
                            nat(0),
                        ],
                    ),
                    ite(
                        local_call("anyUnresolved", vec![var("entries")]),
                        local("Answer.incomplete", Vec::new()),
                        let_in(
                            "costed",
                            list_t(costed_t()),
                            local_call("admitted", vec![var("entries")]),
                            on_list(
                                var("costed"),
                                local("Answer.infeasible", Vec::new()),
                                "head",
                                unused.next(),
                                matching(
                                    project(var("request"), "objective"),
                                    vec![
                                        arm(
                                            "Objective.scalar",
                                            Vec::new(),
                                            let_in(
                                                "best",
                                                nat_t(),
                                                local_call(
                                                    "minimumSteps",
                                                    vec![var("costed"), steps_of("head")],
                                                ),
                                                local(
                                                    "Answer.argmin",
                                                    vec![
                                                        local_call(
                                                            "withSteps",
                                                            vec![var("costed"), var("best")],
                                                        ),
                                                        var("best"),
                                                    ],
                                                ),
                                            ),
                                        ),
                                        arm(
                                            "Objective.vector",
                                            Vec::new(),
                                            local(
                                                "Answer.frontier",
                                                vec![local_call(
                                                    "nondominated",
                                                    vec![var("costed"), var("costed")],
                                                )],
                                            ),
                                        ),
                                    ],
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
        // The answer: a rejected request, or the request's grammar universe
        // evaluated by the calculus denotation.
        axioms(
            &RUN_AXIOMS,
            definition(
                "answer",
                vec![parameter("request", local_t("Request"))],
                local_t("Answer"),
                on_option(
                    local_call("validate", vec![var("request")]),
                    matching(
                        project(var("request"), "carrier"),
                        vec![
                            arm(
                                "Carrier.grammar",
                                binders!["grammar"],
                                local_call("evaluate", vec![var("request"), var("grammar")]),
                            ),
                            arm(
                                "Carrier.internalPlans",
                                Vec::new(),
                                local("Answer.incomplete", Vec::new()),
                            ),
                            arm(
                                "Carrier.optimizerOutput",
                                Vec::new(),
                                local("Answer.incomplete", Vec::new()),
                            ),
                            arm(
                                "Carrier.discovered",
                                binders![unused.next()],
                                local("Answer.incomplete", Vec::new()),
                            ),
                            arm(
                                "Carrier.cached",
                                Vec::new(),
                                local("Answer.incomplete", Vec::new()),
                            ),
                        ],
                    ),
                    "rejection",
                    local("Answer.rejected", vec![var("rejection")]),
                ),
            ),
        ),
    ]
}

// --- the authority's normative fixtures -------------------------------------

/// uor-gnaf/1-draft.2 §16: identities are naturals, costs vectors of
/// naturals, so a table lists `(identity, cost vector)` rows.
fn table_t() -> Json {
    list_t(product_t(nat_t(), list_t(nat_t())))
}

fn cost_tables(unused: &Unused) -> Vec<Json> {
    let nats_t = || list_t(nat_t());
    let cost_of = |row: &str| second(var(row));
    vec![
        recursive(
            "left",
            definition(
                "weaklyBelow",
                vec![parameter("left", nats_t()), parameter("right", nats_t())],
                lx::bool_t(),
                on_list(
                    var("left"),
                    yes(),
                    "leftHead",
                    "leftRest",
                    on_list(
                        var("right"),
                        no(),
                        "rightHead",
                        "rightRest",
                        and(
                            ble(var("leftHead"), var("rightHead")),
                            local_call("weaklyBelow", vec![var("leftRest"), var("rightRest")]),
                        ),
                    ),
                ),
            ),
        ),
        definition(
            "strictlyBelow",
            vec![parameter("left", nats_t()), parameter("right", nats_t())],
            lx::bool_t(),
            and(
                local_call("weaklyBelow", vec![var("left"), var("right")]),
                not(local_call("weaklyBelow", vec![var("right"), var("left")])),
            ),
        ),
        recursive(
            "table",
            definition(
                "tableDominated",
                vec![parameter("cost", nats_t()), parameter("table", table_t())],
                lx::bool_t(),
                on_list(
                    var("table"),
                    no(),
                    "row",
                    "rest",
                    or(
                        local_call("strictlyBelow", vec![cost_of("row"), var("cost")]),
                        local_call("tableDominated", vec![var("cost"), var("rest")]),
                    ),
                ),
            ),
        ),
        recursive(
            "rows",
            definition(
                "tableFrontierFrom",
                vec![parameter("rows", table_t()), parameter("table", table_t())],
                nats_t(),
                on_list(
                    var("rows"),
                    nil(nat_t()),
                    "row",
                    "rest",
                    ite(
                        local_call("tableDominated", vec![cost_of("row"), var("table")]),
                        local_call("tableFrontierFrom", vec![var("rest"), var("table")]),
                        cons(
                            first(var("row")),
                            local_call("tableFrontierFrom", vec![var("rest"), var("table")]),
                        ),
                    ),
                ),
            ),
        ),
        definition(
            "tableFrontier",
            vec![parameter("table", table_t())],
            nats_t(),
            local_call("tableFrontierFrom", vec![var("table"), var("table")]),
        ),
        recursive(
            "left",
            definition(
                "sameIds",
                vec![parameter("left", nats_t()), parameter("right", nats_t())],
                lx::bool_t(),
                on_list(
                    var("left"),
                    on_list(var("right"), yes(), unused.next(), unused.next(), no()),
                    "leftHead",
                    "leftRest",
                    on_list(
                        var("right"),
                        no(),
                        "rightHead",
                        "rightRest",
                        and(
                            beq(var("leftHead"), var("rightHead")),
                            local_call("sameIds", vec![var("leftRest"), var("rightRest")]),
                        ),
                    ),
                ),
            ),
        ),
        recursive(
            "table",
            definition(
                "tableAttains",
                vec![parameter("table", table_t()), parameter("cost", nats_t())],
                lx::bool_t(),
                on_list(
                    var("table"),
                    no(),
                    "row",
                    "rest",
                    or(
                        and(
                            local_call("weaklyBelow", vec![cost_of("row"), var("cost")]),
                            local_call("weaklyBelow", vec![var("cost"), cost_of("row")]),
                        ),
                        local_call("tableAttains", vec![var("rest"), var("cost")]),
                    ),
                ),
            ),
        ),
        recursive(
            "table",
            definition(
                "rowMinimum",
                vec![parameter("table", table_t()), parameter("best", nat_t())],
                nat_t(),
                on_list(
                    var("table"),
                    var("best"),
                    "row",
                    "rest",
                    local_call(
                        "rowMinimum",
                        vec![
                            var("rest"),
                            on_list(
                                cost_of("row"),
                                var("best"),
                                "cost",
                                unused.next(),
                                ite(blt(var("cost"), var("best")), var("cost"), var("best")),
                            ),
                        ],
                    ),
                ),
            ),
        ),
        // A one-step local rewrite system: `normalize` follows rewrites until
        // no rule applies, within `fuel` steps.
        recursive(
            "rules",
            definition(
                "rewriteOnce",
                vec![
                    parameter("rules", list_t(product_t(nat_t(), nat_t()))),
                    parameter("item", nat_t()),
                ],
                option_t(nat_t()),
                on_list(
                    var("rules"),
                    none(nat_t()),
                    "rule",
                    "rest",
                    ite(
                        beq(first(var("rule")), var("item")),
                        some(nat_t(), second(var("rule"))),
                        local_call("rewriteOnce", vec![var("rest"), var("item")]),
                    ),
                ),
            ),
        ),
        recursive(
            "fuel",
            definition(
                "normalize",
                vec![
                    parameter("fuel", nat_t()),
                    parameter("rules", list_t(product_t(nat_t(), nat_t()))),
                    parameter("item", nat_t()),
                ],
                nat_t(),
                on_nat(
                    var("fuel"),
                    var("item"),
                    on_option(
                        local_call("rewriteOnce", vec![var("rules"), var("item")]),
                        var("item"),
                        "next",
                        local_call(
                            "normalize",
                            vec![var("remaining"), var("rules"), var("next")],
                        ),
                    ),
                ),
            ),
        ),
        recursive(
            "costs",
            definition(
                "listMax",
                vec![parameter("costs", nats_t()), parameter("worst", nat_t())],
                nat_t(),
                on_list(
                    var("costs"),
                    var("worst"),
                    "cost",
                    "rest",
                    local_call(
                        "listMax",
                        vec![
                            var("rest"),
                            ite(blt(var("worst"), var("cost")), var("cost"), var("worst")),
                        ],
                    ),
                ),
            ),
        ),
        // The best worst case a single uniform system guarantees over every
        // environment: the minimum over rows of each row's maximum.
        recursive(
            "table",
            definition(
                "uniformWorst",
                vec![parameter("table", table_t()), parameter("best", nat_t())],
                nat_t(),
                on_list(
                    var("table"),
                    var("best"),
                    "row",
                    "rest",
                    local_call(
                        "uniformWorst",
                        vec![
                            var("rest"),
                            let_in(
                                "worst",
                                nat_t(),
                                local_call("listMax", vec![cost_of("row"), nat(0)]),
                                ite(blt(var("worst"), var("best")), var("worst"), var("best")),
                            ),
                        ],
                    ),
                ),
            ),
        ),
    ]
}

fn nats(numbers: &[u64]) -> Json {
    list(nat_t(), numbers.iter().map(|number| nat(*number)).collect())
}
/// A cost table: `(identity, costs)` rows.
fn table(rows: &[(u64, &[u64])]) -> Json {
    list(
        product_t(nat_t(), list_t(nat_t())),
        rows.iter()
            .map(|(identity, costs)| pair(nat(*identity), nats(costs)))
            .collect(),
    )
}
/// `call arguments = expected`, decided by the kernel.
fn decided(name: &str, function: &str, arguments: Vec<Json>, expected: Json) -> Json {
    theorem(
        name,
        eq(local_call(function, arguments), expected),
        decide(),
    )
}

fn authority_vectors() -> Vec<Json> {
    let vec02 = || table(&[(0, &[1, 3]), (1, &[2, 2]), (2, &[3, 1]), (3, &[3, 3])]);
    let unbounded = || nat(1000);
    vec![
        // GNAF-VEC-02 (§16.2): rA=(1,3), rB=(2,2), rC=(3,1), rD=(3,3); the
        // complete frontier is exactly {rA, rB, rC}.
        decided(
            "vec02Frontier",
            "tableFrontier",
            vec![vec02()],
            nats(&[0, 1, 2]),
        ),
        // GNAF-REJ-14 (§16.24): the componentwise minima (1,1) of
        // GNAF-VEC-02 are attained by no candidate, so they are no vector
        // optimum.
        decided(
            "rej14ComponentwiseMinimaUnattained",
            "tableAttains",
            vec![vec02(), nats(&[1, 1])],
            no(),
        ),
        // GNAF-REJ-29 (§16.24): a frontier response omitting rC is not the
        // complete frontier.
        decided(
            "rej29FrontierOmission",
            "sameIds",
            vec![local_call("tableFrontier", vec![vec02()]), nats(&[0, 1])],
            no(),
        ),
        // GNAF-VEC-01 (§16.1): the S0 compositions are [rAB, rBC] at 5 and
        // rAC6 at 6, optimum 5; admitting rAC4 at 4 in S1 makes the optimum
        // 4, so the S0 optimum certifies nothing about S1.
        decided(
            "vec01Optimum",
            "rowMinimum",
            vec![table(&[(0, &[5]), (1, &[6])]), unbounded()],
            nat(5),
        ),
        decided(
            "vec01Extension",
            "rowMinimum",
            vec![table(&[(0, &[5]), (1, &[6]), (2, &[4])]), unbounded()],
            nat(4),
        ),
        // GNAF-VEC-04 (§16.4): with only the local rewrite a -> b, b is
        // normal but c is the global minimum; local irreducibility proves
        // nothing.
        decided(
            "vec04NormalFormIsB",
            "normalize",
            vec![
                nat(10),
                list(product_t(nat_t(), nat_t()), vec![pair(nat(0), nat(1))]),
                nat(0),
            ],
            nat(1),
        ),
        decided(
            "vec04GlobalMinimumIsC",
            "tableFrontier",
            vec![table(&[(0, &[2]), (1, &[1]), (2, &[0])])],
            nats(&[2]),
        ),
        // GNAF-VEC-17 (§16.17): R0 and R1 cost 0 when their index equals the
        // hidden bit h and 10 otherwise. The pointwise envelope is 0 for each
        // h, but every uniform system's worst case is 10, so no executable
        // system may be certified with the pointwise value.
        decided(
            "vec17EnvelopeAtZero",
            "rowMinimum",
            vec![table(&[(0, &[0]), (1, &[10])]), unbounded()],
            nat(0),
        ),
        decided(
            "vec17EnvelopeAtOne",
            "rowMinimum",
            vec![table(&[(0, &[10]), (1, &[0])]), unbounded()],
            nat(0),
        ),
        decided(
            "vec17UniformWorstCase",
            "uniformWorst",
            vec![table(&[(0, &[0, 10]), (1, &[10, 0])]), unbounded()],
            nat(10),
        ),
    ]
}

// --- the module ------------------------------------------------------------

/// Every declaration of the model, in module order: each group may only
/// refer to the groups before it.
fn declarations() -> Vec<Json> {
    let unused = Unused::new();
    let mut out = request_types();
    out.extend(machine_contract(&unused));
    out.extend(validation(&unused));
    out.extend(universe());
    out.extend(realization());
    out.extend(value_equality(&unused));
    out.extend(program_size(&unused));
    out.extend(admission(&unused));
    out.extend(answer(&unused));
    out.extend(cost_tables(&unused));
    out.extend(authority_vectors());
    out
}

/// The `Gnaf` module text.
#[must_use]
pub fn module() -> String {
    lx::module_tex(MODEL, &[SYNTAX, SEMANTICS], declarations())
}
