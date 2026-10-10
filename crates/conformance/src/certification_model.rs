//! The `Certification` module of the `compiler` project: the GNAF
//! certification framework of SPEC.md §17.18, written here as LexLean
//! semantic terms.
//!
//! The committed `compiler/src/Certification.lex.tex` is exactly what
//! [`module`] renders, which `cargo xtask check-calculus` enforces. The
//! module imports `Gnaf` and states, over the request model of §17.15,
//! what a certificate is, what its premises establish about each system of
//! the request's universe, and which verdict a certificate earns: a
//! certified global optimum or Pareto frontier, or an honest weaker status
//! that names the first obligation left open. Nothing here decides
//! optimality by normality: a locally normalized system is a status of its
//! own, and no route turns it into a certified optimum.
//!
//! Binders a branch never reads are named `ignored<n>` from one counter, in
//! the order the terms are built, as in [`crate::gnaf_model`].

use lexlean::calculus::term::{self, SYNTAX};
use lexlean::gnaf::MODEL;
use serde_json::Value as Json;

use crate::gnaf_model::{claims, ExprFold, Unused, RUN_AXIOMS};
use crate::lx::{
    self, add, and, axioms, beq, ble, blt, boolean, call, call_at, cons, definition, first, ite,
    let_in, list, list_t, local_t, nat, nat_t, nil, none, not, option_t, pair,
    parameter, prim, product_t, project, recursive, second, some, string_t, structure, var,
};

/// The path of the generated module, relative to the repository root.
pub const PATH: &str = "compiler/src/Certification.lex.tex";

/// The name of the module.
pub const NAME: &str = "Certification";

/// The verifier a certificate names: the checker of this module, versioned.
pub const VERIFIER: &str = "lexlean/gnaf-certification/1";

/// The transformation schemas, in declaration order (§17.18).
pub const SCHEMAS: [&str; 3] = ["collapse", "specializeSmall", "specializeLarge"];

macro_rules! binders {
    ($($binder:expr),* $(,)?) => {
        vec![$(($binder).to_string()),*]
    };
}

// --- terms of this module ----------------------------------------------------

fn t(name: &str) -> Json {
    local_t(name)
}
fn g(name: &str) -> Json {
    term::named(MODEL, name)
}
fn nats_t() -> Json {
    list_t(nat_t())
}
fn selectors_t() -> Json {
    list_t(g("Selector"))
}
fn value_t() -> Json {
    term::named(SYNTAX, "Value")
}
fn program_t() -> Json {
    term::named(SYNTAX, "Program")
}
fn facts_t() -> Json {
    list_t(t("Fact"))
}
fn bool_t() -> Json {
    lx::bool_t()
}
fn row_t() -> Json {
    product_t(nat_t(), nats_t())
}
fn rows_t() -> Json {
    list_t(row_t())
}
fn yes() -> Json {
    boolean(true)
}
fn no() -> Json {
    boolean(false)
}
fn p(name: &str, ty: Json) -> Json {
    parameter(name, ty)
}

/// A constructor of this module.
fn c(constructor: &str, arguments: Vec<Json>) -> Json {
    lx::construct(lx::member(constructor), arguments)
}
/// A constructor of `Gnaf`.
fn gc(constructor: &str, arguments: Vec<Json>) -> Json {
    lx::construct(term::member(MODEL, constructor), arguments)
}
/// A call of a definition of this module.
fn f(function: &str, arguments: Vec<Json>) -> Json {
    call(lx::member(function), arguments)
}
/// A call of a definition of `Gnaf`.
fn gf(function: &str, arguments: Vec<Json>) -> Json {
    call(term::member(MODEL, function), arguments)
}
/// A branch on a constructor of this module or a builtin.
fn a(constructor: &str, binders: Vec<String>, body: Json) -> Json {
    lx::branch(lx::member(constructor), binders, body)
}
/// A branch on a constructor of `Gnaf`.
fn ga(constructor: &str, binders: Vec<String>, body: Json) -> Json {
    lx::branch(term::member(MODEL, constructor), binders, body)
}
fn sa(constructor: &str, binders: Vec<String>, body: Json) -> Json {
    lx::branch(term::member(SYNTAX, constructor), binders, body)
}
fn m(scrutinee: Json, branches: Vec<Json>) -> Json {
    lx::matching(scrutinee, branches)
}
fn on_list(
    scrutinee: Json,
    empty: Json,
    head: impl Into<String>,
    tail: impl Into<String>,
    nonempty: Json,
) -> Json {
    m(
        scrutinee,
        vec![
            a("List.nil", Vec::new(), empty),
            a("List.cons", vec![head.into(), tail.into()], nonempty),
        ],
    )
}
fn on_option(scrutinee: Json, absent: Json, found: impl Into<String>, present: Json) -> Json {
    m(
        scrutinee,
        vec![
            a("Option.none", Vec::new(), absent),
            a("Option.some", vec![found.into()], present),
        ],
    )
}
fn on_nat(scrutinee: Json, zero: Json, successor: Json) -> Json {
    m(
        scrutinee,
        vec![
            a("Nat.zero", Vec::new(), zero),
            a("Nat.succ", binders!["remaining"], successor),
        ],
    )
}
fn next(value: &str) -> Json {
    add(var(value), nat(1))
}
fn length(items: Json) -> Json {
    prim("length", vec![items], nat_t())
}
fn multiply(left: Json, right: Json) -> Json {
    prim("multiply", vec![left, right], nat_t())
}
fn nat_in(item: Json, items: Json) -> Json {
    gf("natIn", vec![item, items])
}
fn text_equal(left: Json, right: Json) -> Json {
    prim("equal", vec![left, right], bool_t())
}

/// A total match on a `TargetSyntax.Value`: the listed constructors take
/// their bodies, every other one `default`.
fn on_value(
    unused: &Unused,
    scrutinee: Json,
    cases: Vec<(&str, Vec<String>, Json)>,
    default: &Json,
) -> Json {
    const VALUES: [(&str, usize); 23] = [
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
    let branches = VALUES
        .iter()
        .map(|(constructor, fields)| {
            let name = format!("Value.{constructor}");
            match cases.iter().find(|(case, _, _)| case == constructor) {
                Some((_, binders, body)) => sa(&name, binders.clone(), body.clone()),
                None => sa(
                    &name,
                    (0..*fields).map(|_| unused.next()).collect(),
                    default.clone(),
                ),
            }
        })
        .collect();
    m(scrutinee, branches)
}

/// The constructors of `Fact`, with their field counts.
const FACTS: [(&str, usize); 6] = [
    ("inadmissible", 0),
    ("admitted", 2),
    ("atLeast", 2),
    ("reduces", 1),
    ("unresolved", 0),
    ("unproven", 0),
];

/// A total match on a `Fact`: the listed constructors take their bodies,
/// every other one `default`.
fn on_fact(
    unused: &Unused,
    scrutinee: Json,
    cases: Vec<(&str, Vec<String>, Json)>,
    default: &Json,
) -> Json {
    let branches = FACTS
        .iter()
        .map(|(constructor, fields)| {
            let name = format!("Fact.{constructor}");
            match cases.iter().find(|(case, _, _)| case == constructor) {
                Some((_, binders, body)) => a(&name, binders.clone(), body.clone()),
                None => a(
                    &name,
                    (0..*fields).map(|_| unused.next()).collect(),
                    default.clone(),
                ),
            }
        })
        .collect();
    m(scrutinee, branches)
}

/// A match on a `Gnaf.Status` with one body per constructor.
fn on_status(
    scrutinee: Json,
    admitted: (Vec<String>, Json),
    inadmissible: Json,
    unresolved: Json,
) -> Json {
    m(
        scrutinee,
        vec![
            ga("Status.admitted", admitted.0, admitted.1),
            ga("Status.inadmissible", Vec::new(), inadmissible),
            ga("Status.unresolved", Vec::new(), unresolved),
        ],
    )
}

/// `some` obligation.
fn open(obligation: &str, arguments: Vec<Json>) -> Json {
    some(
        t("Obligation"),
        c(&format!("Obligation.{obligation}"), arguments),
    )
}
fn closed() -> Json {
    none(t("Obligation"))
}
fn established(constructor: &str, arguments: Vec<Json>) -> Json {
    c(&format!("Established.{constructor}"), arguments)
}
fn incomplete(obligation: &str, arguments: Vec<Json>, established: Json) -> Json {
    c(
        "Verdict.incomplete",
        vec![
            c(&format!("Obligation.{obligation}"), arguments),
            established,
        ],
    )
}
fn refused(refusal: &str, arguments: Vec<Json>) -> Json {
    c(
        "Verdict.refused",
        vec![c(&format!("Refusal.{refusal}"), arguments)],
    )
}
fn certified(constructor: &str, arguments: Vec<Json>) -> Json {
    c(
        "Verdict.certified",
        vec![c(&format!("Certified.{constructor}"), arguments)],
    )
}
fn fact(constructor: &str, arguments: Vec<Json>) -> Json {
    c(&format!("Fact.{constructor}"), arguments)
}
fn two(left: Json, right: Json) -> Json {
    list(nat_t(), vec![left, right])
}

// --- types -------------------------------------------------------------------

fn types() -> Vec<Json> {
    let plain = |names: &[&'static str]| -> Vec<(&'static str, Vec<Json>)> {
        names.iter().map(|name| (*name, Vec::new())).collect()
    };
    vec![
        // §17.18: the closed transformation schemas.
        lx::inductive("Schema", plain(&SCHEMAS)),
        // What a certificate offers for one system of the universe.
        lx::inductive(
            "Premise",
            vec![
                ("excluded", vec![nat_t()]),
                ("paysAtLeast", vec![nats_t()]),
                ("measured", Vec::new()),
                ("reduced", vec![t("Schema")]),
            ],
        ),
        // §10.6: the representation-to-execution bridge.
        structure("Bridge", vec![("slope", nat_t()), ("offset", nat_t())]),
        lx::inductive(
            "Route",
            vec![
                ("lowerBound", vec![nat_t(), nats_t()]),
                ("frontierCover", vec![nats_t()]),
                ("reduction", Vec::new()),
                ("representation", vec![nat_t(), option_t(t("Bridge"))]),
                ("normality", vec![nat_t()]),
            ],
        ),
        structure(
            "Certificate",
            vec![
                ("verifier", string_t()),
                ("universe", string_t()),
                ("objective", g("Objective")),
                ("claim", g("ClaimClass")),
                ("scope", g("Scope")),
                ("route", t("Route")),
                ("premises", list_t(t("Premise"))),
            ],
        ),
        // What a premise establishes about one system, at the request's
        // fuel and every larger one.
        lx::inductive(
            "Fact",
            vec![
                ("inadmissible", Vec::new()),
                ("admitted", vec![nat_t(), nat_t()]),
                ("atLeast", vec![nat_t(), nat_t()]),
                ("reduces", vec![nat_t()]),
                ("unresolved", Vec::new()),
                ("unproven", Vec::new()),
            ],
        ),
        // A certificate that does not bind the request it is checked
        // against, or is not complete as a certificate.
        lx::inductive(
            "Refusal",
            vec![
                ("verifierMismatch", Vec::new()),
                ("requestRejected", vec![g("Rejection")]),
                ("noGrammar", Vec::new()),
                ("universeMismatch", Vec::new()),
                ("objectiveMismatch", Vec::new()),
                ("claimMismatch", Vec::new()),
                ("scopeMismatch", Vec::new()),
                ("routeIncompatible", Vec::new()),
                ("missingPremise", vec![nat_t()]),
                ("extraPremise", vec![nat_t()]),
            ],
        ),
        // The first proof obligation a certificate leaves open, in the
        // universe's expansion order.
        lx::inductive(
            "Obligation",
            vec![
                ("unresolvedMember", vec![nat_t()]),
                ("memberUnproven", vec![nat_t()]),
                ("memberBelowBound", vec![nat_t()]),
                ("attainerOmitted", vec![nat_t()]),
                ("attainerUnmeasured", vec![nat_t()]),
                ("attainerInadmissible", vec![nat_t()]),
                ("attainerAboveBound", vec![nat_t()]),
                ("attainersMalformed", Vec::new()),
                ("boundUnattained", Vec::new()),
                ("frontierMemberUnmeasured", vec![nat_t()]),
                ("frontierMemberDominated", vec![nat_t()]),
                ("frontierMemberOmitted", vec![nat_t()]),
                ("reductionIncomplete", vec![nat_t()]),
                ("noAdmittedMember", Vec::new()),
                ("bridgeMissing", Vec::new()),
                ("bridgeViolated", vec![nat_t()]),
                ("bridgeUnproved", vec![nat_t()]),
                ("normalOutOfRange", Vec::new()),
                ("normalNotMinimal", vec![nat_t()]),
                ("normalUnmeasured", Vec::new()),
                ("optimalityUnproved", Vec::new()),
                ("systemNotNormal", Vec::new()),
            ],
        ),
        // The strongest status an incomplete certificate establishes, none
        // of which is an optimality claim: the system's position is its
        // index in the universe's expansion.
        lx::inductive(
            "Established",
            vec![
                ("notCertified", Vec::new()),
                ("correctOnly", vec![nat_t(), nat_t(), nat_t()]),
                ("locallyNormalized", vec![nat_t()]),
                ("representationMinimal", vec![nat_t(), nat_t()]),
            ],
        ),
        // A certified result, as positions in the universe's expansion:
        // an optimum some of whose attainers are listed, or the complete
        // argmin or Pareto frontier.
        lx::inductive(
            "Certified",
            vec![
                ("optimum", vec![nats_t(), nat_t()]),
                ("argmin", vec![nats_t(), nat_t()]),
                ("frontier", vec![nats_t()]),
            ],
        ),
        lx::inductive(
            "Verdict",
            vec![
                ("refused", vec![t("Refusal")]),
                ("certified", vec![t("Certified")]),
                ("incomplete", vec![t("Obligation"), t("Established")]),
            ],
        ),
    ]
}

// --- positions and domains -----------------------------------------------------

/// Total lookups by index into lists of the universe, the domain, the
/// facts, and the sizes.
fn lookups() -> Vec<Json> {
    let at = |name: &str, element: Json| {
        recursive(
            "items",
            definition(
                name,
                vec![
                    p("items", list_t(element.clone())),
                    p("index", nat_t()),
                ],
                option_t(element.clone()),
                on_list(
                    var("items"),
                    none(element.clone()),
                    "head",
                    "rest",
                    on_nat(
                        var("index"),
                        some(element.clone(), var("head")),
                        f(name, vec![var("rest"), var("remaining")]),
                    ),
                ),
            ),
        )
    };
    vec![
        // The index of `selector` in `selectors`, counting from `next`.
        recursive(
            "selectors",
            definition(
                "positionOf",
                vec![
                    p("selectors", selectors_t()),
                    p("selector", g("Selector")),
                    p("next", nat_t()),
                ],
                option_t(nat_t()),
                on_list(
                    var("selectors"),
                    none(nat_t()),
                    "head",
                    "rest",
                    ite(
                        gf("selectorEq", vec![var("selector"), var("head")]),
                        some(nat_t(), var("next")),
                        f(
                            "positionOf",
                            vec![var("rest"), var("selector"), add(var("next"), nat(1))],
                        ),
                    ),
                ),
            ),
        ),
        at("selectorAt", g("Selector")),
        at("valueAt", value_t()),
        at("factAt", t("Fact")),
        at("natAt", nat_t()),
        // The domain arguments at `positions`, in order, counting the head
        // of `values` as position `index`: a position the walk passes
        // without finding, or one out of order, contributes nothing, so the
        // result is always a sublist of the domain.
        recursive(
            "values",
            definition(
                "pickFrom",
                vec![
                    p("values", list_t(value_t())),
                    p("index", nat_t()),
                    p("positions", nats_t()),
                ],
                list_t(value_t()),
                on_list(
                    var("values"),
                    nil(value_t()),
                    "head",
                    "rest",
                    on_list(
                        var("positions"),
                        nil(value_t()),
                        "wanted",
                        "later",
                        ite(
                            beq(var("wanted"), var("index")),
                            cons(
                                var("head"),
                                f(
                                    "pickFrom",
                                    vec![var("rest"), next("index"), var("later")],
                                ),
                            ),
                            f(
                                "pickFrom",
                                vec![var("rest"), next("index"), var("positions")],
                            ),
                        ),
                    ),
                ),
            ),
        ),
        // Strictly ascending from `lower`.
        recursive(
            "positions",
            definition(
                "ascending",
                vec![p("positions", nats_t()), p("lower", nat_t())],
                bool_t(),
                on_list(
                    var("positions"),
                    yes(),
                    "head",
                    "rest",
                    and(
                        ble(var("lower"), var("head")),
                        f("ascending", vec![var("rest"), next("head")]),
                    ),
                ),
            ),
        ),
        recursive(
            "positions",
            definition(
                "allBelow",
                vec![p("positions", nats_t()), p("bound", nat_t())],
                bool_t(),
                on_list(
                    var("positions"),
                    yes(),
                    "head",
                    "rest",
                    and(
                        blt(var("head"), var("bound")),
                        f("allBelow", vec![var("rest"), var("bound")]),
                    ),
                ),
            ),
        ),
    ]
}

/// The domain conditions the schemas' eligibility reads: every argument is
/// a list, and its length against a threshold.
fn domain_conditions(unused: &Unused) -> Vec<Json> {
    let per_value = |name: &str, unused: &Unused, test: Box<dyn Fn() -> Json>| {
        definition(
            name,
            vec![p("value", value_t()), p("limit", nat_t())],
            bool_t(),
            on_value(
                unused,
                var("value"),
                vec![("list", binders!["items"], test())],
                &no(),
            ),
        )
    };
    let over_domain = |name: &str, test: &str| {
        recursive(
            "values",
            definition(
                name,
                vec![p("values", list_t(value_t())), p("limit", nat_t())],
                bool_t(),
                on_list(
                    var("values"),
                    yes(),
                    "head",
                    "rest",
                    and(
                        f(test, vec![var("head"), var("limit")]),
                        f(name, vec![var("rest"), var("limit")]),
                    ),
                ),
            ),
        )
    };
    vec![
        per_value("isList", unused, Box::new(yes)),
        per_value(
            "isShorter",
            unused,
            Box::new(|| blt(length(var("items")), var("limit"))),
        ),
        per_value(
            "isLong",
            unused,
            Box::new(|| ble(var("limit"), length(var("items")))),
        ),
        over_domain("allLists", "isList"),
        over_domain("allShorter", "isShorter"),
        over_domain("allLong", "isLong"),
    ]
}

// --- schemas -------------------------------------------------------------------

fn always(_function: Json) -> Json {
    yes()
}
fn not_entry(function: Json) -> Json {
    not(beq(function, nat(0)))
}

/// The transformation schemas and their eligibility (§17.18): a dispatch is
/// rewritten to the fixed system of the plan it always runs on the request's
/// domain. Eligibility is a decision about the request and the plans, never
/// about costs.
#[allow(clippy::too_many_lines)]
fn schemas(unused: &Unused) -> Vec<Json> {
    let mut out = ExprFold {
        group: "ExpressionIsolation",
        expr: "exprIsolated",
        exprs: "exprsIsolated",
        arms: "armsIsolated",
        arm: "armIsolated",
        result: bool_t(),
        empty: yes(),
        combine: and,
        own: yes(),
        reference: always,
        call_reference: Some(not_entry),
        apply_own: Some(no()),
    }
    .declarations(unused);
    let fixed = |plan: Json| gc("Selector.fixed", vec![plan]);
    let selector_option = || option_t(g("Selector"));
    let on_dispatch = |selector: Json, small_large: &dyn Fn(&str, &str) -> Json, otherwise: Json| {
        m(
            selector,
            vec![
                ga("Selector.fixed", binders![unused.next()], otherwise),
                ga(
                    "Selector.dispatch",
                    binders![unused.next(), "small", "large"],
                    small_large("small", "large"),
                ),
            ],
        )
    };
    out.extend([
        // Every plan's code neither calls the entry nor applies a
        // closure, so a system's entry is the only code its plans cannot
        // tell apart from another's.
        recursive(
            "plans",
            definition(
                "plansIsolated",
                vec![p("plans", list_t(g("Plan")))],
                bool_t(),
                on_list(
                    var("plans"),
                    yes(),
                    "plan",
                    "rest",
                    and(
                        f(
                            "exprIsolated",
                            vec![project(project(var("plan"), "function"), "body")],
                        ),
                        f("plansIsolated", vec![var("rest")]),
                    ),
                ),
            ),
        ),
        // The system a schema rewrites a selector to, when it has one.
        definition(
            "rewrite",
            vec![p("schema", t("Schema")), p("selector", g("Selector"))],
            selector_option(),
            m(
                var("schema"),
                vec![
                    a(
                        "Schema.collapse",
                        Vec::new(),
                        on_dispatch(
                            var("selector"),
                            &|small, large| {
                                ite(
                                    beq(var(small), var(large)),
                                    some(g("Selector"), fixed(var(small))),
                                    none(g("Selector")),
                                )
                            },
                            none(g("Selector")),
                        ),
                    ),
                    a(
                        "Schema.specializeSmall",
                        Vec::new(),
                        on_dispatch(
                            var("selector"),
                            &|small, _| some(g("Selector"), fixed(var(small))),
                            none(g("Selector")),
                        ),
                    ),
                    a(
                        "Schema.specializeLarge",
                        Vec::new(),
                        on_dispatch(
                            var("selector"),
                            &|_, large| some(g("Selector"), fixed(var(large))),
                            none(g("Selector")),
                        ),
                    ),
                ],
            ),
        ),
        // Whether a schema applies to a selector on this request: the
        // plans are isolated from the entry, and on every domain argument
        // the dispatch runs the plan the rewritten system runs.
        definition(
            "eligible",
            vec![
                p("request", g("Request")),
                p("grammar", g("Grammar")),
                p("selector", g("Selector")),
                p("schema", t("Schema")),
            ],
            bool_t(),
            and(
                f("plansIsolated", vec![project(var("grammar"), "plans")]),
                m(
                    var("selector"),
                    vec![
                        ga("Selector.fixed", binders![unused.next()], no()),
                        ga(
                            "Selector.dispatch",
                            binders!["threshold", "small", "large"],
                            m(
                                var("schema"),
                                vec![
                                    a(
                                        "Schema.collapse",
                                        Vec::new(),
                                        and(
                                            beq(var("small"), var("large")),
                                            f(
                                                "allLists",
                                                vec![project(var("request"), "domain"), nat(0)],
                                            ),
                                        ),
                                    ),
                                    a(
                                        "Schema.specializeSmall",
                                        Vec::new(),
                                        f(
                                            "allShorter",
                                            vec![
                                                project(var("request"), "domain"),
                                                var("threshold"),
                                            ],
                                        ),
                                    ),
                                    a(
                                        "Schema.specializeLarge",
                                        Vec::new(),
                                        f(
                                            "allLong",
                                            vec![
                                                project(var("request"), "domain"),
                                                var("threshold"),
                                            ],
                                        ),
                                    ),
                                ],
                            ),
                        ),
                    ],
                ),
            ),
        ),
    ]);
    out
}

// --- facts ---------------------------------------------------------------------

/// The code a system can run and its preparation, as `Gnaf.status` adds
/// them to a system's steps.
fn statics() -> Vec<Json> {
    let system = || gf("realize", vec![var("grammar"), var("selector")]);
    let functions = || project(var("system"), "functions");
    let indices = || gf("reachable", vec![functions()]);
    vec![
        definition(
            "staticSize",
            vec![p("grammar", g("Grammar")), p("selector", g("Selector"))],
            nat_t(),
            let_in(
                "system",
                program_t(),
                system(),
                gf("reachableSize", vec![functions(), indices()]),
            ),
        ),
        definition(
            "staticCharge",
            vec![
                p("request", g("Request")),
                p("grammar", g("Grammar")),
                p("selector", g("Selector")),
            ],
            nat_t(),
            let_in(
                "system",
                program_t(),
                system(),
                multiply(
                    length(project(var("request"), "domain")),
                    gf(
                        "reachableCharge",
                        vec![
                            project(project(var("request"), "machine"), "actions"),
                            project(var("grammar"), "plans"),
                            indices(),
                        ],
                    ),
                ),
            ),
        ),
    ]
}

/// A schema improves a system when it applies, the rewritten system is a
/// system of the grammar's expansion by construction, and its code and
/// preparation are no larger (§6.3: improving adjacency). The step saving is
/// the schema's theorem, not a decision.
fn improvement() -> Vec<Json> {
    let size_of = |selector: &str| f("staticSize", vec![var("grammar"), var(selector)]);
    let charge_of = |selector: &str| {
        f(
            "staticCharge",
            vec![var("request"), var("grammar"), var(selector)],
        )
    };
    let improves = |schema: &str| {
        f(
            "improves",
            vec![
                var("request"),
                var("grammar"),
                var("selector"),
                c(&format!("Schema.{schema}"), Vec::new()),
            ],
        )
    };
    vec![
        definition(
            "improves",
            vec![
                p("request", g("Request")),
                p("grammar", g("Grammar")),
                p("selector", g("Selector")),
                p("schema", t("Schema")),
            ],
            bool_t(),
            on_option(
                f("rewrite", vec![var("schema"), var("selector")]),
                no(),
                "target",
                and(
                    f(
                        "eligible",
                        vec![
                            var("request"),
                            var("grammar"),
                            var("selector"),
                            var("schema"),
                        ],
                    ),
                    and(
                        ble(size_of("target"), size_of("selector")),
                        ble(charge_of("target"), charge_of("selector")),
                    ),
                ),
            ),
        ),
        // GNAF-normal: no schema improves the system (§6.4). Necessary for a
        // global optimum and never sufficient for one.
        definition(
            "isNormal",
            vec![
                p("request", g("Request")),
                p("grammar", g("Grammar")),
                p("selector", g("Selector")),
            ],
            bool_t(),
            and(
                not(improves("collapse")),
                and(
                    not(improves("specializeSmall")),
                    not(improves("specializeLarge")),
                ),
            ),
        ),
    ]
}

/// What one premise establishes about one system. The only definitions of
/// this module that evaluate a program.
#[allow(clippy::too_many_lines)]
fn facts(unused: &Unused) -> Vec<Json> {
    let request = |name: &str| project(var("request"), name);
    let fuel = || project(project(var("request"), "machine"), "fuel");
    let unproven = || fact("unproven", Vec::new());
    let cost_here = |steps: Json| {
        add(
            steps,
            f(
                "staticCharge",
                vec![var("request"), var("grammar"), var("selector")],
            ),
        )
    };
    let on_target = on_option(
        f(
            "positionOf",
            vec![var("selectors"), var("target"), nat(0)],
        ),
        unproven(),
        "targetPosition",
        ite(
            f(
                "improves",
                vec![
                    var("request"),
                    var("grammar"),
                    var("selector"),
                    var("schema"),
                ],
            ),
            fact("reduces", vec![var("targetPosition")]),
            unproven(),
        ),
    );
    let premise_body = m(
        var("premise"),
        vec![
            a(
                "Premise.excluded",
                binders!["position"],
                on_option(
                    f("valueAt", vec![request("domain"), var("position")]),
                    unproven(),
                    "argument",
                    on_status(
                        gf(
                            "statusOn",
                            vec![
                                var("system"),
                                request("reference"),
                                fuel(),
                                list(value_t(), vec![var("argument")]),
                            ],
                        ),
                        (binders![unused.next(), unused.next()], unproven()),
                        fact("inadmissible", Vec::new()),
                        unproven(),
                    ),
                ),
            ),
            a(
                "Premise.paysAtLeast",
                binders!["positions"],
                let_in(
                    "picked",
                    list_t(value_t()),
                    f(
                        "pickFrom",
                        vec![request("domain"), nat(0), var("positions")],
                    ),
                    ite(
                        and(
                            beq(length(var("picked")), length(var("positions"))),
                            f("ascending", vec![var("positions"), nat(0)]),
                        ),
                        on_status(
                            gf(
                                "statusOn",
                                vec![
                                    var("system"),
                                    request("reference"),
                                    fuel(),
                                    var("picked"),
                                ],
                            ),
                            (
                                binders!["steps", unused.next()],
                                fact(
                                    "atLeast",
                                    vec![
                                        cost_here(var("steps")),
                                        f(
                                            "staticSize",
                                            vec![var("grammar"), var("selector")],
                                        ),
                                    ],
                                ),
                            ),
                            unproven(),
                            unproven(),
                        ),
                        unproven(),
                    ),
                ),
            ),
            a(
                "Premise.measured",
                Vec::new(),
                on_status(
                    gf(
                        "status",
                        vec![var("request"), var("grammar"), var("selector")],
                    ),
                    (
                        binders!["cost", "size"],
                        fact("admitted", vec![var("cost"), var("size")]),
                    ),
                    fact("inadmissible", Vec::new()),
                    fact("unresolved", Vec::new()),
                ),
            ),
            a(
                "Premise.reduced",
                binders!["schema"],
                on_option(
                    f("rewrite", vec![var("schema"), var("selector")]),
                    unproven(),
                    "target",
                    on_target,
                ),
            ),
        ],
    );
    vec![
        axioms(
            &RUN_AXIOMS,
            definition(
                "premiseFact",
                vec![
                    p("request", g("Request")),
                    p("grammar", g("Grammar")),
                    p("selectors", selectors_t()),
                    p("selector", g("Selector")),
                    p("premise", t("Premise")),
                ],
                t("Fact"),
                let_in(
                    "system",
                    program_t(),
                    gf("realize", vec![var("grammar"), var("selector")]),
                    premise_body,
                ),
            ),
        ),
        // One fact per system of the universe, in expansion order; a
        // system with no premise left proves nothing.
        axioms(
            &RUN_AXIOMS,
            recursive(
                "selectors",
                definition(
                    "factsOf",
                    vec![
                        p("request", g("Request")),
                        p("grammar", g("Grammar")),
                        p("all", selectors_t()),
                        p("selectors", selectors_t()),
                        p("premises", list_t(t("Premise"))),
                    ],
                    facts_t(),
                    on_list(
                        var("selectors"),
                        nil(t("Fact")),
                        "selector",
                        "rest",
                        on_list(
                            var("premises"),
                            cons(
                                unproven(),
                                f(
                                    "factsOf",
                                    vec![
                                        var("request"),
                                        var("grammar"),
                                        var("all"),
                                        var("rest"),
                                        var("premises"),
                                    ],
                                ),
                            ),
                            "premise",
                            "later",
                            cons(
                                f(
                                    "premiseFact",
                                    vec![
                                        var("request"),
                                        var("grammar"),
                                        var("all"),
                                        var("selector"),
                                        var("premise"),
                                    ],
                                ),
                                f(
                                    "factsOf",
                                    vec![
                                        var("request"),
                                        var("grammar"),
                                        var("all"),
                                        var("rest"),
                                        var("later"),
                                    ],
                                ),
                            ),
                        ),
                    ),
                ),
            ),
        ),
        // The static size of every system, in expansion order.
        recursive(
            "selectors",
            definition(
                "sizesOf",
                vec![p("grammar", g("Grammar")), p("selectors", selectors_t())],
                nats_t(),
                on_list(
                    var("selectors"),
                    nil(nat_t()),
                    "selector",
                    "rest",
                    cons(
                        f("staticSize", vec![var("grammar"), var("selector")]),
                        f("sizesOf", vec![var("grammar"), var("rest")]),
                    ),
                ),
            ),
        ),
        // The admitted systems as rows of the order (position, [steps,
        // size]), and those of them that are listed members.
        recursive(
            "facts",
            definition(
                "factRows",
                vec![p("facts", facts_t()), p("position", nat_t())],
                rows_t(),
                on_list(
                    var("facts"),
                    nil(row_t()),
                    "head",
                    "rest",
                    on_fact(
                        unused,
                        var("head"),
                        vec![(
                            "admitted",
                            binders!["cost", "size"],
                            cons(
                                pair(var("position"), two(var("cost"), var("size"))),
                                f("factRows", vec![var("rest"), next("position")]),
                            ),
                        )],
                        &f("factRows", vec![var("rest"), next("position")]),
                    ),
                ),
            ),
        ),
        recursive(
            "rows",
            definition(
                "rowsOfMembers",
                vec![p("rows", rows_t()), p("members", nats_t())],
                rows_t(),
                on_list(
                    var("rows"),
                    nil(row_t()),
                    "row",
                    "rest",
                    ite(
                        nat_in(first(var("row")), var("members")),
                        cons(
                            var("row"),
                            f("rowsOfMembers", vec![var("rest"), var("members")]),
                        ),
                        f("rowsOfMembers", vec![var("rest"), var("members")]),
                    ),
                ),
            ),
        ),
    ]
}

// --- routes --------------------------------------------------------------------

/// The first position whose fact is not good enough for a lower bound:
/// the system listed as an attainer reaches the bound, every other is at or,
/// for an argmin, strictly above it, is excluded, or reduces to a
/// cheaper system.
#[allow(clippy::too_many_lines)]
fn lower_bound(unused: &Unused) -> Vec<Json> {
    let position = || var("position");
    let attained = {
        let a1 = ite(
            beq(var("cost"), var("bound")),
            closed(),
            ite(
                blt(var("cost"), var("bound")),
                open("memberBelowBound", vec![position()]),
                open("attainerAboveBound", vec![position()]),
            ),
        );
        let other = ite(
            blt(var("cost"), var("bound")),
            open("memberBelowBound", vec![position()]),
            ite(
                and(var("strict"), beq(var("cost"), var("bound"))),
                open("attainerOmitted", vec![position()]),
                closed(),
            ),
        );
        ite(var("attainer"), a1, other)
    };
    let at_least = ite(
        var("attainer"),
        open("attainerUnmeasured", vec![position()]),
        ite(
            var("strict"),
            ite(
                blt(var("bound"), var("cost")),
                closed(),
                open("memberUnproven", vec![position()]),
            ),
            ite(
                ble(var("bound"), var("cost")),
                closed(),
                open("memberUnproven", vec![position()]),
            ),
        ),
    );
    let reduces = ite(
        var("attainer"),
        open("attainerUnmeasured", vec![position()]),
        closed(),
    );
    let position_check = on_fact(
        unused,
        var("fact"),
        vec![
            (
                "inadmissible",
                Vec::new(),
                ite(
                    var("attainer"),
                    open("attainerInadmissible", vec![position()]),
                    closed(),
                ),
            ),
            ("admitted", binders!["cost", unused.next()], attained),
            ("atLeast", binders!["cost", unused.next()], at_least),
            ("reduces", binders![unused.next()], reduces),
            (
                "unresolved",
                Vec::new(),
                open("unresolvedMember", vec![position()]),
            ),
            (
                "unproven",
                Vec::new(),
                open("memberUnproven", vec![position()]),
            ),
        ],
        &closed(),
    );
    let scan_arguments = |facts: Json, position: Json| {
        vec![
            var("strict"),
            var("bound"),
            var("attainers"),
            facts,
            position,
        ]
    };
    let scan = recursive(
        "facts",
        definition(
            "scanLower",
            vec![
                p("strict", bool_t()),
                p("bound", nat_t()),
                p("attainers", nats_t()),
                p("facts", facts_t()),
                p("position", nat_t()),
            ],
            option_t(t("Obligation")),
            on_list(
                var("facts"),
                closed(),
                "head",
                "rest",
                on_option(
                    f(
                        "lowerPosition",
                        vec![
                            var("strict"),
                            var("bound"),
                            nat_in(var("position"), var("attainers")),
                            var("head"),
                            var("position"),
                        ],
                    ),
                    f("scanLower", scan_arguments(var("rest"), next("position"))),
                    "obligation",
                    some(t("Obligation"), var("obligation")),
                ),
            ),
        ),
    );
    vec![
        definition(
            "lowerPosition",
            vec![
                p("strict", bool_t()),
                p("bound", nat_t()),
                p("attainer", bool_t()),
                p("fact", t("Fact")),
                p("position", nat_t()),
            ],
            option_t(t("Obligation")),
            position_check,
        ),
        scan,
        // What a failed certificate still establishes: the first listed
        // attainer is correct at its cost, when its fact says so.
        definition(
            "establishedFrom",
            vec![p("facts", facts_t()), p("attainers", nats_t())],
            t("Established"),
            on_list(
                var("attainers"),
                established("notCertified", Vec::new()),
                "head",
                unused.next(),
                on_option(
                    f("factAt", vec![var("facts"), var("head")]),
                    established("notCertified", Vec::new()),
                    "found",
                    on_fact(
                        unused,
                        var("found"),
                        vec![(
                            "admitted",
                            binders!["cost", "size"],
                            established(
                                "correctOnly",
                                vec![var("head"), var("cost"), var("size")],
                            ),
                        )],
                        &established("notCertified", Vec::new()),
                    ),
                ),
            ),
        ),
        // Theorem 11.1's route: a bound every system respects and a
        // system that attains it.
        definition(
            "lowerVerdict",
            vec![
                p("strict", bool_t()),
                p("bound", nat_t()),
                p("attainers", nats_t()),
                p("facts", facts_t()),
            ],
            t("Verdict"),
            ite(
                not(and(
                    f("ascending", vec![var("attainers"), nat(0)]),
                    f("allBelow", vec![var("attainers"), length(var("facts"))]),
                )),
                incomplete(
                    "attainersMalformed",
                    Vec::new(),
                    established("notCertified", Vec::new()),
                ),
                on_option(
                    f("scanLower", scan_arguments(var("facts"), nat(0))),
                    on_list(
                        var("attainers"),
                        incomplete(
                            "boundUnattained",
                            Vec::new(),
                            established("notCertified", Vec::new()),
                        ),
                        unused.next(),
                        unused.next(),
                        ite(
                            var("strict"),
                            certified("argmin", vec![var("attainers"), var("bound")]),
                            certified("optimum", vec![var("attainers"), var("bound")]),
                        ),
                    ),
                    "obligation",
                    c(
                        "Verdict.incomplete",
                        vec![
                            var("obligation"),
                            f("establishedFrom", vec![var("facts"), var("attainers")]),
                        ],
                    ),
                ),
            ),
        ),
    ]
}

/// Whether some member row strictly dominates `cost`.
fn dominated(cost: Json) -> Json {
    call_at(
        term::member(MODEL, "dominatedIn"),
        vec![nat_t()],
        vec![cost, var("memberRows")],
    )
}

/// The frontier route: the members are admitted and none dominates another,
/// and every other system is excluded, reduces, or is strictly dominated by a
/// member.
#[allow(clippy::too_many_lines)]
fn frontier(unused: &Unused) -> Vec<Json> {
    let position = || var("position");
    let cost = || two(var("cost"), var("size"));
    let as_member = on_fact(
        unused,
        var("fact"),
        vec![
            (
                "admitted",
                binders!["cost", "size"],
                ite(
                    dominated(cost()),
                    open("frontierMemberDominated", vec![position()]),
                    closed(),
                ),
            ),
            (
                "unresolved",
                Vec::new(),
                open("unresolvedMember", vec![position()]),
            ),
            (
                "unproven",
                Vec::new(),
                open("memberUnproven", vec![position()]),
            ),
        ],
        &open("frontierMemberUnmeasured", vec![position()]),
    );
    let as_other = on_fact(
        unused,
        var("fact"),
        vec![
            ("inadmissible", Vec::new(), closed()),
            (
                "admitted",
                binders!["cost", "size"],
                ite(
                    dominated(cost()),
                    closed(),
                    open("frontierMemberOmitted", vec![position()]),
                ),
            ),
            (
                "atLeast",
                binders!["cost", "size"],
                ite(
                    dominated(cost()),
                    closed(),
                    open("memberUnproven", vec![position()]),
                ),
            ),
            ("reduces", binders![unused.next()], closed()),
            (
                "unresolved",
                Vec::new(),
                open("unresolvedMember", vec![position()]),
            ),
            (
                "unproven",
                Vec::new(),
                open("memberUnproven", vec![position()]),
            ),
        ],
        &closed(),
    );
    let scan_arguments = |facts: Json, position: Json| {
        vec![
            var("members"),
            var("memberRows"),
            facts,
            position,
        ]
    };
    vec![
        definition(
            "frontierPosition",
            vec![
                p("member", bool_t()),
                p("memberRows", rows_t()),
                p("fact", t("Fact")),
                p("position", nat_t()),
            ],
            option_t(t("Obligation")),
            ite(var("member"), as_member, as_other),
        ),
        recursive(
            "facts",
            definition(
                "scanFrontier",
                vec![
                    p("members", nats_t()),
                    p("memberRows", rows_t()),
                    p("facts", facts_t()),
                    p("position", nat_t()),
                ],
                option_t(t("Obligation")),
                on_list(
                    var("facts"),
                    closed(),
                    "head",
                    "rest",
                    on_option(
                        f(
                            "frontierPosition",
                            vec![
                                nat_in(var("position"), var("members")),
                                var("memberRows"),
                                var("head"),
                                var("position"),
                            ],
                        ),
                        f(
                            "scanFrontier",
                            scan_arguments(var("rest"), next("position")),
                        ),
                        "obligation",
                        some(t("Obligation"), var("obligation")),
                    ),
                ),
            ),
        ),
        definition(
            "frontierVerdict",
            vec![p("members", nats_t()), p("facts", facts_t())],
            t("Verdict"),
            let_in(
                "memberRows",
                rows_t(),
                f(
                    "rowsOfMembers",
                    vec![
                        f("factRows", vec![var("facts"), nat(0)]),
                        var("members"),
                    ],
                ),
                ite(
                    not(and(
                        f("ascending", vec![var("members"), nat(0)]),
                        f("allBelow", vec![var("members"), length(var("facts"))]),
                    )),
                    incomplete(
                        "attainersMalformed",
                        Vec::new(),
                        established("notCertified", Vec::new()),
                    ),
                    on_option(
                        f("scanFrontier", scan_arguments(var("facts"), nat(0))),
                        on_list(
                            var("members"),
                            incomplete(
                                "noAdmittedMember",
                                Vec::new(),
                                established("notCertified", Vec::new()),
                            ),
                            unused.next(),
                            unused.next(),
                            certified("frontier", vec![var("members")]),
                        ),
                        "obligation",
                        c(
                            "Verdict.incomplete",
                            vec![
                                var("obligation"),
                                established("notCertified", Vec::new()),
                            ],
                        ),
                    ),
                ),
            ),
        ),
    ]
}

/// The reduction route: no premise but a measurement, an exclusion, or a
/// schema's reduction, and the answer is the order's own over the systems
/// that were measured.
fn reduction(unused: &Unused) -> Vec<Json> {
    let position = || var("position");
    let step = on_fact(
        unused,
        var("head"),
        vec![
            ("inadmissible", Vec::new(), closed()),
            ("admitted", binders![unused.next(), unused.next()], closed()),
            ("reduces", binders![unused.next()], closed()),
            (
                "unresolved",
                Vec::new(),
                open("unresolvedMember", vec![position()]),
            ),
        ],
        &open("reductionIncomplete", vec![position()]),
    );
    let none_admitted = incomplete(
        "noAdmittedMember",
        Vec::new(),
        established("notCertified", Vec::new()),
    );
    vec![
        recursive(
            "facts",
            definition(
                "scanReduction",
                vec![p("facts", facts_t()), p("position", nat_t())],
                option_t(t("Obligation")),
                on_list(
                    var("facts"),
                    closed(),
                    "head",
                    "rest",
                    on_option(
                        step,
                        f("scanReduction", vec![var("rest"), next("position")]),
                        "obligation",
                        some(t("Obligation"), var("obligation")),
                    ),
                ),
            ),
        ),
        definition(
            "reductionVerdict",
            vec![p("shape", nat_t()), p("facts", facts_t())],
            t("Verdict"),
            on_option(
                f("scanReduction", vec![var("facts"), nat(0)]),
                let_in(
                    "rows",
                    rows_t(),
                    f("factRows", vec![var("facts"), nat(0)]),
                    ite(
                        blt(var("shape"), nat(3)),
                        on_option(
                            call_at(
                                term::member(MODEL, "argmin"),
                                vec![nat_t()],
                                vec![gf("scalarRows", vec![var("rows")])],
                            ),
                            none_admitted.clone(),
                            "found",
                            f(
                                "lowerVerdict",
                                vec![
                                    beq(var("shape"), nat(2)),
                                    second(var("found")),
                                    first(var("found")),
                                    var("facts"),
                                ],
                            ),
                        ),
                        on_list(
                            var("rows"),
                            none_admitted,
                            unused.next(),
                            unused.next(),
                            f(
                                "frontierVerdict",
                                vec![
                                    call_at(
                                        term::member(MODEL, "frontier"),
                                        vec![nat_t()],
                                        vec![var("rows")],
                                    ),
                                    var("facts"),
                                ],
                            ),
                        ),
                    ),
                ),
                "obligation",
                c(
                    "Verdict.incomplete",
                    vec![
                        var("obligation"),
                        established("notCertified", Vec::new()),
                    ],
                ),
            ),
        ),
    ]
}

/// §10.6: a representation metric supports an execution-optimality claim
/// only through a bridge from it to the objective. `bound` is the bridge
/// applied to a system's size.
#[allow(clippy::too_many_lines)]
fn representation(unused: &Unused) -> Vec<Json> {
    let position = || var("position");
    let bridged = |size: Json| {
        add(
            multiply(project(var("bridge"), "slope"), size),
            project(var("bridge"), "offset"),
        )
    };
    let cost_check = ite(
        ble(bridged(var("measuredSize")), var("cost")),
        closed(),
        open("bridgeViolated", vec![position()]),
    );
    let step = on_fact(
        unused,
        var("fact"),
        vec![
            ("inadmissible", Vec::new(), closed()),
            ("admitted", binders!["cost", unused.next()], cost_check),
            (
                "atLeast",
                binders!["cost", unused.next()],
                ite(
                    ble(bridged(var("measuredSize")), var("cost")),
                    closed(),
                    open("bridgeUnproved", vec![position()]),
                ),
            ),
            (
                "reduces",
                binders![unused.next()],
                open("bridgeUnproved", vec![position()]),
            ),
            (
                "unresolved",
                Vec::new(),
                open("unresolvedMember", vec![position()]),
            ),
            (
                "unproven",
                Vec::new(),
                open("bridgeUnproved", vec![position()]),
            ),
        ],
        &closed(),
    );
    // The first system smaller than the normal one.
    let minimal = recursive(
        "sizes",
        definition(
            "firstSmaller",
            vec![
                p("sizes", nats_t()),
                p("limit", nat_t()),
                p("position", nat_t()),
            ],
            option_t(nat_t()),
            on_list(
                var("sizes"),
                none(nat_t()),
                "head",
                "rest",
                ite(
                    blt(var("head"), var("limit")),
                    some(nat_t(), position()),
                    f(
                        "firstSmaller",
                        vec![var("rest"), var("limit"), next("position")],
                    ),
                ),
            ),
        ),
    );
    let scan = recursive(
        "facts",
        definition(
            "scanBridge",
            vec![
                p("bridge", t("Bridge")),
                p("facts", facts_t()),
                p("sizes", nats_t()),
                p("position", nat_t()),
            ],
            option_t(t("Obligation")),
            on_list(
                var("facts"),
                closed(),
                "head",
                "rest",
                on_list(
                    var("sizes"),
                    open("bridgeUnproved", vec![position()]),
                    "measuredSize",
                    "laterSizes",
                    on_option(
                        f(
                            "bridgePosition",
                            vec![
                                var("bridge"),
                                var("head"),
                                var("measuredSize"),
                                position(),
                            ],
                        ),
                        f(
                            "scanBridge",
                            vec![
                                var("bridge"),
                                var("rest"),
                                var("laterSizes"),
                                next("position"),
                            ],
                        ),
                        "obligation",
                        some(t("Obligation"), var("obligation")),
                    ),
                ),
            ),
        ),
    );
    let attained = on_option(
        f("factAt", vec![var("facts"), var("normal")]),
        incomplete(
            "normalUnmeasured",
            Vec::new(),
            established("notCertified", Vec::new()),
        ),
        "found",
        on_fact(
            unused,
            var("found"),
            vec![(
                "admitted",
                binders!["cost", "size"],
                ite(
                    beq(var("cost"), var("limit")),
                    certified("optimum", vec![list(nat_t(), vec![var("normal")]), var("cost")]),
                    incomplete(
                        "boundUnattained",
                        Vec::new(),
                        established(
                            "correctOnly",
                            vec![var("normal"), var("cost"), var("size")],
                        ),
                    ),
                ),
            )],
            &incomplete(
                "normalUnmeasured",
                Vec::new(),
                established("notCertified", Vec::new()),
            ),
        ),
    );
    let with_bridge = on_option(
        f(
            "scanBridge",
            vec![var("bridge"), var("facts"), var("sizes"), nat(0)],
        ),
        let_in(
            "limit",
            nat_t(),
            add(
                multiply(project(var("bridge"), "slope"), var("normalSize")),
                project(var("bridge"), "offset"),
            ),
            attained,
        ),
        "obligation",
        c(
            "Verdict.incomplete",
            vec![
                var("obligation"),
                established("notCertified", Vec::new()),
            ],
        ),
    );
    let measured = on_option(
        f("natAt", vec![var("sizes"), var("normal")]),
        incomplete(
            "normalOutOfRange",
            Vec::new(),
            established("notCertified", Vec::new()),
        ),
        "normalSize",
        on_option(
            f(
                "firstSmaller",
                vec![var("sizes"), var("normalSize"), nat(0)],
            ),
            on_option(
                var("bridgeOption"),
                incomplete(
                    "bridgeMissing",
                    Vec::new(),
                    established(
                        "representationMinimal",
                        vec![var("normal"), var("normalSize")],
                    ),
                ),
                "given",
                // `bridge` names the bridge from here on.
                let_in("bridge", t("Bridge"), var("given"), with_bridge),
            ),
            "smaller",
            incomplete(
                "normalNotMinimal",
                vec![var("smaller")],
                established("notCertified", Vec::new()),
            ),
        ),
    );
    vec![
        definition(
            "bridgePosition",
            vec![
                p("bridge", t("Bridge")),
                p("fact", t("Fact")),
                p("measuredSize", nat_t()),
                p("position", nat_t()),
            ],
            option_t(t("Obligation")),
            step,
        ),
        minimal,
        scan,
        definition(
            "representationVerdict",
            vec![
                p("normal", nat_t()),
                p("bridgeOption", option_t(t("Bridge"))),
                p("facts", facts_t()),
                p("sizes", nats_t()),
            ],
            t("Verdict"),
            measured,
        ),
    ]
}

/// The normality route: a system's correctness and GNAF-normality are
/// facts; they are never an optimality claim.
fn normality(unused: &Unused) -> Vec<Json> {
    vec![definition(
        "normalityVerdict",
        vec![
            p("request", g("Request")),
            p("grammar", g("Grammar")),
            p("selectors", selectors_t()),
            p("normal", nat_t()),
            p("facts", facts_t()),
        ],
        t("Verdict"),
        on_option(
            f("factAt", vec![var("facts"), var("normal")]),
            incomplete(
                "normalUnmeasured",
                Vec::new(),
                established("notCertified", Vec::new()),
            ),
            "found",
            on_fact(
                unused,
                var("found"),
                vec![(
                    "admitted",
                    binders!["cost", "size"],
                    on_option(
                        f("selectorAt", vec![var("selectors"), var("normal")]),
                        incomplete(
                            "normalOutOfRange",
                            Vec::new(),
                            established("notCertified", Vec::new()),
                        ),
                        "selector",
                        ite(
                            f(
                                "isNormal",
                                vec![var("request"), var("grammar"), var("selector")],
                            ),
                            incomplete(
                                "optimalityUnproved",
                                Vec::new(),
                                established("locallyNormalized", vec![var("normal")]),
                            ),
                            incomplete(
                                "systemNotNormal",
                                Vec::new(),
                                established(
                                    "correctOnly",
                                    vec![var("normal"), var("cost"), var("size")],
                                ),
                            ),
                        ),
                    ),
                )],
                &incomplete(
                    "normalUnmeasured",
                    Vec::new(),
                    established("notCertified", Vec::new()),
                ),
            ),
        ),
    )]
}

// --- the checker -----------------------------------------------------------------

/// The claim classes a certificate can earn, as the shape of their result:
/// 1 `globalOptimal`, 2 `argminComplete`, 3 `paretoOptimal`, 4
/// `frontierComplete`; every other class is 0.
fn claim_shape(unused: &Unused) -> Json {
    let branches: Vec<Json> = claims()
        .into_iter()
        .map(|(name, fields)| {
            let shape = match name {
                "globalOptimal" => 1,
                "argminComplete" => 2,
                "paretoOptimal" => 3,
                "frontierComplete" => 4,
                _ => 0,
            };
            ga(
                &format!("ClaimClass.{name}"),
                (0..fields.len()).map(|_| unused.next()).collect(),
                nat(shape),
            )
        })
        .collect();
    definition(
        "claimShape",
        vec![p("claim", g("ClaimClass"))],
        nat_t(),
        m(var("claim"), branches),
    )
}

fn same_objective() -> Json {
    definition(
        "sameObjective",
        vec![p("left", g("Objective")), p("right", g("Objective"))],
        bool_t(),
        m(
            var("left"),
            vec![
                ga(
                    "Objective.scalar",
                    Vec::new(),
                    m(
                        var("right"),
                        vec![
                            ga("Objective.scalar", Vec::new(), yes()),
                            ga("Objective.vector", Vec::new(), no()),
                        ],
                    ),
                ),
                ga(
                    "Objective.vector",
                    Vec::new(),
                    m(
                        var("right"),
                        vec![
                            ga("Objective.scalar", Vec::new(), no()),
                            ga("Objective.vector", Vec::new(), yes()),
                        ],
                    ),
                ),
            ],
        ),
    )
}

fn same_scope() -> Json {
    let scopes = ["grammarUniverse", "calculusPrograms", "rustPrograms"];
    definition(
        "sameScope",
        vec![p("left", g("Scope")), p("right", g("Scope"))],
        bool_t(),
        m(
            var("left"),
            scopes
                .iter()
                .map(|left| {
                    ga(
                        &format!("Scope.{left}"),
                        Vec::new(),
                        m(
                            var("right"),
                            scopes
                                .iter()
                                .map(|right| {
                                    ga(
                                        &format!("Scope.{right}"),
                                        Vec::new(),
                                        boolean(left == right),
                                    )
                                })
                                .collect(),
                        ),
                    )
                })
                .collect(),
        ),
    )
}

/// The checker.
#[allow(clippy::too_many_lines)]
fn checker(unused: &Unused) -> Vec<Json> {
    let certificate = |name: &str| project(var("certificate"), name);
    let routed = {
        let shape = || var("shape");
        let incompatible = || refused("routeIncompatible", Vec::new());
        m(
            certificate("route"),
            vec![
                a(
                    "Route.lowerBound",
                    binders!["bound", "attainers"],
                    ite(
                        blt(shape(), nat(3)),
                        f(
                            "lowerVerdict",
                            vec![
                                beq(shape(), nat(2)),
                                var("bound"),
                                var("attainers"),
                                var("facts"),
                            ],
                        ),
                        incompatible(),
                    ),
                ),
                a(
                    "Route.frontierCover",
                    binders!["members"],
                    ite(
                        ble(nat(3), shape()),
                        f("frontierVerdict", vec![var("members"), var("facts")]),
                        incompatible(),
                    ),
                ),
                a(
                    "Route.reduction",
                    Vec::new(),
                    f("reductionVerdict", vec![shape(), var("facts")]),
                ),
                a(
                    "Route.representation",
                    binders!["normal", "bridge"],
                    ite(
                        beq(shape(), nat(1)),
                        f(
                            "representationVerdict",
                            vec![
                                var("normal"),
                                var("bridge"),
                                var("facts"),
                                f("sizesOf", vec![var("grammar"), var("selectors")]),
                            ],
                        ),
                        incompatible(),
                    ),
                ),
                a(
                    "Route.normality",
                    binders!["normal"],
                    f(
                        "normalityVerdict",
                        vec![
                            var("request"),
                            var("grammar"),
                            var("selectors"),
                            var("normal"),
                            var("facts"),
                        ],
                    ),
                ),
            ],
        )
    };
    // Each binding of the certificate to the request, in order, is a
    // refusal when it does not hold.
    let guarded = |holds: Json, mismatch: &str, rest: Json| {
        ite(not(holds), refused(mismatch, Vec::new()), rest)
    };
    let cover = let_in(
        "selectors",
        selectors_t(),
        gf("expand", vec![var("grammar")]),
        let_in(
            "count",
            nat_t(),
            length(var("selectors")),
            let_in(
                "given",
                nat_t(),
                length(certificate("premises")),
                ite(
                    blt(var("given"), var("count")),
                    refused("missingPremise", vec![var("given")]),
                    ite(
                        blt(var("count"), var("given")),
                        refused("extraPremise", vec![var("count")]),
                        let_in(
                            "facts",
                            facts_t(),
                            f(
                                "factsOf",
                                vec![
                                    var("request"),
                                    var("grammar"),
                                    var("selectors"),
                                    var("selectors"),
                                    certificate("premises"),
                                ],
                            ),
                            let_in(
                                "shape",
                                nat_t(),
                                f("claimShape", vec![certificate("claim")]),
                                routed,
                            ),
                        ),
                    ),
                ),
            ),
        ),
    );
    let bound = guarded(
        text_equal(certificate("universe"), project(var("request"), "universe")),
        "universeMismatch",
        guarded(
            f(
                "sameObjective",
                vec![certificate("objective"), project(var("request"), "objective")],
            ),
            "objectiveMismatch",
            guarded(
                and(
                    not(beq(f("claimShape", vec![certificate("claim")]), nat(0))),
                    beq(
                        f("claimShape", vec![certificate("claim")]),
                        f("claimShape", vec![project(var("request"), "claim")]),
                    ),
                ),
                "claimMismatch",
                guarded(
                    f(
                        "sameScope",
                        vec![certificate("scope"), project(var("request"), "scope")],
                    ),
                    "scopeMismatch",
                    cover,
                ),
            ),
        ),
    );
    let on_carrier = m(
        project(var("request"), "carrier"),
        vec![
            ga("Carrier.grammar", binders!["grammar"], bound),
            ga(
                "Carrier.internalPlans",
                Vec::new(),
                refused("noGrammar", Vec::new()),
            ),
            ga(
                "Carrier.optimizerOutput",
                Vec::new(),
                refused("noGrammar", Vec::new()),
            ),
            ga(
                "Carrier.discovered",
                binders![unused.next()],
                refused("noGrammar", Vec::new()),
            ),
            ga(
                "Carrier.cached",
                Vec::new(),
                refused("noGrammar", Vec::new()),
            ),
        ],
    );
    vec![
        claim_shape(unused),
        same_objective(),
        same_scope(),
        // The verdict a certificate earns for a request. The request must
        // be valid, the certificate must bind it, and the route's premises
        // cover every system of the request's universe.
        axioms(
            &RUN_AXIOMS,
            definition(
                "check",
                vec![
                    p("request", g("Request")),
                    p("certificate", t("Certificate")),
                ],
                t("Verdict"),
                ite(
                    not(text_equal(
                        certificate("verifier"),
                        lx::string(VERIFIER),
                    )),
                    refused("verifierMismatch", Vec::new()),
                    on_option(
                        gf("validate", vec![var("request")]),
                        on_carrier,
                        "rejection",
                        refused("requestRejected", vec![var("rejection")]),
                    ),
                ),
            ),
        ),
    ]
}

/// Every declaration of the module, in order.
fn declarations() -> Vec<Json> {
    let unused = Unused::new();
    let mut out = types();
    out.extend(lookups());
    out.extend(domain_conditions(&unused));
    out.extend(schemas(&unused));
    out.extend(statics());
    out.extend(improvement());
    out.extend(facts(&unused));
    out.extend(lower_bound(&unused));
    out.extend(frontier(&unused));
    out.extend(reduction(&unused));
    out.extend(representation(&unused));
    out.extend(normality(&unused));
    out.extend(checker(&unused));
    out
}

/// The `Certification` module.
#[must_use]
pub fn module() -> String {
    lx::module_tex(NAME, &[term::SYNTAX, term::SEMANTICS, MODEL], declarations())
}
