//! Lowering of a valid target program to the closed Rust AST of a profile
//! (SPEC.md §17.16).

use std::collections::{BTreeMap, BTreeSet};

use super::super::check::Checker;
use super::super::{Arm, Expr as Term, Prim, Program, Shape, Ty, Value};
use super::ast::{
    Block, Callee, CaptureRead, Crate, Ctor, Dispatch, Expr, Ident, ItemDef, Let, Lit, Pat, Type,
};
use super::runtime::Item;
use super::Profile;

/// A type that can contain itself through in-place storage: an ADT, or a
/// function type through its closures' captures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum Owner {
    Adt(u64),
    Fn(usize),
}

fn fail<T>(reason: impl Into<String>) -> Result<T, String> {
    Err(reason.into())
}

pub(super) fn index(number: u64) -> Result<usize, String> {
    usize::try_from(number).map_err(|error| error.to_string())
}

/// Whether local `name` occurs free in `expr`.
pub(super) fn mentions(expr: &Term, name: u64) -> bool {
    let any = |exprs: &[Term]| exprs.iter().any(|expr| mentions(expr, name));
    match expr {
        Term::Value { .. } => false,
        Term::Var { name: used } => *used == name,
        Term::Let {
            name: bound,
            bound: value,
            body,
            ..
        } => mentions(value, name) || (*bound != name && mentions(body, name)),
        Term::Cond {
            condition,
            then_branch,
            else_branch,
        } => {
            mentions(condition, name) || mentions(then_branch, name) || mentions(else_branch, name)
        }
        Term::Match {
            scrutinee, arms, ..
        } => {
            mentions(scrutinee, name)
                || arms
                    .iter()
                    .any(|arm| !arm.binders.contains(&name) && mentions(&arm.body, name))
        }
        Term::Build { operands, .. }
        | Term::Call { operands, .. }
        | Term::Prim { operands, .. } => any(operands),
        Term::Closure { captures, .. } => any(captures),
        Term::Apply { target, operands } => mentions(target, name) || any(operands),
        Term::First { value } | Term::Second { value } | Term::Field { value, .. } => {
            mentions(value, name)
        }
    }
}

/// The pattern binding local `name` in `body`: `_` when the body never
/// reads it.
fn binder(name: u64, body: &Term) -> Pat {
    if mentions(body, name) {
        Pat::Bind(Ident::Local(name))
    } else {
        Pat::Wild
    }
}

/// An expression as a block, without a redundant pair of braces.
fn block_of(expr: Expr) -> Block {
    match expr {
        Expr::Block(block) => *block,
        other => Block::of(other),
    }
}

/// The value a fallible function returns from `expr`: `Ok` is pushed into
/// every tail, and a fallible call or application in tail position returns
/// its result instead of propagating and rewrapping it.
fn succeed(expr: Expr) -> Expr {
    match expr {
        Expr::Block(mut block) => {
            block.tail = succeed(block.tail);
            Expr::Block(block)
        }
        Expr::If {
            condition,
            mut then_branch,
            mut else_branch,
        } => {
            then_branch.tail = succeed(then_branch.tail);
            else_branch.tail = succeed(else_branch.tail);
            Expr::If {
                condition,
                then_branch,
                else_branch,
            }
        }
        Expr::Match { scrutinee, arms } => Expr::Match {
            scrutinee,
            arms: arms
                .into_iter()
                .map(|(pattern, mut body)| {
                    body.tail = succeed(body.tail);
                    (pattern, body)
                })
                .collect(),
        },
        Expr::Call {
            callee,
            args,
            propagate: true,
        } => Expr::Call {
            callee,
            args,
            propagate: false,
        },
        Expr::Apply {
            holder,
            args,
            propagate: true,
        } => Expr::Apply {
            holder,
            args,
            propagate: false,
        },
        other => Expr::Succeed(Box::new(other)),
    }
}

/// The runtime function realizing `operation` at its operand types.
fn item(operation: &Prim, operands: &[Ty]) -> Result<Item, String> {
    let first = operands.first().cloned().unwrap_or(Ty::Unit);
    let fixed = || match &first {
        Ty::Fixed { width } => Ok(*width),
        other => fail(format!("{other:?} is not fixed-width")),
    };
    let sequence = |list: Item, bytes: Item, string: Option<Item>| match (&first, string) {
        (Ty::List { .. }, _) => Ok(list),
        (Ty::Bytes, _) => Ok(bytes),
        (Ty::String, Some(string)) => Ok(string),
        (other, _) => fail(format!("{operation:?} does not apply to {other:?}")),
    };
    Ok(match operation {
        Prim::NatAdd => Item::NatAdd,
        Prim::NatSub => Item::NatSub,
        Prim::NatMul => Item::NatMul,
        Prim::NatQuot => Item::NatQuot,
        Prim::NatRem => Item::NatRem,
        Prim::NatEq => Item::NatEq,
        Prim::NatLe => Item::NatLe,
        Prim::NatLt => Item::NatLt,
        Prim::IntAdd => Item::IntAdd,
        Prim::IntSub => Item::IntSub,
        Prim::IntMul => Item::IntMul,
        Prim::IntNeg => Item::IntNeg,
        Prim::IntQuot => Item::IntQuot,
        Prim::IntRem => Item::IntRem,
        Prim::CheckedAdd => Item::CheckedAdd(fixed()?),
        Prim::CheckedSub => Item::CheckedSub(fixed()?),
        Prim::CheckedMul => Item::CheckedMul(fixed()?),
        Prim::CheckedQuot => Item::CheckedQuot(fixed()?),
        Prim::CheckedNeg => Item::CheckedNeg(fixed()?),
        Prim::BitAnd => Item::BitAnd(fixed()?),
        Prim::BitOr => Item::BitOr(fixed()?),
        Prim::BitXor => Item::BitXor(fixed()?),
        Prim::BitNot => Item::BitNot(fixed()?),
        Prim::ShiftLeft => Item::ShiftLeft(fixed()?),
        Prim::ShiftRight => Item::ShiftRight(fixed()?),
        Prim::Equal => Item::Equal,
        Prim::BoolNot => Item::BoolNot,
        Prim::BoolAnd => Item::BoolAnd,
        Prim::BoolOr => Item::BoolOr,
        Prim::Append => sequence(Item::AppendList, Item::AppendBytes, None)?,
        Prim::Length => sequence(
            Item::LengthList,
            Item::LengthBytes,
            Some(Item::LengthString),
        )?,
        Prim::Index => sequence(Item::IndexList, Item::IndexBytes, None)?,
        Prim::Slice => sequence(Item::SliceList, Item::SliceBytes, None)?,
        Prim::Utf8Encode => Item::Utf8Encode,
        Prim::Utf8Decode => Item::Utf8Decode,
        Prim::CompareBytes => Item::CompareBytes,
        Prim::Compare => Item::Compare,
        Prim::SplitExact => Item::SplitExact,
        Prim::Join => Item::Join,
        Prim::FormatDecimal => match first {
            Ty::Int => Item::FormatInt,
            _ => Item::FormatFixed(fixed()?),
        },
        Prim::Convert { target } => Item::Convert(*target),
        Prim::ParseDecimal { target } => match target {
            Ty::Int => Item::ParseInt,
            Ty::Fixed { width } => Item::ParseFixed(*width),
            other => return fail(format!("parse_decimal does not target {other:?}")),
        },
    })
}

/// The lowering context: the program, its profile, its function types, and
/// fresh names.
pub(super) struct Lowering<'a> {
    pub(super) program: &'a Program,
    pub(super) profile: Profile,
    checker: Checker<'a>,
    /// Every function type of the program, numbered in first-use order.
    pub(super) fn_types: Vec<Ty>,
    /// For each function type, the closures that inhabit it: function index
    /// and captured types.
    pub(super) closures: BTreeMap<usize, Vec<(u64, Vec<Ty>)>>,
    /// For each owner, the owners its in-place storage reaches.
    reaches: BTreeMap<Owner, BTreeSet<Owner>>,
    /// Whether each function can overflow.
    pub(super) fallible: Vec<bool>,
    /// Whether each function type's application can overflow.
    pub(super) apply_fallible: BTreeMap<usize, bool>,
    fresh: u64,
}

impl<'a> Lowering<'a> {
    /// Analyse a valid program: its function types, closures, boxes, and
    /// failures.
    pub(super) fn new(program: &'a Program, profile: Profile) -> Result<Self, String> {
        super::super::check::check(program)?;
        let mut lowering = Lowering {
            program,
            profile,
            checker: Checker { program },
            fn_types: Vec::new(),
            closures: BTreeMap::new(),
            reaches: BTreeMap::new(),
            fallible: vec![false; program.functions.len()],
            apply_fallible: BTreeMap::new(),
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
                lowering.register(ty);
            }
            lowering.collect(&function.body, &mut scope)?;
        }
        for adt in &program.adts {
            for ty in adt.constructors.iter().flatten() {
                lowering.register(ty);
            }
        }
        lowering.reaches = lowering.reachability();
        lowering.failures()?;
        Ok(lowering)
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

    /// Whether any type of the program holds itself, so some field is boxed.
    pub(super) fn indirect(&self) -> bool {
        self.reaches
            .iter()
            .any(|(owner, reached)| reached.contains(owner))
    }

    /// Whether a field of type `ty` stored in `owner` must be boxed: its
    /// in-place storage reaches `owner` again, so the type has no fixed
    /// size. `rust-core` cannot box, so it refuses the program.
    pub(super) fn boxed(&self, owner: Owner, ty: &Ty) -> Result<bool, String> {
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

    /// Which functions and function types can overflow: a function can when
    /// its body reaches a failing primitive, a successor, a call to a
    /// function that can, or an application of a function type one of whose
    /// closures can; the least fixed point of these rules.
    fn failures(&mut self) -> Result<(), String> {
        loop {
            let mut changed = false;
            for (position, function) in self.program.functions.iter().enumerate() {
                if self.fallible[position] {
                    continue;
                }
                let mut scope: Vec<(u64, Ty)> = function
                    .parameters
                    .iter()
                    .copied()
                    .zip(function.types.iter().cloned())
                    .collect();
                if self.fails(&function.body, &mut scope)? {
                    self.fallible[position] = true;
                    changed = true;
                }
            }
            for position in 0..self.fn_types.len() {
                let fails = self.closures.get(&position).is_some_and(|variants| {
                    variants.iter().any(|(function, _)| {
                        usize::try_from(*function)
                            .ok()
                            .and_then(|function| self.fallible.get(function))
                            .copied()
                            .unwrap_or(false)
                    })
                });
                if self.apply_fallible.insert(position, fails) != Some(fails) {
                    changed = true;
                }
            }
            if !changed {
                return Ok(());
            }
        }
    }

    fn fails(&mut self, expr: &Term, scope: &mut Vec<(u64, Ty)>) -> Result<bool, String> {
        let mut children: Vec<&Term> = Vec::new();
        let here = match expr {
            Term::Value { .. } | Term::Var { .. } => false,
            Term::Let {
                name,
                ty,
                bound,
                body,
            } => {
                let bound = self.fails(bound, scope)?;
                scope.push((*name, ty.clone()));
                let body = self.fails(body, scope);
                scope.pop();
                return Ok(bound || body?);
            }
            Term::Match {
                scrutinee, arms, ..
            } => {
                let mut fails = self.fails(scrutinee, scope)?;
                let scrutinee_ty = self.checker.expr(scrutinee, scope)?;
                for arm in arms {
                    let fields = self.checker.shape_fields(arm.shape, &scrutinee_ty)?;
                    let depth = scope.len();
                    scope.extend(arm.binders.iter().copied().zip(fields));
                    let arm_fails = self.fails(&arm.body, scope);
                    scope.truncate(depth);
                    fails |= arm_fails?;
                }
                return Ok(fails);
            }
            Term::Cond {
                condition,
                then_branch,
                else_branch,
            } => {
                children.extend([
                    condition.as_ref(),
                    then_branch.as_ref(),
                    else_branch.as_ref(),
                ]);
                false
            }
            Term::Build {
                shape, operands, ..
            } => {
                children.extend(operands);
                *shape == Shape::Succ
            }
            Term::Call { function, operands } => {
                children.extend(operands);
                self.fallible
                    .get(index(*function)?)
                    .copied()
                    .unwrap_or(false)
            }
            Term::Closure { captures, .. } => {
                children.extend(captures);
                false
            }
            Term::Apply { target, operands } => {
                children.push(target);
                children.extend(operands);
                let ty = self.checker.expr(target, scope)?;
                let position = self.fn_index(&ty);
                self.apply_fallible.get(&position).copied().unwrap_or(false)
            }
            Term::Prim {
                operation,
                operands,
            } => {
                children.extend(operands);
                let types = operands
                    .iter()
                    .map(|operand| self.checker.expr(operand, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                item(operation, &types)?.fallible()
            }
            Term::First { value } | Term::Second { value } | Term::Field { value, .. } => {
                children.push(value);
                false
            }
        };
        let mut fails = here;
        for child in children {
            fails |= self.fails(child, scope)?;
        }
        Ok(fails)
    }

    fn fresh(&mut self) -> u64 {
        self.fresh += 1;
        self.fresh
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

    pub(super) fn ty(&mut self, ty: &Ty) -> Result<Type, String> {
        Ok(match ty {
            Ty::Unit => Type::Unit,
            Ty::Bool => Type::Bool,
            Ty::Nat => Type::Nat,
            Ty::Int => Type::Int,
            Ty::Fixed { width } => Type::Fixed(*width),
            Ty::String => {
                self.heap("a string")?;
                Type::Str
            }
            Ty::Bytes => {
                self.heap("a byte string")?;
                Type::Bytes
            }
            Ty::Ordering => Type::Ordering,
            Ty::Option { value } => Type::Option(Box::new(self.ty(value)?)),
            Ty::Result { ok, error } => {
                Type::Result(Box::new(self.ty(ok)?), Box::new(self.ty(error)?))
            }
            Ty::List { element } => {
                self.heap("a list")?;
                Type::List(Box::new(self.ty(element)?))
            }
            Ty::Pair { left, right } => {
                Type::Pair(Box::new(self.ty(left)?), Box::new(self.ty(right)?))
            }
            Ty::Adt { index } => Type::Adt(*index),
            Ty::Fn { .. } => Type::Fn(self.fn_index(ty) as u64),
        })
    }

    /// A field of `owner`: its type, boxed where it must be.
    fn field_ty(&mut self, owner: Owner, ty: &Ty) -> Result<Type, String> {
        let lowered = self.ty(ty)?;
        Ok(if self.boxed(owner, ty)? {
            Type::Rc(Box::new(lowered))
        } else {
            lowered
        })
    }

    /// A value stored as a field of `owner`.
    fn store(&self, owner: Owner, ty: &Ty, value: Expr) -> Result<Expr, String> {
        Ok(if self.boxed(owner, ty)? {
            Expr::Box(Box::new(value))
        } else {
            value
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
    fn collect(&mut self, expr: &Term, scope: &mut Vec<(u64, Ty)>) -> Result<(), String> {
        let mut children: Vec<&Term> = Vec::new();
        match expr {
            Term::Value { ty, .. } => self.register(ty),
            Term::Var { .. } => {}
            Term::Let {
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
            Term::Cond {
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
            Term::Match {
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
            Term::Build { ty, operands, .. } => {
                self.register(ty);
                children.extend(operands);
            }
            Term::Call { operands, .. } | Term::Prim { operands, .. } => children.extend(operands),
            Term::Closure { function, captures } => {
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
            Term::Apply { target, operands } => {
                children.push(target);
                children.extend(operands);
            }
            Term::First { value } | Term::Second { value } | Term::Field { value, .. } => {
                children.push(value);
            }
        }
        for child in children {
            self.collect(child, scope)?;
        }
        Ok(())
    }

    pub(super) fn value(&mut self, value: &Value, ty: &Ty) -> Result<Expr, String> {
        let number = |text: &str| {
            text.parse::<i128>()
                .map_err(|error| format!("literal `{text}`: {error}"))
        };
        Ok(match (value, ty) {
            (Value::Unit, _) => Expr::Lit(Lit::Unit),
            (Value::Bool { value }, _) => Expr::Lit(Lit::Bool(*value)),
            (Value::Nat { value }, _) => Expr::Lit(Lit::Nat(
                value
                    .parse()
                    .map_err(|error| format!("literal `{value}`: {error}"))?,
            )),
            (Value::Int { value }, _) => Expr::Lit(Lit::Int(
                value
                    .parse()
                    .map_err(|error| format!("literal `{value}`: {error}"))?,
            )),
            (
                Value::U8 { value }
                | Value::U16 { value }
                | Value::U32 { value }
                | Value::U64 { value }
                | Value::I8 { value }
                | Value::I16 { value }
                | Value::I32 { value }
                | Value::I64 { value },
                Ty::Fixed { width },
            ) => Expr::Lit(Lit::Fixed(*width, number(value)?)),
            (Value::String { value }, _) => {
                self.heap("a string")?;
                Expr::Lit(Lit::Str(value.clone()))
            }
            (Value::Bytes { hex }, _) => {
                self.heap("a byte string")?;
                let octets = (0..hex.len())
                    .step_by(2)
                    .map(|at| {
                        hex.get(at..at + 2)
                            .and_then(|digits| u8::from_str_radix(digits, 16).ok())
                            .ok_or_else(|| format!("byte literal `{hex}` is malformed"))
                    })
                    .collect::<Result<_, _>>()?;
                Expr::Lit(Lit::Bytes(octets))
            }
            (Value::Ordering { value }, _) => Expr::Lit(Lit::Ordering(*value)),
            (Value::None, Ty::Option { value: inner }) => Expr::Construct {
                ctor: Ctor::None(self.ty(inner)?),
                args: Vec::new(),
            },
            (Value::Some { value }, Ty::Option { value: inner }) => Expr::Construct {
                ctor: Ctor::Some,
                args: vec![self.value(value, inner)?],
            },
            (Value::Ok { value }, Ty::Result { ok, error }) => Expr::Construct {
                ctor: Ctor::Ok(self.ty(ok)?, self.ty(error)?),
                args: vec![self.value(value, ok)?],
            },
            (Value::Error { value }, Ty::Result { ok, error }) => Expr::Construct {
                ctor: Ctor::Err(self.ty(ok)?, self.ty(error)?),
                args: vec![self.value(value, error)?],
            },
            (Value::List { items }, Ty::List { element }) => {
                let mut out = Expr::Construct {
                    ctor: Ctor::Nil(self.ty(element)?),
                    args: Vec::new(),
                };
                for item in items.iter().rev() {
                    out = Expr::Construct {
                        ctor: Ctor::Cons,
                        args: vec![self.value(item, element)?, out],
                    };
                }
                out
            }
            (
                Value::Pair { left, right },
                Ty::Pair {
                    left: left_ty,
                    right: right_ty,
                },
            ) => Expr::Pair(
                Box::new(self.value(left, left_ty)?),
                Box::new(self.value(right, right_ty)?),
            ),
            (
                Value::Adt {
                    constructor,
                    fields,
                },
                Ty::Adt { index: adt },
            ) => {
                let types = self.adt_fields(*adt, *constructor)?;
                let mut args = Vec::new();
                for (field, ty) in fields.iter().zip(&types) {
                    let lowered = self.value(field, ty)?;
                    args.push(self.store(Owner::Adt(*adt), ty, lowered)?);
                }
                Expr::Construct {
                    ctor: Ctor::Adt {
                        adt: *adt,
                        constructor: *constructor,
                    },
                    args,
                }
            }
            (value, ty) => {
                return fail(format!(
                    "the literal {value:?} cannot be rendered at type {ty:?}"
                ))
            }
        })
    }

    fn exprs(&mut self, exprs: &[Term], scope: &mut Vec<(u64, Ty)>) -> Result<Vec<Expr>, String> {
        exprs.iter().map(|expr| self.expr(expr, scope)).collect()
    }

    /// The arms of a match on `scrutinee_ty` over the bound `holder`.
    #[allow(clippy::too_many_lines)]
    fn arms(
        &mut self,
        holder: &Ident,
        scrutinee_ty: &Ty,
        arms: &[Arm],
        scope: &mut Vec<(u64, Ty)>,
    ) -> Result<Expr, String> {
        let mut bodies: BTreeMap<Shape, (Vec<Pat>, Block)> = BTreeMap::new();
        for arm in arms {
            let fields = self.checker.shape_fields(arm.shape, scrutinee_ty)?;
            let depth = scope.len();
            scope.extend(arm.binders.iter().copied().zip(fields));
            let body = self.expr(&arm.body, scope);
            scope.truncate(depth);
            let patterns = arm
                .binders
                .iter()
                .map(|name| binder(*name, &arm.body))
                .collect();
            bodies.insert(arm.shape, (patterns, block_of(body?)));
        }
        let mut take = |shape: Shape| {
            bodies
                .remove(&shape)
                .ok_or_else(|| format!("the match lacks {shape:?}"))
        };
        let first = |patterns: &[Pat], position: usize| {
            patterns
                .get(position)
                .cloned()
                .ok_or_else(|| format!("an arm lacks binder {position}"))
        };
        let moved = || Box::new(Expr::Move(holder.clone()));
        let rendered = match scrutinee_ty {
            Ty::Option { .. } => {
                let (_, none) = take(Shape::None)?;
                let (patterns, some) = take(Shape::Some)?;
                Expr::Match {
                    scrutinee: moved(),
                    arms: vec![
                        (Pat::None, none),
                        (Pat::Some(Box::new(first(&patterns, 0)?)), some),
                    ],
                }
            }
            Ty::Result { .. } => {
                let (ok_patterns, ok) = take(Shape::Ok)?;
                let (error_patterns, error) = take(Shape::Error)?;
                Expr::Match {
                    scrutinee: moved(),
                    arms: vec![
                        (Pat::Ok(Box::new(first(&ok_patterns, 0)?)), ok),
                        (Pat::Err(Box::new(first(&error_patterns, 0)?)), error),
                    ],
                }
            }
            Ty::List { .. } => {
                let (_, nil) = take(Shape::Nil)?;
                let (patterns, cons) = take(Shape::Cons)?;
                Expr::Match {
                    scrutinee: Box::new(Expr::Uncons(holder.clone())),
                    arms: vec![
                        (Pat::None, nil),
                        (
                            Pat::Some(Box::new(Pat::Tuple(vec![
                                first(&patterns, 0)?,
                                first(&patterns, 1)?,
                            ]))),
                            cons,
                        ),
                    ],
                }
            }
            Ty::Nat => {
                let (_, zero) = take(Shape::Zero)?;
                let (patterns, mut succ) = take(Shape::Succ)?;
                let predecessor = first(&patterns, 0)?;
                if predecessor != Pat::Wild {
                    succ.lets.insert(
                        0,
                        Let {
                            pat: predecessor,
                            ty: None,
                            value: Expr::Predecessor(holder.clone()),
                        },
                    );
                }
                Expr::If {
                    condition: Box::new(Expr::IsZero(holder.clone())),
                    then_branch: Box::new(zero),
                    else_branch: Box::new(succ),
                }
            }
            Ty::Bool => {
                let (_, yes) = take(Shape::True)?;
                let (_, no) = take(Shape::False)?;
                Expr::If {
                    condition: moved(),
                    then_branch: Box::new(yes),
                    else_branch: Box::new(no),
                }
            }
            Ty::Pair { .. } => {
                let (patterns, mut body) = take(Shape::Pair)?;
                body.lets.insert(
                    0,
                    Let {
                        pat: Pat::Tuple(vec![first(&patterns, 0)?, first(&patterns, 1)?]),
                        ty: None,
                        value: Expr::Move(holder.clone()),
                    },
                );
                Expr::Block(Box::new(body))
            }
            Ty::Unit => {
                let (_, mut body) = take(Shape::Unit)?;
                body.lets.insert(
                    0,
                    Let {
                        pat: Pat::Unit,
                        ty: None,
                        value: Expr::Move(holder.clone()),
                    },
                );
                Expr::Block(Box::new(body))
            }
            Ty::Ordering => {
                let (_, less) = take(Shape::Lt)?;
                let (_, equal) = take(Shape::Eq)?;
                let (_, greater) = take(Shape::Gt)?;
                Expr::Match {
                    scrutinee: moved(),
                    arms: vec![
                        (Pat::Ordering(super::super::OrderingValue::Lt), less),
                        (Pat::Ordering(super::super::OrderingValue::Eq), equal),
                        (Pat::Ordering(super::super::OrderingValue::Gt), greater),
                    ],
                }
            }
            Ty::Adt { index: adt } => {
                let count = self
                    .program
                    .adts
                    .get(index(*adt)?)
                    .map_or(0, |declared| declared.constructors.len())
                    as u64;
                let mut out = Vec::new();
                for constructor in 0..count {
                    let (patterns, mut body) = take(Shape::Adt { constructor })?;
                    let types = self.adt_fields(*adt, constructor)?;
                    let mut fields = Vec::new();
                    let mut loads = Vec::new();
                    for (pattern, ty) in patterns.into_iter().zip(&types) {
                        if pattern != Pat::Wild && self.boxed(Owner::Adt(*adt), ty)? {
                            let held = Ident::Boxed(self.fresh());
                            loads.push(Let {
                                pat: pattern,
                                ty: None,
                                value: Expr::Unbox(held.clone()),
                            });
                            fields.push(Pat::Bind(held));
                        } else {
                            fields.push(pattern);
                        }
                    }
                    loads.append(&mut body.lets);
                    body.lets = loads;
                    out.push((
                        Pat::Adt {
                            adt: *adt,
                            constructor,
                            fields,
                        },
                        body,
                    ));
                }
                Expr::Match {
                    scrutinee: moved(),
                    arms: out,
                }
            }
            other => return fail(format!("cannot render a match on {other:?}")),
        };
        if !bodies.is_empty() {
            return fail("a match arm has no rendering");
        }
        Ok(rendered)
    }

    #[allow(clippy::too_many_lines)]
    fn expr(&mut self, expr: &Term, scope: &mut Vec<(u64, Ty)>) -> Result<Expr, String> {
        Ok(match expr {
            Term::Value { ty, value } => self.value(value, ty)?,
            Term::Var { name } => {
                let ty = scope
                    .iter()
                    .rev()
                    .find(|(bound, _)| bound == name)
                    .map(|(_, ty)| ty.clone())
                    .ok_or_else(|| format!("local {name} is unbound"))?;
                if self.ty(&ty)?.copy() {
                    Expr::Copy(Ident::Local(*name))
                } else {
                    Expr::Clone(Ident::Local(*name))
                }
            }
            Term::Let {
                name,
                ty,
                bound,
                body,
            } => {
                let bound = self.expr(bound, scope)?;
                let lowered = self.ty(ty)?;
                scope.push((*name, ty.clone()));
                let rendered = self.expr(body, scope);
                scope.pop();
                let mut block = block_of(rendered?);
                block.lets.insert(
                    0,
                    Let {
                        pat: binder(*name, body),
                        ty: Some(lowered),
                        value: bound,
                    },
                );
                Expr::Block(Box::new(block))
            }
            Term::Cond {
                condition,
                then_branch,
                else_branch,
            } => {
                // The condition's bindings come before the `if`, so the
                // condition itself is one expression.
                let mut block = block_of(self.expr(condition, scope)?);
                let condition = block.tail;
                let then_branch = block_of(self.expr(then_branch, scope)?);
                let else_branch = block_of(self.expr(else_branch, scope)?);
                let literal = |branch: &Block| match (&branch.lets[..], &branch.tail) {
                    ([], Expr::Lit(Lit::Bool(value))) => Some(*value),
                    _ => None,
                };
                block.tail = match (literal(&then_branch), literal(&else_branch)) {
                    // A conditional choosing between the Boolean literals
                    // is its condition, or its negation.
                    (Some(true), Some(false)) => condition,
                    (Some(false), Some(true)) => Expr::Not(Box::new(condition)),
                    _ => Expr::If {
                        condition: Box::new(condition),
                        then_branch: Box::new(then_branch),
                        else_branch: Box::new(else_branch),
                    },
                };
                if block.lets.is_empty() {
                    block.tail
                } else {
                    Expr::Block(Box::new(block))
                }
            }
            Term::Match {
                scrutinee, arms, ..
            } => {
                let scrutinee_ty = self.checker.expr(scrutinee, scope)?;
                let rendered = self.expr(scrutinee, scope)?;
                let holder = Ident::Holder(self.fresh());
                let body = self.arms(&holder, &scrutinee_ty, arms, scope)?;
                let mut block = block_of(body);
                block.lets.insert(
                    0,
                    Let {
                        pat: Pat::Bind(holder),
                        ty: None,
                        value: rendered,
                    },
                );
                Expr::Block(Box::new(block))
            }
            Term::Build {
                shape,
                ty,
                operands,
            } => {
                let mut rendered = self.exprs(operands, scope)?;
                match (shape, ty) {
                    (Shape::None, Ty::Option { value }) => Expr::Construct {
                        ctor: Ctor::None(self.ty(value)?),
                        args: Vec::new(),
                    },
                    (Shape::Some, _) => Expr::Construct {
                        ctor: Ctor::Some,
                        args: rendered,
                    },
                    (Shape::Ok, Ty::Result { ok, error }) => Expr::Construct {
                        ctor: Ctor::Ok(self.ty(ok)?, self.ty(error)?),
                        args: rendered,
                    },
                    (Shape::Error, Ty::Result { ok, error }) => Expr::Construct {
                        ctor: Ctor::Err(self.ty(ok)?, self.ty(error)?),
                        args: rendered,
                    },
                    (Shape::Nil, Ty::List { element }) => Expr::Construct {
                        ctor: Ctor::Nil(self.ty(element)?),
                        args: Vec::new(),
                    },
                    (Shape::Cons, _) => {
                        self.heap("a list cell")?;
                        Expr::Construct {
                            ctor: Ctor::Cons,
                            args: rendered,
                        }
                    }
                    (Shape::Zero, _) => Expr::Lit(Lit::Nat(0)),
                    (Shape::Succ, _) => Expr::Call {
                        callee: Callee::Runtime(Item::NatSucc),
                        args: rendered,
                        propagate: true,
                    },
                    (Shape::Pair, _) if rendered.len() == 2 => {
                        let right = rendered.pop().unwrap_or(Expr::Lit(Lit::Unit));
                        let left = rendered.pop().unwrap_or(Expr::Lit(Lit::Unit));
                        Expr::Pair(Box::new(left), Box::new(right))
                    }
                    (Shape::True, _) => Expr::Lit(Lit::Bool(true)),
                    (Shape::False, _) => Expr::Lit(Lit::Bool(false)),
                    (Shape::Unit, _) => Expr::Lit(Lit::Unit),
                    (Shape::Lt, _) => Expr::Lit(Lit::Ordering(super::super::OrderingValue::Lt)),
                    (Shape::Eq, _) => Expr::Lit(Lit::Ordering(super::super::OrderingValue::Eq)),
                    (Shape::Gt, _) => Expr::Lit(Lit::Ordering(super::super::OrderingValue::Gt)),
                    (Shape::Adt { constructor }, Ty::Adt { index: adt }) => {
                        let types = self.adt_fields(*adt, *constructor)?;
                        let mut args = Vec::new();
                        for (value, ty) in rendered.into_iter().zip(&types) {
                            args.push(self.store(Owner::Adt(*adt), ty, value)?);
                        }
                        Expr::Construct {
                            ctor: Ctor::Adt {
                                adt: *adt,
                                constructor: *constructor,
                            },
                            args,
                        }
                    }
                    (shape, ty) => return fail(format!("cannot render {shape:?} at {ty:?}")),
                }
            }
            Term::Call { function, operands } => Expr::Call {
                callee: Callee::Function(*function),
                args: self.exprs(operands, scope)?,
                propagate: self
                    .fallible
                    .get(index(*function)?)
                    .copied()
                    .unwrap_or(false),
            },
            Term::Closure { function, captures } => {
                let ty = self.checker.expr(expr, scope)?;
                let position = self.fn_index(&ty);
                let rendered = self.exprs(captures, scope)?;
                let types = self
                    .program
                    .functions
                    .get(index(*function)?)
                    .map(|callee| callee.types.clone())
                    .unwrap_or_default();
                let mut args = Vec::new();
                for (value, ty) in rendered.into_iter().zip(&types) {
                    args.push(self.store(Owner::Fn(position), ty, value)?);
                }
                Expr::Construct {
                    ctor: Ctor::Closure {
                        fn_type: position as u64,
                        function: *function,
                    },
                    args,
                }
            }
            Term::Apply { target, operands } => {
                let ty = self.checker.expr(target, scope)?;
                let position = self.fn_index(&ty);
                let callee = Ident::Callee(self.fresh());
                let target = self.expr(target, scope)?;
                let args = self.exprs(operands, scope)?;
                Expr::Block(Box::new(Block {
                    lets: vec![Let {
                        pat: Pat::Bind(callee.clone()),
                        ty: Some(Type::Fn(position as u64)),
                        value: target,
                    }],
                    tail: Expr::Apply {
                        holder: callee,
                        args,
                        propagate: self.apply_fallible.get(&position).copied().unwrap_or(false),
                    },
                }))
            }
            Term::Prim {
                operation,
                operands,
            } => {
                let types = operands
                    .iter()
                    .map(|operand| self.checker.expr(operand, scope))
                    .collect::<Result<Vec<_>, _>>()?;
                let runtime = item(operation, &types)?;
                if runtime.heap() {
                    self.heap(&format!("the primitive {operation:?}"))?;
                }
                let rendered = self.exprs(operands, scope)?;
                // Operands are bound left to right before the call, so the
                // evaluation order is the denotation's.
                let mut lets = Vec::new();
                let mut args = Vec::new();
                for value in rendered {
                    let name = Ident::Operand(self.fresh());
                    lets.push(Let {
                        pat: Pat::Bind(name.clone()),
                        ty: None,
                        value,
                    });
                    args.push(if matches!(operation, Prim::Convert { .. }) {
                        Expr::Widen(Box::new(Expr::Move(name)))
                    } else {
                        Expr::Move(name)
                    });
                }
                Expr::Block(Box::new(Block {
                    lets,
                    tail: Expr::Call {
                        callee: Callee::Runtime(runtime),
                        args,
                        propagate: runtime.fallible(),
                    },
                }))
            }
            Term::First { value } | Term::Second { value } => {
                let held = Ident::Part(self.fresh());
                let parts = if matches!(expr, Term::First { .. }) {
                    vec![Pat::Bind(held.clone()), Pat::Wild]
                } else {
                    vec![Pat::Wild, Pat::Bind(held.clone())]
                };
                Expr::Block(Box::new(Block {
                    lets: vec![Let {
                        pat: Pat::Tuple(parts),
                        ty: None,
                        value: self.expr(value, scope)?,
                    }],
                    tail: Expr::Move(held),
                }))
            }
            Term::Field {
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
                let held = Ident::Part(self.fresh());
                let fields: Vec<Pat> = (0..types.len())
                    .map(|at| {
                        if at == selected {
                            Pat::Bind(held.clone())
                        } else {
                            Pat::Wild
                        }
                    })
                    .collect();
                let read = if self.boxed(Owner::Adt(adt), &ty)? {
                    Expr::Unbox(held)
                } else {
                    Expr::Move(held)
                };
                Expr::Match {
                    scrutinee: Box::new(self.expr(value, scope)?),
                    arms: vec![(
                        Pat::Adt {
                            adt,
                            constructor: 0,
                            fields,
                        },
                        Block::of(read),
                    )],
                }
            }
        })
    }

    /// The crate: the ADTs, the function types and their dispatch, and the
    /// functions.
    pub(super) fn lower(&mut self) -> Result<Crate, String> {
        let mut items = Vec::new();
        for (position, adt) in self.program.adts.iter().enumerate() {
            let owner = Owner::Adt(position as u64);
            let mut variants = Vec::new();
            for (constructor, fields) in adt.constructors.iter().enumerate() {
                let lowered = fields
                    .iter()
                    .map(|ty| self.field_ty(owner, ty))
                    .collect::<Result<Vec<_>, _>>()?;
                variants.push((constructor as u64, lowered));
            }
            items.push(ItemDef::Enum {
                name: Type::Adt(position as u64),
                variants,
            });
        }
        let fn_types = self.fn_types.clone();
        for (position, fn_ty) in fn_types.iter().enumerate() {
            let owner = Owner::Fn(position);
            let Ty::Fn { parameters, result } = fn_ty else {
                return fail("a function type is not a function");
            };
            let closures = self.closures.get(&position).cloned().unwrap_or_default();
            let mut variants = Vec::new();
            let mut arms = Vec::new();
            for (function, captured) in &closures {
                let lowered = captured
                    .iter()
                    .map(|ty| self.field_ty(owner, ty))
                    .collect::<Result<Vec<_>, _>>()?;
                variants.push((*function, lowered));
                let captures = captured
                    .iter()
                    .map(|ty| {
                        Ok(if self.boxed(owner, ty)? {
                            CaptureRead::Unbox
                        } else if self.ty(ty)?.copy() {
                            CaptureRead::Copy
                        } else {
                            CaptureRead::Clone
                        })
                    })
                    .collect::<Result<Vec<_>, String>>()?;
                arms.push(Dispatch {
                    function: *function,
                    captures,
                    function_fallible: self
                        .fallible
                        .get(index(*function)?)
                        .copied()
                        .unwrap_or(false),
                });
            }
            items.push(ItemDef::Enum {
                name: Type::Fn(position as u64),
                variants,
            });
            let parameters = parameters
                .iter()
                .map(|ty| self.ty(ty))
                .collect::<Result<Vec<_>, _>>()?;
            items.push(ItemDef::Apply {
                fn_type: position as u64,
                parameters,
                result: self.ty(result)?,
                fallible: self.apply_fallible.get(&position).copied().unwrap_or(false),
                arms,
            });
        }
        for (position, function) in self.program.functions.iter().enumerate() {
            let mut scope: Vec<(u64, Ty)> = function
                .parameters
                .iter()
                .copied()
                .zip(function.types.iter().cloned())
                .collect();
            let parameters = function
                .parameters
                .iter()
                .zip(&function.types)
                .map(|(name, ty)| Ok((binder(*name, &function.body), self.ty(ty)?)))
                .collect::<Result<Vec<_>, String>>()?;
            let body = self.expr(&function.body, &mut scope)?;
            let result = self.ty(&function.result)?;
            let fallible = self.fallible.get(position).copied().unwrap_or(false);
            let (result, body) = if fallible {
                (Type::Fallible(Box::new(result)), block_of(succeed(body)))
            } else {
                (result, block_of(body))
            };
            items.push(ItemDef::Function {
                name: Ident::Function(position as u64),
                parameters,
                result,
                body,
            });
        }
        Ok(Crate {
            profile: self.profile,
            items,
        })
    }
}
