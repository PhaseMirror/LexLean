# Phase Mirror Axiom Ledger

This ledger tracks all formal proof debts (`AX-*`) and enforcement gaps (`ENF-*`) across the repository. Closing an ADR requires updating this ledger.

## Proof Debts (Axioms)

| ID | Module | Axiom Name | Description | Status | Target Resolution |
|---|---|---|---|---|---|
| AX-UORC-001 | `UorcTheorems.lean` | `thm_uleb128_roundtrip` | ULEB128 encoding is lossless up to `u64::MAX`. | Open | Tie to Rust properties |
| AX-UORC-002 | `UorcTheorems.lean` | `thm_compress_decompress` | Archive payloads reconstruct deterministically. | Open | Tie to Rust properties |
| AX-UORC-003 | `UorcTheorems.lean` | `thm_evaluator_termination` | Bounded iteration guarantees finite O(N) halting. | Open | Extract from formal VM |
| AX-UORC-004 | `UorcTheorems.lean` | `thm_checkpoint_determinism` | Resuming checkpoint yields exact identical state. | Open | Extract from formal VM |

## Enforcement Gaps

None at this time.
| AX-ZENO-001 | `ZenoController.lean` | `zeno_damping_prevents_infinity` | Zeno minimum damping bound. | Open | Expand List/Sum properties |
