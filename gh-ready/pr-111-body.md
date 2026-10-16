**Issue #111 slice 1 (plan item L1.3 of #106, decision D1): the register becomes a LexLean-authored module. Additive; the edit-ban and the RP-06/RP-07 switch are slice 2, after the open register work rebases.**

## What this PR is

The first projection slice of #111, following `UOR-Foundation/compression`'s
`src/Uorc/Registry.lex.tex` → `model/ids.toml` pattern:

- **`model/src/Registry.lex.tex`** — a language-1.1 semantic module (the same
  language and data form compression's authority modules use): one
  `ConformanceRow` structure over the four register fields and one record
  definition per conformance ID, bootstrapped once from the committed register.
  The locked compiler accepts it:
  `lexlean check` → `checked 1 module (source 313fab4bd702697de341a7a271eeec87647221ebccd8d49969b4ec4af0f87e42, semantic 4a9a9aca5f379f20612bf7cd8e66776d728721424151e0014ddd37e299904b31)`.
- **`model/lexlean.toml` + `model/lexlean.lock` + a one-word titles
  glossary** — the module's own project, so it locks, checks and formats
  without the repository root becoming a project (entrypoints must sit under a
  source root; `lean_workspace` is project-relative).
- **`repo-model/src/authority.rs`** — projects the module into `Ids`, in the
  shape compression's `crates/model/src/authority.rs` projects its graph:
  exactly one `semanticdata` payload, spec `lexlean/semantic-module/1`, one
  `ConformanceRow` structure, one definition per row, no unprojected
  declarations. `Ids::check` now carries the register rules (§27.5, §27.3,
  §27.8) that `Model::check_ids` previously inlined, so the committed gate and
  the projection gate share one implementation.
- **`codegen::render_ids`** — renders the projection, header included.
- **`cargo xtask validate-registry [--write]`** + `just registry` /
  `registry-write` — projects the register and requires the committed
  `model/ids.toml` to be its byte-identical rendering.

`model/ids.toml` itself changes by two deleted blank lines: the projection
exposed two hand-edit double-blanks (before AU-06 and AU-08); statements,
suites, levels and the 381-row total are untouched.

## What this PR is not (slice 2, after the register branches rebase)

Per #111's Accepted-when, full closure needs the hand-edit ban, the
`@generated` header, and RP-06/RP-07 checking the projection instead of the
kept bijection. None of that lands here, deliberately: seven open PRs
(#146, #82, #150, #151, #153, #155, #156) still hand-edit `model/ids.toml`,
and wiring the ban now would turn every one of them red. `validate-registry`
is therefore **not** part of `validate`, `just vv`, or CI — it is the
pre-projection those branches rebase onto, under the ordering protocol in
`gh-ready/protocol-31-rebase.md`. The second slice flips the header, wires
the task into `validate-model` and CI, and switches the RP-06/RP-07 checks.

## Why data form, why language 1.1

Compression's authority modules are `\semanticdata` semantic modules under
language 1.1, and this PR follows that pattern exactly. #149 will refuse the
data form for language-1.2 product sources at the end of v0.4; the register
module is a language-1.1 tooling artifact of this repository, not a product
source, and the projection gate reads the canonical JSON payload regardless of
surface form, so a later migration to notation costs nothing here.

## Validation run locally (Linux x86-64, pinned 4.32.1)

```text
cargo xtask validate-registry  → 381 ids, byte-identical projection of model/ids.toml
cargo xtask validate-model     → documents current, 381 ids, 80 codes, meta-gate and audits clean (R1)
cargo xtask validate-spec-links → 381 table rows bijective with the register (RP-07)
cargo test -p repo-model       → 14 passed (incl. the_registry_module_projects_ids_toml_byte_for_byte)
cargo test -p xtask            → 11 passed
cargo test -p repo-conformance --lib → 27 passed
cargo clippy --workspace --all-targets --all-features -- -D warnings → clean
cargo fmt --all -- --check     → clean
lexlean check (model/)         → 1 module accepted, hashes above
```

## Known quirk, recorded not hidden

`lexlean fmt` reports `formatted 0 modules` inside the model project while
`check` processes the entrypoint. The check gate is green and the source hash
is stable; the fmt discovery question is deferred to slice 2 rather than
papered over.

## Ordering note

This branch touches `model/ids.toml` by two blank-line deletions only, so a
merge with any open register PR conflicts at most in whitespace; the
reprojecting side runs `just registry-write` and the byte-identical gate
arbiter is `cargo xtask validate-registry`, never a hand-merged file.

Closes nothing by itself: #111 stays open until slice 2 satisfies the
Accepted-when. Please also post the claim comment (`claim-111.md`) on #111
before leaving this draft.
