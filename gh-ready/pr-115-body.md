**R3 register-first entry for #115 (plan item L3.2 of #106). Draft: the failing test is the point (R3: register row, scenario and failing test before implementation).**

## What this PR is

The mandated first entry of #115 into the normative register:

- **`model/ids.toml` + SPEC.md §31**: new row `PD-08` (suite `production`, level `build`, ref §18.4), statement kept verbatim-faithful to the issue's Item/Accepted-when:
  "A production root's content-addressed artifacts are decoded once at build and appear as data in the lowered program, never decoded per call: the lowered program of a corpus root reading an artifact contains its decoded value as data and no per-call decoding."
- **`features/suites/production.feature`**: scenario `@PD-08 @build`; the `Scenario:` line equals the register statement byte-for-byte (R3, §27.7).
- **`crates/conformance/tests/conformance.rs`**: named test `conformance_pd_08`.
- **CONFORMANCE.md**: regenerated with `cargo xtask validate-model --write` (model-generated, not hand-edited).
- **Counts**: capability total moves 381 → 382 in SPEC.md §31 prose, README, `xtask/src/spec_links.rs` and `cases/repository.rs`.

## Expected state

`conformance_pd_08` fails with `no production case is wired for PD-08` — the R3 failing-first state, identical in shape to the register-first half of #82. CI red on that one test is by design; implementation of the lowering follows in a later PR, which wires the case and records the plant in VERIFICATION.md.

## Validation run locally (Linux x86-64, pinned 4.32.1)

```text
cargo xtask validate-model       → documents current, 382 ids, 80 codes, meta-gate and audits clean (R1)
cargo xtask validate-spec-links  → 382 table rows bijective with the register (RP-07)
cargo fmt --all -- --check       → clean
cargo xtask check-test-inventory → passes
```

No diagnostic code, semantic ID, golden, fixture or lock changes.

## Ordering note

Several open PRs move the §31 count (#130: +1, #82: +13, #146, #144, and the sibling branch for #116). Whichever lands second rebases the four pinned totals; `validate-model`/`validate-spec-links` are the arbiter, never a hand-merged count.

Please also post the claim comment (`claim-115.md`) on #115 before leaving this draft.
