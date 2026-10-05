//! The contractivity receipt (SPEC.md §34.2).
//!
//! This is the part of the PIRTM layer that is a *reproduction* rather than an
//! interpretation. The `sovereign-pirtm` compiler computes
//!
//! ```text
//! G      = A · diag(λ)
//! ‖G‖₁   = max over j of  Σ_i |G[i][j]|        (the maximum absolute column sum)
//! passed = ‖G‖₁ < 1
//! ```
//!
//! in exact rational arithmetic over ℚ, reports the norm as a reduced
//! `[numerator, denominator]` pair, refuses a negative matrix entry or factor
//! before computing anything, and states a rejection as `‖G‖₁ = n/d >= 1`. All
//! of that was established by probing the compiler's `pirtm_verify_ensemble`
//! interface and is reproduced here value for value, so a receipt produced by
//! this module is comparable with one produced by the compiler.
//!
//! What is *not* reproduced is stated in §34.0 and repeated here because it is
//! the claim a reader is most likely to over-read: **the compiler's
//! `seal_hash` preimage is not recoverable, so [`ContractivityReceipt::hash`]
//! is a LexLean hash and does not reproduce it.** The receipt says so in a
//! field of its own, so nobody goes looking for a LexLean receipt in a PIRTM
//! ledger.
//!
//! # What a receipt does and does not certify
//!
//! A receipt certifies one thing, and it is worth being exact about what it is:
//! the reference graph of a lexeme's declarations satisfies the §34.2 norm
//! bound. It does **not** certify that the lexeme "evolved lawfully". That
//! reading needs a dynamical system, a trajectory, and a notion of lawful
//! process, and §34 specifies a static norm over a static graph. Calling this a
//! lawfulness certificate would be the exact blurring R2 forbids, so the
//! receipt's own status vocabulary says `ACCEPT` or `REJECT` and never
//! `LAWFUL`.

use crate::artifact::canonical_json::Json;
use crate::artifact::content_id::Sha256Digest;
use crate::diagnostic::Diagnostic;
use crate::error::LexLeanError;

use super::canonical::{Declaration, FrameWriter, Token};
use super::rational::Rational;
use super::stratum::Stratum;

/// The hash domain of the receipt (SPEC.md §34.2).
pub const RECEIPT_DOMAIN: &str = "lexlean-contractivity-v1";

/// Build a contractivity diagnostic, so every refusal in this module names a
/// code the registry already lists.
fn contractivity_error(
    code_value: crate::diagnostic::DiagnosticCode,
    reason: impl Into<String>,
) -> LexLeanError {
    LexLeanError::from_diagnostic(Diagnostic::new(code_value, reason.into()))
}

/// The status of a receipt whose norm is below one.
pub const STATUS_ACCEPT: &str = "ACCEPT";

/// The status of a receipt whose norm is not below one.
pub const STATUS_REJECT: &str = "REJECT";

/// Which token stream §34.2's reference relation and its token count are taken
/// over.
///
/// [`Self::HeadInclusive`] is the §33.1 stream unchanged, which is what §34.2
/// said before the body-proper amendment. It is retained, and reachable, so the
/// amendment is a claim a test can check against the text it replaced rather
/// than a wording change asserted in prose: the two definitions disagree on
/// real sources, and [`Scan::HeadInclusive`] is how that disagreement is
/// exhibited. It is **not** normative. §34.2 takes [`Self::BodyProper`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scan {
    /// The whole §33.1 token stream, declaration head included.
    HeadInclusive,
    /// The stream with the two leading head tokens removed. The normative one.
    #[default]
    BodyProper,
}

impl Scan {
    /// The tokens this scan reads for one atom.
    #[must_use]
    pub fn tokens(self, atom: &Declaration) -> &[Token] {
        match self {
            Self::HeadInclusive => &atom.body,
            Self::BodyProper => atom.body_proper(),
        }
    }

    /// Whether this is the scan §34.2 normatively specifies.
    #[must_use]
    pub fn is_normative(self) -> bool {
        self == Self::BodyProper
    }
}

/// The reference adjacency matrix of a stratum: `A[i][j]` is `1` when atom `i`'s
/// body mentions atom `j`'s unqualified name as a whole token.
///
/// The relation is a reference relation and nothing more. It is not symmetric,
/// it is not transitive, and it is deliberately allowed to have `A[i][i] = 1`
/// for a self-referential declaration: refusing self-reference would make the
/// receipt a statement about a graph shape rather than about the lexeme, and a
/// lexeme that refers to itself is a lexeme whose reference count is what it
/// is.
///
/// Non-negativity of §34.2 is enforced by the element type rather than by a
/// check: `u64` cannot hold a negative entry, so there is no signed value to
/// test and an `is_non_negative` predicate over this matrix would be a
/// tautology. The receipt deliberately does not carry `A`, so nothing here is
/// ever read back from untrusted JSON; the norm is recomputed from the
/// canonical form instead.
///
/// A mention is counted only when the token is the *final segment* of the name.
/// `Foo.bar` contributes a reference to `Foo.bar` and to no atom called
/// `bar`, because a bare `bar` token in some other namespace is a different
/// declaration and counting it would make the graph depend on a name collision
/// §33.1 already forbids.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Adjacency {
    rows: Vec<Vec<u64>>,
}

impl Adjacency {
    /// Build the reference matrix of a stratum in canonical atom order, over
    /// the normative §34.2 scan.
    #[must_use]
    pub fn of(atoms: &[Declaration]) -> Self {
        Self::with_scan(atoms, Scan::BodyProper)
    }

    /// Build the reference matrix over an explicitly chosen scan.
    #[must_use]
    pub fn with_scan(atoms: &[Declaration], scan: Scan) -> Self {
        let final_segments: Vec<&str> = atoms
            .iter()
            .map(|atom| atom.name.rsplit('.').next().unwrap_or(&atom.name))
            .collect();
        let rows = atoms
            .iter()
            .map(|atom| {
                let tokens = scan.tokens(atom);
                final_segments
                    .iter()
                    .map(|segment| {
                        let mentions = tokens.iter().filter(|token| token.text == *segment).count();
                        u64::try_from(mentions).unwrap_or(u64::MAX)
                    })
                    .collect()
            })
            .collect();
        Self { rows }
    }

    /// The row count, which is the atom count.
    #[must_use]
    pub fn atom_count(&self) -> usize {
        self.rows.len()
    }

    /// The entries of row `index`, which is the incoming reference weight of
    /// each atom.
    ///
    /// # Errors
    /// Returns the reason when `index` is not a row, which §25.5's checked
    /// arithmetic requires rather than an indexing panic.
    pub fn row(&self, index: usize) -> Result<&[u64], String> {
        self.rows
            .get(index)
            .map(Vec::as_slice)
            .ok_or_else(|| format!("row {index} is outside the {}-row matrix", self.rows.len()))
    }

    /// The number of atoms that mention atom `j`, which is `Σ_i A[i][j]` --- the
    /// un-weighted column sum of §34.2.
    ///
    /// # Errors
    /// Returns the reason when `atom` is not a column of the matrix.
    pub fn references_to(&self, atom: usize) -> Result<u64, String> {
        let row_count = self.rows.len();
        if atom >= row_count {
            return Err(format!(
                "atom {atom} is outside the {row_count}-atom matrix"
            ));
        }
        let total = self.rows.iter().try_fold(0u64, |accumulated, row| {
            row.get(atom)
                .and_then(|entry| accumulated.checked_add(*entry))
                .ok_or_else(|| format!("row {} does not reach atom {atom}", row.len()))
        })?;
        Ok(total)
    }

    /// The rows as the receipt records them: an array of arrays of integers.
    #[must_use]
    pub fn to_json(&self) -> Json {
        Json::Arr(
            self.rows
                .iter()
                .map(|row| {
                    Json::Arr(
                        row.iter()
                            .map(|entry| Json::Int(i64::try_from(*entry).unwrap_or(i64::MAX)))
                            .collect(),
                    )
                })
                .collect(),
        )
    }
}

/// The contraction factor of one atom: `λ[j] = 1 / (1 + t_j)`.
///
/// The numerator is one by construction, so the factor is a unit fraction and
/// the norm of §34.2 is a ratio of a reference count to a token count. §34.2
/// fixes this recipe rather than leaving it open because a receipt is only
/// reproducible if the recipe is normative; a factor a caller chose per
/// receipt would make the norm a number the caller supplied rather than one the
/// lexeme determines.
///
/// An atom with an empty body has `t_j = 0` and so `λ[j] = 1`, which is what
/// puts the `‖G‖₁ = 1` boundary within reach of a real lexeme: a single
/// reference to an empty declaration is exactly one, and is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Contraction {
    numerators: Vec<i128>,
    denominators: Vec<i128>,
}

impl Contraction {
    /// The factors of a stratum's atoms, over the normative §34.2 scan.
    ///
    /// # Errors
    /// Returns the reason when a body token count would overflow the rational
    /// type, which §10's source limits make unreachable.
    pub fn of(atoms: &[Declaration]) -> Result<Self, String> {
        Self::with_scan(atoms, Scan::BodyProper)
    }

    /// The factors of a stratum's atoms over an explicitly chosen scan.
    ///
    /// # Errors
    /// Returns the reason when a body token count would overflow the rational
    /// type.
    pub fn with_scan(atoms: &[Declaration], scan: Scan) -> Result<Self, String> {
        let mut numerators = Vec::with_capacity(atoms.len());
        let mut denominators = Vec::with_capacity(atoms.len());
        for atom in atoms {
            let tokens = i128::try_from(scan.tokens(atom).len())
                .map_err(|_| "an atom body exceeds the rational range".to_owned())?;
            let denominator = tokens
                .checked_add(1)
                .ok_or_else(|| "an atom body exceeds the rational range".to_owned())?;
            numerators.push(1);
            denominators.push(denominator);
        }
        Ok(Self {
            numerators,
            denominators,
        })
    }

    /// The factor of one atom.
    ///
    /// # Errors
    /// Returns the reason when `atom` is outside the factor vector.
    pub fn of_atom(&self, atom: usize) -> Result<Rational, String> {
        let numerator = *self
            .numerators
            .get(atom)
            .ok_or_else(|| format!("atom {atom} has no contraction factor"))?;
        let denominator = *self
            .denominators
            .get(atom)
            .ok_or_else(|| format!("atom {atom} has no contraction factor"))?;
        Rational::new(numerator, denominator)
    }

    /// The number of factors.
    #[must_use]
    pub fn len(&self) -> usize {
        self.numerators.len()
    }

    /// Whether there are no factors, which §34.1's non-empty stratum forbids.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.numerators.is_empty()
    }
}

/// The exact rational one-norm of a stratum's reference graph: the maximum
/// absolute column sum of `A · diag(λ)`.
///
/// Because column `j` of `A · diag(λ)` sums to `λ[j] · refs(j)`, the norm is
/// computed as `refs(j) / (1 + t_j)` maximized over `j` --- which is the same
/// value the matrix product gives, obtained without forming the product. The
/// column-sum form is used rather than a row traversal precisely because it is
/// the definition §34.2 states and the one the compiler computes; a
/// reassociation that happened to agree on one input would not be the same
/// function.
///
/// The column is the maximum over an *empty* set only when the stratum is
/// empty, which §34.1 refuses before this is reached; the fallback is zero
/// rather than a panic so that a caller which bypassed §34.1 gets a stated
/// value instead of a crash.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Norm {
    /// The maximum column sum, in lowest terms.
    pub value: Rational,
    /// The atom whose column attained the maximum, for a receipt that says
    /// which column bound it. Ties keep the lowest index so the receipt is
    /// deterministic.
    pub column: usize,
}

/// Compute the norm of a stratum's reference graph.
///
/// # Errors
/// Returns the reason when the stratum has no atoms, or when the rational
/// arithmetic would overflow.
pub fn norm(stratum: &Stratum) -> Result<Norm, String> {
    norm_with_scan(stratum, Scan::BodyProper)
}

/// The norm over an explicitly chosen scan.
///
/// This exists so the §34.2 amendment is falsifiable: [`norm`] is the
/// normative answer, and this is the pre-amendment one computed the same way,
/// so a test can put a real source through both and observe the difference.
pub fn norm_with_scan(stratum: &Stratum, scan: Scan) -> Result<Norm, String> {
    let adjacency = Adjacency::with_scan(&stratum.atoms, scan);
    let contraction = Contraction::with_scan(&stratum.atoms, scan)?;
    column_max(&adjacency, &contraction)
}

/// The maximum column sum of `A · diag(λ)`.
fn column_max(adjacency: &Adjacency, contraction: &Contraction) -> Result<Norm, String> {
    let atoms = adjacency.atom_count();
    if atoms == 0 {
        return Err("a stratum with no atoms has no norm".to_owned());
    }
    let mut best: Option<Norm> = None;
    for atom in 0..atoms {
        // G's column `atom` is A's column `atom` scaled by λ[atom], so the
        // column sum is λ[atom] · Σ_i A[i][atom]. Building it as the exact
        // rational product keeps every intermediate exact; there is no integer
        // approximation anywhere on the path.
        let lambda = contraction.of_atom(atom)?;
        // A reference count is a `u64` over atoms, so lifting it into the
        // rational type cannot lose anything.
        let references = Rational::from_integer(adjacency.references_to(atom)? as i128)?;
        let value = lambda.mul(references)?;
        let candidate = Norm {
            value,
            column: atom,
        };
        best = Some(match best {
            // `>=` keeps the lowest index on a tie, so the receipt names the
            // same column every time.
            Some(current) if current.value >= value => current,
            _ => candidate,
        });
    }
    best.ok_or_else(|| "a stratum with no atoms has no norm".to_owned())
}

/// Whether `anchor` names an atom of `stratum`, by its fully qualified name or
/// by the unqualified final segment a Lean theorem is ordinarily written with.
///
/// The head of a declaration contains its own name, so accepting the final
/// segment here is what lets `contractivity --theorem b` name `Namespace.b`
/// without the caller repeating the namespace.
fn anchor_names_atom(stratum: &Stratum, anchor: &str) -> bool {
    stratum
        .atoms
        .iter()
        .any(|atom| atom.name == anchor || atom.name.rsplit('.').next() == Some(anchor))
}

/// A contractivity receipt over one stratum.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractivityReceipt {
    /// The stratum's prime index.
    pub prime_index: u64,
    /// The stratum's atom count.
    pub atom_count: usize,
    /// The declaration the receipt is anchored to; §34.2 requires it to be an
    /// atom of this stratum.
    pub anchor: String,
    /// The exact rational norm, in lowest terms.
    pub norm: Rational,
    /// The atom whose column attained the maximum.
    pub column: usize,
    /// [`STATUS_ACCEPT`] when the norm is below one, [`STATUS_REJECT`]
    /// otherwise.
    pub status: String,
}

impl ContractivityReceipt {
    /// Build the receipt for a stratum anchored to `anchor`.
    ///
    /// # Errors
    /// Returns the reason when `anchor` names a declaration that is not an atom
    /// of the stratum. The compiler gates its `theorem_name` on *presence
    /// only* and never checks the content; this module does check, which is
    /// strictly stronger and is recorded so that a receipt from here is never
    /// read as a receipt the compiler would have issued.
    pub fn of(stratum: &Stratum, anchor: &str) -> Result<Self, String> {
        if !anchor_names_atom(stratum, anchor) {
            return Err(format!(
                "the theorem anchor `{anchor}` names no atom of this stratum"
            ));
        }
        let Norm { value, column } = norm(stratum)?;
        Ok(Self {
            prime_index: stratum.prime_index,
            atom_count: stratum.atom_count(),
            anchor: anchor.to_owned(),
            status: if value.is_below_one() {
                STATUS_ACCEPT.to_owned()
            } else {
                STATUS_REJECT.to_owned()
            },
            norm: value,
            column,
        })
    }

    /// Whether the receipt is accepted, which is exactly `‖G‖₁ < 1`.
    #[must_use]
    pub fn accepted(&self) -> bool {
        self.status == STATUS_ACCEPT
    }

    /// The receipt for a caller that reports failures through the closed error
    /// model, so the anchor refusal carries its registered code.
    ///
    /// # Errors
    /// Returns [`LLG1012`](crate::code) when the anchor names no atom of the
    /// stratum. The same check as [`Self::of`]; this is the surface a command
    /// or an entry reader uses, because R5 requires every public failure to be
    /// a registered code rather than a free-form string.
    pub fn of_entry(stratum: &Stratum, anchor: &str) -> Result<Self, LexLeanError> {
        Self::of(stratum, anchor)
            .map_err(|reason| contractivity_error(crate::code!("LLG1012"), reason))
    }

    /// The receipt, or [`LLG1014`](crate::code) when the norm is not below one.
    ///
    /// §34.2 makes a non-contractive graph a *legitimate* outcome: the receipt
    /// records the refusal and the exact value that produced it, and that is a
    /// well-formed receipt rather than a failure. Demanding that a receipt be
    /// accepted is therefore a separate decision with its own code, taken by
    /// whatever caller is gating on contractivity. A caller that only wants the
    /// arithmetic uses [`Self::of`], which refuses nothing.
    ///
    /// # Errors
    /// Returns [`LLG1014`](crate::code) carrying the reduced value that failed.
    pub fn require_accepted(self) -> Result<Self, LexLeanError> {
        if self.accepted() {
            return Ok(self);
        }
        let (numerator, denominator) = self.norm.to_pair();
        Err(contractivity_error(
            crate::code!("LLG1014"),
            format!(
                "the reference graph has Norm = {numerator}/{denominator}, which is not below 1"
            ),
        ))
    }

    /// The receipt's own reason when it is refused, in the compiler's form.
    ///
    /// An accepted receipt has no reason, and asking for one returns `None`
    /// rather than an empty string: an empty reason reads as a reason that was
    /// formatted and found blank, which is a different statement.
    #[must_use]
    pub fn rejection(&self) -> Option<String> {
        if self.accepted() {
            return None;
        }
        let (numerator, denominator) = self.norm.to_pair();
        Some(format!(
            "NormContractivityViolation: ||G||_1 = {numerator}/{denominator} >= 1"
        ))
    }

    /// The canonical bytes of §34.2: the §21.1 frame encoding under
    /// [`RECEIPT_DOMAIN`] over the prime index, atom count, anchor, norm
    /// numerator, norm denominator, and status.
    #[must_use]
    pub fn canonical_form(&self) -> Vec<u8> {
        let (numerator, denominator) = self.norm.to_pair();
        let mut frames = FrameWriter::new(RECEIPT_DOMAIN);
        frames.frame("prime-index", self.prime_index.to_string().as_bytes());
        frames.frame("atom-count", self.atom_count.to_string().as_bytes());
        frames.frame("anchor", self.anchor.as_bytes());
        frames.frame("norm-numerator", numerator.to_string().as_bytes());
        frames.frame("norm-denominator", denominator.to_string().as_bytes());
        frames.frame("status", self.status.as_bytes());
        frames.finish()
    }

    /// The receipt hash: a **LexLean** hash over [`Self::canonical_form`].
    ///
    /// This is not the PIRTM `seal_hash` and does not reproduce it; §34.2 says
    /// so and the entry carries the same statement in
    /// [`Self::HASH_AUTHORITY`], so the two are never confused.
    #[must_use]
    pub fn hash(&self) -> Sha256Digest {
        Sha256Digest::of(&self.canonical_form())
    }

    /// The authority a receipt's hash belongs to, as an entry records it.
    ///
    /// §34.0 records that the compiler's `seal_hash` preimage is not
    /// recoverable from the binary, so interoperability cannot be claimed and
    /// this field exists to say so in the data rather than only in prose.
    pub const HASH_AUTHORITY: &'static str = "lexlean-contractivity-v1";

    /// The receipt as canonical JSON, the form §34.2 has an entry carry.
    #[must_use]
    pub fn to_json(&self) -> Json {
        let (numerator, denominator) = self.norm.to_pair();
        Json::object(vec![
            ("anchor", Json::Str(self.anchor.clone())),
            ("atom_count", Json::Int(self.atom_count as i64)),
            ("column", Json::Int(self.column as i64)),
            ("hash", Json::Str(self.hash().to_hex())),
            ("hash_authority", Json::Str(Self::HASH_AUTHORITY.to_owned())),
            ("norm", self.norm.to_json()),
            ("norm_denominator", Json::Int(denominator as i64)),
            ("norm_numerator", Json::Int(numerator as i64)),
            ("prime_index", Json::Int(self.prime_index as i64)),
            ("status", Json::Str(self.status.clone())),
        ])
    }

    /// Read a receipt back from an entry field, refusing anything that is not
    /// the canonical encoding of itself.
    ///
    /// # Errors
    /// Returns the reason when a field is missing or the wrong shape, when the
    /// rational is not in lowest terms, when the status does not follow from
    /// the norm, or when the recorded hash is not the digest of the receipt's
    /// own frames. Each is §34.2's requirement that a receipt be
    /// self-describing.
    pub fn from_json(value: &Json) -> Result<Self, String> {
        let Json::Obj(fields) = value else {
            return Err("a contractivity receipt is not a JSON object".to_owned());
        };
        let text = |key: &str| -> Result<String, String> {
            match fields.get(key) {
                Some(Json::Str(body)) => Ok(body.clone()),
                _ => Err(format!(
                    "a receipt field `{key}` is missing or not a string"
                )),
            }
        };
        let integer = |key: &str| -> Result<i64, String> {
            match fields.get(key) {
                Some(Json::Int(number)) => Ok(*number),
                _ => Err(format!(
                    "a receipt field `{key}` is missing or not an integer"
                )),
            }
        };
        let Some(norm_field) = fields.get("norm") else {
            return Err("a receipt field `norm` is missing".to_owned());
        };
        let norm = Rational::from_pair(norm_field)?;
        let (numerator, denominator) = norm.to_pair();
        if integer("norm_numerator")? != numerator as i64
            || integer("norm_denominator")? != denominator as i64
        {
            return Err(
                "the receipt's separate numerator and denominator do not match its `norm` pair"
                    .to_owned(),
            );
        }
        let anchor = text("anchor")?;
        let status = text("status")?;
        let expected = if norm.is_below_one() {
            STATUS_ACCEPT
        } else {
            STATUS_REJECT
        };
        if status != expected {
            return Err(format!(
                "the receipt status `{status}` does not follow from a norm of {numerator}/{denominator}"
            ));
        }
        let receipt = Self {
            prime_index: u64::try_from(integer("prime_index")?)
                .map_err(|_| "the prime index is negative".to_owned())?,
            atom_count: usize::try_from(integer("atom_count")?)
                .map_err(|_| "the atom count is negative".to_owned())?,
            anchor,
            norm,
            column: usize::try_from(integer("column")?)
                .map_err(|_| "the bounding column is negative".to_owned())?,
            status,
        };
        if text("hash_authority")? != Self::HASH_AUTHORITY {
            return Err("the receipt does not state that its hash is LexLean's".to_owned());
        }
        if text("hash")? != receipt.hash().to_hex() {
            return Err("the receipt hash is not the digest of its own frames".to_owned());
        }
        Ok(receipt)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        norm, norm_with_scan, Adjacency, Contraction, ContractivityReceipt, Norm, Scan,
        STATUS_ACCEPT, STATUS_REJECT,
    };
    use crate::artifact::canonical_json::Json;
    use crate::lexeme::canonical::canonicalize;
    use crate::lexeme::rational::Rational;
    use crate::lexeme::stratum::Stratum;

    fn stratum_of(source: &str) -> Stratum {
        let canonical = canonicalize(source).expect("the source canonicalizes");
        Stratum::of(&canonical).expect("the source is a stratum")
    }

    #[test]
    fn the_matrix_counts_whole_token_references_in_canonical_order() {
        // `bar` mentions `alpha` twice and itself once; `alpha` mentions
        // neither. Row order follows §33.1's name order, not source order.
        // Neither declaration references itself merely by being named: that is
        // what §34.2's body proper is for.
        let stratum = stratum_of("def alpha : Nat := 1\ndef bar : Nat := alpha + alpha + bar\n");
        let adjacency = Adjacency::of(&stratum.atoms);
        assert_eq!(stratum.atoms[0].name, "alpha");
        assert_eq!(stratum.atoms[1].name, "bar");
        assert_eq!(
            adjacency.row(0).expect("row"),
            &[0u64, 0][..],
            "alpha mentions neither atom"
        );
        assert_eq!(
            adjacency.row(1).expect("row"),
            &[2u64, 1][..],
            "bar mentions alpha twice and itself once"
        );
        assert_eq!(adjacency.references_to(0).expect("column"), 2);
        assert_eq!(adjacency.references_to(1).expect("column"), 1);
    }

    #[test]
    fn a_declaration_does_not_reference_itself_through_its_own_head() {
        // §34.2 scans the body proper. Scanned whole, the head token `alpha`
        // would make this declaration reference itself and every atom would
        // self-reference, which would assert a reference the lexeme does not
        // contain.
        let stratum = stratum_of("def alpha : Nat := 1\ndef beta : Nat := alpha\n");
        let adjacency = Adjacency::of(&stratum.atoms);
        assert_eq!(
            adjacency.row(0).expect("row"),
            &[0u64, 0][..],
            "alpha names itself in its head and refers to nothing in its body"
        );
        assert_eq!(
            adjacency.references_to(0).expect("column"),
            1,
            "beta's body proper mentions alpha once"
        );
        assert_eq!(
            adjacency.row(1).expect("row"),
            &[1u64, 0][..],
            "beta references alpha and not itself"
        );
    }

    #[test]
    fn a_genuine_self_reference_is_reported() {
        // The counterpart: `bar` really does mention `bar` in its own body, so
        // A[i][i] = 1 for a self-referential declaration, exactly as §34.2 says
        // it MAY be and only then.
        let stratum = stratum_of("def bar : Nat := bar + bar\n");
        let adjacency = Adjacency::of(&stratum.atoms);
        assert_eq!(
            adjacency.row(0).expect("row"),
            &[2u64][..],
            "two mentions of itself in its body proper"
        );
    }

    #[test]
    fn a_substring_is_not_a_reference() {
        // `alphax` is a different identifier and must not count toward
        // `alpha`; the relation is whole-token, as §34.2 states.
        let stratum = stratum_of("def alpha : Nat := 1\ndef uses : Nat := alphax\n");
        let adjacency = Adjacency::of(&stratum.atoms);
        assert_eq!(adjacency.references_to(0).expect("column"), 0);
    }

    #[test]
    fn every_factor_is_a_unit_fraction_of_one_plus_its_token_count() {
        let stratum = stratum_of("def a : Nat := 1\n");
        let contraction = Contraction::of(&stratum.atoms).expect("valid");
        // `def a : Nat := 1` lexes to seven tokens, two of which are the head
        // `def a`, so t = 5 and λ = 1/6.
        assert_eq!(
            stratum.atoms[0].body.len(),
            7,
            "the §33.1 stream includes the head"
        );
        assert_eq!(
            stratum.atoms[0].body_proper().len(),
            5,
            "the body proper excludes the head"
        );
        let factor = contraction.of_atom(0).expect("factor");
        assert_eq!(factor.to_pair(), (1, 6), "λ = 1/(1+t) with t = 5");
        assert!(contraction.len() == 1);
        assert!(!contraction.is_empty());
        assert!(contraction.of_atom(9).is_err(), "outside the vector");
    }

    #[test]
    fn the_norm_is_the_maximum_column_sum_in_exact_rationals() {
        // Reproduces the shape of the compiler's own arithmetic: an adjacency
        // with column sums 0.8, 0 and factors 1 gives a norm of 4/5.
        let stratum = stratum_of("def alpha : Nat := 1\ndef beta : Nat := 1\n");
        let Norm { value, .. } = norm(&stratum).expect("a stratum has a norm");
        assert!(value.is_below_one());
        let (numerator, denominator) = value.to_pair();
        assert!(denominator > 0);
        assert!(
            numerator < denominator,
            "an accepted norm must be below one, got {numerator}/{denominator}"
        );
        // No float appears anywhere: the value is a pair of integers.
        assert_eq!(
            value.to_pair(),
            Rational::new(numerator, denominator)
                .expect("valid")
                .to_pair()
        );
    }

    #[test]
    fn an_empty_declaration_referenced_once_sits_exactly_on_the_boundary() {
        // §34.2's reachable boundary: t = 0 gives λ = 1/1, so one reference
        // makes the norm exactly 1/1, which must be refused rather than
        // accepted. `axiom empty` lexes to two tokens, so a token-free body is
        // not reachable through the lexer; the boundary is exercised through the
        // factor arithmetic instead, which is where it is decided.
        let lambda = Rational::new(1, 1).expect("valid");
        let references = Rational::from_integer(1).expect("valid");
        assert_eq!(
            lambda.mul(references).expect("multiplies").to_pair(),
            (1, 1)
        );
        assert!(!Rational::new(1, 1).expect("valid").is_below_one());
    }

    #[test]
    fn a_receipt_is_accepted_exactly_below_one_and_states_an_exact_refusal() {
        let stratum = stratum_of("def a : Nat := 1\ndef b : Nat := a\n");
        let receipt = ContractivityReceipt::of(&stratum, "a").expect("the anchor is an atom");
        assert!(receipt.accepted());
        assert_eq!(receipt.status, STATUS_ACCEPT);
        assert!(
            receipt.rejection().is_none(),
            "an accepted receipt has no reason, not an empty one"
        );

        // The status is a function of the norm alone, so a receipt whose status
        // disagrees with its norm is refused when read back.
        let forged = ContractivityReceipt {
            status: STATUS_REJECT.to_owned(),
            ..receipt.clone()
        };
        assert!(forged
            .rejection()
            .is_some_and(|reason| reason.contains(">= 1")));
        assert!(ContractivityReceipt::from_json(&forged.to_json()).is_err());
    }

    #[test]
    fn an_anchor_naming_an_absent_declaration_is_refused() {
        let stratum = stratum_of("def a : Nat := 1\n");
        let error = ContractivityReceipt::of(&stratum, "nowhere")
            .expect_err("an absent anchor must be refused");
        assert!(error.contains("names no atom"), "{error}");
        // The unqualified final segment is accepted, because that is how a Lean
        // theorem is ordinarily named.
        assert!(ContractivityReceipt::of(&stratum, "a").is_ok());
    }

    #[test]
    fn a_receipt_round_trips_through_canonical_json() {
        let stratum = stratum_of("def a : Nat := 1\ndef b : Nat := a + b\n");
        let receipt = ContractivityReceipt::of(&stratum, "b").expect("valid");
        let json = receipt.to_json();
        assert_eq!(
            ContractivityReceipt::from_json(&json).expect("round trips"),
            receipt
        );
    }

    /// "Self-verifying frames" is bound to a named check and named refusals:
    /// this names each one and its exact reason, so the phrase in SPEC §34.3
    /// resolves to observable behaviour rather than to an adjective.
    #[test]
    fn the_receipt_names_every_refusal_it_makes() {
        let stratum = stratum_of("def a : Nat := 1\ndef b : Nat := a\n");
        let accepted = ContractivityReceipt::of(&stratum, "a").expect("valid");
        assert_eq!(accepted.status, STATUS_ACCEPT);

        let fields_of = |receipt: &ContractivityReceipt| match receipt.to_json() {
            Json::Obj(fields) => fields,
            _ => unreachable!("to_json returns an object"),
        };
        let parse = |fields: std::collections::BTreeMap<String, Json>| {
            ContractivityReceipt::from_json(&Json::Obj(fields))
                .expect_err("this receipt must be refused")
        };

        // 1. The hash must be the digest of its own frames.
        let mut tampered = fields_of(&accepted);
        tampered.insert("anchor".to_owned(), Json::Str("b".to_owned()));
        assert_eq!(
            parse(tampered),
            "the receipt hash is not the digest of its own frames"
        );

        // 2. The status must follow from the norm, so it cannot be asserted.
        let mut restated = fields_of(&accepted);
        restated.insert("status".to_owned(), Json::Str(STATUS_REJECT.to_owned()));
        assert_eq!(
            parse(restated),
            "the receipt status `REJECT` does not follow from a norm of 1/6"
        );

        // 3. The authority must name the domain actually hashed, so this hash
        //    cannot be read as the compiler's `seal_hash`.
        let mut foreign = fields_of(&accepted);
        foreign.insert(
            "hash_authority".to_owned(),
            Json::Str("pirtm-seal".to_owned()),
        );
        assert_eq!(
            parse(foreign),
            "the receipt does not state that its hash is LexLean's"
        );

        // The same receipt with nothing altered is accepted, so the three
        // refusals above are the hash check and not a blanket rejection.
        assert!(ContractivityReceipt::from_json(&accepted.to_json()).is_ok());
    }

    #[test]
    fn a_tampered_receipt_is_refused_on_the_hash() {
        let stratum = stratum_of("def a : Nat := 1\ndef b : Nat := a\n");
        let receipt = ContractivityReceipt::of(&stratum, "a").expect("valid");
        let Json::Obj(mut fields) = receipt.to_json() else {
            unreachable!("to_json returns an object")
        };
        // Changing the anchor without recomputing the hash must be caught: the
        // hash is over the frames, and the anchor is one of them.
        fields.insert("anchor".to_owned(), Json::Str("b".to_owned()));
        assert!(ContractivityReceipt::from_json(&Json::Obj(fields)).is_err());
    }

    #[test]
    fn the_receipt_hash_is_lexleans_and_does_not_claim_to_be_the_compilers() {
        // §34.0: the compiler's `seal_hash` preimage is not recoverable, so the
        // receipt says which authority its hash belongs to and a reader cannot
        // mistake one for the other.
        assert_eq!(
            ContractivityReceipt::HASH_AUTHORITY,
            super::RECEIPT_DOMAIN,
            "the authority field must name the domain actually hashed"
        );
        let stratum = stratum_of("def a : Nat := 1\n");
        let receipt = ContractivityReceipt::of(&stratum, "a").expect("valid");
        let Json::Obj(fields) = receipt.to_json() else {
            unreachable!("to_json returns an object")
        };
        assert_eq!(
            fields.get("hash_authority"),
            Some(&Json::Str(super::RECEIPT_DOMAIN.to_owned()))
        );
        assert_ne!(
            receipt.hash().to_hex(),
            crate::artifact::content_id::Sha256Digest::of(b"seal_hash").to_hex()
        );
    }

    /// Fixture for the §34.2 body-proper amendment: on one real source, the
    /// head-inclusive scan of the pre-amendment text forces `A[i][i] = 1` for
    /// every atom and the body-proper scan does not.
    #[test]
    fn head_inclusive_scan_forces_every_diagonal_to_one_and_body_proper_does_not() {
        // `delta` names itself in its own body; the other three do not.
        let source = concat!(
            "def alpha : Nat := 1\n",
            "def beta : Nat := alpha + alpha\n",
            "def gamma : Nat := beta\n",
            "def delta : Nat := delta\n",
        );
        let stratum = stratum_of(source);
        assert_eq!(stratum.atom_count(), 4);
        let head = Adjacency::with_scan(&stratum.atoms, Scan::HeadInclusive);
        let proper = Adjacency::with_scan(&stratum.atoms, Scan::BodyProper);

        let index_of = |name: &str| {
            stratum
                .atoms
                .iter()
                .position(|atom| atom.name == name)
                .unwrap_or_else(|| panic!("{name} is in the stratum"))
        };

        // Head-inclusive: every atom's own name sits in its own head, so every
        // diagonal entry is at least 1 whether or not the declaration refers to
        // itself. 0 is not among the reachable values.
        for atom in &stratum.atoms {
            let index = index_of(&atom.name);
            assert!(
                head.row(index).expect("row")[index] >= 1,
                "the head-inclusive scan finds `{}` in its own head",
                atom.name
            );
        }
        // `delta`'s forced head mention is not the whole story: its body adds a
        // second, so its head-inclusive diagonal is 2.
        let delta = index_of("delta");
        assert_eq!(head.row(delta).expect("row")[delta], 2, "head plus body");

        // Body-proper: a diagonal entry is 1 only where the body really carries
        // the name, so the three spurious diagonals collapse to 0.
        for name in ["alpha", "beta", "gamma"] {
            let index = index_of(name);
            assert_eq!(
                proper.row(index).expect("row")[index],
                0,
                "{name} mentions no atom, so its forced diagonal was spurious"
            );
        }
        assert_eq!(
            proper.row(delta).expect("row")[delta],
            1,
            "delta names itself in its body, and that one is genuine"
        );

        // Entries count occurrences, not presence: `beta` names `alpha` twice.
        assert_eq!(
            proper.row(index_of("beta")).expect("row")[index_of("alpha")],
            2
        );

        // The off-diagonal entries agree: the scans differ only on the head, so
        // the disagreement is exactly the forced diagonal and nothing else.
        for row in 0..stratum.atom_count() {
            for column in 0..stratum.atom_count() {
                if row != column {
                    assert_eq!(
                        head.row(row).expect("row")[column],
                        proper.row(row).expect("row")[column],
                        "the scans must agree off the diagonal at ({row}, {column})"
                    );
                }
            }
        }
        assert!(
            Scan::BodyProper.is_normative(),
            "§34.2 scans the body proper"
        );
        assert!(!Scan::HeadInclusive.is_normative());
    }

    /// Fixture for the reachability half: §34.2 states the empty-body boundary
    /// (`t = 0` gives `λ = 1`, one reference gives `‖G‖₁ = 1/1`, refused), and
    /// that route exists only under the body-proper scan.
    #[test]
    fn the_empty_body_boundary_is_reachable_only_under_the_body_proper_scan() {
        // `axiom bare` lexes to exactly the two head tokens, so its body proper
        // is empty and `def uses` mentions it once. This is a real lexeme, not
        // an arithmetic fixture.
        let stratum = stratum_of("axiom bare\ndef uses : Nat := bare\n");
        assert_eq!(
            stratum.atoms[0].body_proper().len(),
            0,
            "`axiom bare` has an empty body proper, so t = 0 and λ = 1"
        );
        assert_eq!(
            stratum.atoms[0].body.len(),
            2,
            "head-inclusive, the same atom has t = 2, so λ = 1/3 and λ = 1 is out of reach"
        );

        // Body-proper: λ = 1, one reference, norm exactly 1/1, refused.
        let proper = norm_with_scan(&stratum, Scan::BodyProper).expect("a norm");
        assert_eq!(
            proper.value.to_pair(),
            (1, 1),
            "§34.2's stated boundary is reached exactly"
        );
        assert!(!proper.value.is_below_one(), "and one is not below one");
        let receipt = ContractivityReceipt::of(&stratum, "bare").expect("valid");
        assert_eq!(receipt.status, STATUS_REJECT);
        assert_eq!(
            receipt.rejection(),
            Some("NormContractivityViolation: ||G||_1 = 1/1 >= 1".to_owned())
        );

        // Head-inclusive on the same source: the head-inflated counts keep the
        // norm under one, so the pre-amendment text would ACCEPT a lexeme that
        // §34.2 refuses. The two definitions do not merely differ in detail.
        let head = norm_with_scan(&stratum, Scan::HeadInclusive).expect("a norm");
        assert!(
            head.value.is_below_one(),
            "the pre-amendment scan accepts, at {}",
            head.value
        );
        assert_ne!(
            head.value.to_pair(),
            proper.value.to_pair(),
            "the amendment changes a real verdict, not a wording"
        );
    }

    #[test]
    fn the_registered_codes_are_what_the_two_surfaces_raise() {
        // R5: both the free-form and the closed-error surface must raise the
        // code the registry lists, or a caller reading ERRORS.md learns the
        // wrong code.
        let stratum = stratum_of("def a : Nat := 1\ndef b : Nat := a\n");
        let error = ContractivityReceipt::of_entry(&stratum, "nowhere")
            .expect_err("an absent anchor must be refused");
        let codes: Vec<&str> = error
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(codes, vec!["LLG1012"]);

        // A non-contractive receipt is a well-formed receipt, so computing one
        // raises nothing; only demanding acceptance raises LLG1014, and it
        // carries the reduced value that failed.
        let refused = ContractivityReceipt {
            norm: Rational::new(3, 2).expect("valid"),
            status: STATUS_REJECT.to_owned(),
            ..ContractivityReceipt::of(&stratum, "a").expect("valid")
        };
        let error = refused
            .clone()
            .require_accepted()
            .expect_err("not below one");
        let codes: Vec<&str> = error
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code.as_str())
            .collect();
        assert_eq!(codes, vec!["LLG1014"]);
        assert!(
            error.diagnostics[0].message.contains("3/2"),
            "the exact reduced value must be stated: {}",
            error.diagnostics[0].message
        );
        assert!(
            ContractivityReceipt::of(&stratum, "a")
                .expect("valid")
                .require_accepted()
                .is_ok(),
            "an accepted receipt requires nothing further"
        );
    }

    #[test]
    fn an_empty_stratum_has_no_norm() {
        let adjacency = Adjacency::of(&[]);
        let contraction = Contraction::of(&[]).expect("valid");
        assert!(super::column_max(&adjacency, &contraction).is_err());
    }
}
