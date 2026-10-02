//! The reference renderings of target programs to Rust (SPEC.md §17.14,
//! §17.16).
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
//! realized by work bounded by its charge.
//!
//! Rendering lowers the program to the closed Rust AST ([`ast`]), checks it
//! ([`validate`]), and prints it into canonical bytes. A function that
//! cannot overflow returns its value; one that can returns `R<T>` and every
//! call to it propagates with `?`. Packages ([`package`]) add exported
//! functions, a Cargo manifest, and provenance. A rendered function never
//! panics on a valid program; fuel and stack depth are outside the
//! observable contract. Rendering refuses a program it cannot render
//! faithfully.

pub mod ast;
mod lower;
pub mod package;
pub mod runtime;
pub mod validate;

use std::collections::BTreeSet;
use std::fmt::Write as _;

use super::{Program, Ty, Value};
use lower::{index, Lowering, Owner};

fn fail<T>(reason: impl Into<String>) -> Result<T, String> {
    Err(reason.into())
}

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

    /// The profile of a target identifier.
    #[must_use]
    pub fn named(target: &str) -> Option<Self> {
        Self::ALL
            .into_iter()
            .find(|profile| profile.target() == target)
    }
}

/// The calculus elements a lowering realizes: the program's own, and the
/// structural realizations it uses (§17.14): `function` always, `overflow`
/// when a primitive or successor can fail, and `indirection` when a type
/// holds itself.
fn elements(program: &Program, lowering: &Lowering<'_>) -> BTreeSet<String> {
    let mut out = super::realization::program_elements(program);
    out.insert("function".to_owned());
    const FAILING: [&str; 9] = [
        "prim:nat_add",
        "prim:nat_mul",
        "prim:int_add",
        "prim:int_sub",
        "prim:int_mul",
        "prim:int_neg",
        "prim:int_quot",
        "prim:parse_decimal",
        "shape:succ",
    ];
    if FAILING.iter().any(|element| out.contains(*element)) {
        out.insert("overflow".to_owned());
    }
    if lowering.indirect() {
        out.insert("indirection".to_owned());
    }
    out
}

/// Lower a valid program to the closed Rust AST of `profile`, checked.
///
/// # Errors
///
/// Returns the reason the program is invalid, has no rendering in
/// `profile`, or lowers to a crate the checks of [`validate`] refuse.
pub fn lower(program: &Program, profile: Profile) -> Result<ast::Crate, String> {
    let mut lowering = Lowering::new(program, profile)?;
    let krate = lowering.lower()?;
    validate::validate(&krate)?;
    validate::correspond(&krate, &elements(program, &lowering))?;
    Ok(krate)
}

/// Which functions of a valid program can overflow, as the renderings
/// type them: a function that can returns `R<T>`, one that cannot returns
/// its value.
///
/// # Errors
///
/// Returns the reason the program is invalid.
pub fn fallible_functions(program: &Program) -> Result<Vec<bool>, String> {
    Ok(Lowering::new(program, Profile::Std)?.fallible)
}

/// Render a valid program to the canonical bytes of a library crate of
/// `profile`.
///
/// # Errors
///
/// As [`lower`].
pub fn render(program: &Program, profile: Profile) -> Result<String, String> {
    Ok(ast::print(&lower(program, profile)?))
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
    let owned = vec![package::Passing::Own; arguments.len()];
    render_caller(
        program,
        profile,
        entry,
        arguments,
        &Caller {
            library: "program",
            function: &format!("f{entry}"),
            passing: &owned,
        },
    )
}

/// How a harness reaches the function it runs: the library crate it uses,
/// the function's name there, and how each argument is passed.
pub struct Caller<'a> {
    pub library: &'a str,
    pub function: &'a str,
    pub passing: &'a [package::Passing],
}

/// A binary crate that runs `entry` of `program` through `caller` on
/// `arguments` and prints the observable outcome, then the work the runtime
/// counted, as [`render_harness`].
///
/// # Errors
///
/// As [`render_harness`].
pub fn render_caller(
    program: &Program,
    profile: Profile,
    entry: u64,
    arguments: &[Value],
    caller: &Caller<'_>,
) -> Result<String, String> {
    super::check::check_arguments(program, entry, arguments)?;
    let mut renderer = Lowering::new(program, profile)?;
    let function = program
        .functions
        .get(index(entry)?)
        .ok_or_else(|| format!("function {entry} is not declared"))?;
    let mut out = format!("#![forbid(unsafe_code)]\nuse {}::*;\n", caller.library);
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
        .zip(
            caller
                .passing
                .iter()
                .chain(std::iter::repeat(&package::Passing::Own)),
        )
        .map(|((argument, ty), passing)| {
            renderer.value(argument, ty).map(|value| {
                let text = ast::print_expr(&value);
                if *passing == package::Passing::Borrow {
                    format!("&{text}")
                } else {
                    text
                }
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let result_show = show(&function.result, "value")?;
    // An entry that cannot overflow returns its value directly.
    let fallible = renderer
        .fallible
        .get(index(entry)?)
        .copied()
        .unwrap_or(false);
    let run = if fallible {
        format!(
            "    match {}({}) {{\n        Ok(value) => println!(\"{{}}\", {result_show}),\n        Err(Overflow) => println!(\"{{{{\\\"kind\\\":\\\"overflow\\\"}}}}\"),\n    }}\n",
            caller.function,
            rendered.join(", ")
        )
    } else {
        format!(
            "    let value = {}({});\n    println!(\"{{}}\", {result_show});\n",
            caller.function,
            rendered.join(", ")
        )
    };
    let _ = write!(
        out,
        "\nfn main() {{\n{run}    println!(\"work {{}}\", work());\n}}\n"
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
    use super::Profile;

    #[test]
    fn targets_name_their_profiles() {
        for profile in Profile::ALL {
            assert_eq!(Profile::named(profile.target()), Some(profile));
        }
        assert_eq!(Profile::named("rust-wasm"), None);
    }
}
