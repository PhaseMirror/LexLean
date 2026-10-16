# Escalation comments — post as comments on #91 and #92

Both lanes are claimed but have no PR and no delivery date. The ask is not to
revoke the claim; it is a named checkpoint so "claimed" stays equivalent to
"an actor with credentials has accepted the next commit" (#106 C6 discipline).
Timeout horizon: **7 days** (to 2026-10-23).

---

## Comment for #91 (stage 2, claimed by @humuhumu33)

Checkpoint request (not a claim on this issue): stage 2 was claimed here on
2026-10-15 with a measured route (c) plan — decoder takes derived rows from
Lean, the scalar items with the 21 machine primitives stated in `RustSemantics`,
and the two `Item` proofs in the core notation — but no PR exists yet, and
stage 3 (the loop construct, which closes #105 step 2) plus stage 4 wait
behind it. Stage 3 is the last dependency of a closed v0.4 bug (#105), so its
delay now has a name outside this issue.

Asking @humuhumu33, by **2026-10-23**, for one of:

1. a draft PR (any state, even red) so the sequence is visible, or
2. a one-line status here ("still on route (c), PR expected <date>"), or
3. a partial reassignment ("the decoder change is ready for someone else;
   the proofs stay with me"), so the stage can be split.

Absent that, the checkpoint asks the maintainers to reassign stage 2 or split
it, and to say on #105 whether #105 step 2 stays blocked behind a reassigned
stage. The goal is that no v0.4 closure item sits behind an undated claim.

---

## Comment for #92 (stdlib lane, claimed by @humuhumu33; effects boundary owned by @afflom)

Checkpoint request (not a claim on this issue): the stdlib lane was claimed
here on 2026-10-15 (member 1: `stdlib/Text` with the seven byte/text roots),
and the effects boundary (§3) is recorded as @afflom's. No PR exists yet for
either half.

Asking, by **2026-10-23**:

- @humuhumu33: a draft PR for member 1 (even register-first / red, as #82
  did), or a dated status line here.
- @afflom: confirm the effects boundary stays yours with a rough position
  (design comment here, or a pointer to where it will land), or reassign it,
  so the lane's two halves both have visible owners.

Absent that, the checkpoint asks the maintainers to reassign or split both
halves so stage 8 does not sit behind two undated lanes. #92 is the last
stage of #27; its owner map is the release map.
