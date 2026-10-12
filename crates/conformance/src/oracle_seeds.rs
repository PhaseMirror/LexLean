//! Boundary seeds for the differential oracle (SPEC.md §17.19).
//!
//! The seeded inputs of the production differential find the disagreements a
//! random sample finds; they do not reach the edges where an independent
//! implementation of the same arithmetic is most likely to differ: `MIN / -1`
//! of a signed width, the arm of a match on a value no sample happened to
//! build, the split of a string into exactly as many fields as a limit allows.
//! This module derives those inputs from what the root states, with no
//! oracle and no Lean:
//!
//! - **types**: for each parameter type a pool of the values at its edges (the
//!   bounds of each integer width and their neighbours, the empty, one-, and
//!   several-element lists, strings and byte strings, every constructor of
//!   every option, result, and document type, each with the first values of
//!   its fields);
//! - **the program's own literals**: every number the lowered program states
//!   and its two neighbours, every string it states, and the bit widths of the
//!   fixed-width types it uses and their neighbours, so a comparison against
//!   a constant is reached from both sides;
//! - **combinations**: every pair of pool values of two parameters occurs
//!   together in some input (a greedy pairwise cover), the parameters of one
//!   type are also equal, and the strings are joined by the delimiters the
//!   program or its sibling parameters supply, one to six times.
//!
//! The inputs are run by the interpreter, which is the reference, and an input
//! that exhausts its fuel is left out. [`uncovered`] is the audit: it names
//! every constructor of a parameter's type that no input builds.

use std::collections::BTreeSet;

use lexlean::calculus::{Expr, OrderingValue, Outcome, Program, Value};
use lexlean::ir::semantic::SemanticType;
use lexlean::production::certificate::source_type;

use crate::differential::{first_of, sorted_unique, Case, Modules};

/// The fuel of one boundary input. An input that needs more is a recursion
/// over a number at its bound, which the interpreter cannot finish and no
/// generated program is asked to.
const FUEL: u64 = 20_000;

/// The most inputs one root gets from the combinations. The audit
/// ([`uncovered`]) fails when a cut loses a constructor.
const MOST: usize = 64;

/// The numbers and strings the lowered program states.
#[derive(Debug, Default)]
struct Literals {
    numbers: BTreeSet<i128>,
    strings: BTreeSet<String>,
    bytes: BTreeSet<String>,
}

fn harvest_value(value: &Value, into: &mut Literals) {
    match value {
        Value::Nat { value }
        | Value::Int { value }
        | Value::U8 { value }
        | Value::U16 { value }
        | Value::U32 { value }
        | Value::U64 { value }
        | Value::I8 { value }
        | Value::I16 { value }
        | Value::I32 { value }
        | Value::I64 { value } => {
            if let Ok(number) = value.parse::<i128>() {
                into.numbers.insert(number);
            }
        }
        Value::String { value } => {
            into.strings.insert(value.clone());
        }
        Value::Bytes { hex } => {
            into.bytes.insert(hex.clone());
        }
        Value::Some { value } | Value::Ok { value } | Value::Error { value } => {
            harvest_value(value, into);
        }
        Value::Pair { left, right } => {
            harvest_value(left, into);
            harvest_value(right, into);
        }
        Value::List { items } => items.iter().for_each(|item| harvest_value(item, into)),
        Value::Adt { fields, .. } => fields.iter().for_each(|item| harvest_value(item, into)),
        Value::Closure { captures, .. } => {
            captures.iter().for_each(|item| harvest_value(item, into));
        }
        Value::Unit | Value::Bool { .. } | Value::Ordering { .. } | Value::None => {}
    }
}

fn harvest_expr(expr: &Expr, into: &mut Literals) {
    match expr {
        Expr::Value { value, .. } => harvest_value(value, into),
        Expr::Var { .. } => {}
        Expr::Let { bound, body, .. } => {
            harvest_expr(bound, into);
            harvest_expr(body, into);
        }
        Expr::Cond {
            condition,
            then_branch,
            else_branch,
        } => {
            for part in [condition, then_branch, else_branch] {
                harvest_expr(part, into);
            }
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            harvest_expr(scrutinee, into);
            for arm in arms {
                harvest_expr(&arm.body, into);
            }
        }
        Expr::Build { operands, .. }
        | Expr::Call { operands, .. }
        | Expr::Prim { operands, .. } => operands.iter().for_each(|part| harvest_expr(part, into)),
        Expr::Closure { captures, .. } => captures.iter().for_each(|part| harvest_expr(part, into)),
        Expr::Apply { target, operands } => {
            harvest_expr(target, into);
            operands.iter().for_each(|part| harvest_expr(part, into));
        }
        Expr::First { value } | Expr::Second { value } | Expr::Field { value, .. } => {
            harvest_expr(value, into);
        }
    }
}

fn harvest(program: &Program) -> Literals {
    let mut literals = Literals::default();
    for function in &program.functions {
        harvest_expr(&function.body, &mut literals);
    }
    literals
}

/// The range and width of an integer type, and how to write a value of it.
struct Integer {
    min: i128,
    max: i128,
    make: fn(String) -> Value,
}

fn integer(ty: &SemanticType) -> Option<Integer> {
    let signed = |bits: u32, make: fn(String) -> Value| Integer {
        min: -(1_i128 << (bits - 1)),
        max: (1_i128 << (bits - 1)) - 1,
        make,
    };
    let unsigned = |bits: u32, make: fn(String) -> Value| Integer {
        min: 0,
        max: (1_i128 << bits) - 1,
        make,
    };
    Some(match ty {
        SemanticType::Nat => unsigned(64, |value| Value::Nat { value }),
        SemanticType::Int => signed(64, |value| Value::Int { value }),
        SemanticType::Int8 => signed(8, |value| Value::I8 { value }),
        SemanticType::Int16 => signed(16, |value| Value::I16 { value }),
        SemanticType::Int32 => signed(32, |value| Value::I32 { value }),
        SemanticType::Int64 => signed(64, |value| Value::I64 { value }),
        SemanticType::UInt8 => unsigned(8, |value| Value::U8 { value }),
        SemanticType::UInt16 => unsigned(16, |value| Value::U16 { value }),
        SemanticType::UInt32 => unsigned(32, |value| Value::U32 { value }),
        SemanticType::UInt64 => unsigned(64, |value| Value::U64 { value }),
        _ => return None,
    })
}

/// The bit width of a fixed-width type.
fn width(ty: &SemanticType) -> Option<i128> {
    match ty {
        SemanticType::Int8 | SemanticType::UInt8 => Some(8),
        SemanticType::Int16 | SemanticType::UInt16 => Some(16),
        SemanticType::Int32 | SemanticType::UInt32 => Some(32),
        SemanticType::Int64 | SemanticType::UInt64 => Some(64),
        _ => None,
    }
}

/// Every type a type is built from, itself included.
fn parts(
    modules: &Modules<'_>,
    ty: &SemanticType,
    seen: &mut BTreeSet<String>,
    into: &mut Vec<SemanticType>,
) {
    if !seen.insert(format!("{ty:?}")) {
        return;
    }
    into.push(ty.clone());
    match ty {
        SemanticType::Option { value: inner }
        | SemanticType::List { element: inner }
        | SemanticType::Set { element: inner } => parts(modules, inner, seen, into),
        SemanticType::Result { ok: a, error: b }
        | SemanticType::Product { left: a, right: b }
        | SemanticType::Map { key: a, value: b } => {
            parts(modules, a, seen, into);
            parts(modules, b, seen, into);
        }
        SemanticType::Named { .. } => {
            let (_, constructors) = source_type(modules, ty).expect("a document type");
            for constructor in constructors {
                for field in &constructor.fields {
                    parts(modules, field, seen, into);
                }
            }
        }
        _ => {}
    }
}

/// Whether values of `ty` have the order the sorted collections need.
fn orderable(ty: &SemanticType) -> bool {
    integer(ty).is_some()
        || match ty {
            SemanticType::String | SemanticType::Bool => true,
            SemanticType::Product { left, right } => orderable(left) && orderable(right),
            _ => false,
        }
}

struct Seeder<'a, 'm> {
    modules: &'a Modules<'m>,
    literals: Literals,
    /// The widths of the fixed-width types the root uses.
    widths: BTreeSet<i128>,
}

impl Seeder<'_, '_> {
    /// The numbers worth trying in `min..=max`, most informative first: zero,
    /// one, the bounds, minus one; the program's literals and their
    /// neighbours; the bit widths and their neighbours; the next-to-bounds.
    fn numbers(&self, min: i128, max: i128) -> Vec<i128> {
        let mut order = vec![0, 1, max, min, -1];
        for width in &self.widths {
            order.extend([*width, width - 1, width + 1]);
        }
        for literal in &self.literals.numbers {
            order.extend([*literal, literal + 1, literal - 1]);
        }
        order.extend([max - 1, min + 1, 2, 3]);
        let mut seen = BTreeSet::new();
        order
            .into_iter()
            .filter(|number| (min..=max).contains(number) && seen.insert(*number))
            .collect()
    }

    fn strings(&self) -> Vec<String> {
        let mut out: Vec<String> = ["", "a", "ab", "é", "a,b,c"]
            .iter()
            .map(|text| (*text).to_owned())
            .collect();
        out.extend(
            self.literals
                .strings
                .iter()
                .filter(|text| text.chars().count() <= 12)
                .take(4)
                .cloned(),
        );
        out.extend([" ".to_owned(), "Σa".to_owned()]);
        let mut seen = BTreeSet::new();
        out.retain(|text| seen.insert(text.clone()));
        out
    }

    fn bytes(&self) -> Vec<String> {
        let mut out: Vec<String> = ["", "00", "ff", "0001", "00ff0a", "0102030405"]
            .iter()
            .map(|text| (*text).to_owned())
            .collect();
        out.extend(self.literals.bytes.iter().take(2).cloned());
        let mut seen = BTreeSet::new();
        out.retain(|text| seen.insert(text.clone()));
        out
    }

    /// The pool of `ty`: at most `cap` values, the first of each shape first.
    fn pool(&self, ty: &SemanticType, depth: u32, cap: usize) -> Vec<Value> {
        let inner = cap.clamp(2, 5);
        let mut out = match ty {
            SemanticType::Unit => vec![Value::Unit],
            SemanticType::Bool => vec![false, true]
                .into_iter()
                .map(|value| Value::Bool { value })
                .collect(),
            SemanticType::Ordering => [OrderingValue::Lt, OrderingValue::Eq, OrderingValue::Gt]
                .into_iter()
                .map(|value| Value::Ordering { value })
                .collect(),
            SemanticType::String => self
                .strings()
                .into_iter()
                .map(|value| Value::String { value })
                .collect(),
            SemanticType::Bytes => self
                .bytes()
                .into_iter()
                .map(|hex| Value::Bytes { hex })
                .collect(),
            SemanticType::Option { value } => {
                let mut out = vec![Value::None];
                out.extend(self.pool(value, depth + 1, inner).into_iter().map(|item| {
                    Value::Some {
                        value: Box::new(item),
                    }
                }));
                out
            }
            SemanticType::Result { ok, error } => {
                let oks = self.pool(ok, depth + 1, inner);
                let errors = self.pool(error, depth + 1, inner);
                interleave(vec![
                    oks.into_iter()
                        .map(|item| Value::Ok {
                            value: Box::new(item),
                        })
                        .collect(),
                    errors
                        .into_iter()
                        .map(|item| Value::Error {
                            value: Box::new(item),
                        })
                        .collect(),
                ])
            }
            SemanticType::List { element } => {
                lists(&self.pool(element, depth + 1, inner), |items| items)
            }
            SemanticType::Set { element } => {
                let pool = self.pool(element, depth + 1, inner);
                if orderable(element) {
                    lists(&pool, |items| {
                        sorted_unique(items, |item| item)
                    })
                } else {
                    // Values without an order the collections can state are
                    // sets of one member, and the empty set.
                    std::iter::once(Value::List { items: Vec::new() })
                        .chain(pool.into_iter().map(|item| Value::List { items: vec![item] }))
                        .collect()
                }
            }
            SemanticType::Map { key, value } => {
                let keys = self.pool(key, depth + 1, inner);
                let values = self.pool(value, depth + 1, inner);
                let entry = |at: usize| Value::Pair {
                    left: Box::new(keys[at % keys.len()].clone()),
                    right: Box::new(values[at % values.len()].clone()),
                };
                let mut out = vec![Value::List { items: Vec::new() }];
                for count in [1, 2, 3, keys.len().min(5)] {
                    let items: Vec<Value> = (0..count).map(entry).collect();
                    let items = if orderable(key) {
                        sorted_unique(items, first_of)
                    } else {
                        items.into_iter().take(1).collect()
                    };
                    out.push(Value::List { items });
                }
                // A map of each key, alone.
                out.extend(keys.iter().enumerate().map(|(at, _)| Value::List {
                    items: vec![entry(at)],
                }));
                out
            }
            SemanticType::Product { left, right } => {
                let lefts = self.pool(left, depth + 1, inner);
                let rights = self.pool(right, depth + 1, inner);
                covering(&[lefts, rights], 24)
                    .into_iter()
                    .map(|mut pair| {
                        let right = pair.pop().expect("a right");
                        let left = pair.pop().expect("a left");
                        Value::Pair {
                            left: Box::new(left),
                            right: Box::new(right),
                        }
                    })
                    .collect()
            }
            SemanticType::ContractViolation | SemanticType::ReasoningFailure => {
                covering(
                    &[
                        self.pool(&SemanticType::Bool, depth + 1, 2),
                        self.pool(&SemanticType::Bool, depth + 1, 2),
                    ],
                    4,
                )
                .into_iter()
                .map(|mut pair| {
                    let right = pair.pop().expect("a right");
                    let left = pair.pop().expect("a left");
                    Value::Pair {
                        left: Box::new(left),
                        right: Box::new(right),
                    }
                })
                .collect()
            }
            SemanticType::Named { .. } => {
                let (_, constructors) = source_type(self.modules, ty).expect("a document type");
                // Past the depth bound the constructor with the fewest fields
                // ends the value, as the sampler does.
                let chosen: Vec<usize> = if depth >= 3 {
                    vec![constructors
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, constructor)| constructor.fields.len())
                        .map(|(index, _)| index)
                        .expect("a constructor")]
                } else {
                    (0..constructors.len()).collect()
                };
                interleave(
                    chosen
                        .into_iter()
                        .map(|index| {
                            let pools: Vec<Vec<Value>> = constructors[index]
                                .fields
                                .iter()
                                .map(|field| self.pool(field, depth + 1, inner.min(4)))
                                .collect();
                            covering(&pools, 12)
                                .into_iter()
                                .map(|fields| Value::Adt {
                                    constructor: index as u64,
                                    fields,
                                })
                                .collect()
                        })
                        .collect(),
                )
            }
            SemanticType::Function { .. }
            | SemanticType::Type
            | SemanticType::Prop
            | SemanticType::Parameter { .. } => panic!("{ty:?} is not a boundary type"),
            scalar => {
                let Integer { min, max, make } = integer(scalar).expect("an integer type");
                self.numbers(min, max)
                    .into_iter()
                    .map(|number| make(number.to_string()))
                    .collect()
            }
        };
        out.truncate(cap);
        out
    }
}

/// The lists the pool of a type's values gives: empty, each value alone, two
/// values in both orders, and a longer one with a repeat.
fn lists(pool: &[Value], finish: impl Fn(Vec<Value>) -> Vec<Value>) -> Vec<Value> {
    let at = |index: usize| pool[index % pool.len()].clone();
    let mut shapes: Vec<Vec<Value>> = vec![Vec::new()];
    shapes.extend((0..pool.len().min(4)).map(|index| vec![at(index)]));
    shapes.push(vec![at(0), at(1)]);
    shapes.push(vec![at(1), at(0)]);
    shapes.push(vec![at(0), at(1), at(2), at(3), at(0)]);
    shapes.push(vec![at(pool.len() - 1); 3]);
    let mut seen = BTreeSet::new();
    shapes
        .into_iter()
        .map(|items| Value::List {
            items: finish(items),
        })
        .filter(|value| seen.insert(value.clone()))
        .collect()
}

/// The lists taken one value at a time in turn: the first of each, then the
/// second of each.
fn interleave(lists: Vec<Vec<Value>>) -> Vec<Value> {
    let longest = lists.iter().map(Vec::len).max().unwrap_or(0);
    let mut out = Vec::new();
    for at in 0..longest {
        for list in &lists {
            if let Some(value) = list.get(at) {
                out.push(value.clone());
            }
        }
    }
    out
}

/// Tuples of the pools' values such that every pair of values of two pools
/// occurs together in one of them, found greedily and most informative first
/// (pairs of values early in their pools, which are the edges, before pairs of
/// later ones), and such that each value of each pool occurs at least once.
/// At most `limit` tuples, and never fewer than the singles.
fn covering(pools: &[Vec<Value>], limit: usize) -> Vec<Vec<Value>> {
    if pools.is_empty() {
        return vec![Vec::new()];
    }
    assert!(pools.iter().all(|pool| !pool.is_empty()), "empty pool");
    let base: Vec<usize> = pools.iter().map(|pool| 1.min(pool.len() - 1)).collect();
    let mut tuples: Vec<Vec<usize>> = Vec::new();
    for (position, pool) in pools.iter().enumerate() {
        for at in 0..pool.len() {
            let mut tuple = base.clone();
            tuple[position] = at;
            if !tuples.contains(&tuple) {
                tuples.push(tuple);
            }
        }
    }
    type Pair = (usize, usize, usize, usize);
    let mut covered: BTreeSet<Pair> = BTreeSet::new();
    let note = |tuple: &[usize], covered: &mut BTreeSet<Pair>| {
        let mut added = 0;
        for i in 0..tuple.len() {
            for j in i + 1..tuple.len() {
                added += usize::from(covered.insert((i, tuple[i], j, tuple[j])));
            }
        }
        added
    };
    for tuple in &tuples {
        note(tuple, &mut covered);
    }
    let mut pairs: Vec<Pair> = Vec::new();
    for i in 0..pools.len() {
        for j in i + 1..pools.len() {
            for a in 0..pools[i].len() {
                for b in 0..pools[j].len() {
                    pairs.push((i, a, j, b));
                }
            }
        }
    }
    pairs.sort_by_key(|(i, a, j, b)| (a + b, *i, *j, *a));
    let singles = tuples.len();
    for (i, a, j, b) in pairs {
        if tuples.len() >= limit.max(singles) {
            break;
        }
        if covered.contains(&(i, a, j, b)) {
            continue;
        }
        let mut tuple: Vec<Option<usize>> = vec![None; pools.len()];
        tuple[i] = Some(a);
        tuple[j] = Some(b);
        // Each other parameter takes the value that covers the most pairs not
        // yet covered with the values chosen so far.
        for k in 0..pools.len() {
            if tuple[k].is_some() {
                continue;
            }
            let mut best = (0, base[k]);
            for c in 0..pools[k].len() {
                let gain = tuple
                    .iter()
                    .enumerate()
                    .filter_map(|(m, at)| at.map(|at| (m, at)))
                    .filter(|(m, at)| {
                        let key = if *m < k { (*m, *at, k, c) } else { (k, c, *m, *at) };
                        !covered.contains(&key)
                    })
                    .count();
                if gain > best.0 {
                    best = (gain, c);
                }
            }
            tuple[k] = Some(best.1);
        }
        let tuple: Vec<usize> = tuple.into_iter().map(|at| at.expect("chosen")).collect();
        if note(&tuple, &mut covered) > 0 {
            tuples.push(tuple);
        }
    }
    tuples
        .into_iter()
        .map(|tuple| {
            tuple
                .iter()
                .zip(pools)
                .map(|(at, pool)| pool[*at].clone())
                .collect()
        })
        .collect()
}

/// The strings made of `count + 1` parts joined by `delimiter`.
fn joined(count: usize, delimiter: &str, empties: bool) -> String {
    const PARTS: [&str; 7] = ["a", "bb", "c", "d", "e", "f", "g"];
    (0..=count)
        .map(|at| {
            if empties && at % 2 == 1 {
                ""
            } else {
                PARTS[at % PARTS.len()]
            }
        })
        .collect::<Vec<_>>()
        .join(delimiter)
}

/// The inputs of a root: the combinations of the pools of its parameters, the
/// equal parameters, and the joined strings.
fn inputs(seeder: &Seeder<'_, '_>, parameters: &[SemanticType]) -> Vec<Vec<Value>> {
    let cap = match parameters.len() {
        0 | 1 => 14,
        2 => 8,
        3 => 6,
        _ => 5,
    };
    let pools: Vec<Vec<Value>> = parameters
        .iter()
        .map(|ty| seeder.pool(ty, 0, cap))
        .collect();
    let mut out = covering(&pools, MOST);
    // The parameters of one type are also equal.
    for i in 0..parameters.len() {
        for j in i + 1..parameters.len() {
            if parameters[i] != parameters[j] {
                continue;
            }
            for value in &pools[i] {
                let mut tuple: Vec<Value> = pools.iter().map(|pool| pool[1.min(pool.len() - 1)].clone()).collect();
                tuple[i] = value.clone();
                tuple[j] = value.clone();
                out.push(tuple);
            }
        }
    }
    // A string and the delimiter that splits it, one to six times.
    let mut delimiters: Vec<String> = seeder
        .literals
        .strings
        .iter()
        .filter(|text| (1..=3).contains(&text.chars().count()))
        .take(2)
        .cloned()
        .collect();
    for fallback in [",", "::"] {
        if !delimiters.iter().any(|text| text == fallback) {
            delimiters.push(fallback.to_owned());
        }
    }
    delimiters.truncate(2);
    let strings: Vec<usize> = parameters
        .iter()
        .enumerate()
        .filter(|(_, ty)| **ty == SemanticType::String)
        .map(|(at, _)| at)
        .collect();
    for &text in &strings {
        for &delimiter in &strings {
            if text >= delimiter {
                continue;
            }
            for d in &delimiters {
                for count in 0..=5 {
                    for empties in [false, true] {
                        if empties && !(2..=3).contains(&count) {
                            continue;
                        }
                        let mut tuple: Vec<Value> = pools
                            .iter()
                            .map(|pool| pool[1.min(pool.len() - 1)].clone())
                            .collect();
                        tuple[text] = Value::String {
                            value: joined(count, d, empties),
                        };
                        tuple[delimiter] = Value::String { value: d.clone() };
                        out.push(tuple);
                    }
                }
            }
        }
    }
    // A lone string parameter is joined by the literals' delimiters too.
    if strings.len() == 1 {
        for d in &delimiters {
            for count in 0..=5 {
                let mut tuple: Vec<Value> = pools
                    .iter()
                    .map(|pool| pool[1.min(pool.len() - 1)].clone())
                    .collect();
                tuple[strings[0]] = Value::String {
                    value: joined(count, d, false),
                };
                out.push(tuple);
            }
        }
    }
    out
}

/// The boundary cases of a root: the interpreter's outcome on each input of
/// [`inputs`] that is not already among `existing` and finishes within a small
/// fuel.
///
/// # Panics
///
/// Panics when an input does not check against the program, which is a defect
/// of this module.
#[must_use]
pub fn cases(
    modules: &Modules<'_>,
    root: &str,
    program: &Program,
    parameters: &[SemanticType],
    existing: &[Case],
) -> Vec<Case> {
    let mut widths = BTreeSet::new();
    for ty in parameters {
        let mut all = Vec::new();
        parts(modules, ty, &mut BTreeSet::new(), &mut all);
        widths.extend(all.iter().filter_map(width));
    }
    let seeder = Seeder {
        modules,
        literals: harvest(program),
        widths,
    };
    let mut known: BTreeSet<Vec<Value>> = existing.iter().map(|case| case.values.clone()).collect();
    let fresh: Vec<Vec<Value>> = inputs(&seeder, parameters)
        .into_iter()
        .filter(|values| known.insert(values.clone()))
        .collect();
    for values in &fresh {
        lexlean::calculus::check::check_arguments(program, 0, values)
            .unwrap_or_else(|reason| panic!("{root}: a boundary input does not check: {reason}"));
    }
    // One thread with the stack the recursion within the fuel needs runs them
    // all: a thread per input would cost more than the inputs do.
    let outcomes: Vec<Outcome> = std::thread::scope(|scope| {
        std::thread::Builder::new()
            .stack_size(1 << 29)
            .spawn_scoped(scope, || {
                fresh
                    .iter()
                    .map(|values| lexlean::calculus::interp::run(program, FUEL, 0, values))
                    .collect()
            })
            .expect("a thread")
            .join()
            .expect("the interpreter runs")
    });
    let mut out = Vec::new();
    for (values, outcome) in fresh.into_iter().zip(outcomes) {
        if !matches!(outcome, Outcome::Value { .. } | Outcome::Overflow { .. }) {
            continue;
        }
        out.push(Case {
            root: root.to_owned(),
            arguments: values
                .iter()
                .map(|value| serde_json::to_string(value).expect("a value serializes"))
                .collect(),
            values,
            outcome,
            function: 0,
            invalid: None,
        });
    }
    out
}

/// Whether `case` is one of the boundary inputs of [`cases`] and not one of
/// the production differential's: it states its arguments as calculus values
/// in JSON, which no Lean term it writes begins like.
#[must_use]
pub fn is_boundary(case: &Case) -> bool {
    case.arguments
        .first()
        .is_some_and(|argument| argument.starts_with("{\"kind\""))
}

/// The shape of each constructor a value of `ty` is built with, as
/// `type: constructor`.
fn built(
    modules: &Modules<'_>,
    ty: &SemanticType,
    value: &Value,
    into: &mut BTreeSet<String>,
    depth: usize,
) {
    if depth > 24 {
        return;
    }
    let key = format!("{ty:?}");
    match (ty, value) {
        (SemanticType::Bool, Value::Bool { value }) => {
            into.insert(format!("{key}: {value}"));
        }
        (SemanticType::Ordering, Value::Ordering { value }) => {
            into.insert(format!("{key}: {value:?}"));
        }
        (SemanticType::Option { .. }, Value::None) => {
            into.insert(format!("{key}: none"));
        }
        (SemanticType::Option { value: inner }, Value::Some { value }) => {
            into.insert(format!("{key}: some"));
            built(modules, inner, value, into, depth + 1);
        }
        (SemanticType::Result { ok, .. }, Value::Ok { value }) => {
            into.insert(format!("{key}: ok"));
            built(modules, ok, value, into, depth + 1);
        }
        (SemanticType::Result { error, .. }, Value::Error { value }) => {
            into.insert(format!("{key}: error"));
            built(modules, error, value, into, depth + 1);
        }
        (
            SemanticType::List { element: inner } | SemanticType::Set { element: inner },
            Value::List { items },
        ) => {
            into.insert(format!(
                "{key}: {}",
                match items.len() {
                    0 => "empty",
                    1 => "one",
                    _ => "several",
                }
            ));
            for item in items {
                built(modules, inner, item, into, depth + 1);
            }
        }
        (SemanticType::Map { key: k, value: v }, Value::List { items }) => {
            into.insert(format!(
                "{key}: {}",
                match items.len() {
                    0 => "empty",
                    1 => "one",
                    _ => "several",
                }
            ));
            for item in items {
                if let Value::Pair { left, right } = item {
                    built(modules, k, left, into, depth + 1);
                    built(modules, v, right, into, depth + 1);
                }
            }
        }
        (SemanticType::Product { left: l, right: r }, Value::Pair { left, right }) => {
            built(modules, l, left, into, depth + 1);
            built(modules, r, right, into, depth + 1);
        }
        (SemanticType::Named { .. }, Value::Adt {
            constructor,
            fields,
        }) => {
            into.insert(format!("{key}: constructor {constructor}"));
            let (_, constructors) = source_type(modules, ty).expect("a document type");
            for (field, item) in constructors[*constructor as usize].fields.iter().zip(fields) {
                built(modules, field, item, into, depth + 1);
            }
        }
        (SemanticType::String, Value::String { value }) => {
            into.insert(format!(
                "{key}: {}",
                if value.is_empty() { "empty" } else { "text" }
            ));
        }
        (SemanticType::Bytes, Value::Bytes { hex }) => {
            into.insert(format!(
                "{key}: {}",
                if hex.is_empty() { "empty" } else { "bytes" }
            ));
        }
        _ => {}
    }
}

/// Every shape a value of `ty` can be built with.
fn buildable(
    modules: &Modules<'_>,
    ty: &SemanticType,
    seen: &mut BTreeSet<String>,
    into: &mut BTreeSet<String>,
) {
    let key = format!("{ty:?}");
    if !seen.insert(key.clone()) {
        return;
    }
    match ty {
        SemanticType::Bool => {
            into.insert(format!("{key}: false"));
            into.insert(format!("{key}: true"));
        }
        SemanticType::Ordering => {
            for name in ["Lt", "Eq", "Gt"] {
                into.insert(format!("{key}: {name}"));
            }
        }
        SemanticType::Option { value } => {
            into.insert(format!("{key}: none"));
            into.insert(format!("{key}: some"));
            buildable(modules, value, seen, into);
        }
        SemanticType::Result { ok, error } => {
            into.insert(format!("{key}: ok"));
            into.insert(format!("{key}: error"));
            buildable(modules, ok, seen, into);
            buildable(modules, error, seen, into);
        }
        SemanticType::List { element } | SemanticType::Set { element } => {
            for name in ["empty", "one", "several"] {
                into.insert(format!("{key}: {name}"));
            }
            buildable(modules, element, seen, into);
        }
        SemanticType::Map { key: k, value } => {
            for name in ["empty", "one", "several"] {
                into.insert(format!("{key}: {name}"));
            }
            buildable(modules, k, seen, into);
            buildable(modules, value, seen, into);
        }
        SemanticType::Product { left, right } => {
            buildable(modules, left, seen, into);
            buildable(modules, right, seen, into);
        }
        SemanticType::Named { .. } => {
            let (_, constructors) = source_type(modules, ty).expect("a document type");
            for (index, constructor) in constructors.iter().enumerate() {
                into.insert(format!("{key}: constructor {index}"));
                for field in &constructor.fields {
                    buildable(modules, field, seen, into);
                }
            }
        }
        SemanticType::String => {
            into.insert(format!("{key}: empty"));
            into.insert(format!("{key}: text"));
        }
        SemanticType::Bytes => {
            into.insert(format!("{key}: empty"));
            into.insert(format!("{key}: bytes"));
        }
        _ => {}
    }
}

/// The constructors of the parameter types that no input builds: every arm of
/// every match over a parameter, by the type it matches. Empty when the
/// inputs reach every arm that can be reached from the types alone.
#[must_use]
pub fn uncovered(
    modules: &Modules<'_>,
    parameters: &[SemanticType],
    inputs: &[Vec<Value>],
) -> Vec<String> {
    let mut wanted = BTreeSet::new();
    let mut seen = BTreeSet::new();
    for ty in parameters {
        buildable(modules, ty, &mut seen, &mut wanted);
    }
    let mut got = BTreeSet::new();
    for values in inputs {
        for (ty, value) in parameters.iter().zip(values) {
            built(modules, ty, value, &mut got, 0);
        }
    }
    wanted.difference(&got).cloned().collect()
}
