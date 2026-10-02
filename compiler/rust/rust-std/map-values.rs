#![forbid(unsafe_code)]
use core::cmp::Ordering;
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

pub fn f0(v0: List<(u64, Str)>) -> R<List<Str>> { Ok(f1(v0.clone())?) }

pub fn f1(v0: List<(u64, Str)>) -> R<List<Str>> { Ok({ let m1 = v0.clone(); match m1.uncons() { None => { List::<Str>::nil() } Some((v1, v2)) => { List::cons({ let (_, r2) = v1.clone(); r2 }, f1(v2.clone())?) } } }) }
