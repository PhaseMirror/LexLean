#![no_std]
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

pub fn f0(v0: i64, v1: i64, v2: u32) -> R<(Option<i64>, (Option<i64>, (Option<i64>, (Option<i64>, (Option<i64>, (i64, (i64, (i64, (i64, (Option<i64>, (Option<i64>, (Option<i64>, (Option<i64>, (bool, Ordering))))))))))))))> { Ok(({ let a1 = v0.clone(); let a2 = v1.clone(); fixed_i64::checked_add(a1, a2)? }, ({ let a3 = v1.clone(); let a4 = v0.clone(); fixed_i64::checked_sub(a3, a4)? }, ({ let a5 = v0.clone(); let a6 = v1.clone(); fixed_i64::checked_mul(a5, a6)? }, ({ let a7 = v0.clone(); let a8 = v1.clone(); fixed_i64::checked_quot(a7, a8)? }, ({ let a9 = v1.clone(); checked_neg_i64(a9)? }, ({ let a10 = v0.clone(); let a11 = v1.clone(); fixed_i64::bit_and(a10, a11)? }, ({ let a12 = v0.clone(); let a13 = v1.clone(); fixed_i64::bit_or(a12, a13)? }, ({ let a14 = v0.clone(); let a15 = v1.clone(); fixed_i64::bit_xor(a14, a15)? }, ({ let a16 = v1.clone(); fixed_i64::bit_not(a16)? }, ({ let a17 = v1.clone(); let a18 = v2.clone(); fixed_i64::shift_left(a17, a18)? }, ({ let a19 = v0.clone(); let a20 = v2.clone(); fixed_i64::shift_right(a19, a20)? }, ({ let a21 = v1.clone(); let a22 = 64u32; fixed_i64::shift_left(a21, a22)? }, ({ let a23 = 300i64; fixed_i64::convert(i128::from(a23))? }, ({ let a24 = v0.clone(); let a25 = v1.clone(); equal(a24, a25)? }, { let a26 = v0.clone(); let a27 = v1.clone(); compare(a26, a27)? }))))))))))))))) }
