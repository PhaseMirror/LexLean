//! The hand-constructed GNAF certificates of SPEC.md §17.18, and the
//! generated `compiler` project files that state their verdicts to Lean.
//!
//! Every certificate is written here, by hand, against a GNAF request of
//! [`crate::gnaf`]; the premises an author offers are chosen from what the
//! host's trials say about each system, and the expected verdict is
//! computed by the host transcription `lexlean::gnaf::certification::check`.
//! The committed `compiler/certificates/<name>.json` files, the
//! `compiler/src/GnafCertificates` module, the `Certification` module it
//! imports ([`crate::certification_model`]), and the schemas are exactly
//! what [`files`] renders, which `cargo xtask check-calculus` enforces. In
//! the module, each certificate is a `Certification.Certificate` definition
//! with a theorem that the kernel reduces the checker's verdict on it and
//! its request to what the host computes, so the model, not the
//! transcription, is the oracle.

use std::collections::BTreeMap;

use lexlean::calculus::term;
use lexlean::gnaf::certification::{
    self as certification, Bridge, Certificate, CertificateFixture, Premise, Route, Schema, Trial,
    Verdict, CERTIFICATE_SPEC, FIXTURE_SPEC, VERIFIER,
};
use lexlean::gnaf::{self, Answer, ClaimClass, Request, Selector, Status};
use serde_json::{json, Value as Json};

/// The module stating every certificate's verdict.
pub const MODULE: &str = "GnafCertificates";

/// The directory of the committed certificate fixtures.
pub const DIRECTORY: &str = "compiler/certificates";

/// The path of the module, relative to the repository root.
pub const PATH: &str = "compiler/src/GnafCertificates.lex.tex";

/// One system of a request, as the host evaluates it.
struct System {
    selector: Selector,
    status: Status,
    trials: Vec<Trial>,
    charge: u64,
    size: u64,
}

fn systems(request: &Request) -> Vec<System> {
    gnaf::statuses(request)
        .expect("a committed request evaluates")
        .into_iter()
        .map(|(_, selector, status)| {
            let (trials, charge, size) =
                certification::trials(request, selector).expect("a committed request evaluates");
            System {
                selector,
                status,
                trials,
                charge,
                size,
            }
        })
        .collect()
}

/// The position of the first domain argument the system disagrees on.
fn first_disagreement(system: &System) -> Option<u64> {
    system
        .trials
        .iter()
        .position(|trial| *trial == Trial::Disagrees)
        .map(|position| position as u64)
}

/// The shortest prefix of the arguments a system agrees on whose steps, with
/// its preparation, satisfy `enough`: a lower bound without evaluating the
/// rest. `None` when no prefix does.
fn prefix(system: &System, enough: impl Fn(u64, u64) -> bool) -> Option<Premise> {
    let mut positions = Vec::new();
    let mut total = system.charge;
    if enough(total, system.size) {
        return Some(Premise::PaysAtLeast { positions });
    }
    for (position, trial) in system.trials.iter().enumerate() {
        if let Trial::Agrees { steps } = trial {
            positions.push(position as u64);
            total += steps;
            if enough(total, system.size) {
                return Some(Premise::PaysAtLeast { positions });
            }
        }
    }
    None
}

/// Every position the system agrees on.
fn all_agreeing(system: &System) -> Premise {
    Premise::PaysAtLeast {
        positions: system
            .trials
            .iter()
            .enumerate()
            .filter(|(_, trial)| matches!(trial, Trial::Agrees { .. }))
            .map(|(position, _)| position as u64)
            .collect(),
    }
}

/// A premise that offers no evidence: a lower bound over no argument.
const fn nothing() -> Premise {
    Premise::PaysAtLeast {
        positions: Vec::new(),
    }
}

/// A certificate for `request`, binding its universe, objective, claim, and
/// scope.
fn certificate(request: &Request, route: Route, premises: Vec<Premise>) -> Certificate {
    Certificate {
        spec: CERTIFICATE_SPEC.to_owned(),
        verifier: VERIFIER.to_owned(),
        universe: request.universe,
        objective: request.objective,
        claim: request.claim.clone(),
        scope: request.scope,
        route,
        premises,
    }
}

/// The premises of a lower-bound certificate: the attainers are measured,
/// every other system offers the least evidence that puts its cost above the
/// bound (at the bound, for an optimum that need not list its ties).
fn bounded(request: &Request, bound: u64, attainers: &[u64], strict: bool) -> Vec<Premise> {
    systems(request)
        .iter()
        .enumerate()
        .map(|(position, system)| {
            if attainers.contains(&(position as u64)) {
                return Premise::Measured;
            }
            let enough = |cost: u64, _size: u64| {
                if strict {
                    bound < cost
                } else {
                    bound <= cost
                }
            };
            match system.status {
                Status::Inadmissible => first_disagreement(system)
                    .map_or(Premise::Measured, |position| Premise::Excluded { position }),
                Status::Admitted { .. } | Status::Unresolved => {
                    prefix(system, enough).unwrap_or_else(|| match system.status {
                        Status::Admitted { .. } => Premise::Measured,
                        Status::Inadmissible | Status::Unresolved => all_agreeing(system),
                    })
                }
            }
        })
        .collect()
}

/// The premises of a frontier certificate: the members are measured, every
/// other system is excluded, or shown strictly dominated by a member from
/// the least evidence.
fn covered(request: &Request, members: &[u64]) -> Vec<Premise> {
    let all = systems(request);
    let member_costs: Vec<(u64, u64)> = members
        .iter()
        .filter_map(|member| match all[usize::try_from(*member).ok()?].status {
            Status::Admitted { steps, size } => Some((steps, size)),
            Status::Inadmissible | Status::Unresolved => None,
        })
        .collect();
    all.iter()
        .enumerate()
        .map(|(position, system)| {
            if members.contains(&(position as u64)) {
                return Premise::Measured;
            }
            let dominated = |cost: u64, size: u64| {
                member_costs.iter().any(|(steps, member_size)| {
                    *steps <= cost
                        && *member_size <= size
                        && (*steps < cost || *member_size < size)
                })
            };
            match system.status {
                Status::Inadmissible => first_disagreement(system)
                    .map_or(Premise::Measured, |position| Premise::Excluded { position }),
                Status::Admitted { .. } | Status::Unresolved => prefix(system, dominated)
                    .unwrap_or_else(|| match system.status {
                        Status::Admitted { .. } => Premise::Measured,
                        Status::Inadmissible | Status::Unresolved => all_agreeing(system),
                    }),
            }
        })
        .collect()
}

/// The premises of a reduction certificate: every system a schema improves is
/// reduced, every other is measured.
fn reduced(request: &Request) -> Vec<Premise> {
    systems(request)
        .iter()
        .map(|system| {
            certification::improving_schema(request, system.selector)
                .expect("a committed request evaluates")
                .map_or(Premise::Measured, |schema| Premise::Reduced { schema })
        })
        .collect()
}

/// Premises that offer no evidence for any system but the ones in `keep`,
/// which are measured: the route reads only those.
fn only_measured(request: &Request, keep: &[u64]) -> Vec<Premise> {
    systems(request)
        .iter()
        .enumerate()
        .map(|(position, _)| {
            if keep.contains(&(position as u64)) {
                Premise::Measured
            } else {
                nothing()
            }
        })
        .collect()
}

fn answer_members(request: &Request) -> (Vec<u64>, u64) {
    match gnaf::answer(request).expect("a committed request answers") {
        Answer::Argmin { members, steps } => (members, steps),
        Answer::Frontier { members } => (members, 0),
        Answer::Rejected { .. } | Answer::Infeasible | Answer::Incomplete => {
            panic!("a request with an optimum")
        }
    }
}

/// The position of the first system in `request` that satisfies `wanted`.
fn position_where(request: &Request, wanted: impl Fn(&System) -> bool) -> u64 {
    systems(request)
        .iter()
        .position(wanted)
        .expect("a system of the kind") as u64
}

fn dispatch(threshold: u64, small: u64, large: u64) -> Selector {
    Selector::Dispatch {
        threshold,
        small,
        large,
    }
}

/// The names of the GNAF fixtures the certificates are for.
const ARGMIN: &str = "argmin-over-systems";
const TIE: &str = "argmin-tie";
const UNRESOLVED: &str = "bound-over-unresolved";
const UNKNOWN: &str = "unknown-cost-incomplete";
const SHORT: &str = "internal-best-on-short-lists";
const VEC02: &str = crate::gnaf::PARETO_ENVELOPE;
const FRONTIER_TIE: &str = "frontier-tie";
const FRONTIER_PARETO: &str = "frontier-pareto-optimal";
const INFEASIBLE: &str = "infeasible-universe";
const REJECTED_SCOPE: &str = "reject-calculus-program-scope";

/// Every certificate, by name, with the name of its request.
#[allow(clippy::too_many_lines)]
fn certificates(requests: &BTreeMap<String, Request>) -> Vec<(&'static str, &'static str, Certificate)> {
    let request = |name: &str| &requests[name];
    let mut out: Vec<(&'static str, &'static str, Certificate)> = Vec::new();
    let mut add = |name: &'static str, request_name: &'static str, certificate: Certificate| {
        out.push((name, request_name, certificate));
    };

    // --- a universal lower bound and its attainment -----------------------------
    let argmin = request(ARGMIN);
    let (members, steps) = answer_members(argmin);
    add(
        "lb-argmin-over-systems",
        ARGMIN,
        certificate(
            argmin,
            Route::LowerBound {
                bound: steps,
                attainers: members.clone(),
            },
            bounded(argmin, steps, &members, false),
        ),
    );
    let complete = request("argmin-complete");
    let (complete_members, complete_steps) = answer_members(complete);
    add(
        "lb-argmin-complete",
        "argmin-complete",
        certificate(
            complete,
            Route::LowerBound {
                bound: complete_steps,
                attainers: complete_members.clone(),
            },
            bounded(complete, complete_steps, &complete_members, true),
        ),
    );
    let tie = request(TIE);
    let (tied, tied_steps) = answer_members(tie);
    add(
        "lb-argmin-tie",
        TIE,
        certificate(
            tie,
            Route::LowerBound {
                bound: tied_steps,
                attainers: tied.clone(),
            },
            bounded(tie, tied_steps, &tied, true),
        ),
    );
    // A tie with a member omitted from the attainers.
    add(
        "lb-tie-dropped",
        TIE,
        certificate(
            tie,
            Route::LowerBound {
                bound: tied_steps,
                attainers: tied[..1].to_vec(),
            },
            bounded(tie, tied_steps, &tied[..1], true),
        ),
    );
    // The bound is above what the best system pays.
    add(
        "lb-undercut",
        ARGMIN,
        certificate(
            argmin,
            Route::LowerBound {
                bound: steps + 13,
                attainers: members.clone(),
            },
            bounded(argmin, steps + 13, &members, false),
        ),
    );
    // The bound is below what any system attains.
    add(
        "lb-unattained",
        ARGMIN,
        certificate(
            argmin,
            Route::LowerBound {
                bound: steps - 1,
                attainers: members.clone(),
            },
            bounded(argmin, steps - 1, &members, false),
        ),
    );
    // One competitor's premise omitted.
    let mut premises = bounded(argmin, steps, &members, false);
    premises.pop();
    add(
        "lb-premise-omitted",
        ARGMIN,
        certificate(
            argmin,
            Route::LowerBound {
                bound: steps,
                attainers: members.clone(),
            },
            premises,
        ),
    );
    // One competitor's premise offering no evidence.
    let mut premises = bounded(argmin, steps, &members, false);
    premises[0] = nothing();
    add(
        "lb-premise-weakened",
        ARGMIN,
        certificate(
            argmin,
            Route::LowerBound {
                bound: steps,
                attainers: members.clone(),
            },
            premises,
        ),
    );
    // An extra premise.
    let mut premises = bounded(argmin, steps, &members, false);
    premises.push(Premise::Measured);
    add(
        "lb-extra-premise",
        ARGMIN,
        certificate(
            argmin,
            Route::LowerBound {
                bound: steps,
                attainers: members.clone(),
            },
            premises,
        ),
    );
    // The bound over systems whose cost is unknown at the request's fuel:
    // the answer by enumeration is incomplete.
    let unresolved = request(UNRESOLVED);
    let best = position_where(unresolved, |system| {
        matches!(system.status, Status::Admitted { .. })
            && system.selector == (Selector::Fixed { plan: 0 })
    });
    let best_cost = match systems(unresolved)[best as usize].status {
        Status::Admitted { steps, .. } => steps,
        Status::Inadmissible | Status::Unresolved => unreachable!("the best system is admitted"),
    };
    add(
        "lb-over-unresolved",
        UNRESOLVED,
        certificate(
            unresolved,
            Route::LowerBound {
                bound: best_cost,
                attainers: vec![best],
            },
            bounded(unresolved, best_cost, &[best], true),
        ),
    );
    // The same request, with the unresolved systems offering nothing.
    let mut premises = bounded(unresolved, best_cost, &[best], true);
    for (premise, system) in premises.iter_mut().zip(systems(unresolved)) {
        if system.status == Status::Unresolved {
            *premise = nothing();
        }
    }
    add(
        "lb-unresolved-unbounded",
        UNRESOLVED,
        certificate(
            unresolved,
            Route::LowerBound {
                bound: best_cost,
                attainers: vec![best],
            },
            premises,
        ),
    );
    // A diverging plan can never be shown above any bound by its prefix.
    let unknown = request(UNKNOWN);
    let known_best = position_where(unknown, |system| {
        matches!(system.status, Status::Admitted { .. })
    });
    let known_cost = match systems(unknown)[known_best as usize].status {
        Status::Admitted { steps, .. } => steps,
        Status::Inadmissible | Status::Unresolved => unreachable!("an admitted system"),
    };
    add(
        "lb-diverging-plan",
        UNKNOWN,
        certificate(
            unknown,
            Route::LowerBound {
                bound: known_cost,
                attainers: vec![known_best],
            },
            bounded(unknown, known_cost, &[known_best], false),
        ),
    );
    // An inadmissible system cannot attain a bound.
    let infeasible = request(INFEASIBLE);
    add(
        "lb-infeasible-attainer",
        INFEASIBLE,
        certificate(
            infeasible,
            Route::LowerBound {
                bound: 0,
                attainers: vec![0],
            },
            bounded(infeasible, 0, &[0], false),
        ),
    );
    // No system attains the bound.
    add(
        "lb-no-attainer",
        ARGMIN,
        certificate(
            argmin,
            Route::LowerBound {
                bound: steps,
                attainers: Vec::new(),
            },
            bounded(argmin, steps, &[], false),
        ),
    );

    // --- bindings -----------------------------------------------------------------
    let base = certificate(
        argmin,
        Route::LowerBound {
            bound: steps,
            attainers: members.clone(),
        },
        bounded(argmin, steps, &members, false),
    );
    add(
        "verifier-mismatch",
        ARGMIN,
        Certificate {
            verifier: "lexlean/gnaf-certification/0".to_owned(),
            ..base.clone()
        },
    );
    add(
        "claim-mismatch",
        ARGMIN,
        Certificate {
            claim: ClaimClass::ArgminComplete,
            ..base.clone()
        },
    );
    add(
        "scope-mismatch",
        ARGMIN,
        Certificate {
            scope: gnaf::Scope::CalculusPrograms,
            ..base.clone()
        },
    );
    add(
        "objective-mismatch",
        ARGMIN,
        Certificate {
            objective: gnaf::Objective::Vector,
            ..base.clone()
        },
    );
    add(
        "route-incompatible",
        ARGMIN,
        Certificate {
            route: Route::FrontierCover {
                members: members.clone(),
            },
            ..base.clone()
        },
    );
    // A certificate for a request the model refuses.
    add(
        "request-rejected",
        REJECTED_SCOPE,
        Certificate {
            scope: gnaf::Scope::CalculusPrograms,
            ..certificate(
                request(REJECTED_SCOPE),
                Route::Reduction,
                Vec::new(),
            )
        },
    );
    // A certificate that is for another universe: the same grammar and plans
    // with a larger machine.
    add(
        "reused-enlarged",
        "vec08-enlarged",
        Certificate {
            universe: argmin.universe,
            ..certificate(
                request("vec08-enlarged"),
                Route::LowerBound {
                    bound: steps,
                    attainers: members.clone(),
                },
                bounded(argmin, steps, &members, false),
            )
        },
    );

    // --- reduction -----------------------------------------------------------------
    let short = request(SHORT);
    add(
        "reduction-short-lists",
        SHORT,
        certificate(short, Route::Reduction, reduced(short)),
    );
    let mut premises = reduced(short);
    let reducible = position_where(short, |system| {
        certification::improving_schema(short, system.selector)
            .expect("evaluates")
            .is_some()
    });
    premises[reducible as usize] = all_agreeing(&systems(short)[reducible as usize]);
    add(
        "reduction-incomplete",
        SHORT,
        certificate(short, Route::Reduction, premises),
    );
    let mut premises = reduced(short);
    premises[0] = Premise::Reduced {
        schema: Schema::Collapse,
    };
    add(
        "reduction-ineligible",
        SHORT,
        certificate(short, Route::Reduction, premises),
    );
    // A reduction over a request whose dispatches the domain does not collapse
    // reduces only what the schemas allow.
    add(
        "reduction-mixed-domain",
        ARGMIN,
        certificate(argmin, Route::Reduction, reduced(argmin)),
    );

    // --- Pareto frontier ---------------------------------------------------------------
    let vec02 = request(VEC02);
    let (frontier_members, _) = answer_members(vec02);
    add(
        "frontier-vec02",
        VEC02,
        certificate(
            vec02,
            Route::FrontierCover {
                members: frontier_members.clone(),
            },
            covered(vec02, &frontier_members),
        ),
    );
    add(
        "frontier-pareto-optimal",
        FRONTIER_PARETO,
        certificate(
            request(FRONTIER_PARETO),
            Route::FrontierCover {
                members: answer_members(request(FRONTIER_PARETO)).0,
            },
            covered(
                request(FRONTIER_PARETO),
                &answer_members(request(FRONTIER_PARETO)).0,
            ),
        ),
    );
    let frontier_tie = request(FRONTIER_TIE);
    let tie_members = answer_members(frontier_tie).0;
    add(
        "frontier-tie",
        FRONTIER_TIE,
        certificate(
            frontier_tie,
            Route::FrontierCover {
                members: tie_members.clone(),
            },
            covered(frontier_tie, &tie_members),
        ),
    );
    add(
        "frontier-tie-dropped",
        FRONTIER_TIE,
        certificate(
            frontier_tie,
            Route::FrontierCover {
                members: tie_members[..tie_members.len() - 1].to_vec(),
            },
            covered(frontier_tie, &tie_members[..tie_members.len() - 1]),
        ),
    );
    add(
        "frontier-member-omitted",
        VEC02,
        certificate(
            vec02,
            Route::FrontierCover {
                members: frontier_members[1..].to_vec(),
            },
            covered(vec02, &frontier_members[1..]),
        ),
    );
    let outsider = position_where(vec02, |system| {
        matches!(system.status, Status::Admitted { .. })
    }) as usize;
    let mut with_dominated = frontier_members.clone();
    let dominated = (0..systems(vec02).len() as u64)
        .find(|position| !frontier_members.contains(position))
        .expect("a dominated system");
    with_dominated.push(dominated);
    with_dominated.sort_unstable();
    let _ = outsider;
    add(
        "frontier-dominated-member",
        VEC02,
        certificate(
            vec02,
            Route::FrontierCover {
                members: with_dominated.clone(),
            },
            covered(vec02, &with_dominated),
        ),
    );
    let mut premises = covered(vec02, &frontier_members);
    premises[dominated as usize] = nothing();
    add(
        "frontier-premise-weakened",
        VEC02,
        certificate(
            vec02,
            Route::FrontierCover {
                members: frontier_members.clone(),
            },
            premises,
        ),
    );

    // --- local normality is not optimality ---------------------------------------------
    // The dispatch from the long-list plan to the short-list plan is
    // GNAF-normal on a mixed domain and pays more than the optimum.
    let suboptimal = position_where(argmin, |system| system.selector == dispatch(3, 1, 0));
    add(
        "normal-not-global",
        ARGMIN,
        certificate(
            argmin,
            Route::Normality { system: suboptimal },
            only_measured(argmin, &[suboptimal]),
        ),
    );
    // The same system offered as the attainer of a bound it cannot meet.
    let suboptimal_cost = match systems(argmin)[suboptimal as usize].status {
        Status::Admitted { steps, .. } => steps,
        Status::Inadmissible | Status::Unresolved => unreachable!("an admitted system"),
    };
    add(
        "normal-claimed-global",
        ARGMIN,
        certificate(
            argmin,
            Route::LowerBound {
                bound: suboptimal_cost,
                attainers: vec![suboptimal],
            },
            bounded(argmin, suboptimal_cost, &[suboptimal], false),
        ),
    );
    // A system a schema improves is not normal.
    let improvable = position_where(argmin, |system| system.selector == dispatch(3, 0, 0));
    add(
        "normality-improvable",
        ARGMIN,
        certificate(
            argmin,
            Route::Normality { system: improvable },
            only_measured(argmin, &[improvable]),
        ),
    );
    // The optimum itself: normal, and still not certified by normality.
    add(
        "normality-optimum",
        ARGMIN,
        certificate(
            argmin,
            Route::Normality {
                system: members[0],
            },
            only_measured(argmin, &members),
        ),
    );
    out
}

/// The requests the certificates are for: the GNAF fixtures and the ones
/// written for certification.
fn requests() -> BTreeMap<String, Request> {
    crate::gnaf::fixtures()
        .into_iter()
        .map(|fixture| (fixture.name, fixture.request))
        .collect()
}

/// The bridge requests' certificates, over a request of straight-line plans:
/// the representation metric is the size of the code a system can run, and
/// its bridge to the step objective is an affine lower bound.
fn bridge_certificates(
    requests: &BTreeMap<String, Request>,
) -> Vec<(&'static str, &'static str, Certificate)> {
    const NAME: &str = "bridge-straight-line";
    let request = &requests[NAME];
    let all = systems(request);
    let smallest = position_where(request, |system| {
        all.iter().all(|other| system.size <= other.size)
    });
    let normal_cost = match all[smallest as usize].status {
        Status::Admitted { steps, .. } => steps,
        Status::Inadmissible | Status::Unresolved => unreachable!("an admitted system"),
    };
    let slope = request.domain.len() as u64;
    let tight = Bridge {
        slope,
        offset: normal_cost - slope * all[smallest as usize].size,
    };
    let represent = |normal: u64, bridge: Option<Bridge>| {
        certificate(
            request,
            Route::Representation { normal, bridge },
            all.iter().map(|_| Premise::Measured).collect(),
        )
    };
    let larger = position_where(request, |system| system.size > all[smallest as usize].size);
    vec![
        ("bridge-attained", NAME, represent(smallest, Some(tight))),
        ("bridge-missing", NAME, represent(smallest, None)),
        // A bridge that claims more than the minimal system pays.
        (
            "bridge-violated",
            NAME,
            represent(
                smallest,
                Some(Bridge {
                    slope,
                    offset: tight.offset + 11,
                }),
            ),
        ),
        // A valid bridge whose bound the minimal system does not meet.
        (
            "bridge-unattained",
            NAME,
            represent(
                smallest,
                Some(Bridge {
                    slope,
                    offset: tight.offset - 19,
                }),
            ),
        ),
        // The system is not the smallest representation.
        ("bridge-not-minimal", NAME, represent(larger, Some(tight))),
        // A bridge with nothing to attain it: the normal system is out of
        // range.
        (
            "bridge-out-of-range",
            NAME,
            represent(all.len() as u64, Some(tight)),
        ),
    ]
}

/// Every certificate fixture, sorted by name, its verdict computed by the
/// host.
///
/// # Panics
///
/// Panics on a certificate the host does not load, which no generated
/// certificate is.
#[must_use]
pub fn fixtures() -> Vec<CertificateFixture> {
    let requests = requests();
    let mut all = certificates(&requests);
    all.extend(bridge_certificates(&requests));
    let mut out: Vec<CertificateFixture> = all
        .into_iter()
        .map(|(name, request_name, certificate)| {
            let bytes = gnaf::to_file_bytes(&certificate);
            let certificate = certification::load(&bytes)
                .unwrap_or_else(|error| panic!("certificate {name}: {error:?}"));
            let expected = certification::check(&requests[request_name], &certificate)
                .unwrap_or_else(|error| panic!("certificate {name}: {error:?}"));
            CertificateFixture {
                spec: FIXTURE_SPEC.to_owned(),
                name: name.to_owned(),
                request: request_name.to_owned(),
                id: certification::certificate_id(&certificate)
                    .unwrap_or_else(|error| panic!("certificate {name}: {error:?}")),
                certificate,
                expected,
            }
        })
        .collect();
    out.sort_by(|left, right| left.name.cmp(&right.name));
    out
}

/// The declarations stating one fixture: the certificate, and that the
/// model's checker gives it and its request the host's verdict.
///
/// # Panics
///
/// Panics only if the fixture's request is unknown, which no generated
/// fixture's is.
#[must_use]
pub fn fixture_declarations(fixture: &CertificateFixture) -> Vec<Json> {
    let id = crate::calculus::identifier(&fixture.name);
    let certificate_name = format!("{id}Certificate");
    let request_id = crate::calculus::identifier(&fixture.request);
    let request_call = json!({"kind": "call",
        "function": term::member(crate::gnaf::FIXTURES_MODULE, &format!("{request_id}Request")),
        "arguments": []});
    vec![
        json!({"kind": "definition", "name": certificate_name, "parameters": [],
               "result": term::named(certification::MODEL, "Certificate"),
               "body": certification::certificate_term(&fixture.certificate)}),
        json!({"kind": "theorem", "name": format!("{id}Verdict"), "parameters": [],
               "axioms": crate::gnaf_model::RUN_AXIOMS,
               "statement": {"kind": "eq",
                   "left": {"kind": "call",
                            "function": term::member(certification::MODEL, "check"),
                            "arguments": [request_call,
                                          {"kind": "call", "function": {"name": certificate_name},
                                           "arguments": []}]},
                   "right": certification::verdict_term(&fixture.expected)},
               "proof": {"kind": "reflexivity"}}),
    ]
}

/// The `GnafCertificates` module.
///
/// # Panics
///
/// Panics only if `serde_json` cannot serialize a JSON value.
#[must_use]
pub fn module(fixtures: &[CertificateFixture]) -> String {
    let declarations: Vec<Json> = fixtures.iter().flat_map(fixture_declarations).collect();
    crate::lx::module_tex(
        MODULE,
        &[
            term::SYNTAX,
            term::SEMANTICS,
            gnaf::MODEL,
            certification::MODEL,
            crate::gnaf::FIXTURES_MODULE,
        ],
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
            format!("{DIRECTORY}/{}.json", fixture.name),
            gnaf::to_file_bytes(fixture),
        );
    }
    out.insert(PATH.to_owned(), module(&fixtures).into_bytes());
    for (path, schema) in schemas() {
        out.insert(path.to_owned(), gnaf::to_file_bytes(&schema));
    }
    out.insert(
        crate::certification_model::PATH.to_owned(),
        crate::certification_model::module().into_bytes(),
    );
    out
}

/// The definitions of a certificate, a verdict, and their parts.
#[allow(clippy::too_many_lines)]
fn definitions() -> serde_json::Map<String, Json> {
    use crate::gnaf::{closed, named_enum, natural};
    let shared = crate::gnaf::gnaf_definitions();
    let reference = |name: &str| json!({"$ref": format!("#/$defs/{name}")});
    let positions = || json!({"items": natural(), "type": "array"});
    let schema_names = named_enum(&["collapse", "specialize_small", "specialize_large"]);
    let mut out = serde_json::Map::new();
    for name in ["action_kind", "claim", "digest", "rejection"] {
        out.insert(name.to_owned(), shared[name].clone());
    }
    let at = |kind: &str| closed(json!({"kind": {"const": kind}, "position": natural()}));
    let simple = |kinds: &[&str]| closed(json!({"kind": named_enum(kinds)}));
    let entries = [
        (
            "premise",
            json!({"oneOf": [
                at("excluded"),
                closed(json!({"kind": {"const": "pays_at_least"}, "positions": positions()})),
                simple(&["measured"]),
                closed(json!({"kind": {"const": "reduced"}, "schema": schema_names})),
            ]}),
        ),
        ("bridge", closed(json!({"offset": natural(), "slope": natural()}))),
        (
            "route",
            json!({"oneOf": [
                closed(json!({"attainers": positions(), "bound": natural(),
                              "kind": {"const": "lower_bound"}})),
                closed(json!({"kind": {"const": "frontier_cover"}, "members": positions()})),
                simple(&["reduction"]),
                // The bridge is absent when the certificate offers none.
                json!({"additionalProperties": false,
                       "properties": {"bridge": reference("bridge"),
                                      "kind": {"const": "representation"},
                                      "normal": natural()},
                       "required": ["kind", "normal"], "type": "object"}),
                closed(json!({"kind": {"const": "normality"}, "system": natural()})),
            ]}),
        ),
        (
            "refusal",
            json!({"oneOf": [
                simple(&["verifier_mismatch", "no_grammar", "universe_mismatch",
                         "objective_mismatch", "claim_mismatch", "scope_mismatch",
                         "route_incompatible"]),
                closed(json!({"kind": {"const": "request_rejected"},
                              "rejection": reference("rejection")})),
                at("missing_premise"),
                closed(json!({"count": natural(), "kind": {"const": "extra_premise"}})),
            ]}),
        ),
        (
            "obligation",
            json!({"oneOf": [
                simple(&["attainers_malformed", "bound_unattained", "no_admitted_member",
                         "bridge_missing", "normal_out_of_range", "normal_unmeasured",
                         "optimality_unproved", "system_not_normal"]),
                closed(json!({"kind": named_enum(&["unresolved_member", "member_unproven",
                        "member_below_bound", "attainer_omitted", "attainer_unmeasured",
                        "attainer_inadmissible", "attainer_above_bound",
                        "frontier_member_unmeasured", "frontier_member_dominated",
                        "frontier_member_omitted", "reduction_incomplete", "bridge_violated",
                        "bridge_unproved", "normal_not_minimal"]),
                              "position": natural()})),
            ]}),
        ),
        (
            "established",
            json!({"oneOf": [
                simple(&["not_certified"]),
                closed(json!({"kind": {"const": "correct_only"}, "size": natural(),
                              "steps": natural(), "system": natural()})),
                closed(json!({"kind": {"const": "locally_normalized"}, "system": natural()})),
                closed(json!({"kind": {"const": "representation_minimal"}, "size": natural(),
                              "system": natural()})),
            ]}),
        ),
        (
            "certified",
            json!({"oneOf": [
                closed(json!({"kind": named_enum(&["optimum", "argmin"]),
                              "members": positions(), "steps": natural()})),
                closed(json!({"kind": {"const": "frontier"}, "members": positions()})),
            ]}),
        ),
        (
            "verdict",
            json!({"oneOf": [
                closed(json!({"kind": {"const": "refused"}, "refusal": reference("refusal")})),
                closed(json!({"kind": {"const": "certified"},
                              "result": reference("certified")})),
                closed(json!({"established": reference("established"),
                              "kind": {"const": "incomplete"},
                              "obligation": reference("obligation")})),
            ]}),
        ),
        ("certificate", certificate_object()),
    ];
    for (name, definition) in entries {
        out.insert(name.to_owned(), definition);
    }
    out
}

/// The certificate object, without the schema's own identification.
fn certificate_object() -> Json {
    use crate::gnaf::{closed, named_enum};
    let reference = |name: &str| json!({"$ref": format!("#/$defs/{name}")});
    closed(json!({
        "claim": reference("claim"),
        "objective": named_enum(&["scalar", "vector"]),
        "premises": {"items": reference("premise"), "type": "array"},
        "route": reference("route"),
        "scope": named_enum(&["grammar_universe", "calculus_programs", "rust_programs"]),
        "spec": {"const": CERTIFICATE_SPEC},
        "universe": reference("digest"),
        "verifier": {"type": "string"},
    }))
}

/// `schemas/gnaf-certificate.schema.json` and
/// `schemas/gnaf-certificate-fixture.schema.json`.
#[must_use]
pub fn schemas() -> [(&'static str, Json); 2] {
    use crate::gnaf::{closed, schema_id};
    let definitions = definitions();
    let identify = |mut schema: Json, name: &str, title: &str| {
        let object = schema.as_object_mut().expect("an object");
        object.insert("$id".to_owned(), json!(schema_id(name)));
        object.insert(
            "$schema".to_owned(),
            json!("https://json-schema.org/draft/2020-12/schema"),
        );
        object.insert("title".to_owned(), json!(title));
        schema
    };
    let mut certificate = identify(
        certificate_object(),
        "gnaf-certificate.schema.json",
        "LexLean GNAF certificate over a request (SPEC.md §17.18)",
    );
    let mut certificate_definitions = definitions.clone();
    for name in ["certificate", "refusal", "obligation", "established", "certified", "verdict"] {
        certificate_definitions.remove(name);
    }
    certificate["$defs"] = Json::Object(certificate_definitions);
    let mut fixture = identify(
        closed(json!({
            "certificate": {"$ref": "#/$defs/certificate"},
            "expected": {"$ref": "#/$defs/verdict"},
            "id": {"$ref": "#/$defs/digest"},
            "name": {"pattern": "^[a-z0-9]+(-[a-z0-9]+)*$", "type": "string"},
            "request": {"pattern": "^[a-z0-9]+(-[a-z0-9]+)*$", "type": "string"},
            "spec": {"const": FIXTURE_SPEC},
        })),
        "gnaf-certificate-fixture.schema.json",
        "LexLean GNAF certificate fixture (SPEC.md §17.18)",
    );
    fixture["$defs"] = Json::Object(definitions);
    [
        ("schemas/gnaf-certificate.schema.json", certificate),
        ("schemas/gnaf-certificate-fixture.schema.json", fixture),
    ]
}

/// The verdict a fixture is expected to have, for the conformance cases that
/// read one by name.
///
/// # Panics
///
/// Panics on a name no fixture has.
#[must_use]
pub fn expected(name: &str) -> Verdict {
    fixtures()
        .into_iter()
        .find(|fixture| fixture.name == name)
        .unwrap_or_else(|| panic!("no certificate fixture {name}"))
        .expected
}
