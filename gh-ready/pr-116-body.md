**R3 register-first entry for #116 (plan item L3.3 of #106). Draft: the failing test is the point (R3: register row, scenario and failing test before implementation).**

## What this PR is

The mandated first entry of #116 into the normative register:

- **`model/ids.toml` + SPEC.md §31**: new row `PD-09` (suite `production`, level `build`, refs §17.13, §17.17), statement kept verbatim-faithful to the issue's Item/Accepted-when:
  "The `overflow` effect of a production root is discharged where a kernel-checked bound covers every checked operation: a corpus root with a proven bound is admitted without `overflow`, its rendering has no overflow checks, and removing the bound restores both."
- **`features/suites/production.feature`**: scenario `@PD-09 @build`; the `Scenario:` line equals the register statement byte-for-byte (R3, §27.7).
- **`crates/conformance/tests/conformance.rs`**: named test `conformance_pd_09`.
- **CONFORMANCE.md**: regenerated with `cargo xtask validate-model --write`.
- **Counts**: capability total moves 381 → 382 in SPEC.md §31 prose, README, `xtask/src/spec_links.rs` and `cases/repository.rs`.

## Expected state

`conformance_pd_09` fails with `no production case is wired for PD-09` — the R3 failing-first state. CI red on that one test is by design; the bound discharge (kernel-checked bounds read by eligibility, rendering drops the checks) follows in a later PR that wires the case and records the plant.

## Validation run locally (Linux x86-64, pinned 4.32.1)

```text
cargo xtask validate-model       → documents current, 382 ids, 80 codes, meta-gate and audits clean (R1)
cargo xtask validate-spec-links  → 382 table rows bijective with the register (RP-07)
cargo fmt --all -- --check       → clean
```

No diagnostic code, semantic ID, golden, fixture or lock changes.

## Ordering note

Independent of the sibling branch `issue-115-pd08` (PD-08). Both add one production-suite row; the second to merge rebases the four pinned totals. `validate-model`/`validate-spec-links` are the arbiter.

Please also post the claim comment (`claim-116.md`) on #116 before leaving this draft.
