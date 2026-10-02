//! Checks a rendered crate before it is printed (SPEC.md §17.16): its
//! profile admits every type and runtime function it uses, every name it
//! uses is declared and bound exactly once, every value is moved at most
//! once on any path, every fallible call propagates its failure and only a
//! fallible function propagates, and every construct it emits corresponds
//! to an element of the target program it realizes.

use std::collections::{BTreeMap, BTreeSet};

use super::ast::{Block, Callee, Crate, Ctor, Expr, Ident, ItemDef, Lit, Pat, Type};
use super::Profile;

fn fail<T>(reason: impl Into<String>) -> Result<T, String> {
    Err(reason.into())
}

/// What the crate declares: its enums with their variants' arities, its
/// function types' fallibility, and its functions' fallibility.
struct Declared {
    adts: BTreeMap<u64, BTreeMap<u64, usize>>,
    closures: BTreeMap<u64, BTreeMap<u64, usize>>,
    applies: BTreeMap<u64, bool>,
    functions: BTreeMap<u64, bool>,
}

fn declared(krate: &Crate) -> Result<Declared, String> {
    let mut out = Declared {
        adts: BTreeMap::new(),
        closures: BTreeMap::new(),
        applies: BTreeMap::new(),
        functions: BTreeMap::new(),
    };
    for item in &krate.items {
        match item {
            ItemDef::Enum { name, variants } => {
                let arities = variants
                    .iter()
                    .map(|(number, fields)| (*number, fields.len()))
                    .collect();
                let fresh = match name {
                    Type::Adt(n) => out.adts.insert(*n, arities).is_none(),
                    Type::Fn(n) => out.closures.insert(*n, arities).is_none(),
                    other => {
                        return fail(format!("an enum named {other:?} is not a declared type"))
                    }
                };
                if !fresh {
                    return fail(format!("hygiene: {name:?} is declared twice"));
                }
            }
            ItemDef::Apply {
                fn_type, fallible, ..
            } => {
                if out.applies.insert(*fn_type, *fallible).is_some() {
                    return fail(format!("hygiene: Fn{fn_type}::apply is declared twice"));
                }
            }
            ItemDef::Function { name, result, .. } => {
                if let Ident::Function(n) = name {
                    if out
                        .functions
                        .insert(*n, matches!(result, Type::Fallible(_)))
                        .is_some()
                    {
                        return fail(format!("hygiene: f{n} is declared twice"));
                    }
                }
            }
        }
    }
    Ok(out)
}

struct Checker<'a> {
    profile: Profile,
    declared: &'a Declared,
    /// Every name bound in the current function, so none is bound twice.
    bound: BTreeSet<Ident>,
    /// The names in scope at this point.
    scope: Vec<Ident>,
    /// The names moved on the current path.
    moved: BTreeSet<Ident>,
    /// Whether a `?` occurs in the current function.
    propagates: bool,
    /// Whether the current function is fallible, so its tail may return a
    /// fallible call's result without propagating it.
    fallible: bool,
}

impl Checker<'_> {
    fn ty(&self, ty: &Type) -> Result<(), String> {
        if self.profile == Profile::Core && ty.heap() {
            return fail(format!(
                "hidden allocation: rust-core renders the heap type {ty:?}, which rust-core does not provide"
            ));
        }
        match ty {
            Type::Adt(n) if !self.declared.adts.contains_key(n) => {
                fail(format!("type Adt{n} is not declared"))
            }
            Type::Fn(n) if !self.declared.closures.contains_key(n) => {
                fail(format!("type Fn{n} is not declared"))
            }
            Type::Option(inner)
            | Type::List(inner)
            | Type::Rc(inner)
            | Type::Fallible(inner)
            | Type::Ref(inner) => self.ty(inner),
            Type::Result(left, right) | Type::Pair(left, right) => {
                self.ty(left)?;
                self.ty(right)
            }
            _ => Ok(()),
        }
    }

    fn heap(&self, what: &str) -> Result<(), String> {
        if self.profile == Profile::Core {
            return fail(format!(
                "hidden allocation: rust-core renders {what}, which requires heap allocation"
            ));
        }
        Ok(())
    }

    fn bind(&mut self, pattern: &Pat) -> Result<(), String> {
        match pattern {
            Pat::Wild | Pat::Unit | Pat::None | Pat::Ordering(_) => Ok(()),
            Pat::Bind(ident) => {
                if !self.bound.insert(ident.clone()) {
                    return fail(format!(
                        "hygiene: `{}` is bound twice in one function",
                        ident_text(ident)
                    ));
                }
                self.scope.push(ident.clone());
                Ok(())
            }
            Pat::Tuple(items) => items.iter().try_for_each(|item| self.bind(item)),
            Pat::Some(inner) | Pat::Ok(inner) | Pat::Err(inner) => self.bind(inner),
            Pat::Adt {
                adt,
                constructor,
                fields,
            } => {
                let arity = self
                    .declared
                    .adts
                    .get(adt)
                    .and_then(|variants| variants.get(constructor))
                    .ok_or_else(|| format!("pattern Adt{adt}::C{constructor} is not declared"))?;
                if *arity != fields.len() {
                    return fail(format!(
                        "pattern Adt{adt}::C{constructor} binds {} fields of {arity}",
                        fields.len()
                    ));
                }
                fields.iter().try_for_each(|field| self.bind(field))
            }
        }
    }

    /// A read of `ident` that leaves it in place.
    fn read(&self, ident: &Ident) -> Result<(), String> {
        if !self.scope.contains(ident) {
            return fail(format!("`{}` is not bound", ident_text(ident)));
        }
        if self.moved.contains(ident) {
            return fail(format!(
                "ownership: `{}` is read after it was moved",
                ident_text(ident)
            ));
        }
        Ok(())
    }

    fn consume(&mut self, ident: &Ident) -> Result<(), String> {
        if !self.scope.contains(ident) {
            return fail(format!("`{}` is not bound", ident_text(ident)));
        }
        if !self.moved.insert(ident.clone()) {
            return fail(format!("ownership: `{}` is moved twice", ident_text(ident)));
        }
        Ok(())
    }

    fn call(
        &mut self,
        fallible: bool,
        propagate: bool,
        tail: bool,
        what: &str,
    ) -> Result<(), String> {
        match (fallible, propagate) {
            // The tail of a fallible function returns the call's result.
            (true, false) if tail && self.fallible => Ok(()),
            (true, false) => fail(format!(
                "arithmetic: the fallible call of {what} does not propagate its overflow"
            )),
            (false, true) => fail(format!(
                "arithmetic: the infallible call of {what} propagates a failure it cannot have"
            )),
            _ => {
                self.propagates |= propagate;
                Ok(())
            }
        }
    }

    fn block(&mut self, block: &Block, tail: bool) -> Result<(), String> {
        let depth = self.scope.len();
        for binding in &block.lets {
            if let Some(ty) = &binding.ty {
                self.ty(ty)?;
            }
            self.expr(&binding.value, false)?;
            self.bind(&binding.pat)?;
        }
        let checked = self.expr(&block.tail, tail);
        self.scope.truncate(depth);
        checked
    }

    /// Branches are alternatives: each starts from the same moved set, and
    /// a name moved on any of them is moved afterwards.
    fn branches<'b>(
        &mut self,
        branches: impl IntoIterator<Item = (Option<&'b Pat>, &'b Block)>,
        tail: bool,
    ) -> Result<(), String> {
        let before = self.moved.clone();
        let mut after = before.clone();
        for (pattern, body) in branches {
            self.moved = before.clone();
            let depth = self.scope.len();
            if let Some(pattern) = pattern {
                self.bind(pattern)?;
            }
            let checked = self.block(body, tail);
            self.scope.truncate(depth);
            checked?;
            after.extend(self.moved.iter().cloned());
        }
        self.moved = after;
        Ok(())
    }

    /// Check `expr`; `tail` says it is the value the current function
    /// returns.
    #[allow(clippy::too_many_lines)]
    fn expr(&mut self, expr: &Expr, tail: bool) -> Result<(), String> {
        // The value a fallible function returns is `R<T>`: an `Ok`, a
        // fallible call or application returning its own result, or a
        // branch or block whose tails are such values.
        let returns_result = matches!(
            expr,
            Expr::Succeed(_)
                | Expr::If { .. }
                | Expr::Match { .. }
                | Expr::Block(_)
                | Expr::Apply {
                    propagate: false,
                    ..
                }
                | Expr::Call {
                    propagate: false,
                    ..
                }
        );
        if tail && self.fallible && !returns_result {
            return fail(format!(
                "arithmetic: the value a fallible function returns, {}, is not a fallible value",
                constructs_of(expr)
            ));
        }
        match expr {
            Expr::Lit(literal) => match literal {
                Lit::Str(_) => self.heap("a string literal"),
                Lit::Bytes(_) => self.heap("a byte string literal"),
                Lit::Unit
                | Lit::Bool(_)
                | Lit::Nat(_)
                | Lit::Int(_)
                | Lit::Fixed(..)
                | Lit::Ordering(_) => Ok(()),
            },
            Expr::Move(ident) => self.consume(ident),
            Expr::Not(inner) => self.expr(inner, false),
            Expr::Clone(ident)
            | Expr::Copy(ident)
            | Expr::Deref(ident)
            | Expr::Uncons(ident)
            | Expr::IsZero(ident)
            | Expr::Predecessor(ident) => {
                if matches!(expr, Expr::Uncons(_)) {
                    self.heap("a list match")?;
                }
                self.read(ident)
            }
            Expr::Unbox(ident) | Expr::UnboxRef(ident) => {
                self.heap("a boxed read")?;
                self.read(ident)
            }
            Expr::Box(inner) => {
                self.heap("a box")?;
                self.expr(inner, false)
            }
            Expr::Call {
                callee,
                args,
                propagate,
            } => {
                for arg in args {
                    self.expr(arg, false)?;
                }
                match callee {
                    Callee::Function(n) => {
                        let fallible = *self
                            .declared
                            .functions
                            .get(n)
                            .ok_or_else(|| format!("f{n} is not declared"))?;
                        self.call(fallible, *propagate, tail, &format!("f{n}"))
                    }
                    Callee::Runtime(item) => {
                        if item.heap() {
                            self.heap(&format!("the runtime function `{}`", item.path()))?;
                        }
                        self.call(
                            item.fallible(),
                            *propagate,
                            tail,
                            &format!("`{}`", item.path()),
                        )
                    }
                }
            }
            Expr::Apply {
                holder,
                args,
                propagate,
            } => {
                self.read(holder)?;
                for arg in args {
                    self.expr(arg, false)?;
                }
                // The applied closure's type is known only through its
                // binding, so its fallibility is checked by `apply_types`.
                self.propagates |= *propagate;
                Ok(())
            }
            Expr::Construct { ctor, args } => {
                for arg in args {
                    self.expr(arg, false)?;
                }
                match ctor {
                    Ctor::None(ty) | Ctor::Nil(ty) => {
                        if matches!(ctor, Ctor::Nil(_)) {
                            self.heap("an empty list")?;
                        }
                        self.ty(ty)
                    }
                    Ctor::Ok(ok, error) | Ctor::Err(ok, error) => {
                        self.ty(ok)?;
                        self.ty(error)
                    }
                    Ctor::Cons => self.heap("a list cell"),
                    Ctor::Some => Ok(()),
                    Ctor::Adt { adt, constructor } => {
                        let arity = self
                            .declared
                            .adts
                            .get(adt)
                            .and_then(|variants| variants.get(constructor))
                            .ok_or_else(|| format!("Adt{adt}::C{constructor} is not declared"))?;
                        if *arity != args.len() {
                            return fail(format!(
                                "Adt{adt}::C{constructor} receives {} fields of {arity}",
                                args.len()
                            ));
                        }
                        Ok(())
                    }
                    Ctor::Closure { fn_type, function } => {
                        let arity = self
                            .declared
                            .closures
                            .get(fn_type)
                            .and_then(|variants| variants.get(function))
                            .ok_or_else(|| format!("Fn{fn_type}::F{function} is not declared"))?;
                        if *arity != args.len() {
                            return fail(format!(
                                "Fn{fn_type}::F{function} captures {} values of {arity}",
                                args.len()
                            ));
                        }
                        Ok(())
                    }
                }
            }
            Expr::Pair(left, right) => {
                self.expr(left, false)?;
                self.expr(right, false)
            }
            Expr::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.expr(condition, false)?;
                self.branches(
                    [(None, then_branch.as_ref()), (None, else_branch.as_ref())],
                    tail,
                )
            }
            Expr::Match { scrutinee, arms } => {
                self.expr(scrutinee, false)?;
                self.branches(
                    arms.iter().map(|(pattern, body)| (Some(pattern), body)),
                    tail,
                )
            }
            Expr::Block(block) => self.block(block, tail),
            Expr::Widen(inner) | Expr::Succeed(inner) => self.expr(inner, false),
        }
    }
}

/// The kind of an expression, for a diagnostic.
fn constructs_of(expr: &Expr) -> String {
    let mut out = BTreeSet::new();
    expr_constructs(expr, &mut out);
    out.into_iter()
        .next()
        .unwrap_or_else(|| "a read".to_owned())
}

fn ident_text(ident: &Ident) -> String {
    match ident {
        Ident::Local(n) => format!("v{n}"),
        Ident::Holder(n) => format!("m{n}"),
        Ident::Operand(n) => format!("a{n}"),
        Ident::Callee(n) => format!("c{n}"),
        Ident::Boxed(n) => format!("r{n}"),
        Ident::Part(n) => format!("h{n}"),
        Ident::Capture(n) => format!("k{n}"),
        Ident::Param(n) => format!("p{n}"),
        Ident::Function(n) => format!("f{n}"),
        Ident::Export(name) => name.clone(),
    }
}

/// Every closure applied through a binding of function type `n` is bound by
/// `let c: Fn<n> = ...`, so its `apply` is known; a call through it
/// propagates exactly when that `apply` is fallible.
fn apply_types(
    block: &Block,
    declared: &Declared,
    types: &mut BTreeMap<Ident, u64>,
    tail: bool,
) -> Result<(), String> {
    for binding in &block.lets {
        if let (Pat::Bind(ident), Some(Type::Fn(n))) = (&binding.pat, &binding.ty) {
            types.insert(ident.clone(), *n);
        }
        apply_types_expr(&binding.value, declared, types, false)?;
    }
    apply_types_expr(&block.tail, declared, types, tail)
}

/// `tail` says the expression is the value a fallible function returns,
/// where a fallible application returns its result without propagating.
fn apply_types_expr(
    expr: &Expr,
    declared: &Declared,
    types: &mut BTreeMap<Ident, u64>,
    tail: bool,
) -> Result<(), String> {
    match expr {
        Expr::Apply {
            holder,
            args,
            propagate,
        } => {
            let fn_type = types.get(holder).ok_or_else(|| {
                format!(
                    "`{}` is applied without a function type",
                    ident_text(holder)
                )
            })?;
            let fallible = declared
                .applies
                .get(fn_type)
                .ok_or_else(|| format!("Fn{fn_type}::apply is not declared"))?;
            if fallible != propagate && !(tail && *fallible) {
                return fail(format!(
                    "arithmetic: an application of Fn{fn_type} {} a failure its apply {}",
                    if *propagate { "propagates" } else { "drops" },
                    if *fallible { "can have" } else { "cannot have" }
                ));
            }
            args.iter()
                .try_for_each(|arg| apply_types_expr(arg, declared, types, false))
        }
        Expr::Box(inner) | Expr::Widen(inner) | Expr::Succeed(inner) | Expr::Not(inner) => {
            apply_types_expr(inner, declared, types, false)
        }
        Expr::Call { args, .. } | Expr::Construct { args, .. } => args
            .iter()
            .try_for_each(|arg| apply_types_expr(arg, declared, types, false)),
        Expr::Pair(left, right) => {
            apply_types_expr(left, declared, types, false)?;
            apply_types_expr(right, declared, types, false)
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            apply_types_expr(condition, declared, types, false)?;
            apply_types(then_branch, declared, types, tail)?;
            apply_types(else_branch, declared, types, tail)
        }
        Expr::Match { scrutinee, arms } => {
            apply_types_expr(scrutinee, declared, types, false)?;
            arms.iter()
                .try_for_each(|(_, body)| apply_types(body, declared, types, tail))
        }
        Expr::Block(block) => apply_types(block, declared, types, tail),
        Expr::Lit(_)
        | Expr::Move(_)
        | Expr::Copy(_)
        | Expr::Deref(_)
        | Expr::Clone(_)
        | Expr::Unbox(_)
        | Expr::UnboxRef(_)
        | Expr::Uncons(_)
        | Expr::IsZero(_)
        | Expr::Predecessor(_) => Ok(()),
    }
}

/// Check a crate before it is printed.
///
/// # Errors
///
/// Returns the first violation: a heap type, function, or construct in
/// `rust-core` (hidden allocation), an undeclared or doubly declared name, a
/// name bound twice in one function (hygiene), a value moved twice or read
/// after it was moved (ownership), or a failure dropped, invented, or
/// returned from a function that declares none (arithmetic).
pub fn validate(krate: &Crate) -> Result<(), String> {
    let declared = declared(krate)?;
    for item in &krate.items {
        let mut checker = Checker {
            profile: krate.profile,
            declared: &declared,
            bound: BTreeSet::new(),
            scope: Vec::new(),
            moved: BTreeSet::new(),
            propagates: false,
            fallible: false,
        };
        match item {
            ItemDef::Enum { variants, .. } => {
                for ty in variants.iter().flat_map(|(_, fields)| fields) {
                    checker.ty(ty)?;
                }
            }
            ItemDef::Apply {
                fn_type,
                parameters,
                result,
                fallible,
                arms,
            } => {
                for ty in parameters.iter().chain([result]) {
                    checker.ty(ty)?;
                }
                for arm in arms {
                    let is_function = declared.functions.get(&arm.function).ok_or_else(|| {
                        format!("Fn{fn_type} dispatches to undeclared f{}", arm.function)
                    })?;
                    if *is_function != arm.function_fallible || (arm.function_fallible && !fallible)
                    {
                        return fail(format!(
                            "arithmetic: Fn{fn_type}::apply misstates the failure of f{}",
                            arm.function
                        ));
                    }
                    if arm
                        .captures
                        .iter()
                        .any(|read| matches!(read, super::ast::CaptureRead::Unbox))
                    {
                        checker.heap("a boxed capture")?;
                    }
                }
            }
            ItemDef::Function {
                name,
                parameters,
                result,
                body,
            } => {
                checker.ty(result)?;
                for (pattern, ty) in parameters {
                    checker.ty(ty)?;
                    checker.bind(pattern)?;
                }
                let fallible = matches!(result, Type::Fallible(_));
                checker.fallible = fallible;
                checker.block(body, true)?;
                if checker.propagates && !fallible {
                    return fail(format!(
                        "arithmetic: `{}` propagates a failure its result type does not carry",
                        ident_text(name)
                    ));
                }
                let mut types = BTreeMap::new();
                for (pattern, ty) in parameters {
                    if let (Pat::Bind(ident), Type::Fn(n)) = (pattern, ty) {
                        types.insert(ident.clone(), *n);
                    }
                }
                apply_types(body, &declared, &mut types, fallible)?;
            }
        }
    }
    Ok(())
}

// --- correspondence ----------------------------------------------------------

/// Every construct a crate emits, by kind.
#[must_use]
pub fn constructs(krate: &Crate) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    for item in &krate.items {
        match item {
            ItemDef::Enum { name, variants } => {
                out.insert(match name {
                    Type::Fn(_) => "enum:closures".to_owned(),
                    _ => "enum:adt".to_owned(),
                });
                for ty in variants.iter().flat_map(|(_, fields)| fields) {
                    type_constructs(ty, &mut out);
                }
            }
            ItemDef::Apply {
                parameters, result, ..
            } => {
                out.insert("apply:dispatch".to_owned());
                for ty in parameters.iter().chain([result]) {
                    type_constructs(ty, &mut out);
                }
            }
            ItemDef::Function {
                parameters,
                result,
                body,
                ..
            } => {
                out.insert("function".to_owned());
                for (_, ty) in parameters {
                    type_constructs(ty, &mut out);
                }
                type_constructs(result, &mut out);
                block_constructs(body, &mut out);
            }
        }
    }
    out
}

fn type_constructs(ty: &Type, out: &mut BTreeSet<String>) {
    let kind = match ty {
        Type::Unit => "unit",
        Type::Bool => "bool",
        Type::Nat => "nat",
        Type::Int => "int",
        Type::Fixed(_) => "fixed",
        Type::Ordering => "ordering",
        Type::Str => "string",
        Type::Bytes => "bytes",
        Type::Option(_) => "option",
        Type::Result(..) => "result",
        Type::List(_) => "list",
        Type::Pair(..) => "pair",
        Type::Adt(_) => "adt",
        Type::Fn(_) => "fn",
        Type::Rc(_) => "rc",
        Type::Fallible(_) => "fallible",
        Type::Ref(_) => "ref",
    };
    out.insert(format!("type:{kind}"));
    match ty {
        Type::Option(inner)
        | Type::List(inner)
        | Type::Rc(inner)
        | Type::Fallible(inner)
        | Type::Ref(inner) => type_constructs(inner, out),
        Type::Result(left, right) | Type::Pair(left, right) => {
            type_constructs(left, out);
            type_constructs(right, out);
        }
        _ => {}
    }
}

fn block_constructs(block: &Block, out: &mut BTreeSet<String>) {
    for binding in &block.lets {
        if let Some(ty) = &binding.ty {
            type_constructs(ty, out);
            out.insert("let".to_owned());
        }
        match binding.pat {
            Pat::Tuple(_) => {
                out.insert("destructure:pair".to_owned());
            }
            Pat::Unit => {
                out.insert("match:unit".to_owned());
            }
            _ => {}
        }
        expr_constructs(&binding.value, out);
    }
    expr_constructs(&block.tail, out);
}

fn pat_constructs(pattern: &Pat, out: &mut BTreeSet<String>) {
    let kind = match pattern {
        Pat::None | Pat::Some(_) => "match:option",
        Pat::Ok(_) | Pat::Err(_) => "match:result",
        Pat::Ordering(_) => "match:ordering",
        Pat::Adt { .. } => "match:adt",
        Pat::Unit => "match:unit",
        Pat::Wild | Pat::Bind(_) | Pat::Tuple(_) => return,
    };
    out.insert(kind.to_owned());
}

fn expr_constructs(expr: &Expr, out: &mut BTreeSet<String>) {
    match expr {
        Expr::Lit(literal) => {
            let kind = match literal {
                Lit::Unit => "unit",
                Lit::Bool(_) => "bool",
                Lit::Nat(_) => "nat",
                Lit::Int(_) => "int",
                Lit::Fixed(..) => "fixed",
                Lit::Str(_) => "string",
                Lit::Bytes(_) => "bytes",
                Lit::Ordering(_) => "ordering",
            };
            out.insert(format!("lit:{kind}"));
        }
        Expr::Move(_) | Expr::Clone(_) | Expr::Copy(_) | Expr::Deref(_) => {
            out.insert("read".to_owned());
        }
        Expr::Not(inner) => {
            out.insert("if".to_owned());
            expr_constructs(inner, out);
        }
        Expr::Unbox(_) | Expr::UnboxRef(_) | Expr::Box(_) => {
            out.insert("box".to_owned());
            if let Expr::Box(inner) = expr {
                expr_constructs(inner, out);
            }
        }
        Expr::Call { callee, args, .. } => {
            out.insert(match callee {
                Callee::Function(_) => "call:function".to_owned(),
                Callee::Runtime(item) => {
                    let path = item.path();
                    let base = path.rsplit("::").next().unwrap_or(&path).to_owned();
                    format!("call:runtime:{}", runtime_kind(&base))
                }
            });
            args.iter().for_each(|arg| expr_constructs(arg, out));
        }
        Expr::Apply { args, .. } => {
            out.insert("apply".to_owned());
            args.iter().for_each(|arg| expr_constructs(arg, out));
        }
        Expr::Construct { ctor, args } => {
            let kind = match ctor {
                Ctor::None(_) => "none",
                Ctor::Some => "some",
                Ctor::Ok(..) => "ok",
                Ctor::Err(..) => "err",
                Ctor::Adt { .. } => "adt",
                Ctor::Closure { .. } => "closure",
                Ctor::Cons => "cons",
                Ctor::Nil(_) => "nil",
            };
            out.insert(format!("construct:{kind}"));
            args.iter().for_each(|arg| expr_constructs(arg, out));
        }
        Expr::Pair(left, right) => {
            out.insert("construct:pair".to_owned());
            expr_constructs(left, out);
            expr_constructs(right, out);
        }
        Expr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            out.insert(if matches!(condition.as_ref(), Expr::IsZero(_)) {
                "match:nat".to_owned()
            } else {
                "if".to_owned()
            });
            expr_constructs(condition, out);
            block_constructs(then_branch, out);
            block_constructs(else_branch, out);
        }
        Expr::Match { scrutinee, arms } => {
            if matches!(scrutinee.as_ref(), Expr::Uncons(_)) {
                out.insert("match:list".to_owned());
            } else {
                for (pattern, _) in arms {
                    pat_constructs(pattern, out);
                }
            }
            expr_constructs(scrutinee, out);
            for (_, body) in arms {
                block_constructs(body, out);
            }
        }
        Expr::Block(block) => block_constructs(block, out),
        Expr::Uncons(_) | Expr::IsZero(_) | Expr::Predecessor(_) => {}
        Expr::Widen(inner) => {
            out.insert("widen".to_owned());
            expr_constructs(inner, out);
        }
        Expr::Succeed(inner) => {
            out.insert("succeed".to_owned());
            expr_constructs(inner, out);
        }
    }
}

/// The calculus primitive a runtime function realizes, by its path's last
/// segment: fixed-width paths share one name per operation.
fn runtime_kind(base: &str) -> String {
    for (prefix, kind) in [
        ("checked_neg_", "checked_neg"),
        ("format_", "format_decimal"),
        ("parse_", "parse_decimal"),
        ("append_", "append"),
        ("length_", "length"),
        ("index_", "index"),
        ("slice_", "slice"),
    ] {
        if base.starts_with(prefix) {
            return kind.to_owned();
        }
    }
    match base {
        "nat_succ" => "succ".to_owned(),
        other => other.to_owned(),
    }
}

/// For every construct a rendering can emit, the calculus elements (or
/// structural realizations, §17.14) it realizes. A construct is justified
/// in a rendering exactly when the realized program uses one of them.
pub const CORRESPONDENCE: &[(&str, &[&str])] = &[
    ("function", &["function"]),
    ("enum:adt", &["type:adt"]),
    ("enum:closures", &["type:fn"]),
    ("apply:dispatch", &["type:fn"]),
    (
        "let",
        &["expr:let", "expr:match", "expr:prim", "expr:apply"],
    ),
    (
        "read",
        &[
            "expr:var",
            "expr:match",
            "expr:prim",
            "expr:apply",
            "expr:first",
            "expr:second",
            "expr:field",
            "function",
        ],
    ),
    ("box", &["indirection"]),
    (
        "destructure:pair",
        &["expr:first", "expr:second", "shape:pair"],
    ),
    ("type:unit", &["type:unit"]),
    ("type:bool", &["type:bool"]),
    ("type:nat", &["type:nat", "type:fixed"]),
    ("type:int", &["type:int", "type:fixed"]),
    ("type:fixed", &["type:fixed"]),
    ("type:ordering", &["type:ordering"]),
    ("type:string", &["type:string"]),
    ("type:bytes", &["type:bytes"]),
    ("type:option", &["type:option"]),
    ("type:result", &["type:result"]),
    ("type:list", &["type:list"]),
    ("type:pair", &["type:pair"]),
    ("type:adt", &["type:adt"]),
    ("type:fn", &["type:fn"]),
    ("type:rc", &["indirection"]),
    ("type:fallible", &["overflow"]),
    ("type:ref", &["export"]),
    ("lit:unit", &["value:unit", "shape:unit"]),
    ("lit:bool", &["value:bool", "shape:true", "shape:false"]),
    ("lit:nat", &["value:nat", "shape:zero"]),
    ("lit:int", &["value:int"]),
    (
        "lit:fixed",
        &[
            "value:u8",
            "value:u16",
            "value:u32",
            "value:u64",
            "value:i8",
            "value:i16",
            "value:i32",
            "value:i64",
        ],
    ),
    ("lit:string", &["value:string"]),
    ("lit:bytes", &["value:bytes"]),
    (
        "lit:ordering",
        &["value:ordering", "shape:lt", "shape:eq", "shape:gt"],
    ),
    ("construct:none", &["value:none", "shape:none"]),
    ("construct:some", &["value:some", "shape:some"]),
    ("construct:ok", &["value:ok", "shape:ok"]),
    ("construct:err", &["value:error", "shape:error"]),
    ("construct:adt", &["value:adt", "shape:adt"]),
    ("construct:closure", &["expr:closure"]),
    ("construct:cons", &["value:list", "shape:cons"]),
    ("construct:nil", &["value:list", "shape:nil"]),
    ("construct:pair", &["value:pair", "shape:pair"]),
    ("if", &["expr:cond", "shape:true", "shape:false"]),
    ("match:nat", &["shape:zero", "shape:succ"]),
    ("match:list", &["shape:nil", "shape:cons"]),
    ("match:option", &["shape:none", "shape:some", "expr:field"]),
    ("match:result", &["shape:ok", "shape:error"]),
    ("match:ordering", &["shape:lt", "shape:eq", "shape:gt"]),
    ("match:adt", &["shape:adt", "expr:field"]),
    ("match:unit", &["shape:unit"]),
    ("call:function", &["expr:call", "export"]),
    ("apply", &["expr:apply"]),
    ("widen", &["prim:convert"]),
    ("succeed", &["overflow"]),
    ("call:runtime:succ", &["shape:succ"]),
    ("call:runtime:nat_add", &["prim:nat_add"]),
    ("call:runtime:nat_sub", &["prim:nat_sub"]),
    ("call:runtime:nat_mul", &["prim:nat_mul"]),
    ("call:runtime:nat_quot", &["prim:nat_quot"]),
    ("call:runtime:nat_rem", &["prim:nat_rem"]),
    ("call:runtime:nat_eq", &["prim:nat_eq"]),
    ("call:runtime:nat_le", &["prim:nat_le"]),
    ("call:runtime:nat_lt", &["prim:nat_lt"]),
    ("call:runtime:int_add", &["prim:int_add"]),
    ("call:runtime:int_sub", &["prim:int_sub"]),
    ("call:runtime:int_mul", &["prim:int_mul"]),
    ("call:runtime:int_neg", &["prim:int_neg"]),
    ("call:runtime:int_quot", &["prim:int_quot"]),
    ("call:runtime:int_rem", &["prim:int_rem"]),
    ("call:runtime:bool_not", &["prim:bool_not"]),
    ("call:runtime:bool_and", &["prim:bool_and"]),
    ("call:runtime:bool_or", &["prim:bool_or"]),
    ("call:runtime:equal", &["prim:equal"]),
    ("call:runtime:compare", &["prim:compare"]),
    ("call:runtime:checked_add", &["prim:checked_add"]),
    ("call:runtime:checked_sub", &["prim:checked_sub"]),
    ("call:runtime:checked_mul", &["prim:checked_mul"]),
    ("call:runtime:checked_quot", &["prim:checked_quot"]),
    ("call:runtime:checked_neg", &["prim:checked_neg"]),
    ("call:runtime:bit_and", &["prim:bit_and"]),
    ("call:runtime:bit_or", &["prim:bit_or"]),
    ("call:runtime:bit_xor", &["prim:bit_xor"]),
    ("call:runtime:bit_not", &["prim:bit_not"]),
    ("call:runtime:shift_left", &["prim:shift_left"]),
    ("call:runtime:shift_right", &["prim:shift_right"]),
    ("call:runtime:convert", &["prim:convert"]),
    ("call:runtime:append", &["prim:append"]),
    ("call:runtime:length", &["prim:length"]),
    ("call:runtime:index", &["prim:index"]),
    ("call:runtime:slice", &["prim:slice"]),
    ("call:runtime:utf8_encode", &["prim:utf8_encode"]),
    ("call:runtime:utf8_decode", &["prim:utf8_decode"]),
    ("call:runtime:compare_bytes", &["prim:compare_bytes"]),
    ("call:runtime:split_exact", &["prim:split_exact"]),
    ("call:runtime:join", &["prim:join"]),
    ("call:runtime:format_decimal", &["prim:format_decimal"]),
    ("call:runtime:parse_decimal", &["prim:parse_decimal"]),
];

/// Check that every construct `krate` emits is in [`CORRESPONDENCE`] and
/// realizes an element of `elements`, the program's calculus elements and
/// the structural realizations it uses.
///
/// # Errors
///
/// Returns the first construct with no row, or whose row names no element
/// the program uses.
pub fn correspond(krate: &Crate, elements: &BTreeSet<String>) -> Result<(), String> {
    for construct in constructs(krate) {
        let (_, realized) = CORRESPONDENCE
            .iter()
            .find(|(name, _)| *name == construct)
            .ok_or_else(|| {
                format!("the construct `{construct}` has no target-semantics correspondence")
            })?;
        if !realized.iter().any(|element| elements.contains(*element)) {
            return fail(format!(
                "the construct `{construct}` realizes none of {realized:?}, which the program does not use"
            ));
        }
    }
    Ok(())
}
