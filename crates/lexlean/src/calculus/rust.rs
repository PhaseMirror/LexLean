//! The reference rendering of target programs to Rust (SPEC.md §17.14).
//!
//! The Rust profile is fixed: one source file with `#![forbid(unsafe_code)]`,
//! only `core` and `std` items, `nat` as `u64` and `int` as `i64` with every
//! out-of-range result an explicit `Err(Overflow)`, lists as `Vec`, ADTs as
//! enums with boxed fields, function values defunctionalized into one enum
//! per function type, and values cloned on use, so Rust's ownership never
//! changes an observable value. A rendered function never panics on a valid
//! program; fuel and stack depth are outside the observable contract.
//! Rendering refuses a program it cannot render faithfully.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::check::Checker;
use super::{Expr, IntKind, OrderingValue, Prim, Program, Shape, Ty, Value};

/// The fixed runtime every rendered program carries.
pub const PRELUDE: &str = r#"#![forbid(unsafe_code)]
#![allow(dead_code, unused_parens, unused_mut, unused_variables, clippy::all)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Overflow;
pub type R<T> = Result<T, Overflow>;

fn nat_add(a: u64, b: u64) -> R<u64> { a.checked_add(b).ok_or(Overflow) }
fn nat_sub(a: u64, b: u64) -> R<u64> { Ok(a.saturating_sub(b)) }
fn nat_mul(a: u64, b: u64) -> R<u64> { a.checked_mul(b).ok_or(Overflow) }
fn nat_quot(a: u64, b: u64, z: u64) -> R<u64> { Ok(if b == 0 { z } else { a / b }) }
fn nat_rem(a: u64, b: u64, z: u64) -> R<u64> { Ok(if b == 0 { z } else { a % b }) }
fn nat_eq(a: u64, b: u64) -> R<bool> { Ok(a == b) }
fn nat_le(a: u64, b: u64) -> R<bool> { Ok(a <= b) }
fn nat_lt(a: u64, b: u64) -> R<bool> { Ok(a < b) }
fn nat_succ(a: u64) -> R<u64> { a.checked_add(1).ok_or(Overflow) }
fn int_add(a: i64, b: i64) -> R<i64> { a.checked_add(b).ok_or(Overflow) }
fn int_sub(a: i64, b: i64) -> R<i64> { a.checked_sub(b).ok_or(Overflow) }
fn int_mul(a: i64, b: i64) -> R<i64> { a.checked_mul(b).ok_or(Overflow) }
fn int_neg(a: i64) -> R<i64> { a.checked_neg().ok_or(Overflow) }
fn int_quot(a: i64, b: i64, z: i64) -> R<i64> { if b == 0 { Ok(z) } else { a.checked_div(b).ok_or(Overflow) } }
fn int_rem(a: i64, b: i64, z: i64) -> R<i64> { if b == 0 { Ok(z) } else { Ok(a.wrapping_rem(b)) } }
fn bool_not(a: bool) -> R<bool> { Ok(!a) }
fn bool_and(a: bool, b: bool) -> R<bool> { Ok(a && b) }
fn bool_or(a: bool, b: bool) -> R<bool> { Ok(a || b) }
fn equal<T: PartialEq>(a: T, b: T) -> R<bool> { Ok(a == b) }
fn append<T>(mut a: Vec<T>, b: Vec<T>) -> R<Vec<T>> { a.extend(b); Ok(a) }
fn length_list<T>(a: Vec<T>) -> R<u64> { u64::try_from(a.len()).map_err(|_| Overflow) }
fn length_string(a: String) -> R<u64> { u64::try_from(a.chars().count()).map_err(|_| Overflow) }
fn index<T: Clone>(a: Vec<T>, i: u64) -> R<Option<T>> { Ok(usize::try_from(i).ok().and_then(|i| a.get(i)).cloned()) }
fn slice<T: Clone>(a: Vec<T>, start: u64, count: u64) -> R<Option<Vec<T>>> {
    let end = u128::from(start) + u128::from(count);
    if end > a.len() as u128 { return Ok(None); }
    Ok(Some(a[start as usize..end as usize].to_vec()))
}
fn utf8_encode(a: String) -> R<Vec<u8>> { Ok(a.into_bytes()) }
fn utf8_decode(a: Vec<u8>) -> R<Option<String>> { Ok(String::from_utf8(a).ok()) }
fn compare_bytes(a: Vec<u8>, b: Vec<u8>) -> R<std::cmp::Ordering> { Ok(a.cmp(&b)) }
fn compare<T: Ord>(a: T, b: T) -> R<std::cmp::Ordering> { Ok(a.cmp(&b)) }
fn split_exact(text: String, delimiter: String, maximum: u32) -> R<Option<Vec<String>>> {
    if delimiter.is_empty() { return Ok(None); }
    let fields: Vec<String> = text.split(delimiter.as_str()).map(str::to_owned).collect();
    Ok(if fields.len() as u128 <= u128::from(maximum) { Some(fields) } else { None })
}
fn join(texts: Vec<String>, delimiter: String) -> R<String> { Ok(texts.join(&delimiter)) }
fn canonical_decimal(text: &str) -> Option<Option<i128>> {
    let digits = text.strip_prefix('-').unwrap_or(text);
    let canonical = !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit())
        && (digits == "0" || !digits.starts_with('0')) && text != "-0";
    if !canonical { return None; }
    Some(text.parse::<i128>().ok())
}
fn parse_int(text: String) -> R<Option<i64>> {
    match canonical_decimal(&text) {
        None => Ok(None),
        Some(Some(n)) => i64::try_from(n).map(Some).map_err(|_| Overflow),
        Some(None) => Err(Overflow),
    }
}

macro_rules! fixed {
    ($t:ident) => {
        pub mod $t {
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
            pub fn format(a: $t) -> R<String> { Ok(a.to_string()) }
            pub fn parse(text: String) -> R<Option<$t>> {
                Ok(match super::canonical_decimal(&text) { Some(Some(n)) => $t::try_from(n).ok(), _ => None })
            }
        }
    };
}
fixed!(u8); fixed!(u16); fixed!(u32); fixed!(u64); fixed!(i8); fixed!(i16); fixed!(i32); fixed!(i64);
fn checked_neg_i8(a: i8) -> R<Option<i8>> { Ok(a.checked_neg()) }
fn checked_neg_i16(a: i16) -> R<Option<i16>> { Ok(a.checked_neg()) }
fn checked_neg_i32(a: i32) -> R<Option<i32>> { Ok(a.checked_neg()) }
fn checked_neg_i64(a: i64) -> R<Option<i64>> { Ok(a.checked_neg()) }

pub trait Show { fn show(&self) -> String; }
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
impl Show for () { fn show(&self) -> String { "{\"kind\":\"unit\"}".to_owned() } }
impl Show for bool { fn show(&self) -> String { format!("{{\"kind\":\"bool\",\"value\":{}}}", self) } }
impl Show for String { fn show(&self) -> String { format!("{{\"kind\":\"string\",\"value\":{}}}", quote(self)) } }
impl Show for std::cmp::Ordering {
    fn show(&self) -> String {
        let value = match self { std::cmp::Ordering::Less => "lt", std::cmp::Ordering::Equal => "eq", std::cmp::Ordering::Greater => "gt" };
        format!("{{\"kind\":\"ordering\",\"value\":\"{value}\"}}")
    }
}
impl<T: Show> Show for Option<T> {
    fn show(&self) -> String {
        match self { None => "{\"kind\":\"none\"}".to_owned(), Some(v) => format!("{{\"kind\":\"some\",\"value\":{}}}", v.show()) }
    }
}
impl<T: Show, E: Show> Show for Result<T, E> {
    fn show(&self) -> String {
        match self { Ok(v) => format!("{{\"kind\":\"ok\",\"value\":{}}}", v.show()), Err(v) => format!("{{\"kind\":\"error\",\"value\":{}}}", v.show()) }
    }
}
impl<A: Show, B: Show> Show for (A, B) {
    fn show(&self) -> String { format!("{{\"kind\":\"pair\",\"left\":{},\"right\":{}}}", self.0.show(), self.1.show()) }
}
pub struct Nat(pub u64);
pub struct Int(pub i64);
impl Show for Nat { fn show(&self) -> String { format!("{{\"kind\":\"nat\",\"value\":\"{}\"}}", self.0) } }
impl Show for Int { fn show(&self) -> String { format!("{{\"kind\":\"int\",\"value\":\"{}\"}}", self.0) } }
macro_rules! show_fixed { ($t:ident, $k:literal) => { impl Show for $t { fn show(&self) -> String { format!("{{\"kind\":\"{}\",\"value\":\"{}\"}}", $k, self) } } }; }
show_fixed!(u8, "u8"); show_fixed!(u16, "u16"); show_fixed!(u32, "u32"); show_fixed!(i8, "i8"); show_fixed!(i16, "i16"); show_fixed!(i32, "i32");
pub struct Fixed64U(pub u64);
pub struct Fixed64I(pub i64);
impl Show for Fixed64U { fn show(&self) -> String { format!("{{\"kind\":\"u64\",\"value\":\"{}\"}}", self.0) } }
impl Show for Fixed64I { fn show(&self) -> String { format!("{{\"kind\":\"i64\",\"value\":\"{}\"}}", self.0) } }
pub struct Bytes(pub Vec<u8>);
impl Show for Bytes { fn show(&self) -> String { format!("{{\"kind\":\"bytes\",\"hex\":\"{}\"}}", self.0.iter().map(|b| format!("{b:02x}")).collect::<String>()) } }
pub fn show_list(items: Vec<String>) -> String { format!("{{\"kind\":\"list\",\"items\":[{}]}}", items.join(",")) }
pub fn show_adt(constructor: u64, fields: Vec<String>) -> String { format!("{{\"kind\":\"adt\",\"constructor\":{},\"fields\":[{}]}}", constructor, fields.join(",")) }
"#;

/// The rendering context: the program, its function types, and fresh names.
struct Renderer<'a> {
    program: &'a Program,
    checker: Checker<'a>,
    /// Every function type of the program, numbered in first-use order.
    fn_types: Vec<Ty>,
    /// For each function type, the closures that inhabit it: function index
    /// and captured types.
    closures: BTreeMap<usize, Vec<(u64, Vec<Ty>)>>,
    fresh: usize,
}

fn fail<T>(reason: impl Into<String>) -> Result<T, String> {
    Err(reason.into())
}

impl<'a> Renderer<'a> {
    fn fn_index(&mut self, ty: &Ty) -> usize {
        if let Some(position) = self.fn_types.iter().position(|known| known == ty) {
            return position;
        }
        self.fn_types.push(ty.clone());
        self.fn_types.len() - 1
    }

    fn fresh(&mut self, base: &str) -> String {
        self.fresh += 1;
        format!("{base}{}", self.fresh)
    }

    fn ty(&mut self, ty: &Ty) -> String {
        match ty {
            Ty::Unit => "()".to_owned(),
            Ty::Bool => "bool".to_owned(),
            Ty::Nat => "u64".to_owned(),
            Ty::Int => "i64".to_owned(),
            Ty::Fixed { width } => width.name().to_owned(),
            Ty::String => "String".to_owned(),
            Ty::Bytes => "Vec<u8>".to_owned(),
            Ty::Ordering => "std::cmp::Ordering".to_owned(),
            Ty::Option { value } => format!("Option<{}>", self.ty(value)),
            Ty::Result { ok, error } => format!("Result<{}, {}>", self.ty(ok), self.ty(error)),
            Ty::List { element } => format!("Vec<{}>", self.ty(element)),
            Ty::Pair { left, right } => format!("({}, {})", self.ty(left), self.ty(right)),
            Ty::Adt { index } => format!("Adt{index}"),
            Ty::Fn { .. } => format!("Fn{}", self.fn_index(ty)),
        }
    }

    /// Collect every closure site, so each function type's enum is closed.
    fn collect(&mut self, expr: &Expr, scope: &mut Vec<(u64, Ty)>) -> Result<(), String> {
        let mut children: Vec<&Expr> = Vec::new();
        match expr {
            Expr::Value { .. } | Expr::Var { .. } => {}
            Expr::Let {
                name,
                ty,
                bound,
                body,
            } => {
                self.collect(bound, scope)?;
                scope.push((*name, ty.clone()));
                self.collect(body, scope)?;
                scope.pop();
                return Ok(());
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
                scrutinee, arms, ..
            } => {
                self.collect(scrutinee, scope)?;
                let scrutinee_ty = self.checker.expr(scrutinee, scope)?;
                for arm in arms {
                    let fields = self.checker.shape_fields(arm.shape, &scrutinee_ty)?;
                    let depth = scope.len();
                    scope.extend(arm.binders.iter().copied().zip(fields));
                    self.collect(&arm.body, scope)?;
                    scope.truncate(depth);
                }
                return Ok(());
            }
            Expr::Build { operands, .. }
            | Expr::Call { operands, .. }
            | Expr::Prim { operands, .. } => children.extend(operands),
            Expr::Closure { function, captures } => {
                children.extend(captures);
                let ty = self.checker.expr(expr, scope)?;
                let index = self.fn_index(&ty);
                let captured = self.program.functions
                    [usize::try_from(*function).map_err(|e| e.to_string())?]
                .types[..captures.len()]
                    .to_vec();
                let entry = self.closures.entry(index).or_default();
                if !entry.iter().any(|(known, _)| known == function) {
                    entry.push((*function, captured));
                }
            }
            Expr::Apply { target, operands } => {
                children.push(target);
                children.extend(operands);
            }
            Expr::First { value } | Expr::Second { value } | Expr::Field { value, .. } => {
                children.push(value)
            }
        }
        for child in children {
            self.collect(child, scope)?;
        }
        Ok(())
    }

    fn value(&mut self, value: &Value, ty: &Ty) -> Result<String, String> {
        let int_literal = |text: &str, kind: &str, min: &str| -> String {
            if text == min {
                format!("{kind}::MIN")
            } else {
                format!("({text}{kind})")
            }
        };
        Ok(match (value, ty) {
            (Value::Unit, _) => "()".to_owned(),
            (Value::Bool { value }, _) => value.to_string(),
            (Value::Nat { value }, _) => format!("{value}u64"),
            (Value::Int { value }, _) => int_literal(value, "i64", "-9223372036854775808"),
            (Value::U8 { value }, _) => format!("{value}u8"),
            (Value::U16 { value }, _) => format!("{value}u16"),
            (Value::U32 { value }, _) => format!("{value}u32"),
            (Value::U64 { value }, _) => format!("{value}u64"),
            (Value::I8 { value }, _) => int_literal(value, "i8", "-128"),
            (Value::I16 { value }, _) => int_literal(value, "i16", "-32768"),
            (Value::I32 { value }, _) => int_literal(value, "i32", "-2147483648"),
            (Value::I64 { value }, _) => int_literal(value, "i64", "-9223372036854775808"),
            (Value::String { value }, _) => format!("String::from({value:?})"),
            (Value::Bytes { hex }, _) => {
                let bytes: Vec<String> = (0..hex.len())
                    .step_by(2)
                    .map(|at| format!("0x{}u8", &hex[at..at + 2]))
                    .collect();
                format!("vec![{}] as Vec<u8>", bytes.join(", "))
            }
            (Value::Ordering { value }, _) => match value {
                OrderingValue::Lt => "std::cmp::Ordering::Less",
                OrderingValue::Eq => "std::cmp::Ordering::Equal",
                OrderingValue::Gt => "std::cmp::Ordering::Greater",
            }
            .to_owned(),
            (Value::None, ty) => format!("None::<{}>", self.ty(option_inner(ty)?)),
            (Value::Some { value }, Ty::Option { value: inner }) => {
                format!("Some({})", self.value(value, inner)?)
            }
            (Value::Ok { value }, Ty::Result { ok, error }) => {
                format!(
                    "Ok::<{}, {}>({})",
                    self.ty(ok),
                    self.ty(error),
                    self.value(value, ok)?
                )
            }
            (Value::Error { value }, Ty::Result { ok, error }) => {
                format!(
                    "Err::<{}, {}>({})",
                    self.ty(ok),
                    self.ty(error),
                    self.value(value, error)?
                )
            }
            (Value::List { items }, Ty::List { element }) => {
                let rendered = items
                    .iter()
                    .map(|item| self.value(item, element))
                    .collect::<Result<Vec<_>, _>>()?;
                format!(
                    "{{ let items: Vec<{}> = vec![{}]; items }}",
                    self.ty(element),
                    rendered.join(", ")
                )
            }
            (
                Value::Pair { left, right },
                Ty::Pair {
                    left: lt,
                    right: rt,
                },
            ) => {
                format!("({}, {})", self.value(left, lt)?, self.value(right, rt)?)
            }
            (
                Value::Adt {
                    constructor,
                    fields,
                },
                Ty::Adt { index },
            ) => {
                let types = self.program.adts
                    [usize::try_from(*index).map_err(|e| e.to_string())?]
                .constructors[usize::try_from(*constructor).map_err(|e| e.to_string())?]
                .clone();
                let rendered = fields
                    .iter()
                    .zip(&types)
                    .map(|(field, ty)| {
                        self.value(field, ty)
                            .map(|text| format!("Box::new({text})"))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                if rendered.is_empty() {
                    format!("Adt{index}::C{constructor}")
                } else {
                    format!("Adt{index}::C{constructor}({})", rendered.join(", "))
                }
            }
            (value, ty) => {
                return fail(format!(
                    "the literal {value:?} cannot be rendered at type {ty:?}"
                ))
            }
        })
    }

    fn prim(
        &mut self,
        operation: &Prim,
        operands: &[Ty],
        rendered: &[String],
    ) -> Result<String, String> {
        let args = rendered.join(", ");
        let fixed_of = |ty: &Ty| -> Result<&'static str, String> {
            match ty {
                Ty::Fixed { width } => Ok(width.name()),
                other => fail(format!("{other:?} is not fixed-width")),
            }
        };
        let first = operands.first().cloned().unwrap_or(Ty::Unit);
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
            Prim::CheckedAdd => format!("{}::checked_add({args})?", fixed_of(&first)?),
            Prim::CheckedSub => format!("{}::checked_sub({args})?", fixed_of(&first)?),
            Prim::CheckedMul => format!("{}::checked_mul({args})?", fixed_of(&first)?),
            Prim::CheckedQuot => format!("{}::checked_quot({args})?", fixed_of(&first)?),
            Prim::CheckedNeg => format!("checked_neg_{}({args})?", fixed_of(&first)?),
            Prim::BitAnd => format!("{}::bit_and({args})?", fixed_of(&first)?),
            Prim::BitOr => format!("{}::bit_or({args})?", fixed_of(&first)?),
            Prim::BitXor => format!("{}::bit_xor({args})?", fixed_of(&first)?),
            Prim::BitNot => format!("{}::bit_not({args})?", fixed_of(&first)?),
            Prim::ShiftLeft => format!("{}::shift_left({args})?", fixed_of(&first)?),
            Prim::ShiftRight => format!("{}::shift_right({args})?", fixed_of(&first)?),
            Prim::Equal => format!("equal({args})?"),
            Prim::BoolNot => format!("bool_not({args})?"),
            Prim::BoolAnd => format!("bool_and({args})?"),
            Prim::BoolOr => format!("bool_or({args})?"),
            Prim::Append => format!("append({args})?"),
            Prim::Length => match first {
                Ty::String => format!("length_string({args})?"),
                _ => format!("length_list({args})?"),
            },
            Prim::Index => format!("index({args})?"),
            Prim::Slice => format!("slice({args})?"),
            Prim::Utf8Encode => format!("utf8_encode({args})?"),
            Prim::Utf8Decode => format!("utf8_decode({args})?"),
            Prim::CompareBytes => format!("compare_bytes({args})?"),
            Prim::Compare => format!("compare({args})?"),
            Prim::SplitExact => format!("split_exact({args})?"),
            Prim::Join => format!("join({args})?"),
            Prim::FormatDecimal => match first {
                Ty::Int => format!("i64::format({args})?"),
                other => format!("{}::format({args})?", fixed_of(&other)?),
            },
            Prim::Convert { target } => format!("{}::convert(i128::from({args}))?", target.name()),
            Prim::ParseDecimal { target } => match target {
                Ty::Int => format!("parse_int({args})?"),
                other => format!("{}::parse({args})?", fixed_of(other)?),
            },
        })
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
                let rust_ty = self.ty(ty);
                scope.push((*name, ty.clone()));
                let body = self.expr(body, scope);
                scope.pop();
                format!("{{ let v{name}: {rust_ty} = {bound}; {} }}", body?)
            }
            Expr::Cond {
                condition,
                then_branch,
                else_branch,
            } => format!(
                "(if {} {{ {} }} else {{ {} }})",
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
                let mut bodies: BTreeMap<Shape, (Vec<u64>, String)> = BTreeMap::new();
                for arm in arms {
                    let fields = self.checker.shape_fields(arm.shape, &scrutinee_ty)?;
                    let depth = scope.len();
                    scope.extend(arm.binders.iter().copied().zip(fields));
                    let body = self.expr(&arm.body, scope);
                    scope.truncate(depth);
                    bodies.insert(arm.shape, (arm.binders.clone(), body?));
                }
                let take = |bodies: &mut BTreeMap<Shape, (Vec<u64>, String)>, shape: Shape| {
                    bodies
                        .remove(&shape)
                        .ok_or_else(|| format!("the match lacks {shape:?}"))
                };
                let body = match &scrutinee_ty {
                    Ty::Option { .. } => {
                        let (_, none) = take(&mut bodies, Shape::None)?;
                        let (b, some) = take(&mut bodies, Shape::Some)?;
                        format!(
                            "match {holder} {{ None => {{ {none} }}, Some(v{}) => {{ {some} }} }}",
                            b[0]
                        )
                    }
                    Ty::Result { .. } => {
                        let (b, ok) = take(&mut bodies, Shape::Ok)?;
                        let (c, error) = take(&mut bodies, Shape::Error)?;
                        format!(
                            "match {holder} {{ Ok(v{}) => {{ {ok} }}, Err(v{}) => {{ {error} }} }}",
                            b[0], c[0]
                        )
                    }
                    Ty::List { .. } => {
                        let (_, nil) = take(&mut bodies, Shape::Nil)?;
                        let (b, cons) = take(&mut bodies, Shape::Cons)?;
                        format!(
                            "if {holder}.is_empty() {{ {nil} }} else {{ let v{} = {holder}[0].clone(); let v{} = {holder}[1..].to_vec(); {cons} }}",
                            b[0], b[1]
                        )
                    }
                    Ty::Nat => {
                        let (_, zero) = take(&mut bodies, Shape::Zero)?;
                        let (b, succ) = take(&mut bodies, Shape::Succ)?;
                        format!("if {holder} == 0 {{ {zero} }} else {{ let v{} = {holder} - 1; {succ} }}", b[0])
                    }
                    Ty::Bool => {
                        let (_, yes) = take(&mut bodies, Shape::True)?;
                        let (_, no) = take(&mut bodies, Shape::False)?;
                        format!("if {holder} {{ {yes} }} else {{ {no} }}")
                    }
                    Ty::Pair { .. } => {
                        let (b, body) = take(&mut bodies, Shape::Pair)?;
                        format!("{{ let (v{}, v{}) = {holder}; {body} }}", b[0], b[1])
                    }
                    Ty::Unit => take(&mut bodies, Shape::Unit)?.1,
                    Ty::Ordering => {
                        let (_, less) = take(&mut bodies, Shape::Lt)?;
                        let (_, equal) = take(&mut bodies, Shape::Eq)?;
                        let (_, greater) = take(&mut bodies, Shape::Gt)?;
                        format!("match {holder} {{ std::cmp::Ordering::Less => {{ {less} }}, std::cmp::Ordering::Equal => {{ {equal} }}, std::cmp::Ordering::Greater => {{ {greater} }} }}")
                    }
                    Ty::Adt { index } => {
                        let mut out = format!("match {holder} {{ ");
                        let count = self.program.adts
                            [usize::try_from(*index).map_err(|e| e.to_string())?]
                        .constructors
                        .len() as u64;
                        for constructor in 0..count {
                            let (binders, body) = take(&mut bodies, Shape::Adt { constructor })?;
                            if binders.is_empty() {
                                let _ = write!(out, "Adt{index}::C{constructor} => {{ {body} }}, ");
                            } else {
                                let names: Vec<String> =
                                    binders.iter().map(|b| format!("f{b}")).collect();
                                let unboxed: String = binders
                                    .iter()
                                    .map(|b| format!("let v{b} = *f{b}; "))
                                    .collect();
                                let _ = write!(
                                    out,
                                    "Adt{index}::C{constructor}({}) => {{ {unboxed}{body} }}, ",
                                    names.join(", ")
                                );
                            }
                        }
                        out.push('}');
                        out
                    }
                    other => return fail(format!("cannot render a match on {other:?}")),
                };
                if !bodies.is_empty() {
                    return fail("a match arm has no rendering");
                }
                format!("{{ let {holder} = {rendered}; {body} }}")
            }
            Expr::Build {
                shape,
                ty,
                operands,
            } => {
                let rendered = operands
                    .iter()
                    .map(|operand| self.expr(operand, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                match (shape, ty) {
                    (Shape::None, ty) => format!("None::<{}>", self.ty(option_inner(ty)?)),
                    (Shape::Some, _) => format!("Some({})", rendered[0]),
                    (Shape::Ok, Ty::Result { ok, error }) => {
                        format!("Ok::<{}, {}>({})", self.ty(ok), self.ty(error), rendered[0])
                    }
                    (Shape::Error, Ty::Result { ok, error }) => format!(
                        "Err::<{}, {}>({})",
                        self.ty(ok),
                        self.ty(error),
                        rendered[0]
                    ),
                    (Shape::Nil, Ty::List { element }) => {
                        format!("Vec::<{}>::new()", self.ty(element))
                    }
                    (Shape::Cons, _) => {
                        let head = self.fresh("h");
                        let tail = self.fresh("t");
                        format!("{{ let {head} = {}; let mut {tail} = {}; {tail}.insert(0, {head}); {tail} }}", rendered[0], rendered[1])
                    }
                    (Shape::Zero, _) => "0u64".to_owned(),
                    (Shape::Succ, _) => format!("nat_succ({})?", rendered[0]),
                    (Shape::Pair, _) => format!("({}, {})", rendered[0], rendered[1]),
                    (Shape::True, _) => "true".to_owned(),
                    (Shape::False, _) => "false".to_owned(),
                    (Shape::Unit, _) => "()".to_owned(),
                    (Shape::Lt, _) => "std::cmp::Ordering::Less".to_owned(),
                    (Shape::Eq, _) => "std::cmp::Ordering::Equal".to_owned(),
                    (Shape::Gt, _) => "std::cmp::Ordering::Greater".to_owned(),
                    (Shape::Adt { constructor }, Ty::Adt { index }) => {
                        if rendered.is_empty() {
                            format!("Adt{index}::C{constructor}")
                        } else {
                            let boxed: Vec<String> = rendered
                                .iter()
                                .map(|operand| format!("Box::new({operand})"))
                                .collect();
                            format!("Adt{index}::C{constructor}({})", boxed.join(", "))
                        }
                    }
                    (shape, ty) => return fail(format!("cannot render {shape:?} at {ty:?}")),
                }
            }
            Expr::Call { function, operands } => {
                let rendered = operands
                    .iter()
                    .map(|operand| self.expr(operand, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                format!("f{function}({})?", rendered.join(", "))
            }
            Expr::Closure { function, captures } => {
                let ty = self.checker.expr(expr, scope)?;
                let index = self.fn_index(&ty);
                let rendered = captures
                    .iter()
                    .map(|capture| self.expr(capture, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                if rendered.is_empty() {
                    format!("Fn{index}::F{function}")
                } else {
                    let boxed: Vec<String> = rendered
                        .iter()
                        .map(|capture| format!("Box::new({capture})"))
                        .collect();
                    format!("Fn{index}::F{function}({})", boxed.join(", "))
                }
            }
            Expr::Apply { target, operands } => {
                let callee = self.fresh("c");
                let target = self.expr(target, scope)?;
                let rendered = operands
                    .iter()
                    .map(|operand| self.expr(operand, scope))
                    .collect::<Result<Vec<_>, _>>()?;
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
                let rendered = operands
                    .iter()
                    .map(|operand| self.expr(operand, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                // Arguments are bound left to right before the call, so the
                // evaluation order is the denotation's whatever Rust does
                // with nested `?`.
                let names: Vec<String> = rendered.iter().map(|_| self.fresh("a")).collect();
                let bindings: String = names
                    .iter()
                    .zip(&rendered)
                    .map(|(name, text)| format!("let {name} = {text}; "))
                    .collect();
                format!("{{ {bindings}{} }}", self.prim(operation, &types, &names)?)
            }
            Expr::First { value } => format!("({}).0", self.expr(value, scope)?),
            Expr::Second { value } => format!("({}).1", self.expr(value, scope)?),
            Expr::Field { value, index } => {
                let Ty::Adt { index: adt } = self.checker.expr(value, scope)? else {
                    return fail("`field` of a non-ADT value");
                };
                let arity = self.program.adts[usize::try_from(adt).map_err(|e| e.to_string())?]
                    .constructors[0]
                    .len();
                let names: Vec<String> =
                    (0..arity).map(|position| format!("g{position}")).collect();
                format!(
                    "match {} {{ Adt{adt}::C0({}) => *g{index} }}",
                    self.expr(value, scope)?,
                    names.join(", ")
                )
            }
        })
    }
}

fn option_inner(ty: &Ty) -> Result<&Ty, String> {
    match ty {
        Ty::Option { value } => Ok(value),
        other => fail(format!("`none` at {other:?}")),
    }
}

/// Render a valid program to Rust source: the prelude, its ADTs, function
/// types, and functions.
///
/// # Errors
///
/// Returns the reason the program is invalid or cannot be rendered.
pub fn render(program: &Program) -> Result<String, String> {
    super::check::check(program)?;
    let mut renderer = Renderer {
        program,
        checker: Checker { program },
        fn_types: Vec::new(),
        closures: BTreeMap::new(),
        fresh: 0,
    };
    for function in &program.functions {
        let mut scope: Vec<(u64, Ty)> = function
            .parameters
            .iter()
            .copied()
            .zip(function.types.iter().cloned())
            .collect();
        renderer.collect(&function.body, &mut scope)?;
        for ty in function.types.iter().chain([&function.result]) {
            let _ = renderer.ty(ty);
        }
    }
    for adt in &program.adts {
        for ty in adt.constructors.iter().flatten() {
            let _ = renderer.ty(ty);
        }
    }
    let mut out = String::from(PRELUDE);
    for (index, adt) in program.adts.iter().enumerate() {
        let _ = write!(out, "\n#[derive(Clone, Debug)]\npub enum Adt{index} {{ ");
        for (constructor, fields) in adt.constructors.iter().enumerate() {
            if fields.is_empty() {
                let _ = write!(out, "C{constructor}, ");
            } else {
                let boxed: Vec<String> = fields
                    .iter()
                    .map(|ty| format!("Box<{}>", renderer.ty(ty)))
                    .collect();
                let _ = write!(out, "C{constructor}({}), ", boxed.join(", "));
            }
        }
        out.push_str("}\n");
    }
    let fn_types = renderer.fn_types.clone();
    for (index, fn_ty) in fn_types.iter().enumerate() {
        let Ty::Fn { parameters, result } = fn_ty else {
            return fail("a function type is not a function");
        };
        let variants = renderer.closures.get(&index).cloned().unwrap_or_default();
        let _ = write!(out, "\n#[derive(Clone, Debug)]\npub enum Fn{index} {{ ");
        for (function, captured) in &variants {
            if captured.is_empty() {
                let _ = write!(out, "F{function}, ");
            } else {
                let boxed: Vec<String> = captured
                    .iter()
                    .map(|ty| format!("Box<{}>", renderer.ty(ty)))
                    .collect();
                let _ = write!(out, "F{function}({}), ", boxed.join(", "));
            }
        }
        let params: Vec<String> = parameters
            .iter()
            .enumerate()
            .map(|(position, ty)| format!("p{position}: {}", renderer.ty(ty)))
            .collect();
        let args: Vec<String> = (0..parameters.len())
            .map(|position| format!("p{position}"))
            .collect();
        // A function type no closure inhabits is an empty enum, matched
        // without arms: no unreachable panic is rendered.
        let scrutinee = if variants.is_empty() { "*self" } else { "self" };
        let _ = write!(
            out,
            "}}\nimpl Fn{index} {{ pub fn apply(&self, {}) -> R<{}> {{ match {scrutinee} {{ ",
            params.join(", "),
            renderer.ty(result)
        );
        for (function, captured) in &variants {
            let caps: Vec<String> = (0..captured.len())
                .map(|position| format!("k{position}"))
                .collect();
            let mut all: Vec<String> = caps
                .iter()
                .map(|cap| format!("(**{cap}).clone()"))
                .collect();
            all.extend(args.iter().cloned());
            if captured.is_empty() {
                let _ = write!(
                    out,
                    "Fn{index}::F{function} => f{function}({}), ",
                    all.join(", ")
                );
            } else {
                let _ = write!(
                    out,
                    "Fn{index}::F{function}({}) => f{function}({}), ",
                    caps.join(", "),
                    all.join(", ")
                );
            }
        }
        out.push_str("} } }\n");
    }
    for (index, function) in program.functions.iter().enumerate() {
        let mut scope: Vec<(u64, Ty)> = function
            .parameters
            .iter()
            .copied()
            .zip(function.types.iter().cloned())
            .collect();
        let params: Vec<String> = function
            .parameters
            .iter()
            .zip(&function.types)
            .map(|(name, ty)| format!("v{name}: {}", renderer.ty(ty)))
            .collect();
        let body = renderer.expr(&function.body, &mut scope)?;
        let result = renderer.ty(&function.result);
        let _ = write!(
            out,
            "\npub fn f{index}({}) -> R<{result}> {{ Ok({body}) }}\n",
            params.join(", ")
        );
    }
    Ok(out)
}

/// The `show` expression of a rendered value of type `ty` named `name`.
fn show(ty: &Ty, name: &str) -> Result<String, String> {
    Ok(match ty {
        Ty::Unit
        | Ty::Bool
        | Ty::String
        | Ty::Ordering
        | Ty::Fixed {
            width:
                IntKind::U8 | IntKind::U16 | IntKind::U32 | IntKind::I8 | IntKind::I16 | IntKind::I32,
        } => {
            format!("{name}.show()")
        }
        Ty::Fixed {
            width: IntKind::U64,
        } => format!("Fixed64U({name}).show()"),
        Ty::Fixed {
            width: IntKind::I64,
        } => format!("Fixed64I({name}).show()"),
        Ty::Nat => format!("Nat({name}).show()"),
        Ty::Int => format!("Int({name}).show()"),
        Ty::Bytes => format!("Bytes({name}).show()"),
        Ty::Option { value } => {
            let inner = show(value, "x")?;
            format!("(match {name} {{ None => \"{{\\\"kind\\\":\\\"none\\\"}}\".to_owned(), Some(x) => format!(\"{{{{\\\"kind\\\":\\\"some\\\",\\\"value\\\":{{}}}}}}\", {inner}) }})")
        }
        Ty::Result { ok, error } => {
            let ok_show = show(ok, "x")?;
            let error_show = show(error, "x")?;
            format!("(match {name} {{ Ok(x) => format!(\"{{{{\\\"kind\\\":\\\"ok\\\",\\\"value\\\":{{}}}}}}\", {ok_show}), Err(x) => format!(\"{{{{\\\"kind\\\":\\\"error\\\",\\\"value\\\":{{}}}}}}\", {error_show}) }})")
        }
        Ty::List { element } => {
            let inner = show(element, "x")?;
            format!("show_list({name}.into_iter().map(|x| {inner}).collect())")
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

/// A program rendered with a `main` that runs `entry` on `arguments` and
/// prints the observable outcome: the value in its exact JSON form, or
/// `{"kind":"overflow"}`.
///
/// # Errors
///
/// Returns the reason the program, the arguments, or the result type cannot
/// be rendered.
pub fn render_harness(
    program: &Program,
    entry: u64,
    arguments: &[Value],
) -> Result<String, String> {
    super::check::check_arguments(program, entry, arguments)?;
    let mut out = render(program)?;
    let mut renderer = Renderer {
        program,
        checker: Checker { program },
        fn_types: Vec::new(),
        closures: BTreeMap::new(),
        fresh: 0,
    };
    for function in &program.functions {
        let mut scope: Vec<(u64, Ty)> = function
            .parameters
            .iter()
            .copied()
            .zip(function.types.iter().cloned())
            .collect();
        renderer.collect(&function.body, &mut scope)?;
    }
    for (index, adt) in program.adts.iter().enumerate() {
        let _ = write!(
            out,
            "\nfn show_adt{index}(value: Adt{index}) -> String {{ match value {{ "
        );
        for (constructor, fields) in adt.constructors.iter().enumerate() {
            let names: Vec<String> = (0..fields.len())
                .map(|position| format!("x{position}"))
                .collect();
            let shows = fields
                .iter()
                .enumerate()
                .map(|(position, ty)| show(ty, &format!("(*x{position})")))
                .collect::<Result<Vec<_>, _>>()?;
            if fields.is_empty() {
                let _ = write!(
                    out,
                    "Adt{index}::C{constructor} => show_adt({constructor}, Vec::new()), "
                );
            } else {
                let _ = write!(
                    out,
                    "Adt{index}::C{constructor}({}) => show_adt({constructor}, vec![{}]), ",
                    names.join(", "),
                    shows.join(", ")
                );
            }
        }
        out.push_str("} }\n");
    }
    let function = &program.functions[usize::try_from(entry).map_err(|e| e.to_string())?];
    let rendered = arguments
        .iter()
        .zip(&function.types)
        .map(|(argument, ty)| renderer.value(argument, ty))
        .collect::<Result<Vec<_>, _>>()?;
    let result_show = show(&function.result, "value")?;
    let _ = write!(
        out,
        "\nfn main() {{ match f{entry}({}) {{ Ok(value) => println!(\"{{}}\", {result_show}), Err(Overflow) => println!(\"{{{{\\\"kind\\\":\\\"overflow\\\"}}}}\") }} }}\n",
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
