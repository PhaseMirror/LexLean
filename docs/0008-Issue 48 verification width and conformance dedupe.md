# Session record: issue #48, verification width and conformance deduplication

**Commit:** `8728427` — *Make the verification width operational and stop running conformance twice*
**Date:** 2026-10-03
**Base:** `6b198ee6` (PR #47, *Closed Model, Realization, Contract, and Evidence semantics*)
**Scope:** 13 files, +545 / −26

This document records everything done in the session, including the parts that
did not work, the evidence that was thrown away and retaken, and the work that
is deliberately still open. It is a session record, not a specification:
`SPEC.md` §22.11 is normative, and `VERIFICATION.md` holds the falsifiability
records. Where the two overlap, this file is the narrative and those are the
authority.

---

## 1. Two independent changes

The session landed two changes that are unrelated to each other in mechanism but
share one constraint: **neither may alter any published byte.** LexLean's
verification evidence is content-addressed, so a change that reordered,
omitted, or annotated anything would not be a performance change, it would be a
forgery of evidence. Every design decision below follows from that.

### 1.1 The verification width became operational

`crates/lexlean/src/verify/mod.rs` previously contained a module constant:

```rust
const PROCESS_WIDTH: usize = 1;
```

It is gone. In its place, `crates/lexlean/src/verify/profile.rs` (109 lines) adds:

| Item | Value | Purpose |
| --- | --- | --- |
| `WIDTH_VARIABLE` | `LEXLEAN_VERIFY_OPERATIONAL_WIDTH` | how the host selects a width |
| `MAX_WIDTH` | `64` | the widest width ever honoured |
| `ResourceProfile` | `{ width: usize }` | an operational profile, read once per run |
| `ResourceProfile::from_environment()` | — | parse, validate, clamp, or fall back |
| `ResourceProfile::width()` | — | the value used at the three batching sites |

The name says *operational* deliberately: the value is not part of the
language. The same project verifies to the same bytes at every width.

**Selection rules**, all of which land on the conservative side:

- absent → width 1
- unparsable → width 1
- zero or negative → width 1
- non-integral (`4.0`) → width 1
- empty or whitespace-only → width 1
- above the ceiling → clamped down to 64

The asymmetry is the point. Every rejection path produces a width **at most**
the conservative one, so a mistyped value can only cost wall time and can never
cost correctness or memory headroom in the wrong direction. The ceiling clamps
downward for the same reason.

**Why 64 is a ceiling and not a tuning knob.** No width has ever changed what
is proved, so refusing to grow past 64 costs an operator nothing they would
notice, while an unbounded variable would let a mistyped `999999` ask a runner
to hold 999,999 proof processes. The cap is a guard on *host* memory, not a
statement about what is safe on any particular machine.

**Why the default stays at 1.** SPEC.md §22.11 records the reason: one Atlas
environment already approaches the memory available on a GitHub-hosted runner,
and two overlapping Atlas processes made a hosted runner lose its control-plane
heartbeat while swapping. A width above 1 is selected only where a runner's
memory envelope has been measured to hold it. The repository never selects it on
a project's behalf, because a project cannot know the machine that will verify
it.

`PROCESS_WIDTH` was threaded to the three places that batch proof processes.
Each now reads `profile.width()` from a profile resolved once at the top of the
run, so a single `verify` invocation cannot straddle two different widths.

### 1.2 The conformance suite stopped running twice per gate

The `Justfile` had:

```make
test: cargo test --workspace --all-features      # includes repo-conformance
bdd:  cargo test -p repo-conformance             # the same suite, again
```

`repo-conformance` was therefore a member of the workspace selected by `test`
*and* the whole content of `bdd`. Every conformance test ran twice per `just
vv`, and so did every Lean and `leanchecker` process behind it. Measured on
this host: `test` 2609 s, `bdd` 1806 s.

Now:

```make
test: cargo test --workspace --all-features --exclude repo-conformance
bdd:  cargo test -p repo-conformance
```

`bdd` is the sole owner. `test` retains §28.1 classes 1–2 and 4–5: unit,
property, integration, CLI, and the model crate's own tests.

---

## 2. Registering VR-20

Because a wider profile overlaps proof processes, the executor's collection
order became the load-bearing risk. VR-20 exists to pin it.

Added, in the order `AGENTS.md` §"Adding a capability" requires:

1. `model/ids.toml` — row `VR-20`, level `build`, suite `verification`.
2. `SPEC.md` §31 — matching row, and the §22.11 contract it points to.
3. `features/suites/verification.feature` — scenario tagged `@VR-20 @build`.
4. `crates/conformance/tests/conformance.rs` — `conformance_vr_20`.
5. `crates/conformance/src/cases/verification.rs` — the case body.
6. `CONFORMANCE.md` — regenerated with `just model-write`.
7. `xtask/src/spec_links.rs` — expected capability count 290 → 291.

The statement, verbatim:

> The verification resource profile is operational: the width the host selects
> bounds how many proof processes overlap and reaches no identity, and a project
> verified at the conservative width and at a wider profile publishes
> byte-identical evidence under one attestation ID.

**What `conformance_vr_20` actually does.** It verifies the committed
11-module `semantic-1.1` project twice in one project root — once at width 1,
once at width 4 — and requires one attestation ID over a byte-identical set of
normalized `.json` records and `audit/output.txt`. Runtime: 68.66 s.

**Two anti-vacuity guards.** Both assert the fixture is wide enough for the
comparison to mean anything:

1. Before running, `assert!(modules.len() > wide)` — the fixture must hold more
   modules than the wider profile batches at once. Without it, a project of at
   most one module per batch would run everything sequentially at width 4 and
   the comparison could not distinguish input order from completion order at
   all.
2. After running, `assert!(elaborated > 1)` — the run must actually have
   elaborated more than one module, so the two snapshots cannot both be
   single-process and trivially equal.

Neither guard was anticipated in the first draft of the case; both were added
after noticing the comparison could pass trivially. A gate that cannot fail is
worse than no gate, because it reads as evidence.

`SPEC.md` §22.11 also states the four things width may not change — what runs,
import barriers, order, and evidence — and each has a corresponding
consequence recorded below.

---

## 3. Falsifiability: the plant and what it caught

Per `AGENTS.md` §"Writing a gate", a defect was planted in the batch executor
to confirm the new gates could fail. The planted defect: each worker sent its
result through an `mpsc` channel and the batch collected the channel, so
results arrived in **completion order** instead of **input order**.

Both gates fired:

```
$ cargo test -p lexlean --lib verify::tests
test verify::tests::process_batch_is_concurrent_and_result_order_is_stable ... FAILED
  left: [3, 7]
 right: [7, 3]
```

```
$ cargo test -p repo-conformance --test conformance conformance_vr_20 -- --exact
LLV7004: expected 2 audit records, found 41
```

The second failure is the more informative one, and it is why VR-20 was worth
registering. Reordering does not merely permute a list: it interleaves the
axiom-audit members' `stdout` into a byte sequence the exact-axiom parser can
no longer attribute to a member. The published evidence would have been
*wrong* rather than differently ordered — the attestation ID would still have
been internally consistent, and the damage would have been silent. Restoring
input-order collection passed both gates, and the original order-preserving
unit test passes again.

Profile selection is separately falsifiable without Lean at all: `0`, `-2`,
`two`, `4.0`, and the empty string each select width 1, and `999999` selects
64, asserted on every supported host.

---

## 4. Measurements

All figures are from the development host: 8 CPUs, 30 GiB RAM. They are CI
evidence about a host and reach no artifact, record, or identity.

| Recipe | Before | After | Change |
| --- | --- | --- | --- |
| `test` | 2609 s | 579 s | **−2030 s (−78 %)** |
| `bdd` | 1806 s | 2061 s | +255 s, the added `VR-20` case |
| **`test` + `bdd`** | **4415 s** | **2640 s** | **−1775 s (−40 %)** |
| `fmt-check` | 2 s | 3 s | noise |
| `model` | 33 s | 23 s | noise |
| `spec-links` | 1 s | 0 s | noise |
| `lint` | 19 s | 22 s | noise |
| `features` | 6 s | 19 s | noise |
| `deny` | 1 s | 1 s | noise |
| `golden` | 142 s | 115 s | failed, unrelated — §6 |
| `repro` | 225 s | 222 s | failed, unrelated — §6 |
| `examples` | 10652 s, aborted | not re-run | §7 |

`bdd` rises because it now carries the entire suite *plus* VR-20, which spends
~69 s proving width invariance. The pair together is 40 % cheaper per `vv`.
The pair is the honest figure: the suite still has to run, once.

**Peak aggregate RSS across proof processes**, sampled every 2 s:

| Phase | Peak |
| --- | --- |
| `test` | 1.15 GiB |
| `bdd` | 1.17 GiB |
| `examples` (Atlas) | 18.3 GiB |

A single Atlas `leanchecker` process was observed at **7.1 GB RSS** on its own.
The envelope for a wider profile is therefore roughly `width × 7 GiB`, and this
host offered about 21 GiB available while running other work. Width 2 is
arguably plausible here and width 4 is not. Neither was exercised — see §7.

---

## 5. Evidence that was wrong and had to be retaken

The first before-inventory was captured from the working tree *after* this
change had already been restored. Both sides of the comparison therefore
contained `conformance_vr_20`, the diff came out empty, and the empty diff
looked exactly like the success it was supposed to demonstrate. The comparison
was vacuous — a gate that passes because it compared a thing with itself is not
evidence.

It was retaken properly, in a detached worktree at `HEAD`, so the before-state
was genuinely pristine:

| Set | Count |
| --- | --- |
| `HEAD`, `cargo test --workspace --all-features` | 429 |
| after, `test` phase alone | 124 |
| after, `bdd` phase alone | 306 |
| after, union of the two phases | 430 |
| conformance tests, `HEAD` → after | 290 → 291 |

```
$ comm -23 inv-head.txt inv-after.txt                  # lost by the change
                                                     # (empty)
$ comm -13 inv-head.txt inv-after.txt                  # added by the change
conformance_vr_20
$ comm -12 inv-after-ordinary.txt inv-after-bdd.txt     # run twice
                                                     # (empty)
```

The empty `comm -23` matters most: the deduplication is only permissible if it
is lossless. The empty `comm -12` is what the old layout violated — that
intersection previously held all 290 conformance names.

`VERIFICATION.md` records the failed first attempt as well as the corrected one,
because a reader who finds only the successful comparison cannot tell that the
method was ever at risk.

---

## 6. Pre-existing tree breakage, and how it was attributed

The working tree already contained a large unrelated in-progress change —
a `ReasoningOracle` compiler module, GNAF reasoning fixtures, `Compliance.md`,
`Inventorship.md`, and ~315 modified plus 46 untracked files. None of it was
created by this session and none of it is in `stash@{0}`, which contains only
the seven files this work touched.

Two consequences:

- `golden` and `repro` fail with `LLC0102: lexlean.lock is stale or
  noncanonical; run lexlean lock`.
- Eight conformance cases fail: `gn_01`, `gn_02`, `gn_03`, `gn_07`, `rb_07`,
  `tc_03`, `tc_04`, `tc_05`. Six report the same `LLC0102`; `rb_07` compares
  `compiler/rust/rust-std/adt-evaluation/provenance.json`; `gn_02` reports a
  committed module no longer equalling its generator.

`compiler_semantics` is computed only over `language/`, `schemas/`,
`tests/golden/axiom-parser`, and `tests/golden/canonical-json`
(`crates/lexlean/build.rs`), and none of those four trees is modified. The
committed lock carries the committed data's digest (`5e9c21…`) while the working
tree carries a different one (`392bad…`), written at 09:23 by the other work
and never updated back.

**Attribution was measured, not assumed.** Setting this change aside entirely —
`git stash push` over exactly the seven files, leaving every unrelated
modification untouched — and re-running reproduces the failures identically:

```
$ git stash push -- Justfile SPEC.md crates/conformance/src/cases/verification.rs \
    crates/conformance/tests/conformance.rs crates/lexlean/src/verify/mod.rs \
    features/suites/verification.feature model/ids.toml
$ just golden
gate failed: …/compiler: build: LLC0102: lexlean.lock is stale or noncanonical

$ cargo test -p repo-conformance --test conformance -- conformance_gn_01 \
    conformance_gn_02 conformance_gn_03 conformance_gn_07 conformance_rb_07 \
    conformance_tc_03 conformance_tc_04 conformance_tc_05
test result: FAILED. 0 passed; 8 failed; 0 ignored; 0 measured; 282 filtered out
```

A change that fails only while it is present, and stops failing when it is
removed, is the change's own. These stop failing when it is removed, so they
are not, and they were left unfixed and reported **blocked** rather than
quietly folded into this commit.

---

## 7. What was deliberately not done

**The ≥25 % Atlas measurement was not taken.** Issue #48's headline criterion is
a wall-time reduction at a validated width above 1. It requires a runner with a
measured memory envelope, and none exists here: one Atlas proof process reached
7.1 GB RSS, the aggregate peaked near 18.3 GiB, and the host is 30 GiB with
other load. Eight CPUs is explicitly the wrong reason to widen — SPEC.md §22.11
and the issue both say so. No timing in this document is offered as evidence for
that criterion, and **issue #48 remains open on it.**

**`examples` was aborted, not completed.** The before-run was stopped during
`uor-atlas` after ~45 min. Atlas is unaffected by both changes — deduplication
does not touch that recipe, and the width stays at its conservative default — so
its wall time buys nothing for this work. It is recorded as **aborted**, not as
a pass and not as a failure. A full `just vv` has therefore not been observed
end to end on this change.

**The unrelated in-progress work was not touched.** It was neither committed,
stashed, nor reverted, per instruction. Note that the committed tree still
carries it as uncommitted modifications alongside commit `8728427`; that should
be committed or stashed separately before any branch is pushed.

---

## 8. Three gates that caught this session's own incomplete edit

Worth recording separately, because a suite that cannot fail on the recipe it
pins is not pinning anything. All three fired on the first full `bdd` run and
were fixed to follow the change:

| Case | Assertion | Failure |
| --- | --- | --- |
| `conformance_rp_05` | §9.2: `test` runs exactly the specified command | `left: ["cargo test --workspace --all-features --exclude repo-conformance"]`, `right: [... --all-features]` |
| `conformance_rp_07` | §31 has 290 rows | `left: 291`, `right: 290` |
| `conformance_rp_11` | README states the exact register size | `All 291 registered conformance IDs` vs `All 290 …` |

`rp_11` also required the README capability row's claimed range to move from
`VR-01`..`VR-19` to `VR-01`..`VR-20`, and the prose sentence listing which IDs
run Lean to gain `VR-20`. Those are the register's authority over the documents
that describe it (R1).

---

## 9. Final gate state for commit `8728427`

| Recipe | State |
| --- | --- |
| `fmt-check` | pass |
| `model` | pass — 291 ids, 61 codes, audits clean |
| `spec-links` | pass — 291 table rows bijective with the register |
| `lint` | pass |
| `features` | pass |
| `test` | pass — 579 s |
| `bdd` | the three `rp_*` failures fixed and verified; 8 pre-existing failures remain, attributed in §6 |
| `deny` | pass |
| `golden`, `repro` | blocked — pre-existing, §6 |
| `examples` | aborted, §7 |

`conformance_vr_20` passes in 68.66 s. The plant in §3 was reverted and both
gates are green. `PROCESS_WIDTH` has zero remaining references. The new module
contains no `unsafe`.

---

## 10. Files changed in `8728427`

| File | Change |
| --- | --- |
| `crates/lexlean/src/verify/profile.rs` | **new**, 109 lines — the profile |
| `crates/lexlean/src/verify/mod.rs` | `PROCESS_WIDTH` → per-run profile at 3 sites |
| `crates/conformance/src/cases/verification.rs` | `VR-20` case, +129 |
| `crates/conformance/src/cases/repository.rs` | 3 assertions realigned to the change |
| `crates/conformance/tests/conformance.rs` | `conformance_vr_20` |
| `features/suites/verification.feature` | `VR-20` scenario |
| `model/ids.toml` | `VR-20` row |
| `SPEC.md` | §22.11 contract, §31 row, total 291, +58 |
| `xtask/src/spec_links.rs` | expected count 290 → 291 |
| `Justfile` | `test` excludes `repo-conformance` |
| `README.md` | register size 291, `VR-01`..`VR-20`, width-invariance clause |
| `CONFORMANCE.md` | regenerated by `just model-write` |
| `VERIFICATION.md` | 4 falsifiability records + telemetry, +194 |

`docs/` remains untracked and was not part of the commit.
