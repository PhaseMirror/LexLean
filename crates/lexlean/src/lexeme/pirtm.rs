//! The optional §34 layer carried by an entry (SPEC.md §34.4).
//!
//! This type is the whole of the integration between the PIRTM layer and the
//! base ledger, and it is deliberately small: a snapaddr, a prime index, an
//! optional contractivity receipt, and an optional Zeno-Finton signal. There is
//! no other way for §34 to reach an entry.
//!
//! # Why the layer is not in the signed leaf
//!
//! [`PirtmLayer`] is excluded from the §33.5 leaf bytes, exactly as the
//! inclusion proof is, and for the same reason: it must not be able to change a
//! §33 result. Every field here is *derived* --- the snapaddr and prime index
//! from the entry's canonical sources, the receipt from those and the reference
//! graph, the signal from the leaf index --- so a verifier recomputes each one
//! rather than trusting the recorded copy. Signing a derived value would add no
//! integrity while making the §33 leaf hash depend on whether the layer happened
//! to be attached, which is the substitution this section exists to rule out.
//!
//! The practical consequence is that attaching or detaching this layer leaves
//! the §33 verdict bit-identical, and `LP-12` tests exactly that.

use crate::artifact::canonical_json::Json;
use crate::error::LexLeanError;

use super::contractivity::ContractivityReceipt;
use super::stratum::SnapAddr;
use super::zeno::ZenoFinton;

/// Build a §34 layer error, so every refusal in this module names a code the
/// registry already lists rather than an ad-hoc diagnostic.
fn layer_error(code: crate::diagnostic::DiagnosticCode, reason: impl Into<String>) -> LexLeanError {
    use crate::diagnostic::Diagnostic;
    LexLeanError::from_diagnostic(Diagnostic::new(code, reason.into()))
}

/// The optional §34 stratification layer of one entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PirtmLayer {
    /// The §34.1 snapaddr, which also commits to the prime index and the atom
    /// count, so those two are recomputable rather than trusted.
    pub snapaddr: SnapAddr,
    /// The §34.1 prime index.
    pub prime_index: u64,
    /// The §34.2 receipt, when one was computed for this stratum.
    pub contractivity: Option<ContractivityReceipt>,
    /// The §34.3 signal, when one was recorded. Supplementary and carrying no
    /// legal weight; see [`ZenoFinton`].
    pub zeno_finton: Option<ZenoFinton>,
}

impl PirtmLayer {
    /// The layer for a stratum with no receipt and no signal.
    #[must_use]
    pub fn identity(snapaddr: SnapAddr, prime_index: u64) -> Self {
        Self {
            snapaddr,
            prime_index,
            contractivity: None,
            zeno_finton: None,
        }
    }

    /// Attach a contractivity receipt.
    #[must_use]
    pub fn with_contractivity(mut self, receipt: ContractivityReceipt) -> Self {
        self.contractivity = Some(receipt);
        self
    }

    /// Attach a Zeno-Finton signal.
    #[must_use]
    pub fn with_zeno_finton(mut self, signal: ZenoFinton) -> Self {
        self.zeno_finton = Some(signal);
        self
    }

    /// The layer as canonical JSON.
    #[must_use]
    pub fn to_json(&self) -> Json {
        let mut fields = vec![
            ("prime_index", Json::Int(self.prime_index as i64)),
            ("snapaddr", Json::Str(self.snapaddr.to_hex())),
        ];
        if let Some(receipt) = &self.contractivity {
            fields.push(("contractivity", receipt.to_json()));
        }
        if let Some(signal) = self.zeno_finton {
            fields.push(("zeno_finton", signal.to_json()));
        }
        Json::object(fields)
    }

    /// Read a layer back from an entry field.
    ///
    /// # Errors
    /// Returns [`LLG1011`](crate::code) when a field is absent, of the wrong
    /// type, or inconsistent. Inconsistency is refused rather than repaired:
    /// the snapaddr is a digest, so a recorded one that does not parse is a
    /// corrupt entry, and repairing it would hide the corruption.
    pub fn from_json(value: &Json) -> Result<Self, LexLeanError> {
        let Json::Obj(fields) = value else {
            return Err(layer_error(
                crate::code!("LLG1011"),
                "the `pirtm` layer is not a JSON object",
            ));
        };
        let snapaddr = match fields.get("snapaddr") {
            Some(Json::Str(hex)) => SnapAddr::from_hex(hex).map_err(|reason| {
                layer_error(
                    crate::code!("LLG1011"),
                    format!("the `pirtm.snapaddr` field is malformed: {reason}"),
                )
            })?,
            _ => {
                return Err(layer_error(
                    crate::code!("LLG1011"),
                    "the `pirtm.snapaddr` field is missing or not a string",
                ))
            }
        };
        let prime_index = match fields.get("prime_index") {
            Some(Json::Int(number)) => u64::try_from(*number).map_err(|_| {
                layer_error(
                    crate::code!("LLG1011"),
                    "the `pirtm.prime_index` field is negative",
                )
            })?,
            _ => {
                return Err(layer_error(
                    crate::code!("LLG1011"),
                    "the `pirtm.prime_index` field is missing or not an integer",
                ))
            }
        };
        let contractivity = match fields.get("contractivity") {
            None => None,
            Some(receipt) => Some(ContractivityReceipt::from_json(receipt).map_err(|reason| {
                layer_error(
                    crate::code!("LLG1013"),
                    format!("the `pirtm.contractivity` field is malformed: {reason}"),
                )
            })?),
        };
        let zeno_finton = match fields.get("zeno_finton") {
            None => None,
            Some(signal) => Some(ZenoFinton::from_json(signal).map_err(|reason| {
                layer_error(
                    crate::code!("LLG1011"),
                    format!("the `pirtm.zeno_finton` field is malformed: {reason}"),
                )
            })?),
        };
        let layer = Self {
            snapaddr,
            prime_index,
            contractivity,
            zeno_finton,
        };
        // A recorded receipt must agree with the identity recorded beside it, or
        // the entry would carry two different statements about one stratum.
        if let Some(receipt) = &layer.contractivity {
            if receipt.prime_index != layer.prime_index {
                return Err(layer_error(
                    crate::code!("LLG1011"),
                    "the `pirtm.contractivity` receipt names a different prime index than `pirtm`",
                ));
            }
        }
        Ok(layer)
    }
}
