//! The reference renderings of target programs to Rust (SPEC.md §17.14).
//!
//! A program renders to one safe Rust 2021 library crate in one of the two
//! machine profiles of §17.13. `rust-core` is `#![no_std]` and allocates
//! nothing: it admits only types whose values have a fixed size, so a
//! string, a byte string, a list, or a type that contains itself is refused.
//! `rust-std` shares every heap value behind `Rc`: a string, a byte string,
//! and each list cell are immutable shared buffers, and an ADT field or a
//! closure capture is boxed exactly where its type contains its owner. In
//! both, using a local clones a handle, never a structure, and building or
//! taking apart a list cell is constant work, so every evaluation step is
//! realized by work bounded by its charge. The runtime counts the work of
//! every loop it runs (`work`), and the conformance suite checks that count
//! against the denotation's steps. A rendered function never panics on a
//! valid program; fuel and stack depth are outside the observable contract.
//! Rendering refuses a program it cannot render faithfully.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write as _;

use super::check::Checker;
use super::{Arm, Expr, OrderingValue, Prim, Program, Shape, Ty, Value};

/// A machine profile of §17.13.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Profile {
    /// `rust-core`: the core library only, no heap allocation.
    Core,
    /// `rust-std`: the standard library, heap allocation.
    Std,
}

impl Profile {
    /// Both profiles.
    pub const ALL: [Self; 2] = [Self::Core, Self::Std];

    /// The target identifier of the production registry.
    #[must_use]
    pub const fn target(self) -> &'static str {
        match self {
            Self::Core => "rust-core",
            Self::Std => "rust-std",
        }
    }
}

/// The runtime both profiles carry: checked arithmetic, the scalar
/// equality and key order, and the work counter.
pub const CORE_RUNTIME: &str = r#"use core::cmp::Ordering;
use core::sync::atomic::{AtomicU64, Ordering as Memory};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Overflow;
pub type R<T> = Result<T, Overflow>;

static WORK: AtomicU64 = AtomicU64::new(0);
pub fn tick(units: u64) { WORK.fetch_add(units, Memory::Relaxed); }
pub fn work() -> u64 { WORK.load(Memory::Relaxed) }

pub fn nat_add(a: u64, b: u64) -> R<u64> { a.checked_add(b).ok_or(Overflow) }
pub fn nat_sub(a: u64, b: u64) -> R<u64> { Ok(a.saturating_sub(b)) }
pub fn nat_mul(a: u64, b: u64) -> R<u64> { a.checked_mul(b).ok_or(Overflow) }
pub fn nat_quot(a: u64, b: u64, z: u64) -> R<u64> { Ok(if b == 0 { z } else { a / b }) }
pub fn nat_rem(a: u64, b: u64, z: u64) -> R<u64> { Ok(if b == 0 { z } else { a % b }) }
pub fn nat_eq(a: u64, b: u64) -> R<bool> { Ok(a == b) }
pub fn nat_le(a: u64, b: u64) -> R<bool> { Ok(a <= b) }
pub fn nat_lt(a: u64, b: u64) -> R<bool> { Ok(a < b) }
pub fn nat_succ(a: u64) -> R<u64> { a.checked_add(1).ok_or(Overflow) }
pub fn int_add(a: i64, b: i64) -> R<i64> { a.checked_add(b).ok_or(Overflow) }
pub fn int_sub(a: i64, b: i64) -> R<i64> { a.checked_sub(b).ok_or(Overflow) }
pub fn int_mul(a: i64, b: i64) -> R<i64> { a.checked_mul(b).ok_or(Overflow) }
pub fn int_neg(a: i64) -> R<i64> { a.checked_neg().ok_or(Overflow) }
pub fn int_quot(a: i64, b: i64, z: i64) -> R<i64> { if b == 0 { Ok(z) } else { a.checked_div(b).ok_or(Overflow) } }
pub fn int_rem(a: i64, b: i64, z: i64) -> R<i64> { if b == 0 { Ok(z) } else { Ok(a.wrapping_rem(b)) } }
pub fn bool_not(a: bool) -> R<bool> { Ok(!a) }
pub fn bool_and(a: bool, b: bool) -> R<bool> { Ok(a && b) }
pub fn bool_or(a: bool, b: bool) -> R<bool> { Ok(a || b) }

pub trait Same { fn same(&self, other: &Self) -> bool; }
pub trait Key { fn key(&self, other: &Self) -> Ordering; }
macro_rules! scalar {
    ($($t:ty),*) => { $(
        impl Same for $t { fn same(&self, other: &Self) -> bool { self == other } }
        impl Key for $t { fn key(&self, other: &Self) -> Ordering { self.cmp(other) } }
    )* };
}
scalar!(bool, u8, u16, u32, u64, i8, i16, i32, i64);
impl Same for Ordering { fn same(&self, other: &Self) -> bool { self == other } }
impl<A: Key, B: Key> Key for (A, B) {
    fn key(&self, other: &Self) -> Ordering {
        match self.0.key(&other.0) { Ordering::Equal => self.1.key(&other.1), decided => decided }
    }
}
pub fn equal<T: Same>(a: T, b: T) -> R<bool> { Ok(a.same(&b)) }
pub fn compare<T: Key>(a: T, b: T) -> R<Ordering> { Ok(a.key(&b)) }

macro_rules! fixed {
    ($m:ident, $t:ident) => {
        pub mod $m {
            use super::R;
            pub fn checked_add(a: $t, b: $t) -> R<Option<$t>> { Ok(a.checked_add(b)) }
            pub fn checked_sub(a: $t, b: $t) -> R<Option<$t>> { Ok(a.checked_sub(b)) }
            pub fn checked_mul(a: $t, b: $t) -> R<Option<$t>> { Ok(a.checked_mul(b)) }
            pub fn checked_quot(a: $t, b: $t) -> R<Option<$t>> { Ok(a.checked_div(b)) }
            pub fn bit_and(a: $t, b: $t) -> R<$t> { Ok(a & b) }
            pub fn bit_or(a: $t, b: $t) -> R<$t> { Ok(a | b) }
            pub fn bit_xor(a: $t, b: $t) -> R<$t> { Ok(a ^ b) }
            pub fn bit_not(a: $t) -> R<$t> { Ok(!a) }
            pub fn shift_left(a: $t, amount: u32) -> R<Option<$t>> { Ok(if amount < $t::BITS { Some(a.wrapping_shl(amount)) } else { None }) }
            pub fn shift_right(a: $t, amount: u32) -> R<Option<$t>> { Ok(if amount < $t::BITS { Some(a.wrapping_shr(amount)) } else { None }) }
            pub fn convert(a: i128) -> R<Option<$t>> { Ok($t::try_from(a).ok()) }
        }
    };
}
fixed!(fixed_u8, u8); fixed!(fixed_u16, u16); fixed!(fixed_u32, u32); fixed!(fixed_u64, u64);
fixed!(fixed_i8, i8); fixed!(fixed_i16, i16); fixed!(fixed_i32, i32); fixed!(fixed_i64, i64);
pub fn checked_neg_i8(a: i8) -> R<Option<i8>> { Ok(a.checked_neg()) }
pub fn checked_neg_i16(a: i16) -> R<Option<i16>> { Ok(a.checked_neg()) }
pub fn checked_neg_i32(a: i32) -> R<Option<i32>> { Ok(a.checked_neg()) }
pub fn checked_neg_i64(a: i64) -> R<Option<i64>> { Ok(a.checked_neg()) }
"#;

/// The runtime only `rust-std` carries: shared strings, byte strings, and
/// persistent lists, and the primitives over them. Every loop ticks the
/// work counter once per iteration and a bulk copy ticks its length, except
/// the release of a list, whose cells were each counted when built.
pub const STD_RUNTIME: &str = r#"
#[derive(Clone)]
pub struct Str(std::rc::Rc<str>);
impl Str {
    pub fn lit(text: &str) -> Str { Str(std::rc::Rc::from(text)) }
    pub fn text(&self) -> &str { &self.0 }
}
#[derive(Clone)]
pub struct Bytes(std::rc::Rc<[u8]>);
impl Bytes {
    pub fn lit(octets: &[u8]) -> Bytes { Bytes(std::rc::Rc::from(octets)) }
    pub fn octets(&self) -> &[u8] { &self.0 }
}

pub struct Node<T> { head: T, tail: List<T> }
pub struct List<T>(Option<std::rc::Rc<Node<T>>>);
impl<T> Clone for List<T> { fn clone(&self) -> Self { List(self.0.clone()) } }
impl<T> Drop for List<T> {
    fn drop(&mut self) {
        let mut next = self.0.take();
        while let Some(cell) = next { // release
            match std::rc::Rc::try_unwrap(cell) {
                Ok(mut node) => next = node.tail.0.take(),
                Err(_) => break,
            }
        }
    }
}
impl<T: Clone> List<T> {
    pub fn nil() -> Self { List(None) }
    pub fn cons(head: T, tail: List<T>) -> Self { List(Some(std::rc::Rc::new(Node { head, tail }))) }
    pub fn uncons(&self) -> Option<(T, List<T>)> { self.0.as_ref().map(|node| (node.head.clone(), node.tail.clone())) }
    fn items(&self) -> Vec<T> {
        let mut out = Vec::new();
        let mut cursor = self.0.as_ref();
        while let Some(node) = cursor { tick(1); out.push(node.head.clone()); cursor = node.tail.0.as_ref(); }
        out
    }
    fn onto(items: Vec<T>, tail: List<T>) -> List<T> {
        let mut out = tail;
        for item in items.into_iter().rev() { tick(1); out = List::cons(item, out); }
        out
    }
}

fn chars(text: &str) -> u64 {
    let mut count = 0;
    for _ in text.chars() { tick(1); count += 1; }
    count
}

impl Same for Str {
    fn same(&self, other: &Self) -> bool {
        let mut left = self.0.chars();
        let mut right = other.0.chars();
        loop {
            tick(1);
            match (left.next(), right.next()) { (None, None) => return true, (a, b) if a != b => return false, _ => {} }
        }
    }
}
impl Same for Bytes {
    fn same(&self, other: &Self) -> bool { tick(1 + self.0.len().min(other.0.len()) as u64); self.0 == other.0 }
}
impl Key for Str {
    fn key(&self, other: &Self) -> Ordering {
        let mut left = self.0.chars();
        let mut right = other.0.chars();
        loop {
            tick(1);
            match (left.next(), right.next()) {
                (None, None) => return Ordering::Equal,
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some(a), Some(b)) => match a.cmp(&b) { Ordering::Equal => {} decided => return decided },
            }
        }
    }
}
impl<T: Key> Key for List<T> {
    fn key(&self, other: &Self) -> Ordering {
        let mut left = self.0.as_ref();
        let mut right = other.0.as_ref();
        loop {
            tick(1);
            match (left, right) {
                (None, None) => return Ordering::Equal,
                (None, Some(_)) => return Ordering::Less,
                (Some(_), None) => return Ordering::Greater,
                (Some(a), Some(b)) => match a.head.key(&b.head) {
                    Ordering::Equal => { left = a.tail.0.as_ref(); right = b.tail.0.as_ref(); }
                    decided => return decided,
                },
            }
        }
    }
}

pub fn append_list<T: Clone>(a: List<T>, b: List<T>) -> R<List<T>> { Ok(List::onto(a.items(), b)) }
pub fn append_bytes(a: Bytes, b: Bytes) -> R<Bytes> {
    tick((a.0.len() + b.0.len()) as u64);
    let mut out = Vec::with_capacity(a.0.len() + b.0.len());
    out.extend_from_slice(&a.0);
    out.extend_from_slice(&b.0);
    Ok(Bytes(std::rc::Rc::from(out)))
}
pub fn length_list<T>(a: List<T>) -> R<u64> {
    let mut count = 0;
    let mut cursor = a.0.as_ref();
    while let Some(node) = cursor { tick(1); count += 1; cursor = node.tail.0.as_ref(); }
    Ok(count)
}
pub fn length_bytes(a: Bytes) -> R<u64> { Ok(a.0.len() as u64) }
pub fn length_string(a: Str) -> R<u64> { Ok(chars(&a.0)) }
pub fn index_list<T: Clone>(a: List<T>, i: u64) -> R<Option<T>> {
    let mut position = 0;
    let mut cursor = a.0.as_ref();
    while let Some(node) = cursor {
        tick(1);
        if position == i { return Ok(Some(node.head.clone())); }
        position += 1;
        cursor = node.tail.0.as_ref();
    }
    Ok(None)
}
pub fn index_bytes(a: Bytes, i: u64) -> R<Option<u8>> { Ok(usize::try_from(i).ok().and_then(|i| a.0.get(i)).copied()) }
pub fn slice_list<T: Clone>(a: List<T>, start: u64, count: u64) -> R<Option<List<T>>> {
    let end = u128::from(start) + u128::from(count);
    let mut part = Vec::new();
    let mut position: u128 = 0;
    let mut cursor = a.0.as_ref();
    while let Some(node) = cursor {
        tick(1);
        if position >= end { break; }
        if position >= u128::from(start) { part.push(node.head.clone()); }
        position += 1;
        cursor = node.tail.0.as_ref();
    }
    Ok(if position < end { None } else { Some(List::onto(part, List::nil())) })
}
pub fn slice_bytes(a: Bytes, start: u64, count: u64) -> R<Option<Bytes>> {
    let end = u128::from(start) + u128::from(count);
    if end > a.0.len() as u128 { return Ok(None); }
    tick(count);
    Ok(Some(Bytes::lit(&a.0[start as usize..end as usize])))
}
pub fn utf8_encode(a: Str) -> R<Bytes> { tick(a.0.len() as u64); Ok(Bytes::lit(a.0.as_bytes())) }
pub fn utf8_decode(a: Bytes) -> R<Option<Str>> { tick(a.0.len() as u64); Ok(core::str::from_utf8(&a.0).ok().map(Str::lit)) }
pub fn compare_bytes(a: Bytes, b: Bytes) -> R<Ordering> { tick(1 + a.0.len().min(b.0.len()) as u64); Ok(a.0.cmp(&b.0)) }
pub fn split_exact(text: Str, delimiter: Str, maximum: u32) -> R<Option<List<Str>>> {
    if delimiter.0.is_empty() { return Ok(None); }
    chars(&text.0);
    let fields: Vec<Str> = text.0.split(&*delimiter.0).map(Str::lit).collect();
    if fields.len() as u128 > u128::from(maximum) { return Ok(None); }
    Ok(Some(List::onto(fields, List::nil())))
}
pub fn join(texts: List<Str>, delimiter: Str) -> R<Str> {
    let mut out = String::new();
    let mut cursor = texts.0.as_ref();
    let mut first = true;
    while let Some(node) = cursor {
        tick(1);
        if !first { chars(&delimiter.0); out.push_str(&delimiter.0); }
        first = false;
        chars(&node.head.0);
        out.push_str(&node.head.0);
        cursor = node.tail.0.as_ref();
    }
    Ok(Str::lit(&out))
}
fn canonical_decimal(text: &str) -> Option<Option<i128>> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let mut canonical = !digits.is_empty() && (digits == "0" || !digits.starts_with('0')) && text != "-0";
    for c in digits.chars() { tick(1); canonical = canonical && c.is_ascii_digit(); }
    if !canonical { return None; }
    Some(text.parse::<i128>().ok())
}
pub fn parse_int(text: Str) -> R<Option<i64>> {
    match canonical_decimal(&text.0) {
        None => Ok(None),
        Some(Some(n)) => i64::try_from(n).map(Some).map_err(|_| Overflow),
        Some(None) => Err(Overflow),
    }
}
macro_rules! decimal {
    ($t:ident, $format:ident, $parse:ident) => {
        pub fn $format(a: $t) -> R<Str> { let text = a.to_string(); tick(text.len() as u64); Ok(Str::lit(&text)) }
        pub fn $parse(text: Str) -> R<Option<$t>> {
            Ok(match canonical_decimal(&text.0) { Some(Some(n)) => $t::try_from(n).ok(), _ => None })
        }
    };
}
decimal!(u8, format_u8, parse_u8); decimal!(u16, format_u16, parse_u16);
decimal!(u32, format_u32, parse_u32); decimal!(u64, format_u64, parse_u64);
decimal!(i8, format_i8, parse_i8); decimal!(i16, format_i16, parse_i16);
decimal!(i32, format_i32, parse_i32); decimal!(i64, format_i64, parse_i64);
"#;

/// A type that can contain itself through in-place storage: an ADT, or a
/// function type through its closures' captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Owner {
    Adt(u64),
    Fn(usize),
}

/// The rendering context: the program, its profile, its function types, and
/// fresh names.
struct Renderer<'a> {
    program: &'a Program,
    profile: Profile,
    checker: Checker<'a>,
    /// Every function type of the program, numbered in first-use order.
    fn_types: Vec<Ty>,
    /// For each function type, the closures that inhabit it: function index
    /// and captured types.
    closures: BTreeMap<usize, Vec<(u64, Vec<Ty>)>>,
    /// For each owner, the owners its in-place storage reaches.
    reaches: BTreeMap<Owner, BTreeSet<Owner>>,
    fresh: usize,
}

fn fail<T>(reason: impl Into<String>) -> Result<T, String> {
    Err(reason.into())
}

fn index(number: u64) -> Result<usize, String> {
    usize::try_from(number).map_err(|error| error.to_string())
}

/// Whether local `name` occurs free in `expr`.
fn mentions(expr: &Expr, name: u64) -> bool {
    let any = |exprs: &[Expr]| exprs.iter().any(|expr| mentions(expr, name));
    match expr {
        Expr::Value { .. } => false,
        Expr::Var { name: used } => *used == name,
        Expr::Let {
            name: bound,
            bound: value,
            body,
            ..
        } => mentions(value, name) || (*bound != name && mentions(body, name)),
        Expr::Cond {
            condition,
            then_branch,
            else_branch,
        } => {
            mentions(condition, name) || mentions(then_branch, name) || mentions(else_branch, name)
        }
        Expr::Match {
            scrutinee, arms, ..
        } => {
            mentions(scrutinee, name)
                || arms
                    .iter()
                    .any(|arm| !arm.binders.contains(&name) && mentions(&arm.body, name))
        }
        Expr::Build { operands, .. }
        | Expr::Call { operands, .. }
        | Expr::Prim { operands, .. } => any(operands),
        Expr::Closure { captures, .. } => any(captures),
        Expr::Apply { target, operands } => mentions(target, name) || any(operands),
        Expr::First { value } | Expr::Second { value } | Expr::Field { value, .. } => {
            mentions(value, name)
        }
    }
}

/// The binder of local `name` in `body`: `_` when the body never reads it.
fn binder(name: u64, body: &Expr) -> String {
    if mentions(body, name) {
        format!("v{name}")
    } else {
        "_".to_owned()
    }
}

/// A signed literal; the minimum is spelled by its constant, since its
/// magnitude is not a literal of the type.
fn signed_literal(text: &str, kind: &str, minimum: &str) -> String {
    if text == minimum {
        format!("{kind}::MIN")
    } else {
        format!("{text}{kind}")
    }
}

impl<'a> Renderer<'a> {
    fn new(program: &'a Program, profile: Profile) -> Result<Self, String> {
        super::check::check(program)?;
        let mut renderer = Renderer {
            program,
            profile,
            checker: Checker { program },
            fn_types: Vec::new(),
            closures: BTreeMap::new(),
            reaches: BTreeMap::new(),
            fresh: 0,
        };
        for function in &program.functions {
            let mut scope: Vec<(u64, Ty)> = function
                .parameters
                .iter()
                .copied()
                .zip(function.types.iter().cloned())
                .collect();
            for ty in function.types.iter().chain([&function.result]) {
                renderer.register(ty);
            }
            renderer.collect(&function.body, &mut scope)?;
        }
        for adt in &program.adts {
            for ty in adt.constructors.iter().flatten() {
                renderer.register(ty);
            }
        }
        renderer.reaches = renderer.reachability();
        Ok(renderer)
    }

    fn fn_index(&mut self, ty: &Ty) -> usize {
        if let Some(position) = self.fn_types.iter().position(|known| known == ty) {
            return position;
        }
        self.fn_types.push(ty.clone());
        self.fn_types.len() - 1
    }

    /// Number every function type `ty` mentions.
    fn register(&mut self, ty: &Ty) {
        match ty {
            Ty::Option { value } => self.register(value),
            Ty::Result { ok, error } => {
                self.register(ok);
                self.register(error);
            }
            Ty::List { element } => self.register(element),
            Ty::Pair { left, right } => {
                self.register(left);
                self.register(right);
            }
            Ty::Fn { parameters, result } => {
                let _ = self.fn_index(ty);
                for parameter in parameters {
                    self.register(parameter);
                }
                self.register(result);
            }
            Ty::Unit
            | Ty::Bool
            | Ty::Nat
            | Ty::Int
            | Ty::Fixed { .. }
            | Ty::String
            | Ty::Bytes
            | Ty::Ordering
            | Ty::Adt { .. } => {}
        }
    }

    /// The owners whose storage a value of `ty` holds in place: a list's
    /// cells are behind a handle, so its element is not in place.
    fn in_place(&self, ty: &Ty, out: &mut BTreeSet<Owner>) {
        match ty {
            Ty::Option { value } => self.in_place(value, out),
            Ty::Result { ok, error } => {
                self.in_place(ok, out);
                self.in_place(error, out);
            }
            Ty::Pair { left, right } => {
                self.in_place(left, out);
                self.in_place(right, out);
            }
            Ty::Adt { index } => {
                out.insert(Owner::Adt(*index));
            }
            Ty::Fn { .. } => {
                if let Some(position) = self.fn_types.iter().position(|known| known == ty) {
                    out.insert(Owner::Fn(position));
                }
            }
            Ty::Unit
            | Ty::Bool
            | Ty::Nat
            | Ty::Int
            | Ty::Fixed { .. }
            | Ty::String
            | Ty::Bytes
            | Ty::Ordering
            | Ty::List { .. } => {}
        }
    }

    fn stored(&self, owner: Owner) -> Vec<Ty> {
        match owner {
            Owner::Adt(adt) => usize::try_from(adt)
                .ok()
                .and_then(|adt| self.program.adts.get(adt))
                .map(|adt| adt.constructors.iter().flatten().cloned().collect())
                .unwrap_or_default(),
            Owner::Fn(position) => self
                .closures
                .get(&position)
                .map(|variants| {
                    variants
                        .iter()
                        .flat_map(|(_, captured)| captured.iter().cloned())
                        .collect()
                })
                .unwrap_or_default(),
        }
    }

    /// For every owner, every owner its in-place storage reaches.
    fn reachability(&self) -> BTreeMap<Owner, BTreeSet<Owner>> {
        let owners: Vec<Owner> = (0..self.program.adts.len() as u64)
            .map(Owner::Adt)
            .chain((0..self.fn_types.len()).map(Owner::Fn))
            .collect();
        let direct: BTreeMap<Owner, BTreeSet<Owner>> = owners
            .iter()
            .map(|owner| {
                let mut out = BTreeSet::new();
                for ty in self.stored(*owner) {
                    self.in_place(&ty, &mut out);
                }
                (*owner, out)
            })
            .collect();
        owners
            .iter()
            .map(|owner| {
                let mut seen = BTreeSet::new();
                let mut pending: Vec<Owner> = direct
                    .get(owner)
                    .map(|reached| reached.iter().copied().collect())
                    .unwrap_or_default();
                while let Some(next) = pending.pop() {
                    if seen.insert(next) {
                        if let Some(onward) = direct.get(&next) {
                            pending.extend(onward.iter().copied());
                        }
                    }
                }
                (*owner, seen)
            })
            .collect()
    }

    /// Whether a field of type `ty` stored in `owner` must be boxed: its
    /// in-place storage reaches `owner` again, so the type has no fixed
    /// size. `rust-core` cannot box, so it refuses the program.
    fn boxed(&self, owner: Owner, ty: &Ty) -> Result<bool, String> {
        let mut held = BTreeSet::new();
        self.in_place(ty, &mut held);
        let cyclic = held.iter().any(|reached| {
            *reached == owner
                || self
                    .reaches
                    .get(reached)
                    .is_some_and(|onward| onward.contains(&owner))
        });
        if cyclic && self.profile == Profile::Core {
            let named = match owner {
                Owner::Adt(adt) => format!("ADT {adt}"),
                Owner::Fn(position) => format!("function type {position}"),
            };
            return fail(format!(
                "{named} contains itself and requires heap allocation, which {} does not provide",
                self.profile.target()
            ));
        }
        Ok(cyclic)
    }

    fn fresh(&mut self, base: &str) -> String {
        self.fresh += 1;
        format!("{base}{}", self.fresh)
    }

    fn heap(&self, what: &str) -> Result<(), String> {
        if self.profile == Profile::Core {
            return fail(format!(
                "{what} requires heap allocation, which {} does not provide",
                self.profile.target()
            ));
        }
        Ok(())
    }

    fn ty(&mut self, ty: &Ty) -> Result<String, String> {
        Ok(match ty {
            Ty::Unit => "()".to_owned(),
            Ty::Bool => "bool".to_owned(),
            Ty::Nat => "u64".to_owned(),
            Ty::Int => "i64".to_owned(),
            Ty::Fixed { width } => width.name().to_owned(),
            Ty::String => {
                self.heap("a string")?;
                "Str".to_owned()
            }
            Ty::Bytes => {
                self.heap("a byte string")?;
                "Bytes".to_owned()
            }
            Ty::Ordering => "Ordering".to_owned(),
            Ty::Option { value } => format!("Option<{}>", self.ty(value)?),
            Ty::Result { ok, error } => format!("Result<{}, {}>", self.ty(ok)?, self.ty(error)?),
            Ty::List { element } => {
                self.heap("a list")?;
                format!("List<{}>", self.ty(element)?)
            }
            Ty::Pair { left, right } => format!("({}, {})", self.ty(left)?, self.ty(right)?),
            Ty::Adt { index } => format!("Adt{index}"),
            Ty::Fn { .. } => format!("Fn{}", self.fn_index(ty)),
        })
    }

    /// A field of `owner`: its type, boxed where it must be.
    fn field_ty(&mut self, owner: Owner, ty: &Ty) -> Result<String, String> {
        let rendered = self.ty(ty)?;
        Ok(if self.boxed(owner, ty)? {
            format!("std::rc::Rc<{rendered}>")
        } else {
            rendered
        })
    }

    /// A value stored as a field of `owner`.
    fn store(&self, owner: Owner, ty: &Ty, text: String) -> Result<String, String> {
        Ok(if self.boxed(owner, ty)? {
            format!("std::rc::Rc::new({text})")
        } else {
            text
        })
    }

    fn adt_fields(&self, adt: u64, constructor: u64) -> Result<Vec<Ty>, String> {
        self.program
            .adts
            .get(index(adt)?)
            .and_then(|declared| declared.constructors.get(index(constructor).ok()?))
            .cloned()
            .ok_or_else(|| format!("ADT {adt} has no constructor {constructor}"))
    }

    /// Collect every closure site, so each function type's enum is closed,
    /// and number every function type the program states.
    fn collect(&mut self, expr: &Expr, scope: &mut Vec<(u64, Ty)>) -> Result<(), String> {
        let mut children: Vec<&Expr> = Vec::new();
        match expr {
            Expr::Value { ty, .. } => self.register(ty),
            Expr::Var { .. } => {}
            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => {
                self.register(ty);
                self.collect(bound, scope)?;
                scope.push((*name, ty.clone()));
                let collected = self.collect(body, scope);
                scope.pop();
                return collected;
            }
            Expr::Cond {
                condition,
                then_branch,
                else_branch,
            } => {
                children.extend([
                    condition.as_ref(),
                    then_branch.as_ref(),
                    else_branch.as_ref(),
                ]);
            }
            Expr::Match {
                ty,
                scrutinee,
                arms,
            } => {
                self.register(ty);
                self.collect(scrutinee, scope)?;
                let scrutinee_ty = self.checker.expr(scrutinee, scope)?;
                for arm in arms {
                    let fields = self.checker.shape_fields(arm.shape, &scrutinee_ty)?;
                    let depth = scope.len();
                    scope.extend(arm.binders.iter().copied().zip(fields));
                    let collected = self.collect(&arm.body, scope);
                    scope.truncate(depth);
                    collected?;
                }
                return Ok(());
            }
            Expr::Build { ty, operands, .. } => {
                self.register(ty);
                children.extend(operands);
            }
            Expr::Call { operands, .. } | Expr::Prim { operands, .. } => children.extend(operands),
            Expr::Closure { function, captures } => {
                children.extend(captures);
                let ty = self.checker.expr(expr, scope)?;
                self.register(&ty);
                let position = self.fn_index(&ty);
                let captured = self
                    .program
                    .functions
                    .get(index(*function)?)
                    .and_then(|callee| callee.types.get(..captures.len()))
                    .ok_or_else(|| format!("closure of function {function} is malformed"))?
                    .to_vec();
                let entry = self.closures.entry(position).or_default();
                if !entry.iter().any(|(known, _)| known == function) {
                    entry.push((*function, captured));
                }
            }
            Expr::Apply { target, operands } => {
                children.push(target);
                children.extend(operands);
            }
            Expr::First { value } | Expr::Second { value } | Expr::Field { value, .. } => {
                children.push(value);
            }
        }
        for child in children {
            self.collect(child, scope)?;
        }
        Ok(())
    }

    fn value(&mut self, value: &Value, ty: &Ty) -> Result<String, String> {
        Ok(match (value, ty) {
            (Value::Unit, _) => "()".to_owned(),
            (Value::Bool { value }, _) => value.to_string(),
            (Value::Nat { value }, _) => format!("{value}u64"),
            (Value::Int { value }, _) => signed_literal(value, "i64", "-9223372036854775808"),
            (Value::U8 { value }, _) => format!("{value}u8"),
            (Value::U16 { value }, _) => format!("{value}u16"),
            (Value::U32 { value }, _) => format!("{value}u32"),
            (Value::U64 { value }, _) => format!("{value}u64"),
            (Value::I8 { value }, _) => signed_literal(value, "i8", "-128"),
            (Value::I16 { value }, _) => signed_literal(value, "i16", "-32768"),
            (Value::I32 { value }, _) => signed_literal(value, "i32", "-2147483648"),
            (Value::I64 { value }, _) => signed_literal(value, "i64", "-9223372036854775808"),
            (Value::String { value }, _) => {
                self.heap("a string")?;
                format!("Str::lit({value:?})")
            }
            (Value::Bytes { hex }, _) => {
                self.heap("a byte string")?;
                let bytes: Vec<String> = (0..hex.len())
                    .step_by(2)
                    .map(|at| {
                        hex.get(at..at + 2)
                            .map(|digits| format!("0x{digits}"))
                            .ok_or_else(|| format!("byte literal `{hex}` is malformed"))
                    })
                    .collect::<Result<_, _>>()?;
                format!("Bytes::lit(&[{}])", bytes.join(", "))
            }
            (Value::Ordering { value }, _) => match value {
                OrderingValue::Lt => "core::cmp::Ordering::Less",
                OrderingValue::Eq => "core::cmp::Ordering::Equal",
                OrderingValue::Gt => "core::cmp::Ordering::Greater",
            }
            .to_owned(),
            (Value::None, ty) => format!("None::<{}>", self.ty(option_inner(ty)?)?),
            (Value::Some { value }, Ty::Option { value: inner }) => {
                format!("Some({})", self.value(value, inner)?)
            }
            (Value::Ok { value }, Ty::Result { ok, error }) => format!(
                "Ok::<{}, {}>({})",
                self.ty(ok)?,
                self.ty(error)?,
                self.value(value, ok)?
            ),
            (Value::Error { value }, Ty::Result { ok, error }) => format!(
                "Err::<{}, {}>({})",
                self.ty(ok)?,
                self.ty(error)?,
                self.value(value, error)?
            ),
            (Value::List { items }, Ty::List { element }) => {
                let mut out = format!("List::<{}>::nil()", self.ty(element)?);
                for item in items.iter().rev() {
                    out = format!("List::cons({}, {out})", self.value(item, element)?);
                }
                out
            }
            (
                Value::Pair { left, right },
                Ty::Pair {
                    left: left_ty,
                    right: right_ty,
                },
            ) => format!(
                "({}, {})",
                self.value(left, left_ty)?,
                self.value(right, right_ty)?
            ),
            (
                Value::Adt {
                    constructor,
                    fields,
                },
                Ty::Adt { index: adt },
            ) => {
                let types = self.adt_fields(*adt, *constructor)?;
                if fields.is_empty() {
                    format!("Adt{adt}::C{constructor}")
                } else {
                    let mut rendered = Vec::new();
                    for (field, ty) in fields.iter().zip(&types) {
                        let text = self.value(field, ty)?;
                        rendered.push(self.store(Owner::Adt(*adt), ty, text)?);
                    }
                    format!("Adt{adt}::C{constructor}({})", rendered.join(", "))
                }
            }
            (value, ty) => {
                return fail(format!(
                    "the literal {value:?} cannot be rendered at type {ty:?}"
                ))
            }
        })
    }

    fn prim(operation: &Prim, operands: &[Ty], rendered: &[String]) -> Result<String, String> {
        let args = rendered.join(", ");
        let fixed_of = |ty: &Ty| -> Result<&'static str, String> {
            match ty {
                Ty::Fixed { width } => Ok(width.name()),
                other => fail(format!("{other:?} is not fixed-width")),
            }
        };
        let first = operands.first().cloned().unwrap_or(Ty::Unit);
        let sequence = |list: &str, bytes: &str, string: &str| -> Result<String, String> {
            match &first {
                Ty::List { .. } => Ok(list.to_owned()),
                Ty::Bytes => Ok(bytes.to_owned()),
                Ty::String if !string.is_empty() => Ok(string.to_owned()),
                other => fail(format!("{operation:?} does not apply to {other:?}")),
            }
        };
        Ok(match operation {
            Prim::NatAdd => format!("nat_add({args})?"),
            Prim::NatSub => format!("nat_sub({args})?"),
            Prim::NatMul => format!("nat_mul({args})?"),
            Prim::NatQuot => format!("nat_quot({args})?"),
            Prim::NatRem => format!("nat_rem({args})?"),
            Prim::NatEq => format!("nat_eq({args})?"),
            Prim::NatLe => format!("nat_le({args})?"),
            Prim::NatLt => format!("nat_lt({args})?"),
            Prim::IntAdd => format!("int_add({args})?"),
            Prim::IntSub => format!("int_sub({args})?"),
            Prim::IntMul => format!("int_mul({args})?"),
            Prim::IntNeg => format!("int_neg({args})?"),
            Prim::IntQuot => format!("int_quot({args})?"),
            Prim::IntRem => format!("int_rem({args})?"),
            Prim::CheckedAdd => format!("fixed_{}::checked_add({args})?", fixed_of(&first)?),
            Prim::CheckedSub => format!("fixed_{}::checked_sub({args})?", fixed_of(&first)?),
            Prim::CheckedMul => format!("fixed_{}::checked_mul({args})?", fixed_of(&first)?),
            Prim::CheckedQuot => format!("fixed_{}::checked_quot({args})?", fixed_of(&first)?),
            Prim::CheckedNeg => format!("checked_neg_{}({args})?", fixed_of(&first)?),
            Prim::BitAnd => format!("fixed_{}::bit_and({args})?", fixed_of(&first)?),
            Prim::BitOr => format!("fixed_{}::bit_or({args})?", fixed_of(&first)?),
            Prim::BitXor => format!("fixed_{}::bit_xor({args})?", fixed_of(&first)?),
            Prim::BitNot => format!("fixed_{}::bit_not({args})?", fixed_of(&first)?),
            Prim::ShiftLeft => format!("fixed_{}::shift_left({args})?", fixed_of(&first)?),
            Prim::ShiftRight => format!("fixed_{}::shift_right({args})?", fixed_of(&first)?),
            Prim::Equal => format!("equal({args})?"),
            Prim::BoolNot => format!("bool_not({args})?"),
            Prim::BoolAnd => format!("bool_and({args})?"),
            Prim::BoolOr => format!("bool_or({args})?"),
            Prim::Append => format!("{}({args})?", sequence("append_list", "append_bytes", "")?),
            Prim::Length => format!(
                "{}({args})?",
                sequence("length_list", "length_bytes", "length_string")?
            ),
            Prim::Index => format!("{}({args})?", sequence("index_list", "index_bytes", "")?),
            Prim::Slice => format!("{}({args})?", sequence("slice_list", "slice_bytes", "")?),
            Prim::Utf8Encode => format!("utf8_encode({args})?"),
            Prim::Utf8Decode => format!("utf8_decode({args})?"),
            Prim::CompareBytes => format!("compare_bytes({args})?"),
            Prim::Compare => format!("compare({args})?"),
            Prim::SplitExact => format!("split_exact({args})?"),
            Prim::Join => format!("join({args})?"),
            Prim::FormatDecimal => match first {
                Ty::Int => format!("format_i64({args})?"),
                other => format!("format_{}({args})?", fixed_of(&other)?),
            },
            Prim::Convert { target } => {
                format!("fixed_{}::convert(i128::from({args}))?", target.name())
            }
            Prim::ParseDecimal { target } => match target {
                Ty::Int => format!("parse_int({args})?"),
                other => format!("parse_{}({args})?", fixed_of(other)?),
            },
        })
    }

    fn exprs(&mut self, exprs: &[Expr], scope: &mut Vec<(u64, Ty)>) -> Result<Vec<String>, String> {
        exprs.iter().map(|expr| self.expr(expr, scope)).collect()
    }

    /// The arms of a match on `scrutinee_ty`, rendered over `holder`.
    #[allow(clippy::too_many_lines)]
    fn arms(
        &mut self,
        holder: &str,
        scrutinee_ty: &Ty,
        arms: &[Arm],
        scope: &mut Vec<(u64, Ty)>,
    ) -> Result<String, String> {
        let mut bodies: BTreeMap<Shape, (Vec<String>, String)> = BTreeMap::new();
        for arm in arms {
            let fields = self.checker.shape_fields(arm.shape, scrutinee_ty)?;
            let depth = scope.len();
            scope.extend(arm.binders.iter().copied().zip(fields));
            let body = self.expr(&arm.body, scope);
            scope.truncate(depth);
            let names = arm
                .binders
                .iter()
                .map(|name| binder(*name, &arm.body))
                .collect();
            bodies.insert(arm.shape, (names, body?));
        }
        let mut take = |shape: Shape| {
            bodies
                .remove(&shape)
                .ok_or_else(|| format!("the match lacks {shape:?}"))
        };
        let name = |names: &[String], position: usize| {
            names
                .get(position)
                .cloned()
                .ok_or_else(|| format!("an arm lacks binder {position}"))
        };
        let rendered = match scrutinee_ty {
            Ty::Option { .. } => {
                let (_, none) = take(Shape::None)?;
                let (b, some) = take(Shape::Some)?;
                format!(
                    "match {holder} {{ None => {{ {none} }} Some({}) => {{ {some} }} }}",
                    name(&b, 0)?
                )
            }
            Ty::Result { .. } => {
                let (b, ok) = take(Shape::Ok)?;
                let (c, error) = take(Shape::Error)?;
                format!(
                    "match {holder} {{ Ok({}) => {{ {ok} }} Err({}) => {{ {error} }} }}",
                    name(&b, 0)?,
                    name(&c, 0)?
                )
            }
            Ty::List { .. } => {
                let (_, nil) = take(Shape::Nil)?;
                let (b, cons) = take(Shape::Cons)?;
                format!(
                    "match {holder}.uncons() {{ None => {{ {nil} }} Some(({}, {})) => {{ {cons} }} }}",
                    name(&b, 0)?,
                    name(&b, 1)?
                )
            }
            Ty::Nat => {
                let (_, zero) = take(Shape::Zero)?;
                let (b, succ) = take(Shape::Succ)?;
                let predecessor = name(&b, 0)?;
                if predecessor == "_" {
                    format!("if {holder} == 0 {{ {zero} }} else {{ {succ} }}")
                } else {
                    format!(
                        "if {holder} == 0 {{ {zero} }} else {{ let {predecessor} = {holder} - 1; {succ} }}"
                    )
                }
            }
            Ty::Bool => {
                let (_, yes) = take(Shape::True)?;
                let (_, no) = take(Shape::False)?;
                format!("if {holder} {{ {yes} }} else {{ {no} }}")
            }
            Ty::Pair { .. } => {
                let (b, body) = take(Shape::Pair)?;
                format!(
                    "let ({}, {}) = {holder}; {body}",
                    name(&b, 0)?,
                    name(&b, 1)?
                )
            }
            Ty::Unit => {
                let (_, body) = take(Shape::Unit)?;
                format!("let () = {holder}; {body}")
            }
            Ty::Ordering => {
                let (_, less) = take(Shape::Lt)?;
                let (_, equal) = take(Shape::Eq)?;
                let (_, greater) = take(Shape::Gt)?;
                format!("match {holder} {{ Ordering::Less => {{ {less} }} Ordering::Equal => {{ {equal} }} Ordering::Greater => {{ {greater} }} }}")
            }
            Ty::Adt { index: adt } => {
                let count = self
                    .program
                    .adts
                    .get(index(*adt)?)
                    .map_or(0, |declared| declared.constructors.len())
                    as u64;
                let mut out = format!("match {holder} {{ ");
                for constructor in 0..count {
                    let (names, body) = take(Shape::Adt { constructor })?;
                    if names.is_empty() {
                        let _ = write!(out, "Adt{adt}::C{constructor} => {{ {body} }} ");
                        continue;
                    }
                    let types = self.adt_fields(*adt, constructor)?;
                    let mut patterns = Vec::new();
                    let mut loads = String::new();
                    for (position, (bound, ty)) in names.iter().zip(&types).enumerate() {
                        if bound == "_" {
                            patterns.push("_".to_owned());
                        } else if self.boxed(Owner::Adt(*adt), ty)? {
                            let held = format!("r{position}");
                            let _ = write!(loads, "let {bound} = (*{held}).clone(); ");
                            patterns.push(held);
                        } else {
                            patterns.push(bound.clone());
                        }
                    }
                    let _ = write!(
                        out,
                        "Adt{adt}::C{constructor}({}) => {{ {loads}{body} }} ",
                        patterns.join(", ")
                    );
                }
                out.push('}');
                out
            }
            other => return fail(format!("cannot render a match on {other:?}")),
        };
        if !bodies.is_empty() {
            return fail("a match arm has no rendering");
        }
        Ok(rendered)
    }

    #[allow(clippy::too_many_lines)]
    fn expr(&mut self, expr: &Expr, scope: &mut Vec<(u64, Ty)>) -> Result<String, String> {
        Ok(match expr {
            Expr::Value { ty, value } => self.value(value, ty)?,
            Expr::Var { name } => format!("v{name}.clone()"),
            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => {
                let bound = self.expr(bound, scope)?;
                let rust_ty = self.ty(ty)?;
                scope.push((*name, ty.clone()));
                let rendered = self.expr(body, scope);
                scope.pop();
                let pattern = binder(*name, body);
                format!("{{ let {pattern}: {rust_ty} = {bound}; {} }}", rendered?)
            }
            Expr::Cond {
                condition,
                then_branch,
                else_branch,
            } => format!(
                "if {} {{ {} }} else {{ {} }}",
                self.expr(condition, scope)?,
                self.expr(then_branch, scope)?,
                self.expr(else_branch, scope)?
            ),
            Expr::Match {
                scrutinee, arms, ..
            } => {
                let scrutinee_ty = self.checker.expr(scrutinee, scope)?;
                let rendered = self.expr(scrutinee, scope)?;
                let holder = self.fresh("m");
                let body = self.arms(&holder, &scrutinee_ty, arms, scope)?;
                format!("{{ let {holder} = {rendered}; {body} }}")
            }
            Expr::Build {
                shape,
                ty,
                operands,
            } => {
                let rendered = self.exprs(operands, scope)?;
                let operand = |position: usize| {
                    rendered
                        .get(position)
                        .cloned()
                        .ok_or_else(|| format!("{shape:?} lacks operand {position}"))
                };
                match (shape, ty) {
                    (Shape::None, ty) => format!("None::<{}>", self.ty(option_inner(ty)?)?),
                    (Shape::Some, _) => format!("Some({})", operand(0)?),
                    (Shape::Ok, Ty::Result { ok, error }) => {
                        format!(
                            "Ok::<{}, {}>({})",
                            self.ty(ok)?,
                            self.ty(error)?,
                            operand(0)?
                        )
                    }
                    (Shape::Error, Ty::Result { ok, error }) => format!(
                        "Err::<{}, {}>({})",
                        self.ty(ok)?,
                        self.ty(error)?,
                        operand(0)?
                    ),
                    (Shape::Nil, Ty::List { element }) => {
                        format!("List::<{}>::nil()", self.ty(element)?)
                    }
                    (Shape::Cons, _) => format!("List::cons({}, {})", operand(0)?, operand(1)?),
                    (Shape::Zero, _) => "0u64".to_owned(),
                    (Shape::Succ, _) => format!("nat_succ({})?", operand(0)?),
                    (Shape::Pair, _) => format!("({}, {})", operand(0)?, operand(1)?),
                    (Shape::True, _) => "true".to_owned(),
                    (Shape::False, _) => "false".to_owned(),
                    (Shape::Unit, _) => "()".to_owned(),
                    (Shape::Lt, _) => "Ordering::Less".to_owned(),
                    (Shape::Eq, _) => "Ordering::Equal".to_owned(),
                    (Shape::Gt, _) => "Ordering::Greater".to_owned(),
                    (Shape::Adt { constructor }, Ty::Adt { index: adt }) => {
                        if rendered.is_empty() {
                            format!("Adt{adt}::C{constructor}")
                        } else {
                            let types = self.adt_fields(*adt, *constructor)?;
                            let mut stored = Vec::new();
                            for (text, ty) in rendered.iter().zip(&types) {
                                stored.push(self.store(Owner::Adt(*adt), ty, text.clone())?);
                            }
                            format!("Adt{adt}::C{constructor}({})", stored.join(", "))
                        }
                    }
                    (shape, ty) => return fail(format!("cannot render {shape:?} at {ty:?}")),
                }
            }
            Expr::Call { function, operands } => {
                format!("f{function}({})?", self.exprs(operands, scope)?.join(", "))
            }
            Expr::Closure { function, captures } => {
                let ty = self.checker.expr(expr, scope)?;
                let position = self.fn_index(&ty);
                let rendered = self.exprs(captures, scope)?;
                if rendered.is_empty() {
                    format!("Fn{position}::F{function}")
                } else {
                    let types = self
                        .program
                        .functions
                        .get(index(*function)?)
                        .map(|callee| callee.types.clone())
                        .unwrap_or_default();
                    let mut stored = Vec::new();
                    for (text, ty) in rendered.iter().zip(&types) {
                        stored.push(self.store(Owner::Fn(position), ty, text.clone())?);
                    }
                    format!("Fn{position}::F{function}({})", stored.join(", "))
                }
            }
            Expr::Apply { target, operands } => {
                let callee = self.fresh("c");
                let target = self.expr(target, scope)?;
                let rendered = self.exprs(operands, scope)?;
                format!(
                    "{{ let {callee} = {target}; {callee}.apply({})? }}",
                    rendered.join(", ")
                )
            }
            Expr::Prim {
                operation,
                operands,
            } => {
                let types = operands
                    .iter()
                    .map(|operand| self.checker.expr(operand, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                let rendered = self.exprs(operands, scope)?;
                // Operands are bound left to right before the call, so the
                // evaluation order is the denotation's.
                let names: Vec<String> = rendered.iter().map(|_| self.fresh("a")).collect();
                let bindings: String = names
                    .iter()
                    .zip(&rendered)
                    .map(|(name, text)| format!("let {name} = {text}; "))
                    .collect();
                format!("{{ {bindings}{} }}", Self::prim(operation, &types, &names)?)
            }
            Expr::First { value } => {
                let held = self.fresh("l");
                format!(
                    "{{ let ({held}, _) = {}; {held} }}",
                    self.expr(value, scope)?
                )
            }
            Expr::Second { value } => {
                let held = self.fresh("r");
                format!(
                    "{{ let (_, {held}) = {}; {held} }}",
                    self.expr(value, scope)?
                )
            }
            Expr::Field {
                value,
                index: position,
            } => {
                let Ty::Adt { index: adt } = self.checker.expr(value, scope)? else {
                    return fail("`field` of a non-ADT value");
                };
                let types = self.adt_fields(adt, 0)?;
                let selected = index(*position)?;
                let ty = types
                    .get(selected)
                    .cloned()
                    .ok_or_else(|| format!("ADT {adt} has no field {position}"))?;
                let patterns: Vec<&str> = (0..types.len())
                    .map(|at| if at == selected { "g" } else { "_" })
                    .collect();
                let read = if self.boxed(Owner::Adt(adt), &ty)? {
                    "(*g).clone()"
                } else {
                    "g"
                };
                format!(
                    "match {} {{ Adt{adt}::C0({}) => {read} }}",
                    self.expr(value, scope)?,
                    patterns.join(", ")
                )
            }
        })
    }

    /// The library crate: the runtime, the ADTs, the function types, and the
    /// functions.
    fn render(&mut self) -> Result<String, String> {
        let mut out = match self.profile {
            Profile::Core => format!("#![no_std]\n#![forbid(unsafe_code)]\n{CORE_RUNTIME}"),
            Profile::Std => format!("#![forbid(unsafe_code)]\n{CORE_RUNTIME}{STD_RUNTIME}"),
        };
        for (position, adt) in self.program.adts.iter().enumerate() {
            let owner = Owner::Adt(position as u64);
            let _ = write!(out, "\n#[derive(Clone)]\npub enum Adt{position} {{ ");
            for (constructor, fields) in adt.constructors.iter().enumerate() {
                if fields.is_empty() {
                    let _ = write!(out, "C{constructor}, ");
                } else {
                    let rendered = fields
                        .iter()
                        .map(|ty| self.field_ty(owner, ty))
                        .collect::<Result<Vec<_>, _>>()?;
                    let _ = write!(out, "C{constructor}({}), ", rendered.join(", "));
                }
            }
            out.push_str("}\n");
        }
        let fn_types = self.fn_types.clone();
        for (position, fn_ty) in fn_types.iter().enumerate() {
            let owner = Owner::Fn(position);
            let Ty::Fn { parameters, result } = fn_ty else {
                return fail("a function type is not a function");
            };
            let variants = self.closures.get(&position).cloned().unwrap_or_default();
            let _ = write!(out, "\n#[derive(Clone)]\npub enum Fn{position} {{ ");
            for (function, captured) in &variants {
                if captured.is_empty() {
                    let _ = write!(out, "F{function}, ");
                } else {
                    let rendered = captured
                        .iter()
                        .map(|ty| self.field_ty(owner, ty))
                        .collect::<Result<Vec<_>, _>>()?;
                    let _ = write!(out, "F{function}({}), ", rendered.join(", "));
                }
            }
            // A function type no closure inhabits is an empty enum, matched
            // without arms: no unreachable panic is rendered, and its
            // parameters are never read.
            let unread = if variants.is_empty() { "_" } else { "" };
            let params = parameters
                .iter()
                .enumerate()
                .map(|(at, ty)| Ok(format!("{unread}p{at}: {}", self.ty(ty)?)))
                .collect::<Result<Vec<_>, String>>()?;
            let scrutinee = if variants.is_empty() { "*self" } else { "self" };
            let _ = write!(
                out,
                "}}\nimpl Fn{position} {{ pub fn apply(&self, {}) -> R<{}> {{ match {scrutinee} {{ ",
                params.join(", "),
                self.ty(result)?
            );
            for (function, captured) in &variants {
                let mut passed = Vec::new();
                for (at, ty) in captured.iter().enumerate() {
                    passed.push(if self.boxed(owner, ty)? {
                        format!("(**k{at}).clone()")
                    } else {
                        format!("k{at}.clone()")
                    });
                }
                passed.extend((0..parameters.len()).map(|at| format!("p{at}")));
                if captured.is_empty() {
                    let _ = write!(
                        out,
                        "Fn{position}::F{function} => f{function}({}), ",
                        passed.join(", ")
                    );
                } else {
                    let names: Vec<String> =
                        (0..captured.len()).map(|at| format!("k{at}")).collect();
                    let _ = write!(
                        out,
                        "Fn{position}::F{function}({}) => f{function}({}), ",
                        names.join(", "),
                        passed.join(", ")
                    );
                }
            }
            out.push_str("} } }\n");
        }
        for (position, function) in self.program.functions.iter().enumerate() {
            let mut scope: Vec<(u64, Ty)> = function
                .parameters
                .iter()
                .copied()
                .zip(function.types.iter().cloned())
                .collect();
            let params = function
                .parameters
                .iter()
                .zip(&function.types)
                .map(|(name, ty)| {
                    let rendered = self.ty(ty)?;
                    Ok(format!("{}: {rendered}", binder(*name, &function.body)))
                })
                .collect::<Result<Vec<_>, String>>()?;
            let body = self.expr(&function.body, &mut scope)?;
            let result = self.ty(&function.result)?;
            let _ = write!(
                out,
                "\npub fn f{position}({}) -> R<{result}> {{ Ok({body}) }}\n",
                params.join(", ")
            );
        }
        Ok(out)
    }
}

fn option_inner(ty: &Ty) -> Result<&Ty, String> {
    match ty {
        Ty::Option { value } => Ok(value),
        other => fail(format!("`none` at {other:?}")),
    }
}

/// Render a valid program to a library crate of `profile`: the runtime, its
/// ADTs, function types, and functions.
///
/// # Errors
///
/// Returns the reason the program is invalid or has no rendering in
/// `profile`.
pub fn render(program: &Program, profile: Profile) -> Result<String, String> {
    Renderer::new(program, profile)?.render()
}

/// The ADTs whose values a value of `ty` can hold, and whether it can hold
/// a string.
fn shown(program: &Program, ty: &Ty, adts: &mut BTreeSet<u64>) -> bool {
    match ty {
        Ty::String => true,
        Ty::Option { value } => shown(program, value, adts),
        Ty::List { element } => shown(program, element, adts),
        Ty::Result { ok, error } => shown(program, ok, adts) | shown(program, error, adts),
        Ty::Pair { left, right } => shown(program, left, adts) | shown(program, right, adts),
        Ty::Adt { index } => {
            if !adts.insert(*index) {
                return false;
            }
            usize::try_from(*index)
                .ok()
                .and_then(|adt| program.adts.get(adt))
                .map(|adt| {
                    adt.constructors
                        .iter()
                        .flatten()
                        .cloned()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default()
                .iter()
                .fold(false, |found, field| shown(program, field, adts) | found)
        }
        Ty::Unit
        | Ty::Bool
        | Ty::Nat
        | Ty::Int
        | Ty::Fixed { .. }
        | Ty::Bytes
        | Ty::Ordering
        | Ty::Fn { .. } => false,
    }
}

/// The expression printing a value of type `ty` held in `name` in its exact
/// JSON form.
fn show(ty: &Ty, name: &str) -> Result<String, String> {
    let number = |kind: &str| {
        format!(
            "format!(\"{{{{\\\"kind\\\":\\\"{kind}\\\",\\\"value\\\":\\\"{{}}\\\"}}}}\", {name})"
        )
    };
    Ok(match ty {
        Ty::Unit => format!("{{ let () = {name}; \"{{\\\"kind\\\":\\\"unit\\\"}}\".to_owned() }}"),
        Ty::Bool => {
            format!("format!(\"{{{{\\\"kind\\\":\\\"bool\\\",\\\"value\\\":{{}}}}}}\", {name})")
        }
        Ty::Nat => number("nat"),
        Ty::Int => number("int"),
        Ty::Fixed { width } => number(width.name()),
        Ty::String => format!(
            "format!(\"{{{{\\\"kind\\\":\\\"string\\\",\\\"value\\\":{{}}}}}}\", quote({name}.text()))"
        ),
        Ty::Bytes => format!(
            "format!(\"{{{{\\\"kind\\\":\\\"bytes\\\",\\\"hex\\\":\\\"{{}}\\\"}}}}\", {name}.octets().iter().map(|b| format!(\"{{b:02x}}\")).collect::<String>())"
        ),
        Ty::Ordering => format!(
            "format!(\"{{{{\\\"kind\\\":\\\"ordering\\\",\\\"value\\\":\\\"{{}}\\\"}}}}\", match {name} {{ core::cmp::Ordering::Less => \"lt\", core::cmp::Ordering::Equal => \"eq\", core::cmp::Ordering::Greater => \"gt\" }})"
        ),
        Ty::Option { value } => {
            let inner = show(value, "x")?;
            format!("match {name} {{ None => \"{{\\\"kind\\\":\\\"none\\\"}}\".to_owned(), Some(x) => format!(\"{{{{\\\"kind\\\":\\\"some\\\",\\\"value\\\":{{}}}}}}\", {inner}) }}")
        }
        Ty::Result { ok, error } => {
            let ok_show = show(ok, "x")?;
            let error_show = show(error, "x")?;
            format!("match {name} {{ Ok(x) => format!(\"{{{{\\\"kind\\\":\\\"ok\\\",\\\"value\\\":{{}}}}}}\", {ok_show}), Err(x) => format!(\"{{{{\\\"kind\\\":\\\"error\\\",\\\"value\\\":{{}}}}}}\", {error_show}) }}")
        }
        Ty::List { element } => {
            let inner = show(element, "x")?;
            format!("{{ let mut parts = Vec::new(); let mut cursor = {name}; while let Some((x, rest)) = cursor.uncons() {{ parts.push({inner}); cursor = rest; }} format!(\"{{{{\\\"kind\\\":\\\"list\\\",\\\"items\\\":[{{}}]}}}}\", parts.join(\",\")) }}")
        }
        Ty::Pair { left, right } => {
            let left_show = show(left, "l")?;
            let right_show = show(right, "r")?;
            format!("{{ let (l, r) = {name}; format!(\"{{{{\\\"kind\\\":\\\"pair\\\",\\\"left\\\":{{}},\\\"right\\\":{{}}}}}}\", {left_show}, {right_show}) }}")
        }
        Ty::Adt { index } => format!("show_adt{index}({name})"),
        Ty::Fn { .. } => return fail("a function value is not observable"),
    })
}

/// Prints a string in the exact JSON form.
const QUOTE: &str = r#"
fn quote(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
"#;

/// A binary crate that runs `entry` of the rendered library crate `program`
/// on `arguments` and prints the observable outcome, the value in its exact
/// JSON form or `{"kind":"overflow"}`, then the work the runtime counted.
///
/// # Errors
///
/// Returns the reason the program, the arguments, or the result type cannot
/// be rendered in `profile`.
pub fn render_harness(
    program: &Program,
    profile: Profile,
    entry: u64,
    arguments: &[Value],
) -> Result<String, String> {
    super::check::check_arguments(program, entry, arguments)?;
    let mut renderer = Renderer::new(program, profile)?;
    let function = program
        .functions
        .get(index(entry)?)
        .ok_or_else(|| format!("function {entry} is not declared"))?;
    let mut out = String::from("#![forbid(unsafe_code)]\nuse program::*;\n");
    let mut adts = BTreeSet::new();
    if shown(program, &function.result, &mut adts) {
        out.push_str(QUOTE);
    }
    for adt in adts {
        let declared = program
            .adts
            .get(index(adt)?)
            .ok_or_else(|| format!("ADT {adt} is not declared"))?;
        let _ = write!(
            out,
            "\nfn show_adt{adt}(value: Adt{adt}) -> String {{ match value {{ "
        );
        for (constructor, fields) in declared.constructors.iter().enumerate() {
            let mut patterns = Vec::new();
            let mut shows = Vec::new();
            for (position, ty) in fields.iter().enumerate() {
                let held = format!("x{position}");
                let read = if renderer.boxed(Owner::Adt(adt), ty)? {
                    format!("(*{held}).clone()")
                } else {
                    held.clone()
                };
                patterns.push(held);
                shows.push(show(ty, &read)?);
            }
            let tagged = |fields: &str| {
                format!("format!(\"{{{{\\\"kind\\\":\\\"adt\\\",\\\"constructor\\\":{constructor},\\\"fields\\\":[{{}}]}}}}\", {fields})")
            };
            if fields.is_empty() {
                let _ = write!(out, "Adt{adt}::C{constructor} => {}, ", tagged("\"\""));
            } else {
                let _ = write!(
                    out,
                    "Adt{adt}::C{constructor}({}) => {}, ",
                    patterns.join(", "),
                    tagged(&format!("[{}].join(\",\")", shows.join(", ")))
                );
            }
        }
        out.push_str("} }\n");
    }
    let rendered = arguments
        .iter()
        .zip(&function.types)
        .map(|(argument, ty)| renderer.value(argument, ty))
        .collect::<Result<Vec<_>, _>>()?;
    let result_show = show(&function.result, "value")?;
    let _ = write!(
        out,
        "\nfn main() {{\n    match f{entry}({}) {{\n        Ok(value) => println!(\"{{}}\", {result_show}),\n        Err(Overflow) => println!(\"{{{{\\\"kind\\\":\\\"overflow\\\"}}}}\"),\n    }}\n    println!(\"work {{}}\", work());\n}}\n",
        rendered.join(", ")
    );
    Ok(out)
}

/// The observable outcome a rendered harness prints for an outcome of the
/// denotation: the value, or `overflow`; steps are not observable in Rust.
#[must_use]
pub fn observable(outcome: &super::Outcome) -> Option<serde_json::Value> {
    match outcome {
        super::Outcome::Value { value, .. } => serde_json::to_value(value).ok(),
        super::Outcome::Overflow { .. } => Some(serde_json::json!({"kind": "overflow"})),
        super::Outcome::Stuck | super::Outcome::Exhausted => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{CORE_RUNTIME, STD_RUNTIME};

    #[test]
    fn the_core_runtime_names_no_heap_type() {
        for heap in ["Vec", "String", "Box", "Rc", "std::", "alloc::", "format!"] {
            assert!(!CORE_RUNTIME.contains(heap), "{heap}");
        }
    }

    /// Every loop of the runtime counts its iterations, except the release
    /// of a list, whose cells were counted when built.
    #[test]
    fn every_runtime_loop_ticks() {
        for runtime in [CORE_RUNTIME, STD_RUNTIME] {
            let lines: Vec<&str> = runtime.lines().collect();
            for (number, line) in lines.iter().enumerate() {
                let trimmed = line.trim_start();
                let looping = trimmed.starts_with("while ")
                    || trimmed.starts_with("for ")
                    || trimmed.starts_with("loop ")
                    || trimmed.contains("{ while ")
                    || trimmed.contains("{ for ");
                if looping && !line.ends_with("// release") {
                    let body = lines[number..(number + 3).min(lines.len())].join("\n");
                    assert!(body.contains("tick("), "an uncounted loop: {line}");
                }
            }
        }
    }
}
