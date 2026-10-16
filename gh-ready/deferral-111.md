# Comment for #111 — post on the issue

Deferral record with protocol (plan item L1.3 of #106, decision D1 resolved).

#111 turns SPEC.md and `model/*.toml` into generated projections of LexLean
modules, and its acceptance replaces the hand-kept RP-06/RP-07 bijection with
projection checks. Starting it now would make the generator fight every open
register-moving branch at once (#130 +1, #146 +1, #144 +4, #82 +13, and the
#115/#116 register-first drafts +1 each): a moving §31 is the exact thing
#111 exists to eliminate, and writing the generator against it produces churn
in both directions.

Recording, not claiming:

- **State**: deferred, owner unassigned. #93 (the listed dependency) is
  closed, so nothing technical blocks the item; the collision surface does.
- **Reopen trigger**: every count-moving branch above has merged or closed,
  so §31 is settled and the generator is written once against final rows.
- **Protocol meanwhile**: the merge-order/rebase protocol attached as
  `protocol-31-rebase.md` applies to all register edits until this item
  replaces it; its arbiter is `cargo xtask validate-model` +
  `cargo xtask validate-spec-links` after every rebase, never a hand-merged
  total.
- **Asking maintainers**: if you want #111 started earlier anyway, name the
  owner and the collision-resolution order here and the deferral is lifted.

This comment is a deferral record under #106 C6 ("nothing deferred" applies
to code, not to sequencing; the plan itself gates L1.3 and orders items). It
is not a claim on the issue.
