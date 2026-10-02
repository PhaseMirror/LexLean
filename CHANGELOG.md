# Changelog

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and
the version axes are the ones SPEC.md §30.1 separates: the compiler crate and
binary carry the SemVer below, each project selects the supported language
identifier (`1.0`, `1.1`, or `1.2`), and each language's compiler-semantics ID is a
digest over its normative language data, schemas, and pinned golden fixtures.

SPEC.md §2.3 fixes `0.1.0` as the initial implementation version and `1.0.0` as
the first release satisfying the complete specification. No tag before `1.0.0`
is therefore a §30 release: `cargo xtask release-check` reads the complete
§30.3 artifact set and §30.4 completion criterion and refuses, naming every
criterion that does not hold. That refusal is the accurate answer at these
versions, and the entries below say what each tag does and does not claim.

## Unreleased

- Language 1.2 (SPEC.md §17.12): a strict extension of language 1.1 selected
  by `language = "1.2"`, with builtin packages at `1.2.0`, the lock schema
  `lexlean/lock/2`, the semantic-module schema `lexlean/semantic-module/2`,
  and the snapshot envelope `lexlean/semantic-snapshot/2`. Routing is fixed
  by the project language: a `/1` module under 1.2, a `/2` module under 1.1,
  a mismatched lock schema, and a cross-version package or glossary are
  rejected before either backend runs.
- The first 1.2-only construct is the typed, nonrecursive `let` term. It
  lowers to a Lean `let`, and language 1.1 rejects it in linking (`SM-23`).
- The compiler-semantics IDs of languages 1.0 and 1.1 are unchanged: each
  language digests a fixed nested partition of the embedded tree, and every
  committed 1.0 and 1.1 artifact keeps its bytes. Migration from 1.1 to 1.2 is
  explicit and never implicit.
- Language-1.2 recursive data (§17.12): inductives may recurse uniformly and
  strictly positively, directly or nested under `List`, `Option`, `Result`,
  and products; `mutual` labels form contiguous mutual groups; every member
  must have a buildable base case; structures stay nonrecursive. Structural
  recursion uses exactly the direct recursive fields, and induction binds one
  hypothesis per recursive field. Products, pairs, `first`, `second`, and
  `Prod.mk` matches are new (`DF-12`, `DF-13`, `SM-24`, `examples/recursive-data`).
  A standalone recursive definition decreases only over a self-recursive
  inductive, and no 1.2 declaration may take the name of a built-in
  constructor owner (`Bool`, `List`, `Nat`, `Option`, `Prod`, `Result`).
- Language-1.2 binder hygiene (§17.12 rule 10). No binder may be spelled like:
  - a declaration of its module;
  - a built-in name the backend emits;
  - the root of the module prefix.

  The rule covers type parameters, value parameters, and pattern, `let`,
  quantifier, and proof binders. Such a binder would capture the
  unqualified reference in generated Lean after linking had accepted it;
  linking now refuses it. Also new in language 1.2:
  - the canonical LaTeX states the type parameters of parameterized data;
  - every declaration is its own source-map node in both artifacts;
  - type parameters, constructors, and mutual labels are charged to
    `max_ir_nodes`.

  Link-time type diagnostics now spell types as the document does, not as
  internal structures.
  Eight new negative fixtures cover positivity, non-uniformity, an
  uninhabited cycle, a recursive structure, a non-contiguous group, bad type
  arguments, a constructor mismatch, and a forward reference.
- Language-1.2 higher-order code (§17.12): function types, lambdas with exact
  explicit captures, full applications, definition references, and generic
  definitions and theorems with explicit type arguments and no polymorphic
  recursion. `executable` definitions are production-eligible only with
  non-escaping closures and executable callees: no type they state or write
  may hold a function, directly or through a document type's fields, and a
  function parameter is used only where a closure may be. Type parameters
  never capture a backend type, a document type, or a value binder, and the
  universe is never a type argument. Snapshots record a per-definition alpha
  identity (`DF-14`, `DF-15`, `SM-25`, `SM-26`, `examples/higher-order`),
  exposed as `SnapshotSemanticDeclaration::alpha_identity` and
  `SemanticSnapshot::alpha_ids`. Fifteen new negative fixtures cover
  capture, arity, typing, escaping closures (returned, stored in an
  `Option`, and passed in or out of a structure), polymorphic ambiguity and
  recursion, recursion under a lambda, formal callees, a captured type name,
  and a universe type argument.
  Changes after review:
  - Binder hygiene (§17.12 rule 10) covers generic type parameters and
    lambda parameters, so neither the declaration's own name, a sibling
    declaration, nor the module prefix can be captured.
  - A type parameter its declaration never mentions lowers as `(_T : Type)`.
  - The alpha identity numbers binders in evaluation order rather than in
    serialized member order.
  - The 1.2 LaTeX states every parameter with its type and every closure
    with what it binds and captures.
- Language-1.2 recursion (§17.12): mutual definition groups recurse
  structurally over one recursive family (naturals, lists, or an inductive
  group with its containers) and lower with `termination_by structural`;
  well-founded definitions carry a measure and one statement-exact evidence
  theorem per call site, checked in linking and proved under Lean. A
  well-founded call may sit under `if` and `match`: its obligation is
  quantified over the enclosing match binders and hypotheses, and lowers
  through `match (generalizing := false) __decreaseN : s` with the evidence
  applied to `_` for each enclosing binder and substituted by `subst_vars`
  (`examples/recursion` reassociates and prunes a syntax tree by a weight
  measure, the pruning under binders its branches ignore). The new `linear_arithmetic` proof form
  discharges linear obligations and may unfold named prior definitions.
  Recursion that escapes its check (a group member called inside a lambda or
  referenced as a value, a mutual label shared by an inductive and a
  definition group) is rejected; eleven new negative fixtures cover these
  and the decrease and evidence rules (`DF-16`, `DF-17`, `PF-19`, `SM-27`).
  Changes after review:
  - Mutual groups may be well-founded: every member has a measure, and each
    call's obligation compares the callee's measure at the arguments with
    the caller's. `ping` and `pong` in `examples/recursion` call each other
    on the same argument and terminate by their two measures.
  - Every mutual group's call graph must be strongly connected, with every
    member calling into the group.
  - The example adds a structural mutual constant-folding rewriter over
    `Expr` and `Stmt` that returns the rewritten tree.
  - A false evidence theorem stating the exact obligation is refused by
    verification (`recursion-false-evidence`), and SPEC states what linking
    and verification each establish.
  - The negative fixture suite requires every diagnostic of a class to
    carry its prescribed code, and the absolute-path guard no longer
    mistakes an escaped newline after a colon for a Windows drive.
- Language-1.2 ordered collections (§17.12): `map` and `set` types over
  closed ordered key types, canonical map/set/graph literals (reordered
  source links to identical semantic data), insertion/lookup/set-algebra
  operations, ordered folds, bounded iteration, and graph successors,
  reachability, and topological order, all over the fixed
  `LexLeanCollections` runtime, which every module applying a collection
  primitive emits (`SM-28`, `SM-29`, `SM-30`, `DF-18`). Literal keys are
  checked as terms before they are ordered. `examples/collections` proves
  each of the 24 operations under Lean and, for every key type (negative and
  fixed-width integers, Booleans, non-ASCII strings, pairs), that linking's
  canonical order equals Lean's own insertion order. Ten new negative
  fixtures cover duplicate, unordered, non-literal, noncanonical, and
  out-of-range keys, graph references and duplicate edges, fold typing,
  unbounded iteration, and collections under language 1.1.
  Changes after review:
  - A graph's nodes are its keys and every successor, so a successor
    inserted without an entry of its own is a node with no successors;
    reachability and topological order now agree on it.
  - `SM-28` and `SM-30` check every operation, under Lean, against an
    independent `BTreeMap`/`BTreeSet` model on seeded operation sequences,
    including a chain as deep as its node count allows.
  - `SM-29` compares the snapshot and every published artifact that does
    not record source positions.
  - Operation costs are stated, `iterate_until`'s bound is named as fuel,
    and an oversized collection literal is `LLS8002`
    (`collection-literal-limit`).
  - Literal normalization walks the typed module instead of a JSON round
    trip, and the canonical document names map and set types.
- Every semantic-module member must survive into the typed value: an extra
  member of a unit variant (for example `{"kind":"reflexivity","tactic":...}`)
  was silently ignored and is now rejected in every language.
- Language 1.1 definitions now reject a type parameter written inside their
  body: the scope is empty, so such a definition could never elaborate.
- A semantic-module definition parameter, quantifier, `let`, or lambda binder
  that its scope never mentions now lowers as `_name`. Such a module type
  checked and built but failed verification with `LLV7006` on pinned Lean's
  unused-variable warning; only Lean text that never verified changes
  (`SM-08`).
- The public snapshot DTO grows with language 1.2. `SnapshotTerm` gains
  `Let`, `Pair`, `First`, `Second`, `Lambda`, `Apply`, `FunctionRef`,
  `MapLiteral`, `SetLiteral`, and `GraphLiteral` (with `SnapshotMapEntry` and
  `SnapshotEdge`) and call type arguments; `SnapshotType` gains `Product`,
  `Function`, `Map`, and `Set`; `SnapshotPrimitive` gains the collection
  operations; inductives gain `mutual`; definitions gain `type_parameters`,
  `executable`, `mutual`, and `termination` (`SnapshotTermination`);
  theorems gain `type_parameters`; `SnapshotProof` gains `Apply` type
  arguments and `LinearArithmetic`. Downstream exhaustive matches must add
  them.
- New conformance IDs `CF-17`, `CF-18`, `GL-17`, `GL-18`, and `SM-23`; new
  example `examples/language-1.2`, verified with real Lean; ten new negative
  fixtures for version mismatches in both directions, a package of another
  language, and malformed versions.
- Schemas for the later languages, in the 1.2-only partition so the frozen
  1.0 and 1.1 identities are untouched: `project-v2`, `lexicon-v2`, and
  `build-manifest-v2` admit languages 1.1 and 1.2, and `lock-1.1` describes
  the language-1.1 lock. The v1 project, lexicon, manifest, and lock schemas
  had pinned `language` to `1.0`, so every committed 1.1 document violated
  them; every example's documents are now validated against its language's
  schemas.
- `SnapshotSemanticModule::parse` takes the project language, which routes
  the `semanticdata` discriminator; a downstream caller must pass it.

## 0.3.0

- Support exhaustive Boolean matches and keep imported list construction
  definitions exposed for kernel reduction across generated modules.
- Preserve digit runs inside `semanticdata` JSON string values as exact string
  content. Exact content identities beginning with zero no longer trigger the
  standalone LexLean numeral canonicalization diagnostic; JSON numeric tokens
  and ordinary source numerals retain their existing leading-zero rejection.
  Payload tracking is linear in source size and respects whitespace, escaped
  quotes, and control-shaped bytes inside strings.
- Treat Lean comment delimiters inside generated string literals as data while
  retaining the backend invariant that generated Lean syntax contains no line
  or block comments.
- Generic language support for production modeling: generic semantic identifier
  quotation for reserved Lean keywords/segments, full Lean string escape sequence
  support without parser divergence, and bounded semantic elaboration for large byte
  constants.
- Immutable typed authority binding and oracle evidence: Lean 4.32.1, Lake,
  leanchecker, and `#print axioms` bound to upstream Lean FRO release 4.32.1, source commit
  `f054605aea4b840552cca2e725580bffd1e1b704`, and archive SHA-256
  `6dec8667fbf57ba480a18a8b0c353b2ee157346b2630b211ccbefeedf20545f8`, with positive
  and negative oracle execution records.
- First-party package identity and publishing bootstrap: owner-controlled initial token
  upload support in release workflow before OIDC trusted publishing transition, and
  standalone offline package verification (`cargo xtask check-package`).
- Release identity provenance and tree manifest package: `release/release-identity.json`
  and `release/MANIFEST.sha256` binding all 712 packaged files, consumed by PrismPM
  without source-only assumptions.

## 0.2.0

Language 1.1 gains the closed portable data and operation vocabulary required
by generated application runtimes. This is the immutable PrismPM integration
line; it does not weaken the separate LexLean 1.0.0 full-spec release gate.

### Implemented

- Distinct mathematical `Int` and fixed-width signed/unsigned integer types,
  UTF-8 strings, byte sequences, ordering, option, and result values, with
  canonical checked literals and no implicit host-width conversion.
- Twenty-eight generic typed operations covering checked arithmetic and
  conversion, quotient/remainder zero cases, bit operations and bounded
  shifts, collections and byte ranges, UTF-8, byte ordering, bounded exact
  split/join, and canonical decimal parse/format.
- One fixed Lean 4.32.1 runtime lowering and canonical LaTeX rendering for the
  portable vocabulary. Definitions and theorems both carry exact observed
  axiom policies; verification still elaborates, replays with `leanchecker`,
  and audits every policy.
- `SM-17` through `SM-22`, an all-operation semantic fixture, exhaustive
  fixed-width bound/signature tests, schema validation, malformed-input cases,
  and cross-root snapshot identity checks. The register now contains 222
  implemented capability IDs.

### Compatibility

- Language 1.0 syntax and meaning are unchanged. Language 1.1 and
  `lexlean/semantic-snapshot/1` are extended in place under the ecosystem's
  pre-freeze policy; their compiler-semantics digest and affected expected
  artifacts are regenerated.
- No Prism-, Holo-, or Calculator-specific node, raw Lean/Rust field, macro,
  tactic, or backend escape hatch is added.

### Not claimed

- This remains an integration release rather than the complete LexLean §30
  release, which is intentionally reserved for version 1.0.0.

## 0.1.1

The UOR Atlas becomes the foundation model of LexLean: every accepted document
carries the Atlas-derived header, and the Atlas formalization is closed inside
the repository.

### Implemented

- The one-time Atlas conversion is closed and recorded in
  `examples/uor-atlas/MIGRATION.md`; its independently authored Lean source and
  exporter are absent from the release tree. `VR-19` now permanently audits
  the native source graph itself: every source module has one generated Lean
  module, public imports stay within `Init` and the generated graph, the only
  backend-support import is `Lean`, and no second Atlas implementation exists.
  `just vv` verifies the generated native Atlas
  under `leanprover/lean4:v4.32.1`, replays it through `leanchecker` — a
  same-kernel replay, not an independent checker (§22.4) — and runs the
  standing exact axiom gate.
- Two new built-in lexicon packages, `lexlean.std.int@1.0.0` and
  `lexlean.uor.atlas@1.0.0`. The Atlas package is registered under
  `[[builtin_package]]`, locked into every project, and unconditionally
  visible in every document, so the header is carried whether or not a
  document names an Atlas entry; its visibility closes transitively over
  `lexlean.core`, `lexlean.std.nat`, and `lexlean.std.int`.
- The frozen Atlas pack is complete against the native source register: every
  label carries exactly one disposition, the registers key on exact
  identifiers, and every frozen entry refers to a declaration owned by the
  native source rather than importing an independently authored module.
  Coverage begins at `Atlas.lex.tex`; exercise, denotation,
  surface-disjointness, and authority-scope audits run as gates,
  each with a planted-defect record in VERIFICATION.md.
- `examples/uor-atlas/` verifies under the pinned toolchain with its
  committed verification records, and the negative fixture suite grows to 28
  classes with `atlas-level-conflation`, rejected by `LLR3005`: a native Atlas
  document declaration cannot be consumed as an external glossary atom.
- `examples/uor-atlas/src/Atlas.lex.tex` is the single native semantic and
  proof source for 5,519 environment declarations; 58 private source-compiler
  implementation records remain hashed provenance and are emitted by neither
  backend. Both backends traverse that closed DAG, generated Lean publicly
  imports only `Init` and generated Atlas modules, privately imports only the
  generic `Lean` support module, and contains no independently authored Atlas
  implementation.
  `S43` is authoritatively the proved integer-uniqueness statement.
  `SM-15` and `VR-19` bring the register to 211 IDs, all implemented at level
  `build`.
- Language `1.1` adds a closed generic semantic declaration, term, and proof
  language for structures, classes, instances, finite inductives, total
  structural recursion, exhaustive matches, Boolean validators, exact theorem
  application, and axiom-free Boolean reflection. Its seven-module
  `semantic-1.1` fixture contains no handwritten Lean and exercises every
  closed variant through elaboration, `leanchecker`, and exact axiom audit.
- The stable seventh `Engine` operation returns an owned, read-only,
  path-independent `lexlean/semantic-snapshot/1` DTO. The public DTOs and
  complete closed JSON Schema expose every legal semantic module variant while
  keeping mutable compiler internals and both fixed backends private. `SM-16`,
  `DF-11`, `CF-16`, `CL-19`, and `CL-20` bring the register to 216 IDs, all
  implemented at level `build`.

### Changed

- Generated-module verification passes Lean an explicit package root with
  `-R`. This removes random staging paths from `.olean` serialization, so a
  same-platform verification has byte-identical oleans and an identical
  attestation across absolute project roots.
- The compiler-semantics ID moves from
  `fa171c7a2d78cf17e6cb49bbec5c1eed8bee20033472b1953211104068589ba7` to
  `95deb33a8d416d7bf60f02a36e251a71c3ee6f046b7e475d7bae2fc5ddc3767d`:
  the accepted language changed, so §30.1 requires a new ID. Every committed
  lock and verification record is regenerated against it.
- Language `1.1` has its independent compiler-semantics ID
  `0accaf7b80d572e21451d5fa650d92a1a28dae799823931b2f756e448fe89996`;
  language-1.0 locks and generated artifacts remain byte-identical.
- The four 0.1.0 examples still format byte-identically and generate
  byte-identical Lean and LaTeX modules; their source maps and manifests
  differ only in the source and semantic digests those artifacts embed. No
  previously-accepted run changed its generated bytes (§30.2).

### Not claimed

- This is not a §30 release. `cargo xtask release-check` refuses at `0.1.1`,
  and the release criterion is met only at `1.0.0`.
- The native Atlas graph's verification status is a `build` claim: it elaborates,
  replays, and reports no axiom outside Lean's own. The mathematical content
  is the specification's, cited at `some-true`, and the Lean kernel and
  elaborator beneath it are cited, not verified; the honest claim is that the
  Atlas is as sound as Lean 4.32.1, not that it is sound absolutely.
- Verified status is claimed by `verify` alone. `check` and `build` never
  claim it (`VR-18`), and `leanchecker` is a same-kernel replay, never
  described as an independent checker (§22.4).

## 0.1.0

The initial implementation of `LEXLEAN-SPEC-1`.

### Implemented

All 216 conformance IDs of SPEC.md §31 are implemented at honesty level
`build`: constructed in this repository and validated against an oracle by the
test named `conformance_<id>`. [CONFORMANCE.md](CONFORMANCE.md) is the
generated register, [ERRORS.md](ERRORS.md) the closed diagnostic registry, and
[VERIFICATION.md](VERIFICATION.md) the falsifiability record for every gate.

- Closed project configuration, canonical lock file, and offline dependency
  policy (`CF-01`..`CF-16`).
- Total lexical closure over every accepted atom (`LX-01`..`LX-14`) and
  versioned lexicon packages with closed schemas, denotations, and renderer
  tokens (`GL-01`..`GL-16`).
- The fixed structural, mathematical, and proposition grammar with closed
  ambiguity handling (`GR-01`..`GR-16`), the typed closed IR with canonical
  serialization, native modules, and language-1.1 snapshots (`SM-01`..`SM-16`),
  and generic semantic declarations plus document definitions with exact
  self-application and acyclicity rules (`DF-01`..`DF-11`).
- The structured proof language with pinned Lean lowerings (`PF-01`..`PF-18`),
  prose-free deterministic generated Lean with complete token traceability
  (`LN-01`..`LN-12`), and canonical LaTeX regeneration with the optional
  hash-checked PDF provider (`TX-01`..`TX-12`).
- Canonical diagnostics, source maps, coverage, manifests, and reproducible
  builds (`AR-01`..`AR-14`); fifteen-stage verification with `leanchecker`
  replay and exact axiom audit (`VR-01`..`VR-19`); the exact CLI contract and
  the stable seven-method Rust `Engine` API (`CL-01`..`CL-20`); filesystem
  confinement, no shell, no hidden network, and the closed failure model
  (`SE-01`..`SE-12`).
- Six example projects that verify under the pinned `leanprover/lean4:v4.32.1`
  toolchain, and the complete negative fixture suite (`EX-01`..`EX-08`).

### Not claimed

- This is not a §30 release. `cargo xtask release-check` refuses at `0.1.0`,
  and the release criterion is met only at `1.0.0`.
- Verified status is claimed by `verify` alone. `check` and `build` never claim
  it (`VR-18`), and `leanchecker` is a same-kernel replay, never described as
  an independent checker (§22.4).
- Facts about external tools are level `some-true` rows in
  [`model/ledger.toml`](model/ledger.toml): reproduced from cited authorities,
  not established here.
- The normative verification and reproducibility gate runs on Linux x86-64
  (§8.3). The other four supported hosts build the crate and run every test
  that does not need the pinned toolchain or a POSIX shell; each such case
  reports which assertions it did not run.

### Known deviations

The README's "Documented deviations" section lists every place the generated
bytes differ from a literal reading of SPEC.md, with the reason. Each is
enforced by the same golden and conformance gates as everything else.
