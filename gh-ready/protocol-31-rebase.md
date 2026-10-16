# §31 register-collision protocol (draft; for #111 and for every count-moving PR)

Status: proposed 2026-10-16 by the register-first lanes (#115, #116). Applies
until #111 replaces hand-kept bijection with generated projections, at which
point RP-06/RP-07 check the projection and this protocol's merge-order half
becomes obsolete (the rebase half remains).

## The arbiter

`cargo xtask validate-model` and `cargo xtask validate-spec-links` are the
only arbiters of register state. A hand-merged total is a defect. After every
rebase of a count-moving branch:

```sh
cargo xtask validate-model        # R1: model, generated docs, honesty meta-gate, audits
cargo xtask validate-spec-links   # RP-07: §31 rows bijective with model/ids.toml
```

Both must pass before the branch is pushed again. Never resolve a totals
conflict by choosing either side's number; recompute by running the gates.

## What each open branch moves (vs main @ 381, inventory 2026-10-16)

| Branch / PR | Rows added | New total if alone |
| --- | --- | --- |
| pr-130 (#130, PX-13) | 1 | 382 |
| pr-146 (#146, SP-14) | 1 | 382 |
| pr-144 (#144, RT-01..RT-04) | 4 | 385 (branch predates later main) |
| pr-82 (#82, LM-01..LM-13, rebased copy `pr-82-rebased`) | 13 | 394 |
| `issue-115-pd08` (#115, PD-08) | 1 | 382 |
| `issue-116-pd09` (#116, PD-09) | 1 | 382 |

## Merge order (mechanical, not seniority)

1. **Merge in whatever order reviews finish**; do not serialize reviews behind
   this table.
2. **The losing side of each merge rebases**, in this sequence per branch:
   - `git rebase main`
   - resolve `model/ids.toml` by keeping **both** row blocks (insertion points
     are adjacent `[[id]]` blocks; never reorder existing rows — RP-07 is a
     zip over order),
   - resolve SPEC.md §31 by keeping **both** rows and setting the prose total
     to `main's row count + this branch's rows`,
   - resolve the three pinned totals (`README.md`, `xtask/src/spec_links.rs`,
     `cases/repository.rs`) the same way — they are four copies of one number,
   - `cargo xtask validate-model --write` to regenerate `CONFORMANCE.md`
     (never hand-edit it),
   - run both gates, then push.
3. **ID reservations**: PD-08 = #115, PD-09 = #116, SP-14 = #146, PX-13 = #130,
   RT-01..RT-04 = #144, LM-01..LM-13 = #82. A new row takes the next free ID in
   its suite; suites and prefixes are not renamed in a rebase.
4. **Draft register-first PRs (#82, #115, #116) may sit red** on their own
   named tests; they may not sit red on the gates above.

## Adoption

- Recorded for #111 by the deferral comment (`deferral-111.md`): #111's
  projections work starts only after every count-moving branch in the table
  has merged or closed, so the generator is written against a settled §31.
- Owner of the protocol until #111 lands: whoever holds credentials on the
  register-first lanes; the reopen trigger for #111 is the table emptying.
