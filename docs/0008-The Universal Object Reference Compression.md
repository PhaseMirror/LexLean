# UORC — Universal Object Reference Compression

**Document:** `UORC-IMPLEMENTATION-SPEC-001`  
**Specification revision:** `1.0-draft.1`  
**Reference-machine profile:** `uorc/reference-machine/1`  
**Archive format:** `uorc/archive/1`  
**Date:** 2026-10-03  
**Status:** implementation contract; no implementation, proof completion, compression record, or dependency compatibility is asserted by this document.

**Navigation:** [Agent instructions](#0-instructions-to-the-implementing-agent) · [Dependency/bootstrap contract](#2-exact-dependency-baseline-and-bootstrap) · [Repository](#3-repository-ownership-and-layout) · [UOR semantics](#4-uor-native-semantic-contract) · [Reference machine](#6-closed-reference-algebra) · [Wire format](#7-exact-wire-format) · [Synthesis](#11-compression-as-reference-synthesis) · [Certification](#13-certificates-and-verification-receipts) · [Clients](#16-native-client-and-filesystem-behavior) · [External oracles](#21-mandatory-external-validation-oracles) · [Benchmarks](#22-benchmarks-and-the-compression-performance-program) · [Acceptance](#28-release-definition-of-done) · [Exact descriptors](#appendix-a-exact-query-and-identity-descriptors) · [Certificate encoding](#appendix-b-exact-prefix-certificate-payload) · [Wire vectors](#appendix-c-concrete-wire-and-generative-reference-examples) · [Authorities](#appendix-e-authoritative-source-register-for-implementation)

## 0. Instructions to the implementing agent

This document specifies UORC, a lossless compression client whose compressed object is an exact, executable Universal Object Reference. UORC MUST be implemented in LexLean and modeled, built, packaged, and validated through PrismPM, in a repository instantiated from `UOR-Foundation/template`.

This Markdown document is an **external implementation brief**, not an additional repository authority. Do not commit it as `SPEC.md`. Translate its requirements, semantic definitions, contracts, proofs, claim registry, diagnostic registry, application model, and verification plan into the authoritative closed LexLean source graph. A repository specification is the generated canonical presentation of that graph. A generated Markdown projection may be committed only if its complete content is produced from that graph by an admitted generator; otherwise use LexLean's generated LaTeX. Unchanged inherited template policy documents are not UORC application semantics.

Every requirement in this brief is mandatory unless expressly described as an optional user action, an experimental measurement, or an extension outside version 1. Implement all mandatory behavior. Resource exhaustion is a defined result, not permission to omit a capability. An implementation is not complete merely because it builds, passes a small round-trip test, or reproduces the literal fallback.

The intended advance is **global synthesis and sharing of typed generative references**, with exact reconstruction and explicit optimality evidence. It is not a portfolio of existing compression engines with UOR names attached. Novelty and record compression remain empirical research claims until demonstrated; this specification does not assume them.

The uppercase requirement words MUST, MUST NOT, REQUIRED, SHALL, SHOULD, and MAY have their BCP 14 meanings [A19]. Mathematical definitions and explicitly specified observable behavior are normative for implementation. Pseudocode and mathematical type signatures are specifications to translate into accepted LexLean syntax, not permission to insert raw Lean, Rust, JavaScript, Python, SMT, or shell as application authority.

### 0.1 Precedence

Within UORC, the following order applies:

1. The user's implementation-language restriction and the inherited template's universal policy.
2. The immutable imported authorities within their explicitly declared scope.
3. The UORC LexLean semantic definitions and independently stated theorems formalizing this brief.
4. Generated registries, schemas, documentation, packages, and test plans.
5. Observed oracle results and measurements, which cannot alter definitions or prove stronger claims than their evidence supports.

An inconsistency MUST be resolved in the authoritative source and its relevant upstream integration, with reviewed regeneration. Never resolve it with a handwritten backend escape, a silent restriction, a changed expected answer, an ignored test, or an unsupported claim.

## 1. Product, objectives, and claim boundaries

### 1.1 Product

UORC provides:

- a reusable generated compression/decompression library;
- a generated native command-line client for files and ordered byte-object corpora;
- an import-free Core-Wasm computational artifact;
- a Prism/Hologram application with a bounded text transport for inspection and small examples;
- an offline verifier for reconstruction receipts and scoped optimality certificates;
- a reproducible external-oracle and benchmark system.

All product-specific computation, including parsing, serialization, search, accounting, validation, command semantics, test generation, oracle-input generation, and report interpretation, is LexLean-authored. PrismPM supplies the locked, generic compilation, execution, transport, filesystem, packaging, and validation machinery.

### 1.2 Definition

For a sealed machine/profile and a declared decoding context `B`, an archive `a` denotes an ordered sequence of exact byte objects:

\[
\operatorname{Resolve}_{M,E}(B,a)\in\operatorname{Result}(\operatorname{List}(\operatorname{Bytes}),\operatorname{Error}).
\]

`M` fixes reference semantics. `E` fixes the abstract resource envelope. `B` is either empty or an explicitly supplied, immutable basis. The reference is the complete archive, not merely a short identifier whose referent is unavailable to the receiver.

Compression seeks an admitted archive resolving exactly to the input objects `X` while minimizing the declared complete storage objective. The encoder is permitted to discover better references incrementally. The decoder performs no search, network lookup, proof search, or model training.

### 1.3 Separate claims

UORC MUST distinguish:

| Claim | Meaning |
|---|---|
| `reconstruction_checked` | Full byte comparison established that this archive resolved to this input during the identified execution. |
| `lossless_by_theorem` | The kernel-checked encoder-success theorem establishes exact reconstruction in the formal UORC semantics. Deployment retains its declared compiler/runtime trust boundary. |
| `best_found` | The selected archive is the least-cost member among explicitly verified candidates found so far. |
| `locally_normal` | No improving replacement in a specified, completely checked local replacement family remains. |
| `minimum_certified` | A checked certificate establishes minimum size over the entire specified admissible universe. It does not imply identity-complete enumeration of all ties. |
| `canonical_minimum_certified` | Minimum size and the defined lexicographic tie-break are both certified. |
| `frontier_complete` | The entire nondominated set for a separately declared bounded multiobjective request, including the required tie policy, has complete coverage evidence. |
| `benchmark_result` | Measured archive sizes and resource use on exact named inputs under exact rules. |
| `record_validated` | An identified external benchmark authority has accepted the measured record under its own current rules. |

`best_found` is not a global certificate. Local non-adjacency is not global minimality. A minimum description is not a fastest decoder. A hash is not a proof of equality. A finite test corpus is not a universal theorem. These distinctions follow the imported UOR-GNAF authority [A04].

### 1.4 Research objective

The research objective is to outperform authoritative lossless-compression baselines through shorter exact generative references, not by moving data into uncharged dictionaries, model weights, executable code, external services, or proofs.

There are two separate completion decisions:

- **Implementation acceptance:** every mandatory semantic, functional, proof, oracle, integration, security, and reproducibility obligation in this document passes.
- **Compression-performance success:** independently reproducible measurements meet a separately sealed performance target. A world-record claim additionally requires the applicable external authority's acceptance.

The agent MUST deliver and report both decisions separately. An accepted implementation MUST NOT be labeled a record-breaking compressor without record evidence. Failure to beat a comparator MUST NOT be disguised by dropping it, selecting favorable files after measurement, changing accounting, or weakening the target.

## 2. Exact dependency baseline and bootstrap

### 2.1 Observed upstream main revisions

The following main revisions were read while preparing this specification:

| Repository | Observed full commit |
|---|---|
| `UOR-Foundation/template` | `1bea460bac6ea50bae53a7eeb674589d7900e6cb` |
| `UOR-Foundation/LexLean` | `eaa9ab85bb66e21a99a761b6b94fdfd527145ca8` |
| `UOR-Foundation/PrismPM` | `4aba0ca1bcdb4a990e9d2fbb27faff301465f83c` |

These identify the researched baseline, not a floating dependency promise. At implementation bootstrap, resolve each repository's actual `refs/heads/main` exactly once, record its full commit and source digest, and review any differences from this table. Then freeze those revisions. Every subsequent update is explicit and reviewed. A build MUST NOT resolve `main`, `latest`, an unpinned action, or a mutable container tag.

### 2.2 Actual compatibility facts

The observed PrismPM main:

- declares the production application language as LexLean language 1.1;
- embeds LexLean `0.3.0` at `9c1456d34b896ab77db01c5e8ebbfed584edee26`, not the LexLean main listed above;
- uses pinned `lean4-prod` in its current production pipeline;
- supplies generated Cargo, Core-Wasm, View, and Hologram artifacts, with independent Hologram execution checks.

The observed template requires a digest-pinned PrismPM SDK and template binding; its README says its own public SDK locks are not yet present. The template forbids replacing the SDK with an adjacent, vendored, submodule, Git/path, host-tool, or workflow-only implementation [A01–A03].

Therefore, “use latest LexLean and latest PrismPM main” MUST be implemented as a **validated SDK dependency update**, not as two unrelated checkouts presumed compatible.

### 2.3 Mandatory bootstrap procedure

1. Instantiate the repository from the exact resolved template main policy revision. Record provenance without editing universal policy to bypass its checks.
2. Resolve and freeze the requested LexLean and PrismPM mains and their dependency inventories.
3. In the PrismPM dependency/release process, update the embedded LexLean source/package and its identity to the selected main-derived artifact. Preserve existing language 1.1 semantics; do not silently migrate application language identifiers.
4. Run PrismPM's complete acceptance and SDK reproducibility/security gates on the updated dependency closure.
5. Publish an immutable SDK index with exact `linux/amd64` and `linux/arm64` children. Generate `prismpm.lock`, `template.lock`, the devcontainer, standards binding, and shared-action pins using the template/SDK's supported rendering process.
6. Open UORC inside that SDK. Verify actual tool identity against every lock before compiling UORC.
7. Commit a capability/compatibility report, generated from tests rather than a hand-maintained assertion list.

No unpublished digest may be invented. No package version may be reused for different bytes. Publication requiring unavailable credentials is an explicit dependency acceptance failure, not a reason to substitute a host build and report success.

### 2.4 Language choice

The baseline UORC computational representation SHALL be compatible with the intersection of the selected LexLean and PrismPM production subsets. On the observed baseline this means language 1.1, using flat typed tables, finite enums, records, lists, byte operations, and explicit structurally decreasing fuel.

The reference machine in this specification is recursive as a mathematical object, but its implementation need not depend on recursive user-defined types or higher-order values. Encode instructions, blocks, frames, stacks, work queues, and proof traces as first-order records and lists. A flat typed instruction table is structured semantic data, not a raw backend payload.

Language 1.2 may be selected only after the actual locked PrismPM SDK demonstrates support for every used construct and the exact source-language identity. Do not assume LexLean's support alone implies PrismPM export support. There is one authoritative UORC implementation, not separate 1.1 and 1.2 algorithms maintained in parallel.

### 2.5 Capability acceptance before feature implementation

The bootstrap report MUST execute minimal probes for:

- byte sequences including all 256 values, checked integer arithmetic and shifts, and the actual declared SHA-256/JCS implementation boundaries;
- records, finite variants, lists of records, explicit evaluator stack and bounded recursion;
- generated root export and every application result/error boundary;
- arbitrary binary buffer transport for the native library and Core-Wasm;
- native command/file adapters with transactional output and no application-specific handwritten code;
- invocation of immutable validation-only tools through the SDK's generic oracle process interface;
- generated conformance entry points, negative tests, and template register/scenario/test reconciliation;
- instruction-table and proof-trace sizes representative of UORC;
- SDK identity, offline operation, and package regeneration.

Each probe names actual SDK public symbols and schema versions in the generated integration manifest. This document does not invent undocumented PrismPM commands or APIs. Where a generic required transport/export/testing capability is absent, implement and validate that capability **upstream in the appropriate dependency**, regenerate its SDK, and lock it through the reviewed update process. Never add a UORC-specific Rust/Python/JavaScript fallback. That integration work is a mandatory dependency of final UORC acceptance.

### 2.6 Compiler trust boundary

The currently observed PrismPM production use of `lean4-prod` is not the same as LexLean's planned oracle-only role for it. Report the actual pipeline. Do not label that pipeline a completed LexLean-native production compiler or a proved source-to-machine-code compiler.

If the selected upstream revisions have completed the native production bridge, use that bridge and retain `lean4-prod` only as a differential oracle. Otherwise the SDK's existing compiler remains an explicit implementation dependency and trust assumption. UORC's own semantics still reside only in LexLean. The stronger claim “the entire compiler is LexLean-authored and semantics-preserving” remains unavailable until upstream evidence actually discharges it. Do not create a competing compiler in UORC.

## 3. Repository ownership and layout

### 3.1 Source ownership

The following are authoritative product sources:

- closed `*.lex.tex` modules, lexicons, and admitted package descriptors;
- Prism/LexLean application and system values within those modules;
- generated immutable bindings that point to those values;
- inherited template policy and its exact locked infrastructure.

The following MUST NOT contain independently authored UORC behavior: `.rs`, `.lean`, `.py`, `.js`, `.ts`, `.c`, `.cpp`, `.wat`, shell scripts, workflow expressions, handwritten JSON instruction payloads, or prose.

Generated target code is allowed and MUST be manifest-owned, source-bound, and byte-reproducible. Imported external oracle implementations are allowed **only as immutable validation dependencies**, never as UORC production code. Unmodified inherited generic template tooling may remain; extending it with UORC-specific logic is forbidden. Oracle adapters, test decisions, fixture generation, and diagnostic interpretation are themselves LexLean-authored.

### 3.2 Required layout

```text
UORC/
  <template universal policy files, retained and locked>
  prismpm.lock
  template.lock
  standards.lock
  Cargo.toml                       # inherited infrastructure / generated package metadata
  Cargo.lock
  Justfile                         # inherited gate plus declarative invocations of generated gates
  lexlean.toml
  lexlean.lock
  lakefile.toml
  lake-manifest.json
  lean-toolchain
  prismpm.toml
  src/
    Uorc/
      Specification.lex.tex
      Registry.lex.tex
      Types.lex.tex
      Bytes.lex.tex
      Naturals.lex.tex
      Wire.lex.tex
      Syntax.lex.tex
      Typing.lex.tex
      ReferenceSemantics.lex.tex
      StepMachine.lex.tex
      Resources.lex.tex
      Resolve.lex.tex
      Identity.lex.tex
      Basis.lex.tex
      Universe.lex.tex
      Observation.lex.tex
      Equivalence.lex.tex
      Discovery.lex.tex
      AntiUnify.lex.tex
      ChartSynthesis.lex.tex
      Sharing.lex.tex
      Layout.lex.tex
      Envelope.lex.tex
      Search.lex.tex
      Coverage.lex.tex
      Certificate.lex.tex
      Encode.lex.tex
      Receipts.lex.tex
      Streaming.lex.tex
      Protocol.lex.tex
      Commands.lex.tex
      Application.lex.tex
      System.lex.tex
      OracleModel.lex.tex
      OracleRequests.lex.tex
      OracleResponses.lex.tex
      Benchmark.lex.tex
      Tests.lex.tex
      MutationTests.lex.tex
      Acceptance.lex.tex
      Main.lex.tex
  packages/                        # closed LexLean lexicons / project packages
  model/
    ids.toml                       # generated projection of Registry
    authorities.toml               # generated projection of exact imports
    ledger.toml                    # generated typed claim disposition
    dependencies.toml              # generated acquisition bindings, no fabricated hashes
    diagnostics.toml               # generated diagnostic projection
    uor-gnaf.manifest.json          # generated §20-style dependency/claim manifest
    integration.json               # generated SDK capability/symbol bindings
  features/suites/                 # generated one-scenario-per-claim projections
  schemas/                        # generated closed interface and evidence schemas
  authority/
    manifests/                     # immutable source/license/role manifests
    data/                          # redistributed authority vectors where permitted
  fixtures/
    wire/
    resolution/
    synthesis/
    certificates/
    negative/
    transport/
    oracle/
  expected/                       # reviewed generated canonical outputs
  benchmark/
    corpora.lock                  # actual acquired object digests and licenses
    baselines.lock                # exact external binaries/source/settings
    plans/                        # generated sealed benchmark plans
  evidence/                       # content-bound semantic/validation evidence manifests
  .github/workflows/              # template-compatible generic SDK action consumption
  .devcontainer/                  # generated from the exact SDK binding
  .prism/                         # ignored build and verified outputs
  .lexlean/                       # ignored generated workspace artifacts
```

Paths may be split into more modules when needed to keep proof/resource limits finite. A rename must preserve the generated declaration/requirement map. The agent may not omit a module's specified responsibility merely by merging files.

`SPEC.md` is intentionally absent from the source tree. `README.md`, `CONFORMANCE.md`, `ERRORS.md`, and project verification documentation are generated summaries of the authoritative model and collected evidence. Template-owned policy documentation remains unchanged.

### 3.3 PrismPM configuration

Use the actual closed `prismpm/project/1` configuration shape supported by the locked SDK. The observed shape is:

```toml
spec = "prismpm/project/1"
project = "UORC"
lexlean_project = "lexlean.toml"
build_root = ".prism"

[limits]
max_holo_bytes = 16777216
max_entities = 100000
max_diagnostics = 256
```

These govern Prism artifacts, not UORC decompression limits. Increase values only within the real schema after measured need and an explicit model update. They do not authorize changing backends, verifier stages, toolchain pins, or standards.

## 4. UOR-native semantic contract

### 4.1 Required separation

Maintain distinct typed objects for:

1. **Content:** exact finite bytes or an ordered tuple of byte objects.
2. **Reference:** a typed generative description resolving to content.
3. **Semantic equivalence:** exact equality of observations, with an explicit witness or complete comparison.
4. **Realization:** an implementation of reference resolution under the fixed machine.
5. **Wire representation:** exact serialization, including its sharing layout and identifiers.
6. **Query:** a sealed request specifying input, basis, admissibility, resources, objective, and claim.
7. **Evidence:** reconstruction, equivalence, coverage, lower bounds, and execution observations, each with its true scope.

A source-semantic ID identifies the source representation it hashes; it does not decide extensional equivalence of arbitrary generators. A content hash is an indexing/provenance device, not a finite name from which an unknown object can be reconstructed.

### 4.2 Profile, not an invented universal theorem

UORC is a concrete profile of the imported UOR-GNAF admission and accounting discipline. Its first observation carrier is exact byte content. Its generators are ordinary typed programs in the closed reference algebra below. Its contexts include shared definitions, captured values, bases, complete reference layouts, and resource envelopes.

Do not introduce Atlas constants, a fixed finite coordinate space, numerology, or a supposed universal decoder dictionary unless a separately imported authority and a proved exact bridge require them. No such bridge is assumed in version 1. “UOR-native” means the actual typed content/reference/operation/evidence structure and global accounting are load-bearing; it does not mean renaming familiar opcodes or asserting unexplained mathematical compression.

### 4.3 General operators, not codec delegation

The production machine MUST NOT expose `zstd`, `gzip`, `brotli`, `xz`, `paq`, arithmetic coding, neural decoding, a downloaded model, an opaque foreign function, or a custom external decoder as a primitive.

Repetition, symbolic templates, indexed tables, finite recurrences, and noncontiguous shared structures arise from ordinary construction, application, indexing, and bounded iteration. The external conventional compressors in §22 are benchmark comparators only. Version 1 does not append a conventional compressor as an unmodeled final stage.

### 4.4 Semantic quotient and operational envelope

Search state retains both:

- exact equivalence classes of observed content or context-qualified computations; and
- their distinct realizations, sharing options, parameterizations, serialization layouts, resource behavior, and provenance.

Equal observed outputs on sampled arguments do not justify merging functions on every argument. Hash collisions do not justify merging content. A shorter isolated expression does not necessarily dominate a longer expression when the latter supplies a shared definition to several parents.

Pruning requires a sound context-qualified domination theorem. Otherwise retain the alternative or classify discarding it as a heuristic search restriction that prevents an optimality certificate until complete coverage is restored.

## 5. Input domain, context, and limits

### 5.1 Input

The semantic input is `X : List Bytes`, with at least one element. Each byte is in `0..255`. Empty byte objects are valid. The ordered object sequence preserves boundaries and order; it does not interpret encodings, filenames, file permissions, timestamps, or directory structure.

The single-file client supplies a one-element sequence. The corpus client supplies an explicit ordered file list. File names in a corpus manifest are transport metadata, not decoding inputs. If a release offers preservation of paths or filesystem metadata, that is a separately versioned archival model whose encoded metadata is charged; it is not part of this byte-object profile.

### 5.2 Basis

`B = []` is self-contained mode. Conditional mode receives an immutable, already acquired ordered list of byte objects. Its physical basis file is itself a self-contained `uorc/archive/1` archive. Recursive external bases are forbidden in version 1.

`BasisGet(k)` denotes exactly the kth object after that self-contained basis archive has been validated and resolved. A basis identity must resolve from locally supplied bytes. No decoder network access, implicit registry, user home directory, or ambient previous-run cache is permitted.

Content comparison for a basis is exact where a proof requires exact identity. A digest match provides a cryptographic integrity check under its declared assumption, not a theorem that distinct bytes cannot collide.

### 5.3 Two different resource budgets

`DecodeEnvelope` is part of the admissible universe. It has finite bounds for archive bytes, aggregate output bytes, basis bytes, blocks, total instructions, parameters per block, operand-stack frames, abstract steps, and abstract live cells. Every applicable bound is positive; basis bytes may be zero only to prohibit conditional mode. Values use checked unsigned 64-bit representations at the runtime boundary.

`SearchBudget` is an operational request: deterministic search work, discovery work, certificate-checking work, and retained-state limits. It is **not** a definition of which competing archives exist. Increasing or exhausting search budget MUST NOT silently change `UniverseId` or turn unexplored candidates into inadmissible candidates.

Physical RAM/time watchdogs are separate from the abstract envelope. A host timeout, allocation failure, killed process, or unavailable oracle yields an operational failure or incomplete search, not a proof that a candidate violates the formal envelope.

### 5.4 Feasibility

A query is accepted for compression only if its canonical raw archive fits its archive/output limits and the reference raw path fits its abstract decode limits. Implement a proved `rawRequirement(X,B)` and check `DecodeEnvelope >= rawRequirement` componentwise before search.

The encoder therefore always has a feasible exact baseline for every admitted input. It MUST return a typed admission error for a request whose limits cannot accommodate that baseline. It MUST NOT silently reduce the input, truncate output, or change resource profiles.

## 6. Closed reference algebra

### 6.1 Types

The machine has four value types:

| Type code | Type | Carrier |
|---|---|---|
| `00` | `N` | integers from 0 through `2^64 - 1`, used for coordinates/counts |
| `01` | `B` | a byte, 0 through 255 |
| `02` | `P` | Boolean false/true |
| `03` | `S` | a finite byte sequence |

`N` arithmetic is checked. It does not wrap. Byte arithmetic explicitly specified modulo 256 is arithmetic in that finite ring. This is a UORC type/profile definition, not an unsupported claim that these four types exhaust UOR.

### 6.2 Program structure

A graph body is a nonempty ordered table of blocks. A block contains:

- an ordered list of parameter types;
- one declared result type;
- an ordered list of typed instructions;
- a return reference.

Parameters occupy the first local slots. Each instruction appends exactly one result slot. Operands refer only to earlier local slots. A block may call only earlier blocks. The final block is the entry, has no parameters, and returns `S`. Its result is the concatenation of the declared output objects; the frame's exact length vector recovers their boundaries.

There is no external code, dynamic loading, self-call, mutual block recursion, arbitrary jump, implicit state, or host-width behavior. Iteration is expressed only by `Tabulate` and `Fold`. They call an earlier block and have an explicit finite count. Thus the structural call graph is acyclic; the resource-bounded small-step evaluator is total even on malformed or resource-exhausting inputs.

Blocks are generators. Block parameters are typed reference arguments. Local values and earlier blocks are shareable objects. A generator authored in LexLean may enter an archive only through this closed representation and a checked correspondence; its name or source hash alone does not provide executable meaning.

### 6.3 Instruction registry

In the table, `r` denotes a local operand reference; `f` an earlier-block reference; `v` a shortest unsigned LEB128 immediate; `u8` one byte. Exact wire field order is the order shown. Every fixed-arity instruction omits an operand-count field. `captures` is a count followed by that many local references.

| Opcode | Instruction / fields | Result and semantics |
|---|---|---|
| `00` | `NatLiteral v` | `N`, value `v` |
| `01` | `ByteLiteral u8` | `B`, exact byte |
| `02` | `BoolLiteral u8` | `P`; immediate must be 0 or 1 |
| `03` | `BytesLiteral v bytes[v]` | `S`, exact bytes |
| `04` | `NatAdd r r` | `N`; fail on overflow |
| `05` | `NatSubtract r r` | `N`; fail when right > left |
| `06` | `NatMultiply r r` | `N`; fail on overflow |
| `07` | `NatQuotient r r` | `N`, floor quotient; fail on zero divisor |
| `08` | `NatRemainder r r` | `N`; fail on zero divisor |
| `09` | `Equal r r` | `P`; operands same type; exact equality, including full bytes |
| `0a` | `NatLess r r` | `P`, unsigned strict comparison |
| `0b` | `Choose r r r` | first operand `P`; other two same type `T`; result `T` |
| `0c` | `ByteOfNat r` | `B`, argument modulo 256 |
| `0d` | `NatOfByte r` | `N`, exact widening |
| `0e` | `ByteAdd r r` | `B`, sum modulo 256 |
| `0f` | `ByteMultiply r r` | `B`, product modulo 256 |
| `10` | `ByteNegate r` | `B`, additive inverse modulo 256 |
| `11` | `ByteComplement r` | `B`, `255 - b` |
| `12` | `ByteXor r r` | `B`, bitwise exclusive-or |
| `13` | `ByteAnd r r` | `B`, bitwise and |
| `14` | `ByteOr r r` | `B`, bitwise or |
| `15` | `Length r` | `N`, exact length of `S` |
| `16` | `Index r r` | `B`, byte sequence then `N`; fail out of range |
| `17` | `Slice r r r` | `S`, sequence, offset, length; require offset <= size and length <= size-offset |
| `18` | `Construct r r` | `S`, exact concatenation; checked output length |
| `19` | `Singleton r` | `S`, one `B` |
| `1a` | `Apply f v r[v]` | earlier block result; count/types exactly match its parameters |
| `1b` | `Tabulate f r captures` | `S`; `r:N`; body has parameters `(N,captureTypes...)` and result `B` |
| `1c` | `Fold f r r captures` | first `r:N` count; second seed of `T`; body parameters `(N,T,captureTypes...)`, result `T` |
| `1d` | `BasisGet v` | `S`; conditional mode only; exact basis object at index `v` |
| `1e` | `ByteShiftLeft r r` | `B`, byte then `N`; amount <8; result modulo 256 |
| `1f` | `ByteShiftRight r r` | `B`, byte then `N`; amount <8; logical shift |
| `20` | `Bit r r` | `P`, byte then `N`; index <8; least-significant bit has index 0 |

All opcodes `21..ff` are invalid. Unused or future codes MUST be rejected, not ignored. There is no extension blob or escape opcode.

All local instruction operands are already evaluated; `Choose` selects between values and does not suppress errors in earlier instructions. Conditional computation with expensive branches must use explicit typed constructions admitted by this machine; do not pretend this opcode implements lazy evaluation.

### 6.4 Iteration semantics

`Tabulate(f,n,c)` yields bytes `[f(0,c), ..., f(n-1,c)]`. Count 0 yields empty bytes without evaluating `f`. The body signature is still validated.

`Fold(f,n,s,c)` starts with `s_0=s`; for `i=0..n-1`, set `s_(i+1)=f(i,s_i,c)`. Return `s_n`. Count 0 returns the seed. Each invocation and its actual work is accounted. An error propagates; no incomplete value is an accepted result.

There is no implicit access to the original uncompressed input. A body sees only its explicit arguments, earlier-block definitions, and an admitted basis through `BasisGet`.

### 6.5 Completeness and expressivity

Raw mode represents every admitted finite input. Graph mode represents exact finite computations expressible in this profile. Do not claim it represents every total LexLean function, every compression algorithm, arbitrary unbounded programs, or the shortest unrestricted program.

New kinds, richer state, or additional operators require an explicit new machine profile, with complete semantics, wire coding, admission, resource rules, proofs, external validation, and a new identity. No input-specific opaque opcode is allowed. A profile update does not retroactively enlarge the universe certified by an old certificate.

## 7. Exact wire format

### 7.1 Integer coding

`V(n)` is the **shortest unsigned LEB128** encoding of `n`, for `0 <= n <= 2^64-1`. Emit low seven-bit groups, least significant first; set the continuation bit on every nonfinal group. Zero is exactly `00`. Encodings longer than ten bytes, a tenth-byte payload greater than 1, unterminated encodings, overflow, and redundant most-significant zero groups are invalid.

Let `vlen(n)=max(1,ceil(bitLength(n)/7))`. `length(V(n))=vlen(n)` MUST be proved. Decoding and then re-encoding an accepted integer MUST reproduce the original bytes exactly.

WebAssembly's unsigned LEB128 is an authoritative independent reference for numeric values and width restrictions, but WebAssembly permits some nonminimal spellings. UORC intentionally rejects them. Oracle adapters MUST represent that stricter rule instead of treating a permissive WebAssembly decoder as UORC's canonicality oracle [A06].

### 7.2 Frame

All fields are byte-aligned. There are no padding bits, optional fields not listed here, extension sections, comments, timestamps, paths, platform names, or embedded search traces.

```text
magic                4 bytes: 55 4f 52 43                 # "UORC"
format_version       1 byte: 01
machine_profile      1 byte: 01
scope                1 byte: 00=self-contained, 01=conditional
representation       1 byte: 00=raw, 01=graph
object_count         V(k), k >= 1
object_lengths       V(n_0) ... V(n_(k-1))
body_length          V(m)
basis_identity       32 bytes, present iff scope=01
body                 exactly m bytes
content_integrity    exactly 32 bytes
end                  immediate EOF
```

The aggregate length `N=sum(n_i)` is computed with checked arithmetic. It MUST fit the declared envelope. The frame is valid only if resolution produces exactly `N` bytes and splitting at the cumulative lengths yields exactly `k` objects. Zero-length objects are retained, including consecutive empty objects.

The content integrity field is SHA-256 of:

```text
ASCII("UORC-CONTENT-1") || 00 || V(k) ||
  V(n_0) || X_0 || ... || V(n_(k-1)) || X_(k-1)
```

This hash detects ordinary corruption and binds receipts cryptographically. It is not an authentication mechanism and not a proof of collision-free object identity. Exact input comparison and formal reconstruction do not reduce to this field.

`basis_identity` is SHA-256 of:

```text
ASCII("UORC-BASIS-1") || 00 || V(length(basisArchive)) || basisArchive
```

The basis archive MUST have scope `00`. Verify its structure, complete byte length, and content integrity before resolving a conditional archive. No substitution by a different archive with the same decoded objects is implicit: the byte binding is exact. A deliberate content-equivalent replacement requires an explicit context update and a new basis identity.

### 7.3 Raw body

For representation `00`, `m=N`, and the body is `X_0 || ... || X_(k-1)` without transformation. Raw conditional frames are allowed but still bind and charge their supplied basis. The encoder's default literal baseline is self-contained unless the query expressly requests conditional mode.

The exact self-contained raw archive length is:

\[
L_{raw}(X)=8+vlen(k)+\sum_i vlen(n_i)+vlen(N)+N+32.
\]

Conditional mode adds 32 bytes to that expression. This is an expansion bound, not a promise that every input shrinks. A zero-byte one-object input has a 43-byte self-contained frame. The native client reports bytes and ratios without division by zero for empty input.

### 7.4 Graph body

A graph body is:

```text
block_count          V(b), b >= 1
for each block j=0..b-1:
  parameter_count    V(p)
  parameter_types    p one-byte type codes
  result_type        one-byte type code
  instruction_count  V(t)
  instructions       t instructions from §6.3
  return_reference   V(r)
```

The final block is the entry. All field counts and section consumption are exact. There is no implicit block length or alternate parse; instruction arities and counted fields determine boundaries. At end of block table, the body cursor MUST equal `body_length`.

### 7.5 References

Before instruction `i` in a block with `p` parameters, there are `q=p+i` local slots. Operand reference `r` denotes slot `q-r` and must satisfy `1 <= r <= q`. References are unsigned LEB128 positive backward distances, not hash identifiers. Before return, use `q=p+t` with the same rule.

Within block `j`, earlier-block reference `f` denotes block `j-f`, with `1 <= f <= j`. A block's `Apply` argument count must equal the referenced signature's parameter count. For `Tabulate` and `Fold`, the captured argument count must match the signature after their explicitly supplied leading arguments.

Index 0 in `BasisGet` denotes the first basis object. It is not a backward distance.

### 7.6 Canonical syntax versus optimal layout

Every accepted wire spelling is canonical **for its parsed structured representation**: minimal integers, valid type/opcode tags, exact fields, exact counts, no trailing bytes. The theorem `serialize(parse(a))=a` applies to accepted frames.

Do not enforce a globally hash-sorted object order, deduplicate all equal values, or force a specific sharing layout merely to obtain this property. Different legal block/slot layouts can have different reference widths and total byte lengths. They are different candidate archives even when they resolve to equal content. Layout choices belong to search and complete cost accounting.

Syntactic canonicality does not mean there is one unique archive for each decoded object. A canonical minimum is a stronger, separately certified result.

### 7.7 Size function

`wireBytes(a)` is the exact length of `serialize(a)`. Implement a structural size function and prove that it equals serialized byte length for every admitted frame. The size includes every literal, block definition, count, parameter, local reference, basis identifier, integrity field, and header byte.

The primary optimization unit is stored bytes. `wireBits(a)=8*wireBytes(a)`. Do not claim bit-level minima for a non-byte-aligned format that UORC does not implement. Do not substitute node count, AST size, an estimated entropy, the LexLean target calculus's program-size field, or a hash count for wire size.

## 8. Resolution semantics and implementation

### 8.1 Two models

Author two representations in LexLean:

- `ReferenceSemantics`: a clear mathematical evaluation relation over well-typed block tables and values, with exact errors and abstract costs.
- `StepMachine`: an executable finite-frame evaluator, implemented with first-order state and a decreasing budget.

They must not be defined by calling each other. Prove the executable evaluator refines the reference relation, and that a successful reference derivation within the specified resource envelope is reproduced by the executable machine. Shared elementary type definitions are allowed; a second wrapper around the same evaluator is not an independent semantics model.

### 8.2 Evaluation order

Blocks execute their instruction sequence from first to last. Primitive operands are read in listed order. `Apply` creates a new frame, evaluates the earlier block completely, and returns its value. `Tabulate` invokes its body in ascending coordinate order. `Fold` invokes its body in ascending coordinate order with the previous result as explicit state.

Validation of all block signatures, references, and instruction types precedes evaluation. Unreachable blocks are still syntactically/type validated and their bytes are counted. Only executed blocks incur evaluation work. No implementation may execute archive-supplied native code.

Errors are deterministic for the formal evaluator. The first error under this order is returned. The host may interrupt physically; that is a separately reported operational status and does not alter formal error priority.

### 8.3 Value semantics

`N` operations use mathematical integers for the specification and range checks before conversion. The implementation uses the admitted checked operations with an exact refinement theorem. In particular, subtraction underflow and division by zero are not default-zero results.

Byte addition/multiplication/negation explicitly reduce modulo 256. Their implementation MUST widen safely before the reduction; it MUST NOT rely on an accidental overflow policy of Rust, Lean, or Wasm.

Byte equality is length equality plus equality at every index. Interning may use a digest to find candidate matches, but full comparison or an already checked equality witness decides a merge. An injected all-colliding hash implementation MUST not make distinct objects equal.

### 8.4 Resolution result

`resolve` returns exactly one of:

- `Ok(objects, abstractUsage)`;
- `FormatError(...)`;
- `TypeError(...)`;
- `ArithmeticError(...)`;
- `ReferenceError(...)`;
- `IntegrityError(...)`;
- `DecodeLimitExceeded(limitKind, requiredOrObserved)`.

Runtime allocation/IO/timeout/cancellation failures belong to the client/SDK result layer, not the mathematical relation. The API must distinguish them.

### 8.5 Resource model

Define canonical abstract costs independently of implementation wall time. Use this v1 model:

- Each parsing transition consuming a byte charges one parse step. Each record/type/reference validation decision charges one validation step. Validation over an array/list charges once per visited element.
- Each instruction dispatch charges one execution step.
- A local reference lookup charges `r` abstract lookup steps for backward distance `r`. An earlier-block signature lookup charges `f` lookup steps. The reference machine is therefore explicit about its list-addressing representation; an accelerated implementation must simulate this accounting even if physical access is faster.
- Scalar values have weight 1. A byte sequence has weight `1 + length`.
- A primitive additionally charges the sum of operand weights plus result weight. This charge is a specified abstract upper-work model, not a hardware-cycle claim. It deliberately charges complete input/output extents, including equality and slicing.
- `Apply` additionally charges one per argument passed, then all callee work.
- `Tabulate` charges its count, each call's work, and one per output byte installed. `Fold` charges its count, each call's work, and each state replacement's value weight. Count-zero cases still incur dispatch and operand-read costs but no body calls.
- The live-cell count is the sum of the weights of all values retained by active frames, plus one per frame/control record, plus bytes in the parsed archive, basis materialization, and pending output buffers. Parameter passage and stored aliases are charged as distinct logical values even when an implementation shares physical storage. A returned value is counted in the caller after the callee frame is discarded. `Tabulate` counts its growing result, captures, and active body frame simultaneously. `Fold` counts the old state while evaluating the next and counts the new state until replacement completes.

The model MUST specify every transition's step and live-cell changes in a closed table in `Resources.lex.tex`. Appendix D fixes the precise work and live-state elaboration of the rules above; no implicit zero-cost constructor, argument transfer, preloaded basis, or retained decoder state is allowed. Count administrative allocations deterministically, and prove the derived cumulative/peak functions from the table.

Parsing and hashing work belong to end-to-end decoding reports. Where the primitive implementation of SHA-256 comes from the SDK, report a separate deterministic bytes-hashed counter unless a complete abstract step bridge is proved. Do not claim an exact total CPU cost from a partial counter.

The certificate's storage-minimum claim is constrained by the explicitly named abstract envelope, not by a measured CPU/RAM envelope. Report physical wall time, peak RSS, and IO separately. Never relabel abstract cells as physical bytes of peak RAM.

### 8.6 Safe rejection

Validate field lengths before allocation, and validate indices before dereference. Use checked addition/multiplication for every length, offset, budget, and cumulative object boundary. Permit no decoder network, filesystem reads beyond explicitly supplied frame/basis streams, environment access, clock, randomness, callbacks, or foreign function.

A byte stream whose syntax requests astronomical output must fail against the caller's output bound before allocating it. A valid compressed object may inherently have a large expansion ratio; safety is enforced by limits, not a heuristic ban on high ratios.

### 8.7 Publication

Library resolution may use temporary bounded buffers. The native client MUST write to a new temporary output in the destination directory and publish by an SDK-provided atomic operation only after full validation/integrity success. A failure must not overwrite an existing destination or publish partially reconstructed bytes under the requested name.

For standard output, buffer or spool until validation completes before emitting accepted bytes. Explicit streaming of unauthenticated/unvalidated prefixes is not part of the default v1 client. It may be offered only by a separately named API returning provisional chunks and never labeling them verified.

## 9. Identity, evidence, and accounting scopes

### 9.1 Identity domains

Use domain-separated SHA-256 identities with length-prefixed canonical preimages. Every label below includes its ASCII spelling followed by a zero byte:

| Identity | Preimage after domain label |
|---|---|
| `ArchiveId`, `UORC-ARCHIVE-1` | `V(length(a)) || a` |
| `ContentId`, `UORC-CONTENT-1` | `V(k) || V(n_0) || X_0 || ... || V(n_(k-1)) || X_(k-1)` |
| `BasisId`, `UORC-BASIS-1` | `V(length(basisArchive)) || basisArchive` |
| `MachineId`, `UORC-MACHINE-1` | canonical machine/typing/resource/wire profile descriptor |
| `UniverseId`, `UORC-UNIVERSE-1` | canonical admissibility descriptor, scope, basis binding, output-length vector, and maximum length |
| `QueryId`, `UORC-QUERY-1` | `UniverseId`, exact input binding, objective, tie policy, requested claim |
| `SearchStateId`, `UORC-SEARCH-1` | `QueryId`, canonical retained state, cursor and proof coverage |
| `CertificateId`, `UORC-CERTIFICATE-1` | canonical certificate bytes |

For descriptors use the closed canonical serialization declared by `Identity.lex.tex`. JSON evidence uses the restricted JCS profile in §18. Archive and certificate binary identities do not depend on JSON display formatting.

`QueryId` and source/compiler/SDK IDs are evidence identifiers, not required per-node wire references. A compressor may not remove their needed evidence binding merely to report a smaller proof bundle. The archive contains only fields required by §7.

### 9.2 Storage scopes

Every benchmark/result names exactly one accounting scope:

**Self-contained archive:** `wireBytes(a)`, with no input-dependent out-of-band context. The common decoder implementation is fixed and identified separately.

**Standalone executable delivery:** `wireBytes(a) + decoderPackageBytes + requiredRuntimeBytes + allInputDependentSideData`. The platform/runtime baseline is explicit and identical for comparisons. Excluding a required shared library is not self-contained delivery.

**Conditional:** `wireBytes(a | B)` with the exact already-installed basis identified. Also report the non-amortized total `basisArchiveBytes + wireBytes(a | B)` and basis resolution resources.

**Corpus:** one joint archive containing an ordered object tuple and all shared definitions. Charge it once, without dividing away its model cost. A conditional corpus additionally reports the full basis cost.

**Proof-carrying delivery:** add every delivered proof, certificate, query descriptor, and receipt to the chosen underlying scope. Report archive-only and proof-bundle size side by side. An optional proof sidecar may be excluded from an archive-only metric only because decoding does not require it.

No selection among accounting modes is automatic. Conditional results must never be ranked against standalone results under an ambiguous “compression ratio.”

### 9.3 GNAF dependency manifest

Generate a manifest satisfying the imported UOR-GNAF dependency-boundary obligations. It names exact revisions and identities for content kinds, operations, equivalence witnesses, reference-machine semantics, wire profile, cost model, source language, compiler, runtime, proof checker, oracle adapters, and measured comparators.

Classify each dependency as semantic, verification-only, realization-only, or measured. State explicit universe restrictions and the strongest actual claim. Imported UOR prose is an authority source, not a local proof. The current LexLean GNAF scalar step objective and `(steps,size)` objective MUST NOT be silently reinterpreted as archive bytes; UORC supplies its own proved storage-cost profile and an explicit bridge when invoking any upstream GNAF API.

## 10. The complete admissible universe

### 10.1 Sealed descriptor

Before discovery, construct and validate:

```text
UniverseDescriptor {
  machineProfile
  decodeEnvelope
  scope
  basisBinding
  orderedOutputLengths
  maximumArchiveBytes
  allowedRepresentationKinds = {raw, graph}
}
```

For an ordinary compression request, `maximumArchiveBytes` is the exact length of that request's feasible raw frame. Any longer archive cannot improve the primary storage objective, so this ceiling is justified by the literal incumbent, not by the optimizer's generator inventory.

The machine profile admits every well-typed block table and every parameter/literal value allowed by §§5–8 within that length and decode envelope. It is not a list of discovered generators, a hand-picked collection of templates, an observed e-graph, a search beam, or a library chosen after seeing favorable results.

### 10.2 Formal candidate set

Let `L` be the sealed maximum length. Define:

\[
\mathcal U(q)=\{a\in\{0,\ldots,255\}^{\leq L}:\operatorname{Admitted}(q,a)\}.
\]

`Admitted` checks the exact frame/profile/scope/length constraints, canonical syntax, typing, basis binding, and formal resource-envelope termination. It does **not** include equality to the target bytes. That is the distinct eligibility predicate:

\[
\operatorname{Eligible}(q,X,a)\iff a\in\mathcal U(q)\land\operatorname{Resolve}_{M,E}(B,a)=\operatorname{Ok}(X,u).
\]

Do not define a candidate as ineligible because it is unknown to search or because an external evaluator timed out physically.

The complete raw-byte universe is finite. A typed-grammar enumerator is acceptable only with a proved coverage theorem that it contains every admitted archive in this set, including all legal layouts and inline/shared alternatives.

### 10.3 Ordering

For primary optimization use:

\[
\operatorname{key}(a)=(\operatorname{wireBytes}(a),a)
\]

where the second component is unsigned lexicographic order of the full archive bytes. `best_found` chooses the minimum key among found eligible archives. `minimum_certified` proves only its minimum first component. `canonical_minimum_certified` additionally proves no equal-length eligible archive has a smaller second component.

No source name, hash-table iteration order, pointer value, wall clock, host identity, or thread completion order is a tie-breaker.

### 10.4 Minimum theorem

The minimum certificate must establish:

\[
\operatorname{Eligible}(q,X,a^*)\quad\land\quad
\forall a\in\mathcal U(q),\ \operatorname{Eligible}(q,X,a)\Rightarrow|a^*|\leq|a|.
\]

For a canonical minimum replace the conclusion with `key(a*) <= key(a)`.

The witness set is nonempty because of the checked raw baseline. Existence and finite enumeration do not imply practical tractability. Exact certification may take substantial finite work; operational exhaustion returns an honest incomplete status without invalidating an already checked archive.

### 10.5 Local non-adjacency

A rewrite has:

- a typed source context with all captured/sharing dependencies;
- a replacement context;
- exact observational equivalence under explicit preconditions;
- a whole-archive storage comparison;
- an admissibility/resource-preservation obligation.

A result is locally normal only after every rewrite in its named finite family has been checked in every admitted matching context. This is not a certificate for all of `U(q)`. The agent MUST implement and test a case in which a local fixed point is globally suboptimal within a small fully enumerated universe.

### 10.6 Multiobjective requests

Version 1 MUST expose a separate bounded analysis operation over `(wireBytes, abstractDecodeSteps)`; it must not silently change the ordinary compression objective. Componentwise dominance is strict in at least one component. Equal-cost distinct identities are retained when the request asks for an identity-complete frontier; a representative-only request states that restriction.

A pointwise minimum of different candidates' cost coordinates is not a realizable candidate. A whole-corpus optimum is not the concatenation of per-file optima when sharing or framing couples files. Every frontier completeness result names the exact universe and result identity policy. Version 1 implements frontier verification by bounded exhaustive replay of that universe, comparing the complete independently recomputed nondominated result to the supplied ordered candidate set. It does not reuse a scalar prefix certificate to assert a frontier theorem. Budget exhaustion is `frontier_incomplete`; the checked report is a separately versioned `uorc/frontier-receipt/1`, not a `UORP` scalar certificate.

## 11. Compression as reference synthesis

### 11.1 Required state

The encoder maintains an explicit, serializable state:

```text
SearchState {
  query
  immutableInputSnapshot
  literalIncumbent
  verifiedIncumbent
  observedObjects
  equivalenceWitnesses
  generatorCandidates
  parameterizedFamilies
  sharingAlternatives
  completeArchiveCandidates
  contextualEnvelope
  discoveryTasks
  exhaustiveCursor
  coverageState
  consumedWorkForThisInvocation
}
```

The original input is available to encoder discovery and eligibility checking only. It MUST NOT be visible through the reference machine's decoder environment. Search caches contain proposed or checked facts; they are not receiver-side side information.

### 11.2 Mandatory encoder algorithm

The following behavior is required, not an optional sketch:

1. Validate the query and immutable input snapshot. Construct and resolve the canonical raw incumbent. Compare all decoded bytes and boundaries against `X`.
2. Initialize semantic observations with the complete input objects, their lengths, and their byte values. Literal references store exact bytes. Observed span descriptors are encoder-side views until their data or generators are explicitly serialized.
3. Initialize the generic typed grammar enumerator and its coverage cursor independently of discovery.
4. Run a deterministic fair schedule of the discovery engines in §§11.3–11.7 and the exhaustive engine in §12. An individual engine's task can be suspended at a modeled small step so one expensive proposal cannot starve all others.
5. Type-check every proposed generator. Build complete candidate archives, including definitions, parameters, reference layout, framing, and integrity bytes.
6. Resolve each candidate with the reference implementation under the sealed decode envelope. Eligibility is full exact output equality, not hash equality or agreement on a sample.
7. Discard failed proposals from the incumbent set, retaining their reason/provenance as required for diagnostics or coverage. Update the incumbent only when a candidate has a smaller exact `(length,bytes)` key.
8. Update the context-qualified frontier and coverage facts only through checked inference rules.
9. At an operational budget boundary, return the incumbent plus `best_found` or the strongest **actually checked** stronger status. Return an optional resumable checkpoint. Never claim a minimum merely because no more tasks were scheduled.
10. Before publication, independently reparse and resolve the selected serialized bytes, compare them to the immutable input, and generate a receipt binding the actual archive.

Compression MUST NOT execute an external conventional codec, call a language model, or substitute a precomputed answer for a recognized benchmark input.

### 11.3 Generic typed generator enumeration

Generate well-typed block fragments in nondecreasing serialized cost and deterministic structural order. Parameters range over the four admitted types. Constants and byte literals range over the complete finite values permitted by the current length budget. Calls range over earlier blocks, not a named codec registry.

The full enumerator has a coverage proof against the wire universe. A discovery enumerator may prioritize small constants, observed coordinates, observed bytes, and short expressions, but its priority restriction is not a universe restriction. Its incompleteness is compensated only by the exhaustive engine or a complete certificate.

Cache evaluation on explicit argument tuples to accelerate proposal rejection. Such a cache is observational; it cannot prove two blocks equivalent on all inputs. Universal replacement of blocks requires a proved parametric identity.

### 11.4 Address-chart synthesis

An output coordinate is an `N` argument to a byte-valued block. Search derives byte content from coordinates, typed parameters, and explicitly represented shared objects.

Required proposal families are ordinary instances of the grammar:

- compositions of checked coordinate addition, multiplication, quotient and remainder;
- byte-ring expressions over coordinates converted modulo 256;
- indexing into explicitly stored or computed objects;
- piecewise byte-valued expressions using exact predicates and `Choose`;
- nested coordinate decompositions expressed through quotient/remainder and generator parameters;
- `Tabulate` applied to the resulting byte-valued generator.

A chart is eligible only after exact reconstruction over every coordinate in the proposed output extent. Testing selected coordinates is allowed only as an early rejection optimization.

For example, the family

\[
b_i=\operatorname{ByteOfNat}(\lfloor i/w\rfloor+c(i\bmod w)+d)
\]

is an ordinary composed reference. `w,c,d`, the extent, and the generator definition must all be represented and charged. They are not special compression opcodes. Inputs with exceptions can be represented by an explicitly encoded exception object and a more general generator; the exceptions and lookup mechanism are charged in full.

### 11.5 Structural anti-unification

Implement deterministic typed anti-unification over two or more generator expressions:

- equal operator/signature nodes retain their shared shape;
- unequal subterms of the same type may become parameters;
- repeated unequal pairs must reuse the same parameter consistently;
- different types cannot be unified by coercion;
- captured values become explicit call arguments;
- substitution of the recorded arguments into the generalized block must reproduce each source expression's denotation in its stated context.

Prove substitution correctness. Generalization itself is not a size improvement: evaluate the complete archive containing the shared block, argument lists, call sites, and all remaining literals. Retain both shared and unshared alternatives until a valid whole-context comparison eliminates one.

Do not infer a total generator merely because it fits several observed outputs. The generated block already has total error/resource semantics; observed fits establish eligibility only on those observations.

### 11.6 Joint sharing synthesis

Discover reusable byte objects and reusable generator bodies across nonadjacent spans and across input objects. Content equality must be exact. Search can propose:

- one shared literal passed to multiple calls;
- one generalized block used with different parameters;
- shared lower-level parameter objects;
- different materialization choices for the same computed content;
- an explicitly stored basis object in conditional mode;
- whole-corpus compositions where common definitions are paid once.

Do not equate sharing with unconditional deduplication. Introducing a definition and extra references can cost more than repetition of a short literal. Backward-distance widths, block placement, parameter lists, and all frame length-field transitions must participate in the comparison.

### 11.7 Bounded-state synthesis

The encoder MUST use the ordinary `Fold` operator to propose finite-state generators, not just independent-coordinate formulas. State types are the four admitted types. The proposal engine may search byte or natural state updates first, then byte-sequence state within the envelope.

The state update, initial state, captures, iteration count, emitted/selected result, and all intermediate costs are explicit. A recurrence involving prior symbols must encode its state evolution; it may not consult the source input through an encoder-only table during decoding.

### 11.8 Layout optimization

After constructing a semantic candidate, explore legal block orders, local slot orders consistent with dependencies, inlining/materialization, parameterization, and sharing. Each layout is serialized and measured exactly.

An accelerated dynamic program may discard a state only when its retained signature contains every property observable by enclosing contexts: content/type, external references, sharing interfaces, required definitions, placement-sensitive cost, resource use, and side conditions. Prove its contextual dominance rule. A key containing only `(decodedBytes,localSize)` is insufficient for general sharing-sensitive pruning.

For small instances, compare layout optimization against every legal topological layout and both inline/shared alternatives. Include examples where a ULEB128 width boundary reverses a local preference.

### 11.9 Scheduling and repeatability

The default search is deterministic and single logical scheduler. Its priority tuple is `(phaseOrdinal, declaredWorkRound, intrinsicCandidateCost, canonicalTaskBytes)`; the precise phase cycle is generated from a closed registry:

```text
typed-generation -> address-charts -> anti-unification -> sharing/layout ->
state-generators -> complete-enumeration -> repeat
```

Each phase receives the same configured quantum of **modeled small steps**, not a time slice. The schedule advances after the quantum or an empty queue. Resume preserves queue order and cursors. Parallel evaluation is allowed only if acceptance and incumbent updates are committed in that same logical order; otherwise identical budgets could yield different results.

Budget increase or valid resume MUST never worsen the incumbent key for the same query and deterministic search state. A wall-time-limited user cancellation can stop at a nondeterministic point, but its receipt must say so and must not claim byte-identical search outcomes across machines.

## 12. Complete enumeration and certified search

### 12.1 Independent baseline enumerator

Implement a deliberately simple complete enumerator in LexLean, independent of discovery and pruning. Enumerate byte strings by length and unsigned lexicographic order up to the raw ceiling, with a finite resumable cursor. Parse, type-check, resolve, and compare each candidate according to the exact sealed query.

A practical typed/prefix enumerator may replace that execution only with a proved bijection or complete coverage relation. The simple definition remains the specification of the covered universe; it does not have to be rerun literally on every large file.

### 12.2 Prefix space

A prefix `p` denotes every candidate byte string extending `p` within the sealed length ceiling and, for certification, within the region that can improve the incumbent. This space includes the word `p` itself where its length is admitted; split nodes MUST account for that terminal possibility as well as extensions.

Sound prefix rejection reasons are closed and individually proved. Version 1 requires:

- impossible fixed header byte/version/profile/scope;
- invalid or overlong integer encoding that no extension can repair;
- impossible field count/declared section length;
- unknown type/opcode or irrevocably invalid backward reference;
- a type contradiction determined entirely by the consumed prefix;
- an already complete frame whose nonempty extensions are trailing data;
- an unavoidable lower bound on completed archive length above the improvement threshold.

A parser's “need more input” is not a rejection reason. A sampled decoder mismatch cannot reject all completions. A physical timeout cannot reject a prefix. Any stronger pruning rule needs a new registered soundness theorem before it can contribute to coverage.

### 12.3 Lower bounds

The minimum supported lower bound is the number of bytes already consumed plus the minimum required bytes to finish the partially parsed structure, including counts, references, outstanding literal payload, remaining object metadata, and the 32-byte integrity field.

Prove that this bound never exceeds the serialized length of a valid completion. Updating the bound after an unresolved block/signature decision must not assume the optimizer's preferred decision. A heuristic estimated compression ratio is never a certificate lower bound.

Optional additional sound lower bounds may use independently proved structural facts. They cannot use empirical entropy estimates as unconditional per-input lower bounds.

### 12.4 Coverage invariant

At every search step, the set of potentially improving candidates is partitioned into:

1. checked terminal candidates;
2. regions rejected by a sound impossibility rule;
3. regions eliminated by a sound lower bound;
4. unvisited regions.

Coverage complete means category 4 is empty. Every split must preserve exact union and disjointness. A set of visited hashes is not coverage. A changed grammar, basis, decode envelope, length ceiling, or objective invalidates prior coverage unless an explicit checked transfer proves applicability.

### 12.5 Exact request semantics

`compress --certify` requests a minimum certificate. The encoder still runs under an operational budget. If complete evidence is obtained, return `minimum_certified` or `canonical_minimum_certified` as requested. Otherwise return the valid incumbent and `certification_incomplete`, and exit with the distinct requested-certification-not-met code.

This is a fully implemented result branch, not a postponed certification feature. Acceptance requires nontrivial fixtures on which exact certification does complete, fixtures on which it does not, and a proof that no incomplete state can be promoted to a minimum.

### 12.6 Exhaustive enumeration is not the proposed performance advantage

The baseline enumerator establishes a rigorous completeness boundary and supports small exact instances. Practical compression performance must come from typed generalization, address-chart synthesis, global sharing, contextual pruning, and retained state. A release that only enumerates raw bytes or dispatches among a few hand-authored templates does not satisfy §11 or the native-synthesis acceptance requirements.

## 13. Certificates and verification receipts

### 13.1 Certificate semantics

A certificate binds:

- exact query and universe descriptors;
- machine, wire, resource, and cost-profile identities;
- basis bytes/identity and exact output-length vector;
- selected archive identity and its actual bytes supplied to verification;
- claim kind and tie policy;
- reconstruction eligibility;
- complete improving-region coverage or another specifically registered proof route.

The formal predicate is over actual bytes and definitions. Cryptographic digests provide transport bindings; no theorem assumes SHA-256 is injective.

### 13.2 Mandatory certificate routes

Version 1 implements two routes:

**Exhaustive replay.** The verifier independently enumerates the complete potentially improving region and rejects the claim if any eligible improvement exists. It consumes its own explicit verifier budget. Exhaustion yields `certificate_check_incomplete`, never success.

**Prefix coverage.** A finite coverage tree accounts for all candidate words in the improving region. Every node is checked by the proved prefix rules. A split accounts for its terminal word and every next-byte branch that remains in range. Version 1 encodes each such branch explicitly in ascending byte order; there is no compressed range or wildcard proof constructor. No missing or duplicate branch may pass.

These routes are ordinary LexLean code with a generic soundness theorem. An external SMT `unsat` string or an optimizer's “done” flag is not a third proof route.

### 13.3 Portable certificate encoding

Use the following binary envelope; every integer is shortest `V` and every byte blob is `V(length)||bytes`:

```text
magic                   55 4f 52 50                        # "UORP"
version                 01
claim                   00=minimum, 01=canonical-minimum
route                   00=exhaustive-replay, 01=prefix-coverage
query_descriptor        length-prefixed canonical descriptor
chosen_archive_id       32 bytes
chosen_archive_length   V
route_payload           length-prefixed bytes
end                     immediate EOF
```

For route 00 the payload is empty. Route 01 uses exactly the node table in Appendix B. Its four constructors are `RejectPrefix`, `BoundPrefix`, `Terminal`, and `Split`, with fixed numeric tags, fields, and independently recomputed premises. There is no arbitrary reason string, general proof program, imported solver answer, or caller-provided coverage region.

The root covers the entire improving region derived from the selected archive and requested claim. The verifier, not the certificate producer, derives that region. The header descriptor is the exact restricted-JCS query in Appendix A; unknown/missing fields fail. The original input and actual basis/archive bytes are supplied separately and rechecked. Proof nodes, depth, byte length, and verification work have caller-supplied finite limits. Cycle, forward-reference, missing/duplicate-child, uncovered-terminal, and stale-query tests are mandatory.

### 13.4 Formal theorem

Prove, with the actual executable checker on the left:

\[
\operatorname{checkCertificate}(q,X,B,a,c)=\operatorname{Verified}
\Rightarrow\operatorname{Eligible}(q,X,a)\land\operatorname{ClaimHolds}(q,X,a,c).
\]

Do not define `ClaimHolds` as the checker's Boolean result. Its minimum predicate is the independent quantified statement of §10.4. Prove coverage, lower-bound, parsing, and evaluation lemmas needed for this implication.

### 13.5 Kernel-instance evidence versus runtime checking

The generic checker soundness theorem is kernel-verified at build time. Runtime execution of the generated checker is an application of that verified algorithm within the declared compiler/runtime trust boundary; its log is not itself a Lean proof term.

For fixtures labeled `kernel_instance`, generate closed LexLean instance modules binding the exact query/archive/certificate data and prove the checker result through admitted proof-producing reduction. Verify them with real Lean and `leanchecker`. No `native_decide`, `sorry`, `admit`, `sorryAx`, source axioms, raw tactic payloads, or evaluator-success-as-proof substitutions are permitted.

Large runtime certificates may remain runtime-checked unless a kernel instance was actually produced and checked. Reports distinguish those evidence kinds exactly. Sidecar proofs are never needed to execute the decoder, and their delivered size is separately reported.

### 13.6 Reconstruction receipt

Generate a closed canonical receipt with schema `uorc/reconstruction-receipt/1` containing:

- source semantic, compiler, SDK, runtime, machine, and wire identities;
- archive ID, byte length, scope, basis binding, ordered output lengths;
- input ContentId and resolved ContentId;
- `comparison = full-byte-equality`, count of compared bytes and boundaries;
- abstract decode usage and explicit physical-measurement references;
- result/claim kind;
- certificate ID and evidence kind, or an explicit absence;
- no unearned `verified`, `optimal`, `independent`, or `record` flags.

Matching input/resolved digests alone MUST NOT set `comparison` to full equality. Receipts created without the original input may report successful decoding and integrity, but not input byte comparison.

## 14. Resume, retained state, and corpus behavior

### 14.1 Checkpoints

A checkpoint includes exact query bindings, incumbent archive, deterministic task queues, complete-enumerator cursor, equivalence witnesses, and coverage state. Serialize it canonically and hash it separately from the archive. Checkpoint bytes are encoder state, not decoder dependencies.

On resume:

1. revalidate query and input/basis bindings;
2. fully reconstruct the incumbent;
3. check every stored equivalence/pruning witness used as proof;
4. validate queue/cursor coverage and task typing;
5. reject stale or malformed state;
6. continue the same logical schedule.

Unproved discovery hints may be accepted only as hints, rechecked before use. They may not contribute coverage. A malicious checkpoint cannot certify a missing region merely by stating it was visited earlier. A complete coverage proof may be retained and rechecked; otherwise the uncertified region must be visited again.

### 14.2 Corpus objective

`compress_many` jointly synthesizes one graph whose entry returns the concatenated ordered objects. Shared blocks and values are paid once. Per-object length boundaries are charged in the frame. Empty objects and order remain observable.

Do not optimize each file separately and describe the result as the globally optimal corpus representation. Such a composition is an eligible candidate, not a proof of corpus optimality. Include a fixture where a joint shared generator beats the sum of separately selected representations after all framing is counted.

### 14.3 Basis construction

`basis build` accepts an ordered training/object set and creates a self-contained archive. It uses the same synthesis semantics and explicitly reports its full byte size, content list, derivation provenance, and creation cost. No hidden binary or external model is generated.

`compress --basis` treats that exact preexisting basis as supplied context. A test/benchmark plan seals basis bytes before evaluation inputs are made available. A basis intentionally constructed from the evaluation input is allowed only in a fully charged corpus/standalone accounting experiment and must be reported as such.

### 14.4 Incremental state law

For the same sealed query and deterministic schedule, resuming after `w1` modeled work and then consuming `w2` additional work must produce the same semantic search state as uninterrupted execution for `w1+w2`, subject to the same retained-state policy and exact arithmetic range. Prove this law at the state-machine level and test checkpoint round trips and corruption.

Across different queries or changed bases, reuse is a new admission event. There is no implicit global shared dictionary whose cost or provenance disappears.

## 15. Public computational API

These are required **UORC-owned semantic entry points**, not claims that the named functions already exist in PrismPM. Register their actual generated qualified names in the application model and integration manifest.

| Entry | Inputs | Output |
|---|---|---|
| `compress` | ordered byte objects, mode, optional basis archive, decode envelope, search budget, claim request | checked archive, reconstruction receipt, optional certificate/checkpoint, claim disposition |
| `decompress` | archive, optional basis archive, decode envelope | exact ordered objects and abstract usage, or typed error |
| `inspect` | archive, parse limits | exact header, reference-table summaries, static validation disposition; no claimed successful execution unless requested separately |
| `verify` | archive, optional original object tuple, optional basis, optional certificate, separate decode/checker limits | distinct decoding-integrity, input-equality, and certificate dispositions |
| `resume` | checkpoint, original objects, optional basis, additional search budget | same result form as compress |
| `buildBasis` | ordered objects and self-contained compression configuration | self-contained basis archive and its complete accounting receipt |
| `analyzeFrontier` | input tuple, exact universe descriptor, resource budget, identity policy | found/certified frontier with explicit completeness disposition |
| `compare` | two byte-object tuples | exact equality and first differing object/offset, or size mismatch |
| `runAcceptance` | generated suite ID and immutable fixture/oracle bindings | structured test transcript; cannot select away mandatory release tests |
| `runCoreRequest` | bounded transport bytes | bounded transport response bytes, with strict version/error behavior |

`compress` MUST NOT return success before having a checked incumbent. `verify` with no original input MUST NOT claim original-input equality. `inspect` MUST NOT imply decompression succeeded. A certificate-verification budget is independent of the decoder limit.

### 15.1 Result records

`CompressionResult` contains `archive`, `archiveId`, `receipt`, `status`, `certificate?`, `checkpoint?`, and exact integer counters. Its `status` is one of:

- `best_found`;
- `locally_normal` with an exact local-family identity;
- `minimum_certified`;
- `canonical_minimum_certified`;
- `certification_incomplete` with a valid archive and named outstanding coverage;
- `cancelled_with_incumbent` with a valid archive and explicit cancellation origin.

Admission/format/runtime failures are errors, not a success status containing empty bytes. A zero-length input is not confused with an absent result.

`VerificationResult` has separate fields `decoded`, `integrity`, `inputComparison`, `certificate`, and `deploymentEvidence`. Each field is a typed sum, not a Boolean defaulting to true. Missing evidence is `not_requested` or `absent`, not `passed`.

### 15.2 Runtime representations

The first-order runtime encoding of records may use finite variants and ordinary lists. All public lengths and per-invocation budgets are checked U64 values. Mathematical proofs may use `Nat` with exact conversion lemmas. A counter that would overflow fails or uses an explicitly implemented arbitrary-precision representation; it never wraps. Cursors should be structural prefix/task stacks, so completing a search does not depend on cramming the cardinality of an enormous universe into U64.

## 16. Native client and filesystem behavior

### 16.1 Required commands

The generated native executable is named `uorc`. Its command grammar is:

```text
uorc compress INPUT --output ARCHIVE [--basis BASIS]
     [--profile PROFILE] [--work N] [--certify | --certify-canonical]
     [--certificate FILE] [--receipt FILE] [--checkpoint FILE] [--json]

uorc decompress ARCHIVE --output OUTPUT [--basis BASIS]
     [--profile PROFILE] [--json]

uorc inspect ARCHIVE [--json]

uorc verify ARCHIVE [--original INPUT] [--basis BASIS]
     [--certificate FILE] [--profile PROFILE] [--check-work N] [--json]

uorc compress-many MANIFEST --output ARCHIVE [compression options]
uorc decompress-many ARCHIVE --output-directory DIRECTORY [decode options]
uorc resume CHECKPOINT --input MANIFEST --output ARCHIVE [--basis BASIS]
     [--work N] [--certificate FILE] [--receipt FILE] [--checkpoint FILE] [--json]
uorc basis build MANIFEST --output BASIS [--profile PROFILE] [--work N] [--json]
uorc frontier MANIFEST --query QUERY --output REPORT [--work N] [--json]
uorc benchmark PLAN --output-directory DIRECTORY
uorc selftest [--json]
uorc version [--json]
```

`INPUT=-` and `ARCHIVE=-` mean standard input only where one stream is unambiguous. `--output -` means standard output for the artifact/decoded bytes; diagnostics and status then go to standard error. `--json` cannot mix structured status with binary artifact bytes on the same stream. Unknown, duplicate, conflicting, or missing arguments fail closed. Decimal numbers are canonical unsigned base ten, not locale-dependent or floating point.

`MANIFEST` is the closed canonical JSON schema `uorc/input-manifest/1`, containing `spec`, an ordered nonempty `objects` array of `{path}`, and nothing else. Resolve relative paths against the manifest directory. Reject duplicate physical input identities only when requested by an explicit policy; repeated identical objects are otherwise legitimate corpus inputs and must be preserved. Symlinks are rejected by default, and path traversal outside the selected input root requires explicit SDK capability authorization rather than implicit acceptance.

The corpus does not encode source filenames. `decompress-many` stages the complete object set in a new temporary directory and atomically publishes the requested destination directory only after every object and the full archive validate. It creates `object-000000.bin`, `object-000001.bin`, etc., in a new destination directory, with widths of at least six decimal digits. The returned receipt preserves the object order. There is no archive-supplied path to execute or extract.

### 16.2 Defaults

Define and generate these named policy profiles in LexLean; they do not change wire semantics:

| Bound | `native-safe-1` | `view-small-1` |
|---|---:|---:|
| Maximum archive bytes | `2147483648` | `32768` |
| Maximum aggregate output bytes | `4294967295` | `65536` |
| Maximum supplied basis archive bytes | `2147483648` | `0` (conditional mode unsupported by this transport) |
| Maximum blocks | `65535` | `256` |
| Maximum instructions | `1048576` | `4096` |
| Maximum parameters in one block | `64` | `16` |
| Maximum active frames | `4096` | `128` |
| Maximum abstract steps | `281474976710656` | `16777216` |
| Maximum abstract live cells | `17179869184` | `4194304` |

A zero basis maximum denotes that the profile authorizes only self-contained mode; other applicable limits must be positive. These are semantic safety ceilings, not RAM availability or throughput promises. Physical SDK limits may be smaller and their failures must be reported explicitly.

Default deterministic search work is `1000000` modeled search steps for the native client and `65536` for the View. The default phase quantum is `256`. These defaults are part of the search-policy identity, not the decoding universe definition. An explicit `--work` changes effort, not archive semantics. Profiles and larger limits may be supplied only by closed validated descriptors, not code.

At bootstrap, exercise these profiles through the real exporters and transports. If a runtime cannot represent a bound, reject that target/profile combination explicitly; do not truncate a U64 into a U32 or advertise support. Native production acceptance includes handling the full locked benchmark inputs within declared, provisioned physical resources.

### 16.3 Exit codes

| Code | Meaning |
|---|---|
| `0` | requested operation completed; every requested claim was met |
| `2` | command/configuration/input admission error |
| `3` | malformed or invalid archive/certificate syntax, type, or reference |
| `4` | reconstruction/integrity/eligibility failure |
| `5` | requested optimality/certificate verification incomplete; a valid best-found artifact may exist only if explicitly reported |
| `6` | decoder or operational resource limit, allocation, IO, or cancellation failure |
| `7` | dependency/authority/SDK identity failure |
| `8` | external oracle disagreement or acceptance failure |
| `9` | internal invariant violation; fail closed and preserve diagnostic evidence |

The precise typed diagnostic code is always emitted in addition to the exit category. Success flags cannot override a nonzero exit. A user-requested `best_found` result can succeed with code 0; an unmet `--certify` request cannot.

### 16.4 Files and privacy

Never overwrite a destination by default. Input and output inode/path aliasing is rejected. Use capability-scoped, race-resistant SDK file operations, temporary output files, complete flush/close error handling, and atomic publication. Do not follow archive-controlled paths or launch archive-provided commands.

Keep file content, literals, and extracted models out of ordinary telemetry. Compression can expose properties of input through output length and timing; UORC does not claim confidentiality, encryption, or immunity to compression side channels. No remote telemetry is enabled. Receipts may contain hashes and lengths but must not publish private input automatically.

## 17. Prism application and deployment

### 17.1 Artifacts

The Prism application model MUST declare the exact qualified computational roots, exported types, argument/result representations, resource limits, acceptance vectors, package metadata, and target-specific boundary constraints.

Required generated outputs are:

- native `uorc` client for the SDK's supported native platforms;
- generated Rust library package, with no handwritten UORC code;
- import-free Core-Wasm computational guest;
- `UORC.holo` containing the admitted computational and portable View layers;
- browser/View assets generated by PrismPM;
- source-semantic, compiler, SDK, manifest, and validation evidence.

`UORC.holo` is the application package, **not** the compressed data format. A compressed file begins with `UORC`, not `HOLO`. Do not confuse application-package compression with UORC archive size.

### 17.2 View uses an existing bounded transport

The observed PrismPM text-application profile accepts UTF-8 request/response transport, with an independent portable intent maximum of 65,536 request bytes and 1,048,576 response bytes [A03]. Use that actual contract for the first View instead of inventing unrestricted browser file IO.

The View request is one ASCII line:

```text
C1:<lowercase-even-hex>       # compress one byte object
D1:<lowercase-even-hex>       # decode one self-contained archive
I1:<lowercase-even-hex>       # inspect one archive
```

No leading/trailing whitespace, uppercase hex, odd nibble, unrecognized prefix, extra newline, or non-ASCII character is accepted. Compression input is limited to 16,384 decoded bytes. Decode/inspect archive input is limited to 32,768 bytes and is further constrained by the whole 65,536-byte transport bound: after the three-character prefix the maximum complete even-length hex payload represents 32,766 bytes (65,535 request bytes). Reject an over-limit input; do not silently clamp it.

Use `view-small-1`. The response is canonical UTF-8 JSON containing either a typed error or operation, exact byte lengths, claim disposition, and the appropriate lowercase hex result/summary. For decoded output up to 65,536 bytes, the hex response fits the response envelope after checked metadata accounting. A profile overflow must produce a bounded error.

Hex is a View transport only. Native binary compression ratios count the binary archive, not the hex presentation. No UTF-8 validation is imposed on the original byte objects.

### 17.3 View behavior

The View displays input type, input/output byte counts, the actual evidence status, scope, and a clear indication when certification is absent or incomplete. It must not label every successful compression “optimal.” Its buttons, labels, error text, intent handling, focus behavior, and output rendering are model values, generated through PrismPM.

The View neither trains nor calls an external model and requests no network capability. Large benchmark/file operations belong to the native interface, not to an unsupported browser transport claim.

### 17.4 Deployment acceptance

Execute the exact generated Core-Wasm and View through the pinned independent Hologram oracle. Run every modeled request directly and through the actual generated View. Include all 256 input byte values encoded as transport hex, malformed UTF-8 transport, empty input, exact request/response boundaries, output-limit rejection, detach, and idempotent shutdown.

Generated-package compilation or mounting a View is not evidence that the browser executed the request. Use PrismPM's real application-acceptance transcript. Missing root export, truncated buffer, absent response, incorrect error mode, and a compiler/runtime mismatch are mandatory failures.

## 18. Schemas and diagnostics

### 18.1 Closed schema policy

Every public descriptor, record, manifest, receipt, checkpoint, certificate metadata, oracle request/result, and benchmark plan has a named versioned schema generated from LexLean-owned types. Unknown fields, duplicate fields, invalid enum strings, absent required fields, and invalid numeric encodings fail closed.

For evidence JSON use JCS [A07] restricted as follows: field names and enums are ASCII; no floating-point values; all potentially large integers are canonical decimal **strings**; hashes are lowercase fixed-length hex; byte strings are lowercase even-length hex; arrays preserve specified order; maps are represented in canonical key order. Canonical schema tags are mandatory. JSON is metadata/transport, not a second application-program source.

Large binary input and output do not have to pass through JSON in the native API. Canonical JSON helpers must be validated against an imported JCS authority implementation and vectors; user-authored expectation files alone are not enough.

### 18.2 Diagnostic inventory

Generate the exact text templates and field schemas in LexLean. Codes are stable within v1; one code may have multiple modeled parameterized reasons, not untyped exception text.

| Code | Class |
|---|---|
| `UC1001` | malformed command or incompatible options |
| `UC1002` | invalid closed configuration/schema/version |
| `UC1003` | input/query not admitted; raw baseline infeasible |
| `UC1004` | path/capability/aliasing/overwrite refusal |
| `UC2001` | truncated frame or field |
| `UC2002` | bad magic, format, machine, scope, or representation tag |
| `UC2003` | noncanonical/overflowing/unterminated integer |
| `UC2004` | inconsistent count, length, trailing or missing bytes |
| `UC2005` | unknown opcode/type or malformed instruction |
| `UC2006` | illegal local/block reference or recursive dependency |
| `UC2007` | type/signature/capture/entry mismatch |
| `UC2008` | missing, stale, conditional, or invalid basis |
| `UC3001` | arithmetic overflow/underflow/zero divisor |
| `UC3002` | byte index/slice/bit/shift out of range |
| `UC3003` | abstract decode envelope exceeded |
| `UC3004` | resolved output length/boundary mismatch |
| `UC3005` | integrity mismatch |
| `UC3006` | full reconstruction differs from original |
| `UC4001` | malformed certificate or unsupported proof constructor |
| `UC4002` | certificate query/universe/archive binding mismatch |
| `UC4003` | invalid pruning/lower-bound/equivalence witness |
| `UC4004` | missing/duplicate/overlapping coverage region |
| `UC4005` | an eligible improving candidate refutes the claim |
| `UC4006` | requested certification not obtained within budget |
| `UC4007` | certificate verification incomplete within checker budget |
| `UC4008` | invalid or stale checkpoint |
| `UC5001` | SDK/dependency/authority identity or license-manifest failure |
| `UC5002` | required production capability/export missing |
| `UC5003` | external oracle rejected, disagreed, or produced malformed output |
| `UC5004` | required oracle unavailable, timed out, or returned unknown |
| `UC5005` | benchmark plan/corpus/comparator/accounting mismatch |
| `UC6001` | physical memory/time/IO/cancellation failure |
| `UC6002` | transaction/publication failure |
| `UC9001` | internal invariant failure |

Every code requires a reachable negative test. A generic error is not a substitute for the expected code. Diagnostics include a field path or byte offset where applicable, stage, and stable reason enum. Unbounded file contents or external process output must not leak into user diagnostics.

## 19. Required formal proof inventory

The named theorems below must be implemented in LexLean, compiled to Lean, kernel-checked, replayed, and axiom-audited. Names may be namespace-qualified but their statements cannot be replaced by weaker reflexive assertions.

| Theorem | Required statement |
|---|---|
| `varint_encode_decode` | shortest V encoding decodes to the original admitted integer |
| `varint_decode_encode` | every accepted integer spelling is reproduced exactly |
| `varint_size_exact` | structural integer cost equals emitted bytes |
| `wire_parse_serialize` | every well-formed frame serializes and parses to the same structured value |
| `wire_serialize_parse` | every accepted frame byte string is reproduced exactly |
| `wire_size_exact` | whole-frame structural byte cost equals serialization length |
| `local_reference_sound` | every admitted local reference denotes the uniquely typed earlier slot |
| `block_reference_acyclic` | calls cannot introduce a block dependency cycle |
| `typing_sound` | successful execution of a well-typed instruction/block produces the declared result type |
| `primitive_refinement` | runtime checked/modular primitives implement §6.3 exactly, including errors |
| `step_reference_sound` | successful executable evaluation has a reference-semantics derivation with matching result/errors/costs |
| `step_reference_complete_bounded` | every reference evaluation within the abstract bounds is reproduced by the executable evaluator |
| `evaluator_terminates` | any bounded evaluator invocation returns a result; no unbounded recursive escape |
| `resource_accounting_exact` | the modeled resource ledger equals the sum/peak defined by the complete transition table |
| `object_split_join` | frame lengths split the concatenation into the exact ordered input objects |
| `raw_roundtrip` | every admitted input's raw archive resolves exactly to it |
| `raw_size_bound` | literal incumbent length equals §7.3 and bounds published compression output from above |
| `eligibility_reflects` | the computational full-byte eligibility check is equivalent to the independent eligibility predicate |
| `encoder_success_lossless` | any successful encoder result resolves to the original exact byte tuple |
| `incumbent_monotone` | additional deterministic work/resume never worsens the incumbent key |
| `generator_substitution` | typed anti-unification followed by recorded substitution preserves source observations |
| `contextual_rewrite_sound` | each admitted rewrite preserves exact observations and its stated admissibility conditions |
| `contextual_dominance_sound` | pruning cannot remove an alternative capable of improving a represented enclosing context |
| `universe_enumerator_complete` | every and only admitted candidate archives within the ceiling are covered |
| `prefix_rejection_sound` | each prefix rejection excludes all admissible completions of its region |
| `completion_lower_bound_sound` | each certified bound is <= every valid completion's actual byte length |
| `coverage_partition` | every update preserves exact union, disjointness, and unvisited-region accounting |
| `certificate_sound` | checker acceptance implies the independently stated full-universe claim |
| `canonical_tie_sound` | canonical-minimum acceptance excludes smaller equal-length eligible archives |
| `frontier_sound_complete` | complete frontier status proves nondominance and complete coverage under the requested tie policy |
| `checkpoint_resume` | validated resume matches uninterrupted logical execution |
| `basis_context_sound` | conditional resolution uses only the explicitly bound basis and charged definitions |
| `input_not_decoder_state` | the decoder's environment contains no encoder-only input or search state |
| `counting_no_universal_shrink` | an injective lossless coding cannot make every fixed-length input strictly shorter under a fixed decoder/context |

The last theorem is a guard against invalid universal marketing or an unsound constant-size identifier claim, not a restriction on pursuing unusually strong measured compression.

### 19.1 Proof policy

No UORC-owned axiom, proof hole, `sorry`, `admit`, `native_decide`, `sorryAx`, raw tactic/code payload, or oracle-result axiom is allowed. Prefer constructive proofs over finite sequences and explicit natural-number induction. Exact axiom policies are per declaration. Any nonempty inherited axiom set must be traceable to an exact imported dependency and explicitly accepted in the model; a blanket allowance to hide new assumptions is forbidden.

Losslessness and certificate-soundness predicates are independently stated. `resolve(x)=resolve(x)`, `check(c)=true` used as the definition of optimality, or “the encoder returns only values our checker accepts” without the checker's independent correctness theorem is not acceptance.

Every theorem's runtime-dependent premise is either validated computationally or explicitly supplied as a verified value. An erased `Prop` field is not runtime validation of untrusted archive bytes.

## 20. External authority acquisition

### 20.1 Manifest

Every external authority/oracle/comparator is imported using an immutable manifest containing:

```text
id, issuer, canonicalSource, role, editionOrVersion,
resolvedCommitOrRelease, artifactUrl, acquiredSha256,
mediaType, byteLength, license, redistributionDisposition,
retrievedAt, buildRecipeId, binarySha256ByPlatform,
interfaceVersion, allowedArguments, expectedExitSemantics,
usedByRequirements, updatePolicy
```

No fabricated digest, `checksum=none` for actually acquired bytes, mutable download at test time, or executable selected from ambient PATH is allowed. When an issuer publishes no digest, acquire from its authenticated authoritative location, record the observed digest and this provenance limitation, and freeze the exact bytes. A locally calculated hash is not an issuer signature.

Acquisition is explicit setup. Build, verification, and release gates run offline afterwards. Tools are supplied through the immutable SDK/validation image model, not copied into UORC as handwritten application code. Preserve required source/license notices when redistributing; do not assume every benchmark corpus is freely redistributable. A restricted corpus may be user-acquired by an explicit locked setup step, but its absence is not a passing benchmark result.

### 20.2 Independence rules

An imported authority is not the implementing agent's own reimplementation of an expected answer. UORC-owned oracle translators are necessary, but they are not external oracles and must be tested/proved separately.

A new wire format has no presumed preexisting authoritative UORC decoder. Do not invent one or falsely call an internal reference interpreter external. Validation therefore combines real external solver/runtime/primitive authorities with UORC's independent formal semantics and full byte comparisons. Each result states precisely which property that oracle checked.

The oracle graph MUST be acyclic in its justification. The encoder cannot generate expected optimal answers, store them, and then pass a test by reproducing them. Local goldens are regression fixtures; they are not sufficient external validation.

### 20.3 Trust classes

- **Formal authority:** the pinned Lean kernel and its documented behavior. `leanchecker` is same-kernel replay in another process, not an independently implemented checker [A05].
- **Independent implementation oracle:** a separately maintained solver, runtime, primitive implementation, or official test suite used for an explicit comparison.
- **Benchmark comparator:** a conventional compressor measuring competing output size and resources, not validating UORC's grammar.
- **Local proof/check:** UORC-owned semantics and certificate checking.
- **Measurement:** observed speed, memory, compression ratio, and reproducibility.

Never merge these into a single undifferentiated `verified=true`.

## 21. Mandatory external validation oracles

### 21.1 Lean authority and replay

Import the exact Lean source/toolchain identity used by the validated SDK. The researched releases use Lean 4.32.1; the release's actual lock is authoritative. Run elaboration, separate-process `leanchecker`, exact per-declaration axiom audit, and the source/closure checks for every UORC module. Preserve the source/compiler identity chain.

Kernel checking verifies formal statements. It does not by itself show that the executable compiler and deployed runtime preserve those statements on every machine.

### 21.2 NIST SHA-256 vectors

Import NIST CAVP SHA-256 byte-oriented response files from the authoritative secure-hashing page [A08], including short-message, long-message, and applicable Monte Carlo vectors. Validate the exact identity hashing implementation used by the generated artifact, whether LexLean-owned or an explicitly imported generated SDK primitive.

Compare expected digests from the NIST files with actual generated-runtime output. Validate importer completeness and record vector counts from the acquired archive. Missing files, malformed vector parsing, an empty selected corpus, or dropping failing vectors fails acceptance.

Passing these vectors is not NIST certification and does not prove collision resistance. NIST expressly distinguishes using vectors from obtaining CAVP validation [A08].

### 21.3 Z3 and cvc5 numeric/constraint oracles

Import immutable official Z3 and cvc5 releases/sources [A09–A10]. Use their command-line SMT-LIB interfaces through the generic SDK oracle runner; no handwritten Python solver program is permitted.

`OracleRequests.lex.tex` constructs a closed typed SMT formula representation and renders only the admitted SMT-LIB subset. The renderer is not a semantic escape: formulas are projections of typed UORC validation queries and cannot define production behavior.

Required checks:

- all 256 values for byte negation, complement, conversion, shifts, and bit observation;
- all 65,536 ordered byte pairs for every byte binary operation, including exact modulo behavior;
- U64 boundary values for every arithmetic operation, with at least `{0,1,2,127,128,255,256,2^32-1,2^32,2^63-1,2^63,2^64-2,2^64-1}` and all ordered pairs plus deterministic interior cases;
- bitvector/integer agreement for the range-checked encoding of these operations;
- bounded evaluator instances with independently formulated block/slot constraints;
- small complete minimum-size queries, including satisfiable “a better archive exists” and unsatisfiable “a better archive exists” cases.

Both solvers must agree with the explicit expected claim on mandatory deterministic suites. Parse `sat`, `unsat`, `unknown`, diagnostics, and model data strictly. `unknown`, timeout, unsupported logic, and truncated stdout are incomplete/unavailable, not `unsat`.

### 21.4 Independent constraint formulation

For small grammar instances, encode candidate opcode/type/reference/literal fields as symbolic variables. Encode reference semantics by explicit bounded transition equations, not by inserting the encoder's computed answer as a formula constant. Ask whether a well-typed, envelope-admitted candidate resolving to the target has lower exact serialized cost.

Use a separate LexLean module from the production search/pruner to generate these equations. Prove the encoding's bounded soundness/completeness correspondence to the micro-profile. Candidate solver models are decoded to actual archives and rechecked by full UORC resolution. An UNSAT answer is external validation evidence only; production minimum status still requires §13's checked certificate.

Micro-profiles are explicitly restricted test universes, not the production universe. Their reports contain all restrictions. The required suite includes every opcode/type in at least one nontrivial symbolic instance; all local/block reference boundary cases; zero and positive iteration; several sharing/layout choices; and at least one case where discovery's best-found result is refuted by an external better witness. Enumerate all target words of length 0 through 6 over the alphabet `{00,01,02,03}` for the smallest tractable fixed micro-profile, and use the complete input inventory rather than a favorable selection.

The SDK may partition this suite across jobs. It may not silently reduce the declared input set to obtain green results.

### 21.5 WebAssembly execution and binary authorities

Import the WebAssembly reference interpreter and official test suite from the WebAssembly specification project, plus a pinned Bytecode Alliance Wasmtime runtime [A06, A11–A12]. Select a Wasm feature profile both oracles actually support; generated code must stay within it.

Execute identical generated UORC Core-Wasm exports on both runtime implementations and compare full results against the independently stated reference semantics. Run the official tests relevant to every binary/runtime feature used by the artifact. A missing feature is a target-eligibility failure, not permission to skip that runtime.

A generic oracle runner may adapt the guest's memory/export protocol. That runner must be an immutable SDK capability; UORC-specific request and expected-result semantics remain in LexLean. Import-free guest execution and bounded memory do not imply a whole-runtime proof; report the trust boundary.

### 21.6 Prism/Hologram oracle

Use PrismPM's exact imported independent Hologram Live/uor-hologram authority bindings and its real application verification. Do not replace them with a local emulator or assume PrismPM `build` is `verify`. Exercise direct guest invocation and actual generated View intents, including all errors and transport edges described in §17.

### 21.7 Compiler differential

Use the selected SDK's existing production exporter and generated native/Wasm artifacts. Where the completed LexLean-native production compiler is available, also import the exact `lean4-prod` revision as a differential oracle only. Compare observables, overflow/error behavior, root coverage, and byte buffers across the actual paths.

Where `lean4-prod` is itself the current production exporter, comparing it to itself is not an independent compiler differential. In that case mandatory external runtime/solver comparisons still apply, and the report explicitly marks the stronger independent-compiler check as not applicable to that pipeline rather than counting it as passed. Do not advertise a completed native-compiler requirement that upstream has not implemented.

### 21.8 Exact equality outside the codec

The SDK validation plan MUST also compare decompressed files against immutable source snapshots using an authoritative external byte comparison utility, such as GNU `cmp` [A13], and an independently acquired SHA-256 utility for transcript bindings. Full comparison decides round-trip equality; matching hashes alone are insufficient.

This comparison catches transport/file/buffer mistakes outside the formal core. It does not certify optimality, and it does not replace the formal encoder-success theorem.

## 22. Benchmarks and the compression-performance program

### 22.1 Required input suites

The accepted repository MUST contain a sealed, generated inventory for each of these suites:

| Suite | Required inputs and role |
|---|---|
| `wire-boundaries` | empty objects, every one-byte object, every ordered byte pair, lengths around each used integer-field boundary, arbitrary binary data, and malformed frames; correctness, not compression marketing |
| `native-structure` | the deterministic coordinate, parameter-sharing, recurrence, and corpus families in §23; demonstrates that the required native synthesis engines actually work |
| `enwik8` | the exact 100,000,000-byte object identified by the corpus maintainer [A14]; public text-compression measurement |
| `silesia` | every member in the acquired authoritative Silesia distribution, in the distribution's declared inventory [A15]; heterogeneous public measurement |
| `holdout` | a separately acquired, license-permitted corpus with its exact identity and selection rule sealed before performance tuning; no benchmark-recognition code |

The maintainer's `enwik8` identification includes MD5 `a1fa5ffddb56f4953e226637dabbb36a` and SHA-1 `57b8363b814821dc9d47aa4d41f58733519076b2`. Use those only to reconcile the published historical identity. Acquisition also computes and locks SHA-256 over the actual bytes. Do not invent that SHA-256 or describe MD5/SHA-1 as the product's security primitive.

The full `enwik9` object and a Hutter Prize submission are **separately declared research/record workloads**, not prerequisites for a v1 wire-format theorem. Before any such claim, acquire the exact input and current rules from the appropriate authority, including time, memory, executable, shared-state, and score accounting. A historical leaderboard, a locally chosen scoring formula, or this brief is not that authority's acceptance.

Corpus acquisition, decompression of the authority's distribution archive, member ordering, and digest verification happen during explicit setup. The tested objects are immutable afterwards. No line-ending normalization, Unicode normalization, file filtering, deleted headers, reordered records, or deduplication may change the workload unnoticed. Any intentionally transformed workload is a distinct named input, with the transformation and its storage cost recorded.

### 22.2 External comparators

Import official, immutable releases of Zstandard, Brotli, XZ, and cmix [A16–A18, A20] as **validation/measurement-only tools**. They MUST NOT appear in UORC's decoder, encoder candidate evaluator, shipped dynamic dependencies, or generated compression implementation.

A comparator binding includes the exact source revision, binary hashes, licensed distribution, executable arguments, thread count, memory settings, input convention, output convention, and decompression command. Seal settings before running a comparison. Use documented settings from that acquired release; do not invent a common compression-level scale across different tools.

At least two plans are required:

- `public-default`: every required public corpus, each comparator's documented default setting, and UORC's published default search budget;
- `public-ratio`: the same corpus inventory, explicitly fixed high-ratio settings and larger declared UORC search budgets, with equal accounting and disclosed resource differences.

Every comparator output must itself decompress and compare byte-for-byte with the immutable source. Report comparator failure or inability to complete; never silently omit a poor or inconvenient result. A tool's use as a comparator does not make its implementation an oracle for UORC's unique reference language.

cmix is a separately licensed, resource-intensive comparator. Its source and artifacts remain validation-only, with its actual license recorded; do not link it into UORC or copy its application code. The same production/validation separation applies to every comparator regardless of license.

### 22.3 Sealed benchmark plan

A `uorc/benchmark-plan/1` value contains exactly the following logical fields, with their closed schema generated from LexLean:

```text
schema, name, corpusIds, orderedObjectIds, grouping,
accountingScope, basisBinding, decoderDeliveryBinding,
uorcSourceId, sdkId, machineId, decodeEnvelope,
searchBudget, requestedClaim, comparatorBindings,
physicalLimits, repetitions, runOrder, cachePolicy,
measurementProtocolId, acceptanceExpectations
```

`grouping` is `per_object` or `joint_corpus`. `accountingScope` is one of §9.2's explicit scopes. A basis chosen using the evaluation objects is charged as input-dependent corpus data; it cannot be presented as a previously installed free model.

Use three repetitions for ordinary timing reports. Run order is a deterministic rotation over the plan's ordered implementations, preventing one implementation from always following the same cache state. Record cold/warm setup policy and actual completion of each repetition. A separate high-cost plan may explicitly declare one repetition; that report must not invent a variance estimate or compare its single observation to another implementation's best-of-many timing.

Physical limits are test conditions, not claims about the admissible archive universe. A timed-out compressor is reported as timed out. A completed valid UORC incumbent may still be reported with its actual `best_found` status. No timeout is treated as proof that another implementation cannot produce a better representation.

### 22.4 Measurements

For every input/group and implementation record:

- original bytes, exact output bytes, and every charged side artifact;
- scope, basis/model bytes, decoder/runtime bytes when applicable, proof bytes when delivered;
- complete reconstruction comparison result;
- encoder and decoder wall time, peak RSS, and process exit status;
- UORC abstract decode steps/live cells and actual search work;
- deterministic archive identity and claim/certificate status;
- source, compiler, SDK, binary, corpus, and plan identities;
- any warm-up, setup, preprocessing, or acquisition time excluded from the operation metric, explicitly labeled.

Use rational ratios represented by integer numerator/denominator. `bits_per_input_byte = 8 * chargedBytes / originalBytes` is undefined for zero input length and must be reported as `not_applicable`, not a division error or fabricated zero. The aggregate ratio is computed from aggregate byte counts; an unweighted average of per-file ratios is a different statistic and must be labeled separately.

Timing and RSS are noncanonical measurement artifacts. They MUST NOT perturb source semantics, archive bytes, canonical proof evidence, or content identity. Reports may reference their separately hashed transcripts.

### 22.5 Performance acceptance and claims

Implementation acceptance requires genuine compression on the native structural suite, execution of the required sealed public comparison plans, and honest reports. It does **not** require falsifying a measurement until UORC appears to beat all comparators. Public-corpus superiority remains an open measured claim until it is achieved.

A headline claim must identify the corpus, scope, exact versions/settings, resources, date, and reconstruction evidence. “Highest compression ever” is not a permissible unqualified release claim. An external record needs the exact authority's acknowledgment; a scoped minimum certificate for UORC's machine is not a world-record certificate.

Maintain a model-owned research result with one of `not_measured`, `measured_target_unmet`, `measured_target_met`, or `externally_accepted_record`. No status is implied by implementation version or number of formal theorems.

## 23. Required fixtures and constructive witnesses

### 23.1 Exact wire fixtures

Appendix C supplies concrete framing and generative-reference examples. Import their literal bytes into LexLean fixture data and verify them, rather than treating this document's examples as already established implementation evidence.

In addition, generate fixtures for all integer values at `0`, `1`, `127`, `128`, `16383`, `16384`, every higher seven-bit transition through U64, and U64 maximum. For each legal spelling include an overlong spelling, a truncated spelling where possible, and an overflowing final group. Include every undefined opcode and type tag.

Every frame field must have at least one negative case that changes only that field and reaches the owning validator. A parser failure earlier in the frame does not exercise a later field's negative requirement. Cover return references with no locals, zero/forward references, missing blocks, incorrect parameter counts/types, a non-byte-sequence entry, extra body bytes, an incomplete integrity field, and trailing bytes.

Test raw and graph forms, single and multiple objects, zero-length members, and self-contained and conditional forms. A raw archive that expands slightly due to framing is correct; the guarantee is bounded expansion relative to the specified raw frame, not shrinking every input.

### 23.2 Native structural suite

The suite is generated by a **separate LexLean test-data module**, using direct sequence definitions rather than invoking the encoder. The encoder receives only the resulting bytes and ordinary request, not the generating expression, its parameters, or a fixture ID.

Required families include:

1. **Coordinate charts:** byte arrays of length 4096 defined by `ByteOfNat(floor(i/w) + c*(i mod w) + d)` for `w` in `{7,16,31}`, `c` in `{3,11}`, and `d` in `{5,173}`. Supply these as independent bytes; the production chart engine must synthesize and validate a charged representation.
2. **Byte-ring generators:** `ByteAdd(ByteMultiply(a,ByteOfNat(i)),b)` for every `a` in `{1,3,5,17,129}` and `b` in `{0,23,255}` over length 2048, including wraparound. Neither the scalar parameters nor the generator are free side information.
3. **Nonlocal shared parameter families:** at least eight objects that use one nontrivial parameterized generator with different explicit parameters, interspersed with unrelated literal fragments. Joint synthesis must create a shared block and exact argument substitutions rather than merely store identical substrings.
4. **Finite-state evolution:** byte state `s_(i+1)=ByteAdd(ByteMultiply(5,s_i),1)` for at least four distinct seeds, with output `s_i` at coordinate `i`, represented by an ordinary earlier-block `Fold` inside a tabulated generator. Include a state-observation variant where index-only short expressions in the restricted fixture search do not suffice. Report the fixture's restricted discovery configuration honestly; do not infer a mathematical lower bound against all coordinate expressions.
5. **Exception-bearing charts:** the first family with 17 deterministic, distinct coordinates changed to explicitly stored replacement bytes. Exact exceptions and their selection/indexing machinery are charged.
6. **Sharing counterexamples:** short repeated literals where adding a definition costs more than repetition, and layouts crossing local/block reference distances 127/128 and 16383/16384 where wire size changes.
7. **Information-poor input:** deterministic externally validated pseudo-random fixtures and already-compressed official comparator output; UORC must retain a correct raw incumbent and must not invent a strong size claim.

For families 1–3, require at least one complete generated representation per parameter combination with archive bytes strictly below its raw frame, obtained by the production synthesis path. For family 4, require the production `Fold` engine to create and validate a nontrivial compressed witness; seed and recurrence description costs must be present. For family 5, require exact handling of every exception and a genuine improvement on at least one fixture. Family 6 explicitly requires sometimes **not** sharing.

These are functional checks that the mandatory synthesis mechanisms exist, not proof of a compression record. Fix a finite documented test search budget before accepting a fixture; do not feed the correct representation directly to the compressor. Unit tests may separately supply known witness graphs to exercise resolution.

### 23.3 Algebra and identity fixtures

Prove and independently test the byte-ring operation identities used by rewriting. For example, `ByteNegate(ByteComplement(b)) = ByteAdd(b,1)` for all 256 bytes is a valid consequence of the exact v1 definitions; it is not an imported statement about every UOR domain. Every rewrite used by the optimizer needs its own actual theorem and operand/type/side-condition tests.

Inject a test hash function that maps every value to the same digest at the interning boundary. Distinct contents, blocks, and basis objects must still remain distinct unless full comparison or a valid witness establishes equality. The test hook must not become a selectable production hash algorithm.

### 23.4 Exact-search fixtures

Required complete-search instances are not limited to an empty input:

- a minimum certified by exhaustive replay over a deliberately small, explicitly bounded micro-universe;
- a production-format minimum established by the prefix-coverage route, including a nonempty target;
- a prefix tree with a candidate at an internal terminal, proving that terminal coverage is not lost;
- two distinct eligible equal-size archives, checking minimum versus canonical-minimum treatment;
- a restricted complete frontier with incomparable `(wireBytes,steps)` results;
- a locally normal but globally suboptimal candidate with a found counterexample;
- insufficient discovery, certification, and verifier budgets, each with its own incomplete result;
- a changed input, basis, resource envelope, or profile rejected against stale evidence.

A micro-universe fixture states its restrictions in its test descriptor. A production-format certificate must quantify over the full §10 wire universe for its declared envelope; it cannot silently substitute the micro-universe. Tiny production envelopes may be used, but must still admit the raw baseline and all other candidates those envelopes allow.

### 23.5 Stateful, transport, and failure fixtures

For a fixed query and deterministic schedule, a run of `a+b` work units and a run of `a` units followed by a validated resume of `b` units must have the same logical search state/incumbent, excluding invocation-local telemetry. Exercise checkpoints before and after a candidate commit, a scheduler quantum, a sharing merge, and a coverage split.

Test cancellation, immutable input snapshot changes, partial reads/writes, disk exhaustion, output aliasing, output already existing, interrupted publication, and paths outside the capability root. No failure may leave an accepted destination containing partial output. Test caller-requested limits at, below, and above the exact boundary.

For every core test, exercise generated native and Core-Wasm execution where admitted by that target profile. Text View tests are separately bounded; they do not stand in for large binary native tests.

## 24. Claim registry and generated test ownership

### 24.1 Required registry families

`Registry.lex.tex` owns the requirement/claim IDs, statements, evidence kinds, diagnostic bindings, scenarios, and test-root mappings. The following IDs are required. Split an ID into additional registered rows when its coverage would otherwise be ambiguous; do not silently delete its requirement.

| ID | Required claim / owning acceptance responsibility |
|---|---|
| `UC-DEP-01` | template provenance, exact SDK/main-derived dependency identities, and platform inventories agree |
| `UC-DEP-02` | actual LexLean/PrismPM production-capability probes pass; no local compiler/runtime substitute exists |
| `UC-SRC-01` | every product-specific computational/proof/test/report root is LexLean-authored and generated artifacts are not independent authority |
| `UC-SRC-02` | required model/scenario/test/diagnostic projections form exact checked inventories |
| `UC-WIR-01` | shortest ULEB128 parsing, serialization, range, and byte cost are exact |
| `UC-WIR-02` | frame parsing/serialization, lengths, scopes, integrity preimages, and EOF are exact |
| `UC-WIR-03` | graph-table wire fields and whole-archive byte accounting are exact |
| `UC-TYP-01` | local/block references, types, parameters, captures, and entry constraints fail closed |
| `UC-TYP-02` | every admitted opcode/type has exactly one semantic and runtime implementation with exhaustive handling |
| `UC-SEM-01` | all checked and modular primitive results/errors agree with independent semantics |
| `UC-SEM-02` | Apply, Tabulate, and Fold implement their exact argument, iteration, and result semantics |
| `UC-SEM-03` | executable evaluator refines the independent reference relation and terminates under its envelope |
| `UC-RES-01` | complete abstract work/live-cell accounting is deterministic and separate from physical telemetry |
| `UC-RES-02` | malformed/expanding inputs and physical failures cannot bypass limits or become false ineligibility evidence |
| `UC-ID-01` | identities have exact domain-separated preimages and hashing passes imported vector checks |
| `UC-ID-02` | hash collisions cannot establish content/reference/basis equivalence |
| `UC-RAW-01` | admitted inputs have a feasible exact raw frame and the encoder never returns a larger incumbent |
| `UC-ENC-01` | every successful encoded archive reconstructs all input bytes and object boundaries |
| `UC-SYN-01` | generic typed enumeration proposes ordinary admitted generators without a codec-name dispatch |
| `UC-SYN-02` | coordinate/byte-ring chart synthesis is exercised on the full native parameter suite |
| `UC-SYN-03` | typed anti-unification has exact substitution witnesses and charged arguments |
| `UC-SYN-04` | joint nonlocal/corpus sharing explores shared and unshared alternatives with full context costs |
| `UC-SYN-05` | finite-state Fold synthesis creates and validates nontrivial generated references |
| `UC-SYN-06` | layout search accounts for all reference widths, framing transitions, and admissibility constraints |
| `UC-SYN-07` | fair deterministic scheduling, incumbent monotonicity, and resumed work agree |
| `UC-EQV-01` | every semantic quotient merge/rewrite/pruning fact has exact observations or checked parametric evidence |
| `UC-ENV-01` | contextual nondominance never discards a candidate by incomplete local size information |
| `UC-UNI-01` | candidate universe is independent of discovery/search budget and complete for the sealed descriptor |
| `UC-UNI-02` | exact enumeration covers every candidate word/layout allowed by the descriptor |
| `UC-COV-01` | prefix impossibility and completion lower bounds are sound |
| `UC-COV-02` | coverage splits account for terminals and every in-range byte branch exactly |
| `UC-CER-01` | exhaustive certificate replay verifies the independent minimum predicate or returns incomplete/refuted |
| `UC-CER-02` | prefix certificate serialization/checking and its generic soundness theorem hold |
| `UC-CER-03` | minimum, canonical ties, local normality, and incomplete states cannot be conflated |
| `UC-FRN-01` | bounded frontier analysis preserves incomparable members and its explicit identity-tie policy |
| `UC-BAS-01` | conditional basis resolution is explicit, nonrecursive, exact, and fully accounted |
| `UC-COR-01` | ordered corpus boundaries, joint sharing, and aggregate accounting are exact |
| `UC-CHK-01` | checkpoint validation cannot promote stale/forged hints into proof coverage |
| `UC-API-01` | generated typed library roots and every result/error boundary match their modeled contracts |
| `UC-CLI-01` | generated command parsing, options, diagnostics, and exit dispositions are exact |
| `UC-IO-01` | binary transport and transactional publication preserve bytes and capabilities |
| `UC-VIW-01` | bounded text transport and generated View expose accurate counts and evidence status |
| `UC-PKG-01` | generated native, Core-Wasm, Hologram, browser, and package artifacts have complete root/provenance closure |
| `UC-ORA-01` | authority manifests, imported inventories, actual tool identities, and offline invocation are validated |
| `UC-ORA-02` | both external solvers check the numeric and bounded relational suites with strict result parsing |
| `UC-ORA-03` | independent WebAssembly runtimes execute the exact generated module and agree with reference results |
| `UC-ORA-04` | independent Hologram and real generated View execution satisfy all application vectors |
| `UC-ORA-05` | external full-byte comparison and primitive/canonicalization oracles validate their actual boundaries |
| `UC-BEN-01` | mandatory corpus/comparator plans execute with exact reconstruction and complete accounting |
| `UC-REP-01` | clean builds and deterministic search yield identical canonical artifacts under the declared identity boundary |
| `UC-NEG-01` | every public diagnostic is reachable through its own negative case and exact expected code |
| `UC-MUT-01` | every mutation class in §26 is caught by its named owning gate, not an unrelated failure |
| `UC-REL-01` | release acceptance requires all mandatory inventories, proof/oracle evidence, and actual target executions |
| `UC-PERF-01` | measured public-corpus compression improvement, with exact target and scope; open until actually measured |
| `UC-REC-01` | externally accepted compression record; open unless the external authority actually accepts it |

### 24.2 Honesty levels

Imported authority facts have the template's imported/`some-true` disposition and are not reclassified as UORC proofs. The implemented conformance claims above are `build` evidence unless the exact generated schema uses the template's equivalent spelling. Their attached Lean theorems are explicitly identified mathematical proof evidence, not an excuse to rename the template's honesty levels.

`UC-PERF-01` and `UC-REC-01` are open measurements and MUST NOT be used as passing implementation evidence. They cannot become positive merely because a fixture ratio improved. A result requires its exact measured target or external acceptance.

### 24.3 Test identity and anti-vacuity

Each implemented claim has a generated scenario and an exact named test entry, using `conformance_` followed by its lower-case ID with hyphens replaced by underscores. The generic template/SDK runner may discover those generated tests, but may not inject UORC behavior itself.

The inventory comparison must inspect actual executed test roots and returned results, not merely count files, registered IDs, or log lines. Empty oracle corpora, missing declared features, zero assertions, skipped tests, platform-disabled normative cases, early-success branches, stale logs, or a test registered under the wrong ID fail acceptance.

No test named for losslessness may only check `encode(x)=encode(x)`. No test named for minimum may only check the selected candidate against itself. The independently quantified theorem and external witness/counterexample cases remain load-bearing.

## 25. Complete validation and CI

### 25.1 Single acceptance boundary

`just vv` remains the repository's only normative implementation acceptance gate. It invokes inherited policy gates and the SDK's generated UORC acceptance roots. It must include, in this logical dependency order:

1. immutable template/SDK/authority checks and actual capability probes;
2. lexical/source closure, generation drift checks, model/scenario/test reconciliation, formatting, and dependency/license/security policy;
3. all unit/property/negative/generated conformance tests;
4. real Lean elaboration, kernel replay, exact axiom policies, and required closed-instance proofs;
5. imported vector, solver, canonicalization, and independent runtime oracles;
6. generated native/Wasm/package tests and full Hologram/View application verification;
7. all mandatory benchmark plans and full byte comparisons from §22, including both success and unsuccessful performance outcomes as typed measured data;
8. golden, clean-build reproducibility, checkpoint/scheduler invariance, and mutation-evidence validation;
9. complete acceptance inventory reconciliation and release-readiness disposition.

A benchmark's honest failure to beat a comparator is not a failed implementation check; inability to execute the mandatory plan, omission of a comparator, wrong source bytes, or incorrect reconstruction is. Physical test limits and available runners must be selected accordingly before marking the gate complete.

Targeted development commands may run subsets and must label them as subsets. They cannot produce the repository's full acceptance result, replace a required merge/release check, or be called a second normative `vv-fast` gate.

### 25.2 Isolation and reproducibility

Run in the exact SDK-selected devcontainer/runner with setup completed, network disabled for build/verification, and a clean repository checkout. Install immutable external tools during setup with their licenses and hashes verified. Do not restore generated UORC proof trees, prior attestations, generated application binaries, or previous-commit acceptance as substitutes for work required on the current checkout.

External published toolchain packages are locked dependencies, not locally proved UORC artifacts. Record that distinction. Any acceleration of dependency acquisition must preserve the exact source/binary inventory checks and must not turn a cache hit into a UORC proof result.

Build twice in distinct absolute paths, with no common UORC build output. Compare the exact canonical source-derived artifacts and generated archive fixtures. For host-dependent executable objects or process hashes, use only the existing explicitly specified platform-bound normalization; never normalize away semantic differences, missing roots, diagnostic changes, or source identities.

### 25.3 Parallelism

Independent test jobs and bounded proof processes may run in parallel when resources allow. Deterministic scheduling is required for product search; collected proof/canonical records must have their specified order regardless of completion order.

Use a measured memory envelope. Do not increase proof concurrency solely because a machine has more CPUs. All children must be joined or terminated before failed staging trees are discarded. The acceptance inventory must still contain every required item. Per-job cancellation cannot silently produce aggregate success.

Timing, RSS, swap, queue time, and process completion order belong in separate performance telemetry. They are not input to `MachineId`, `UniverseId`, source semantics, normalized proof output, or compressed archive identity. The SDK/compiler identities themselves remain part of their appropriate evidence bindings.

### 25.4 Generated acceptance artifact

Generate `uorc/implementation-acceptance/1` with:

```text
sourceId, requirementInventoryId, theoremInventoryId,
scenarioInventoryId, executedTestInventoryId,
sdkId, dependencyInventoryId, authorityInventoryId,
platformResults, proofResults, oracleResults,
applicationAcceptanceId, benchmarkPlanResults,
reproducibilityResults, mutationEvidenceId,
mandatoryFailures, implementationAccepted,
performanceDisposition, recordDisposition
```

`implementationAccepted` is true exactly when every mandatory current-build item succeeds and every inventory reconciliation is exact. It cannot be set by a CLI option, a certificate supplied by an untrusted user, or a stale expected file. `performanceDisposition` and `recordDisposition` are separate values from §22.5.

Actual PrismPM application acceptance remains required; this aggregate artifact does not manufacture a substitute for `prismpm/application-acceptance/1`.

## 26. Falsifiability and adversarial mutation requirements

For every mutation, record the unmodified source identity, exact changed semantic/generator location, intended violated property, targeted command, expected diagnostic/test, observed failing result, and restored passing result. A syntax error, unrelated compiler failure, or missing tool does not count as detecting the intended semantic defect.

| Mutation | Required observing gate |
|---|---|
| accept a redundant ULEB128 high zero group | canonical integer/wire tests |
| omit a header/reference length byte from size accounting | actual serialization-versus-cost comparison |
| reverse or offset a local backward reference | typing/reference and independent evaluator cases |
| allow a block to refer to itself or a future block | block acyclicity/format gate |
| wrap U64 overflow instead of returning the defined error | solver-backed numeric and generated-runtime tests |
| replace byte modulo-256 arithmetic with checked U64 behavior | byte-ring oracle fixtures |
| swap quotient and remainder, or use a wrong zero-divisor result | numeric/coordinate reconstruction oracles |
| run Tabulate/Fold one iteration too few or too many | nontrivial iteration and output-boundary tests |
| consult encoder-only source bytes while resolving a candidate | isolated decoder and no-hidden-input tests |
| merge distinct contents solely on equal hash | all-colliding-hash fixture |
| omit generator arguments/shared definitions from charged size | standalone/conditional/joint accounting tests |
| prune by `(content,localSize)` without sharing context | contextual dominance counterexample |
| call a local fixed point globally optimal | exact micro-universe counterexample |
| interpret search timeout or solver `unknown` as completeness | incomplete-status/strict oracle response tests |
| omit the terminal word of a prefix split | prefix coverage counterexample |
| omit, duplicate, or overlap a split's byte branch | coverage inventory and certificate checker |
| increase a completion lower bound without proof | independently recomputed lower-bound check |
| discard equal-cost identities in identity-complete frontier mode | tie/frontier fixture |
| change query/basis/envelope while reusing checkpoint coverage | binding and resume tests |
| declare full comparison after only comparing digests | receipt/oracle byte-comparison gate |
| ignore a later byte mismatch after a matching prefix | full-length generated/native/external comparison |
| truncate a guest response or mishandle a byte-buffer boundary | independent Wasm/Hologram/native transport tests |
| publish an output before integrity validation | transactional IO/error-injection gate |
| run no imported vectors but report their register count | actual oracle invocation/inventory anti-vacuity |
| accept an oracle response from another request or tool binary | request/tool/source binding tests |
| use prior-commit generated code as current proof/application output | source provenance and clean regeneration gate |
| insert wall time into a canonical identity or output | clean reproducibility and identity separation tests |
| add a handwritten target-specific compression branch | source ownership/production dependency audit |
| alter a generated expectation to match a defect without authority change | model/oracle/source identity reconciliation |
| use a malformed byte sequence as UTF-8 View output | generated text transport and browser tests |

The mutation mechanism is itself modeled. Temporary source variants are generated from typed mutation descriptions, not ad hoc product code. Imported upstream mutation tools may be generic validation infrastructure only. Do not exempt the mutation harness, source audit, certificate checker, or cost model from their own source/closure checks.

## 27. Implementation order and integration ownership

### 27.1 Required order

Implement vertical slices in this order, following the template's ID → scenario → failing test → implementation → planted defect → complete gate discipline:

1. **Bootstrap:** immutable template/SDK bindings, latest-main dependency reconciliation, actual production/oracle/transport capability probes.
2. **Types and bytes:** closed numeric types, exact byte operations, errors, primitive external vectors.
3. **Wire and raw path:** parsers/serializers, exact size, identity preimages, raw reconstruction theorem, native binary round trip through the real SDK.
4. **Reference machine:** blocks, typing, instructions, pure reference semantics, executable stack, complete resource ledger, generated native and Wasm refinement tests.
5. **Native synthesis:** generic enumeration, coordinate charts, anti-unification, sharing, Fold, exact layout costs, final eligibility checks.
6. **Global state/evidence:** typed equivalence/warrants, contextual envelope, independent universe, resumable deterministic search.
7. **Certification:** shortlex reference enumeration, prefix rules, exact certificate encoding/checker, soundness theorem, small production-format and micro-universe certificates, external solver refutations.
8. **Corpus and basis:** full joint accounting, ordered boundaries, explicit conditional context, checkpoint safety.
9. **Client and application:** complete modeled command set, generic filesystem transports, bounded View, actual Hologram/browser validation, packages.
10. **Acceptance:** complete source/claim/test/proof/oracle inventories, public benchmark plans, mutations, clean reproducibility, release manifest.

This order is not permission to label an incomplete intermediate slice the completed v1 product. Intermediate revisions carry exact implemented claims and explicit development status.

### 27.2 Upstream changes

A missing SDK primitive, generalized exporter capability, generic oracle process adapter, generated CLI/file adapter, or package-test hook is an upstream integration task. It must be implemented generically, tested by that project's existing discipline, and consumed through an immutable SDK update. UORC must not acquire a second handwritten implementation while waiting for it.

Do not change UORC's semantic meaning to make an exporter accept it. A first-order flat representation is allowed because it denotes the specified machine exactly; replacing a byte-buffer result with an approximate text summary is not.

A required upstream publication needing credentials remains an accurately reported dependency gate. A locally successful implementation does not imply the published SDK or downstream clean build is accepted.

### 27.3 Non-goals of version 1

Version 1 does not claim arbitrary-program Kolmogorov minima, universal runtime optimality, a complete general LexLean-to-Rust compiler proof, learned neural weights, an archive filesystem with metadata/permissions, browser access beyond the actual View profile, online external object lookup, adaptive decoder training, encryption, or a world record.

These exclusions prevent false claims; they do not exclude arbitrary admitted byte inputs, required native synthesis, real certification on supported bounded instances, or any mandatory client/oracle behavior specified here.

## 28. Release definition of done

UORC version 1 is implementation-complete only when all the following hold together:

1. The authoritative implementation and proof graph is entirely LexLean-authored; PrismPM generates every product artifact. No independent hand-authored UORC implementation or Markdown specification has been committed.
2. The repository conforms to the locked template and SDK boundary, actually uses the selected main-derived LexLean/PrismPM integration, and passes offline clean-checkout bootstrap.
3. Every opcode, field, result, error, resource transition, synthesis engine, client command, certificate route, and data schema in this specification exists and is exercised.
4. The independent reference relation, runtime refinement, losslessness, cost, enumeration, coverage, and certificate theorems are genuinely kernel-checked under exact axiom policies. All required concrete instance proofs run.
5. Every imported oracle is acquired from its named authoritative source, identity-checked, actually invoked on its required nonempty complete corpus, and interpreted with strict failure/incomplete behavior.
6. The generated native/Wasm/Hologram/View artifacts execute and reconstruct identical bytes through the real independent runtime and file boundaries.
7. The native structural suite demonstrates real generative compression, nonlocal parameter sharing, and bounded-state synthesis. Literal-only or codec-portfolio implementations fail this condition.
8. Both mandatory public benchmark plans are complete and reproducible, including comparator failures as failures where a plan required completion, and all successful outputs round-trip exactly.
9. Every planted defect is caught by its intended observing gate; no skipped/ignored/stale result is counted as evidence.
10. The complete `just vv` and actual PrismPM application acceptance pass. The generated release inventory binds every source, dependency, schema, executable, test/oracle result, proof, package, and relevant license.
11. Every public statement reports its true scope. Implementation acceptance, losslessness, scoped minimum, benchmark improvement, and external record acceptance are distinct.

Version numbers use independent axes: product/package SemVer, archive version, reference-machine profile, source-language identity, and evidence schema version. A new opcode/changed arithmetic/resource semantics requires a new machine profile; a changed wire interpretation requires a new archive version. Search heuristics may evolve without changing decoder semantics, but their source identity, deterministic budget behavior, test expectations, and evidence bindings change explicitly.

A release ships its generated library/client, supported-platform inventory, decoder compatibility statement, `.holo`/Core-Wasm artifacts, source and proof manifests, authoritative import licenses, canonical schemas, fixture vectors, complete acceptance receipt, and benchmark reports. Release publication is an explicit authorized action, not an automatic side effect of running tests.

## Appendix A. Exact query and identity descriptors

### A.1 Canonical metadata scalar conventions

`D` below is a JSON string containing canonical unsigned decimal U64, never a JSON number. `H` is exactly 64 lowercase hexadecimal characters. `BytesHex` is an even-length lowercase hexadecimal string. Every listed object has exactly the listed fields; no duplicate, absent, unknown, or null substitute is allowed unless `null` is explicitly admitted. Canonical bytes are UTF-8 restricted JCS with no final newline. A human-readable log may add a newline outside those bytes; a hashed descriptor may not.

### A.2 Decode envelope

The exact JSON field names are:

```text
{
  "schema": "uorc/decode-envelope/1",
  "maxArchiveBytes": D,
  "maxOutputBytes": D,
  "maxBasisBytes": D,
  "maxBlocks": D,
  "maxInstructions": D,
  "maxParameters": D,
  "maxFrames": D,
  "maxSteps": D,
  "maxLiveCells": D
}
```

Validation and the named profile values are as in §§5 and 16.2. A basis archive is resolved using this same envelope with its `maxArchiveBytes` replaced by `maxBasisBytes`, self-contained scope required. `maxOutputBytes` separately bounds the resolved basis aggregate and the requested output aggregate. Logical live-cell accounting includes any basis retained during the main decode. Basis decoding work is added once to complete conditional-decoding work; the peak is the maximum of basis preparation and main decoding with the retained basis. Decoder limits never authorize recursive basis resolution.

### A.3 Universe descriptor

```text
{
  "schema": "uorc/universe/1",
  "machineProfile": "uorc/reference-machine/1",
  "machineId": H,
  "archiveFormat": "uorc/archive/1",
  "decodeEnvelope": <A.2 object>,
  "scope": "self_contained" | "conditional",
  "basisId": null | H,
  "outputLengths": [D, ...],
  "maximumArchiveBytes": D,
  "representations": ["raw", "graph"]
}
```

`outputLengths` is nonempty. `basisId` is null exactly in self-contained scope. `maximumArchiveBytes` is at most the envelope's archive maximum and equals the feasible raw-frame ceiling for ordinary compression. The `representations` list is exactly the displayed ordered pair; no caller-controlled subset is admitted by this production schema. Restricted micro-universes use a distinct validation-only schema and cannot appear in a production scalar certificate.

### A.4 Query descriptor

```text
{
  "schema": "uorc/query/1",
  "universe": <A.3 object>,
  "inputContentId": H,
  "objective": "archive_bytes",
  "tiePolicy": "size_only" | "wire_lexicographic",
  "requestedClaim": "best_found" | "minimum" | "canonical_minimum"
}
```

For a `UORP` certificate, `requestedClaim` is `minimum` or `canonical_minimum` and matches its header tag; `tiePolicy` is respectively `size_only` or `wire_lexicographic`. A general compression request may use `best_found`, with `wire_lexicographic` for deterministic incumbent selection.

The verifier receives the actual input `X`, actual supplied basis bytes, and chosen archive. It recomputes identities, lengths, candidate eligibility, and claim coverage with those actual values. Its formal soundness theorem quantifies over those values; it never infers their equality from digest equality. A receipt without original input cannot assert full reconstruction comparison.

Corpus, conditional, and ordinary archive minimization all use the same primary `archive_bytes` objective with their fixed context. Standalone/proof-delivery measurements add §9.2's costs but do not thereby become optimization certificates for a varying decoder/proof language. A request to optimize those larger spaces is outside this scalar certificate profile, not a silently accepted option.

### A.5 Other identity encodings

For identity domains whose payload is one binary blob or one canonical descriptor, use `domain || 00 || V(payloadLength) || payload`. `ContentId`, `BasisId`, and `ArchiveId` use their already explicitly specified preimages instead, without an additional duplicate domain or length prefix. `QueryId` is the hash of the complete A.4 canonical descriptor under `UORC-QUERY-1`; `UniverseId` hashes A.3 under `UORC-UNIVERSE-1`.

`MachineId` hashes the generated canonical descriptor containing the exact type table, opcode field/type/denotation table, frame/integer rules, error-order rules, and abstract resource rules of this profile. That descriptor contains no compiler hash, scheduler setting, benchmark, date, or measurement. UORC source/compiler/runtime identities are separate evidence bindings. An implementation optimization cannot change machine rules while retaining their identity.

The machine descriptor has schema `uorc/machine-description/1` and fields `schema`, `profile`, `types`, `operations`, `wire`, `admission`, `errors`, and `resources`. Each component is the canonical generated projection of its corresponding closed model datatype, with arrays in numeric tag order. Those projection schemas and their complete generated bytes are locked release artifacts. No arbitrary free-form source string supplies an operation's meaning.

### A.6 Search and verifier resources

A search policy has schema `uorc/search-policy/1` and exactly `schema`, `totalWork`, `phaseQuantum`, `maxRetainedTasks`, `maxCandidateBytes`, `maxCheckpointBytes`, `certificateWork`, `checkerWork`, and `deterministic`. All counts are `D`; `deterministic` is true for the standard profiles. Work may be zero to request the raw incumbent only; positive memory limits must accommodate that incumbent.

A verifier policy has schema `uorc/verifier-policy/1` and exactly `schema`, `maxCertificateBytes`, `maxProofNodes`, `maxProofDepth`, `maxWork`, and `maxLiveCells`. It is operational input, never a field that removes competing archives from A.3.

Default native verifier limits are 268435456 certificate bytes, 1048576 proof nodes, 4096 proof depth, 100000000 modeled verifier steps, and 17179869184 abstract live cells. Default native retained-task count is 1048576, candidate bytes equal the selected archive envelope, and checkpoint bytes are 1073741824. Default certificate/checker work equals 100000000 each when explicitly requested. A default View does not request certificate production, and retains at most 4096 tasks with 1048576 checkpoint bytes if a typed core caller requests a checkpoint. It does not expose checkpoint files through its text UI.

## Appendix B. Exact prefix-certificate payload

### B.1 Improving region

Let `a` be the selected eligible archive and `L` the sealed ceiling. For `minimum`, define the comparison region as all byte words `w` with `length(w) < length(a)` and `length(w) <= L`. For `canonical_minimum`, additionally include words of length `length(a)` that are unsigned-lexicographically smaller than `a`.

Call that finite set `I`. It includes malformed/ineligible words; their rejection is part of coverage. For a prefix `p`, `R(p) = {w in I : p is a prefix of w}`. The root prefix is empty. The checker can determine whether `R(p)` is empty from lengths and lexicographic comparison alone; it does not consult discovery.

### B.2 Table layout

For route 01, the payload is:

```text
V(node_count), node[0], ..., node[node_count-1]
```

The count is positive. Each node begins with a tag byte and `Blob(prefix)`, where `Blob(b)=V(length(b))||b`. Its remaining fields are exactly:

| Tag | Constructor | Remaining fields |
|---|---|---|
| `00` | `RejectPrefix` | one reason byte from B.3 |
| `01` | `BoundPrefix` | `V(claimedLowerBound)` |
| `02` | `Terminal` | one disposition byte from B.4 |
| `03` | `Split` | one disposition byte; `V(childCount)`; `V(distance)` for each child |

For node `j`, every distance is positive and at most `j`, denoting `node[j-distance]`. No forward/cyclic node is admitted. The last node is the root and has empty prefix. Every node must be reachable from it. Serialize the proof in depth-first postorder, traversing child prefixes in ascending next-byte order; no unused duplicate node or alternative topological spelling is canonical in v1. Prefixes identify nodes uniquely in this tree profile.

### B.3 RejectPrefix reasons

The reason byte is one of:

| Reason | Checker-recomputed permanent condition |
|---|---|
| `00` | `R(p)` is empty |
| `01` | a consumed fixed header/profile/scope byte contradicts the frame/query |
| `02` | a consumed integer prefix has irrevocable overflow/nonminimal termination/invalid width |
| `03` | a completed count or declared length irrevocably contradicts query shape, framing, or static envelope |
| `04` | a consumed opcode or type code is unknown |
| `05` | a fully consumed local/block reference is invalid at that fixed location |
| `06` | known argument/signature/entry types already contradict typing |
| `07` | the prefix already contains at least one byte after a completely delimited frame |

There are no reason arguments. The checker reparses the prefix against the query and derives the stated condition. It must prove that any extension in `R(p)` cannot be eligible. A prefix ending exactly at a potentially valid frame is **not** rejected by reason 07, because that terminal may be eligible. A truncated field is `NeedMore`, not reason 02/03. Evaluation of one completion cannot supply a permanent condition for all others.

When more than one permanent reason is derivable, the canonical reason is the lowest numeric reason. A producer's wrong reason is rejected even if another rule could have rejected the region; this keeps certificates reproducible.

### B.4 Terminal dispositions

| Disposition | Exact recomputed condition |
|---|---|
| `00` | `p` is outside `I`; no candidate evaluation is needed |
| `01` | `p` is in `I` but is not admitted by the complete bounded resolution/admission relation |
| `02` | `p` is in `I`, admitted, and its actual resolved objects differ from the supplied target |

No other value is accepted. An eligible `p` in `I` refutes the certificate with `UC4005`. A physical timeout, unavailable basis, or exhausted checker budget is not disposition 01; it returns operational failure/incomplete. Ineligibility due to the **formal** decode envelope is decided by that exact machine relation, not by a host watchdog.

A `Terminal` node is accepted only if every `R(p)` word is the word `p` itself. Equivalently, the checker derives that every next-byte prefix region is empty. `Split` uses the same disposition for its own terminal, then covers its extensions.

### B.5 Split validation

From `p`, derive the ordered list `C = [b in 0..255 : R(p||b) is nonempty]`. `childCount` must equal `length(C)` exactly. The kth child must have prefix `p||C[k]`, and its backward distance must resolve to that exact node. Missing, duplicate, misordered, or wrong-prefix children fail.

`Split` requires nonempty `C`; otherwise canonical encoding uses `Terminal`. Its verified conclusion is the union of its terminal disposition and the checked child regions. This union is exact and disjoint. Successful processing of only the available children does not suffice.

### B.6 BoundPrefix validation

The checker recomputes a structural lower bound `lb(q,p)`; it must equal the claimed U64 value. It accepts the bound only when every word of length at least that bound within `R(p)` is incapable of improving the chosen archive under the requested size/tie policy.

A sufficient rule is `lb >= length(a)` for `minimum`, and `lb > length(a)` for `canonical_minimum`. Canonical mode may also use the independently proved fact that no same-length extension of `p` precedes `a`; the length/lexicographic calculation is recomputed. Never assume that all equal-length archives are interchangeable.

The baseline bound is the maximum of:

- `length(p)`;
- the query's unavoidable frame overhead plus the minimum possible body representation;
- bytes already committed by parsed counts/section lengths and still-required fields.

For a query with total output `N`, object lengths `n_i`, and optional basis-field length `b` (0 or 32), a valid completion has at least

```text
8 + vlen(k) + sum(vlen(n_i)) + b + 32
  + min(vlen(N) + N, vlen(7) + 7)
```

bytes: raw mode has `N` body bytes; the shortest possible well-typed graph body has at least seven bytes. Once representation/body length is consumed, use its exact applicable lower bound instead of the minimum across alternatives. A graph body of seven bytes can produce empty bytes but is not thereby assumed capable of producing the query's arbitrary target.

Outstanding known literal payload lengths and remaining declared fields can raise the bound. For unknown field values use their minimum permitted encoding length, not a guessed favorable value. Avoid double-counting body bytes already included in a known body length. Implement the lower-bound ledger as disjoint byte intervals/reservations and prove it never exceeds a valid completion's actual extent.

This structural bound may be weak. Weakness costs search time; an unjustified strong bound breaks correctness. No entropy estimate, semantic hash, learned prediction, or observed best ratio may replace it.

### B.7 Proof result

Validate the chosen archive's actual eligibility before checking coverage. Check the complete table under explicit checker resources. Success means every `R(empty)` word is ineligible; together with chosen eligibility, this establishes the requested independent minimum statement. A malformed proof fails; an insufficient checker budget returns incomplete. Neither state emits `minimum_certified`.

## Appendix C. Concrete wire and generative-reference examples

These examples fix byte-level interoperability. Their hashes are calculated over the specified preimages and must be independently checked by the implementation's imported primitive oracles. They are fixtures, not claims that UORC has already been implemented.

### C.1 Empty object

Object lengths: `[0]`. Archive length: **43 bytes**.

Complete archive, lowercase hexadecimal:

```text
554f52430101000001000065949c484a11ad21ef30a7ac31cc350873be038a2dbefda610756b563efdbfb9
```

Content-integrity field: `65949c484a11ad21ef30a7ac31cc350873be038a2dbefda610756b563efdbfb9`.

### C.2 One zero byte

Object lengths: `[1]`. Archive length: **44 bytes**.

Complete archive, lowercase hexadecimal:

```text
554f5243010100000101010067cb5df740199b66400f2dd332cd4f529b598574607dc3ceef00d15d70257f20
```

Content-integrity field: `67cb5df740199b66400f2dd332cd4f529b598574607dc3ceef00d15d70257f20`.

### C.3 Ordered empty and nonempty objects

Object lengths: `[0, 2, 0]`. Archive length: **47 bytes**.

Complete archive, lowercase hexadecimal:

```text
554f524301010000030002000200ff242c57d3a63e03c4b2f56a4781c2705052b442e6080e1aa7362c0c55e3c9be6a
```

Content-integrity field: `242c57d3a63e03c4b2f56a4781c2705052b442e6080e1aa7362c0c55e3c9be6a`.

### C.4 A constant byte-valued generator

Object lengths: `[64]`. Archive length: **61 bytes**.

Graph body:

```text
020100010101410100030200401b01010001
```

Block 0 takes one `N` coordinate and returns a `B`. It contains `ByteLiteral 65`. Block 1 contains `NatLiteral 64` and `Tabulate` of block 0, with no captures. It returns 64 copies of byte `41`. There is no repetition opcode or free generator dictionary.

The corresponding raw frame has 107 bytes; this explicitly encoded generator frame has 61 bytes. This is a calculated witness size, not a measured compressor discovery result or an optimality certificate.

Complete archive, lowercase hexadecimal:

```text
554f524301010001014012020100010101410100030200401b010100016956b670a250c22d69ee3fd406d8dc912c387320fd328406f18a5c4e11e52347
```

Content-integrity field: `6956b670a250c22d69ee3fd406d8dc912c387320fd328406f18a5c4e11e52347`.

### C.5 An address-derived byte sequence

Object lengths: `[256]`. Archive length: **63 bytes**.

Graph body:

```text
02010001010c01010003020080021b01010001
```

Block 0 takes one `N` coordinate and returns a `B`. It contains `ByteOfNat` applied to its parameter. Block 1 tabulates it over count 256. The result is the ordered byte sequence `00` through `ff`. The count 256 uses the two-byte integer `8002`.

The corresponding raw frame has 301 bytes; this explicitly encoded generator frame has 63 bytes. This is a calculated witness size, not a measured compressor discovery result or an optimality certificate.

Complete archive, lowercase hexadecimal:

```text
554f5243010100010180021302010001010c01010003020080021b010100018745a7d079e52929307bdab2be1759ccbe17491cfd40bd51c54e12ae2f68143d
```

Content-integrity field: `8745a7d079e52929307bdab2be1759ccbe17491cfd40bd51c54e12ae2f68143d`.

### C.6 Canonicality negatives

For C.1, replace the single `01` object-count byte immediately after the eight-byte fixed header with `8100`: reject `UC2003`, not an alternate valid empty archive. Remove the last integrity byte: reject `UC2001`. Append `00` after the complete archive: reject `UC2004`. Change the machine-profile byte to `02`: reject `UC2002`. These cases reach different owners and must not be conflated into one generic error.

For C.4, change the entry Tabulate block distance from `01` to `00`: reject `UC2006`. Change the generator result type from `01` (byte) to `03` (bytes), without changing its body: reject `UC2007`. Change only the literal `41` to `42`, retaining the old integrity field: decoding fails integrity; it must not publish reconstructed bytes as accepted.


## Appendix D. Deterministic resource-ledger elaboration

This appendix fixes the accounting choices needed to elaborate §8.5. It describes an abstract reference machine, not the physical allocations or instruction count of any particular compiler output.

### D.1 Work units

All ledger arithmetic is mathematical nonnegative integer arithmetic. Before a runtime U64 counter update, test whether it would exceed its finite bound without overflowing the counter. Exceeding the bound returns `DecodeLimitExceeded`; arithmetic wrap is not allowed in bookkeeping.

For a successful parse, parsing work is exactly the number of archive bytes consumed, including framing and the integrity field. The syntax scanner reads bytes left to right; every consumed byte increments that counter once. Inspecting an already parsed value is not another parse-byte read.

Static validation visits a canonical list of obligations, in this order:

1. one frame-shape obligation;
2. one object-length obligation per object, in order;
3. one block-signature obligation per block, in order;
4. one parameter-type obligation per parameter;
5. one instruction-signature obligation per instruction;
6. one operand-reference/type obligation per encoded local operand;
7. one earlier-block signature/capture obligation per Apply/Tabulate/Fold instruction;
8. one return-reference/type obligation per block;
9. one entry-signature obligation;
10. one body-consumption/EOF obligation.

Raw mode has no block/parameter/instruction/operand/return obligations and no entry-signature obligation. Each listed obligation charges one validation unit. A failed parse/validation reports work through its first failed obligation in this canonical order; implementation traversal shortcuts must simulate the same accepted-result ledger and preserve specified error priority.

Each dynamic instruction charges one dispatch unit and the sum of its local backward operand distances. An ordinary primitive (all opcodes other than `Apply`, `Tabulate`, `Fold`, and `BasisGet`) additionally charges the sum of its operand value weights and its result weight. Literal operands encoded as immediates are not local reads; their result weight still applies. Value weight is 1 for `N`, `B`, or `P`, and `1+length` for `S`.

Invoking an earlier block at distance `f` with arguments `args` charges `f + 1 + length(args) + sum(weight(args))`, followed by its instruction execution and return. A block return charges `1 + returnDistance + weight(returnedValue)`. Invoking the final entry from the outer machine charges one unit, with no artificial backward distance or arguments.

For `Apply`, after dispatch and operand reads, add the invocation cost and `weight(result)` for installation in the caller. For `Tabulate`, add the count `n`, all `n` body invocations, one unit per byte installed, and the final result weight. For `Fold`, add the count `n`, all `n` body invocations, the weight of each newly produced state at replacement, and the final result weight. The already-read captures/count/seed contribute their weights once when the iteration control record is created. A zero-count iterator pays that creation and final result-installation cost but makes no body invocation.

`BasisGet(k)` charges dispatch, `k+1` logical list-lookup units, and result weight. Basis preparation is not repeated on every lookup, but its complete preparation work is included once for the operation as specified in A.2.

After producing the root value, splitting charges one unit per object boundary and one per output byte installed in the resulting object sequence. Raw mode charges one initial dispatch plus one per body byte copied before that same split. Integrity digest comparison charges 32 comparison units. SHA-256 input processing has its separate exact bytes-hashed ledger; do not add an unspecified hash cost to `abstractDecodeSteps`.

Thus `abstractDecodeSteps` is the sum of parse, validation, instruction, reference, call/iteration, raw-copy, splitting, and digest-comparison units above. It explicitly excludes SHA-256 internal work, physical IO, compiler/runtime setup, and host allocation overhead. Every report includes the exclusions and the separate hashing count. Any claim concerning **all** decoder execution cost must first supply a machine-specific bridge for those excluded activities.

### D.2 Live cells

The reference state retains the immutable archive bytes and, in conditional decoding, the supplied basis archive and resolved basis values. It also has an explicit list of call frames and iterator controls. A frame contains its block index, program counter, local values, and return continuation. A control contains its iteration index/bound, explicit captures, old state or growing output, and pending child result when present.

Count one administrative cell per active frame/control and the complete value weight of every typed value stored in each frame/control. Count archive bytes once, supplied basis archive bytes once, each retained basis value by its value weight, and each pending final output buffer byte once. Parsed type/opcode records are represented by the archive-byte component of this abstract model, not by a claim about their physical object size. The final ordered result uses one administrative cell per object plus its byte length.

Slots are retained until their frame exits; v1's abstract machine does not opportunistically garbage-collect dead local slots. Physical sharing/copy elimination is allowed only as a refinement that preserves this logical ledger. Two slots aliasing one physical buffer are still two logical stored values.

At a call, the caller remains live while the callee stores its arguments. At a return, the returned value is pending while the callee frame is discarded, then installed in the caller; measure the peak at each transition. At an iterator step, old accumulator/captures remain while the child runs. Replacing an accumulator removes the old stored value only after obtaining and accounting for the new one. Tabulation retains its growing result throughout. Measure peak cells before and after every materialization or transfer; a buffer may not be omitted because it lives only for one transition.

The explicit transition state and this weight function determine peak abstract cells. The verifier cannot substitute observed allocator statistics for it. Every transition constructor has a test with its before/after live set and work increment; every such test is tied to `resource_accounting_exact`.

### D.3 Encoder and checker work

An encoder/checker task is a resumable state machine, not an opaque callback. One logical step advances exactly one typed task transition: queue operation, parser transition, typing obligation, primitive/reference-machine transition, substitution traversal node, layout transition, incumbent comparison byte, or coverage-node/child check. A transition that iterates over a collection must expose that iteration as successive steps rather than charging one unit for unbounded work.

No invocation may hide a complete large candidate decode behind a single search-work unit. A yielded task stores its exact cursor. The same step definition drives budget exhaustion, deterministic scheduler quanta, and resumed-run equivalence. Its source identity is part of the encoder/checker policy evidence, not the decoder's semantic universe.

## Appendix E. Authoritative source register for implementation

The sources below identify the authorities and roles used by this brief. They are **acquisition origins**, not permission for floating dependencies. At setup, resolve exact source/release bytes, read the applicable license and full interface, acquire the required vectors/tools, record actual SHA-256 digests, and freeze them under §20. An HTML documentation URL alone does not constitute an imported executable oracle.

### [A01] UOR template policy

Issuer: UOR Foundation. Role: inherited repository/SDK policy.

- `https://github.com/UOR-Foundation/template/tree/1bea460bac6ea50bae53a7eeb674589d7900e6cb`
- `https://github.com/UOR-Foundation/template/blob/1bea460bac6ea50bae53a7eeb674589d7900e6cb/AGENTS.md`
- `https://github.com/UOR-Foundation/template/blob/1bea460bac6ea50bae53a7eeb674589d7900e6cb/TEMPLATE-CONTRACT.md`
- `https://github.com/UOR-Foundation/template/blob/1bea460bac6ea50bae53a7eeb674589d7900e6cb/README.md`

### [A02] LexLean implementation/language contract

Issuer: UOR Foundation. Role: source language, compilation/verification interface, exact authority provenance.

- `https://github.com/UOR-Foundation/LexLean/tree/eaa9ab85bb66e21a99a761b6b94fdfd527145ca8`
- `https://github.com/UOR-Foundation/LexLean/blob/eaa9ab85bb66e21a99a761b6b94fdfd527145ca8/SPEC.md`
- `https://github.com/UOR-Foundation/LexLean/blob/eaa9ab85bb66e21a99a761b6b94fdfd527145ca8/model/authorities.toml`

### [A03] PrismPM application/SDK contract

Issuer: UOR Foundation. Role: actual compilation, application, transport, SDK, and oracle integration.

- `https://github.com/UOR-Foundation/PrismPM/tree/4aba0ca1bcdb4a990e9d2fbb27faff301465f83c`
- `https://github.com/UOR-Foundation/PrismPM/blob/4aba0ca1bcdb4a990e9d2fbb27faff301465f83c/SPEC.md`
- `https://github.com/UOR-Foundation/PrismPM/blob/4aba0ca1bcdb4a990e9d2fbb27faff301465f83c/model/dependencies.toml`
- `https://github.com/UOR-Foundation/PrismPM/blob/4aba0ca1bcdb4a990e9d2fbb27faff301465f83c/examples/Calculator/prismpm.toml`

### [A04] UOR-GNAF normative draft

Issuer identified by LexLean's authority manifest: UOR Foundation. Identifier: `uor-gnaf/1-draft.2`. Role: typed admission, global/complete-system accounting, quotient/envelope distinction, and claim boundaries.

- Authoritative copy used by the inspected LexLean revision: `https://github.com/UOR-Foundation/LexLean/blob/eaa9ab85bb66e21a99a761b6b94fdfd527145ca8/model/authorities/UOR-GNAF-v1-draft.2.md`
- Its cited downstream-vendored source: `https://github.com/afflom/wasm-gemm-gnaf/blob/917306fd2b5a397ab02c5d38918fb8620fcc5ae0/authority/UOR-GNAF-v1-draft.2.md`
- Exact acquired SHA-256 recorded by LexLean: `5c342373b2ff809bfd607c413cafd0582d32bb097544c6597ff7d674fe99200a`.

The inspected manifest explicitly says the issuer did not publish an upstream copy and identifies this pinned downstream copy. Preserve that provenance limitation; do not describe it as an independently fetched canonical issuer publication. UORC proves its profile's claims locally rather than treating imported prose as a proof.

### [A05] Lean kernel/toolchain

Issuer: Lean project / Lean FRO. Role: formal proof authority and same-kernel replay.

- `https://github.com/leanprover/lean4/tree/f054605aea4b840552cca2e725580bffd1e1b704`
- `https://github.com/leanprover/lean4/releases/tag/v4.32.1`

Use the actual SDK-selected toolchain and complete source/binary identity after bootstrap. This researched source pin explains the observed baseline; it is not permission to ignore a reviewed later toolchain update.

### [A06] WebAssembly binary integer specification

Issuer: WebAssembly specification project. Role: independent unsigned-LEB128 value/width reference, with UORC's stricter minimality rule separately checked.

- `https://webassembly.github.io/spec/core/binary/values.html#integers`
- `https://github.com/WebAssembly/spec`

### [A07] JSON Canonicalization Scheme

Issuer/publication: RFC Editor, RFC 8785. Role: evidence JSON serialization; this is an Informational RFC, not a product certification.

- `https://www.rfc-editor.org/rfc/rfc8785.html`
- Reference implementations and vectors maintained at `https://github.com/cyberphone/json-canonicalization`.

Acquire an actual maintained implementation/vector set independently of UORC. Imported validation source may be in another language; it is not UORC production implementation.

### [A08] NIST secure-hash vectors

Issuer: NIST CSRC/CAVP. Role: authoritative SHA-256 test-vector oracle.

- `https://csrc.nist.gov/projects/cryptographic-algorithm-validation-program/secure-hashing`

Acquire the byte-oriented SHA test-vector archive linked by that page and its README. Select the complete SHA-256 short/long/Monte Carlo sets applicable to the supported byte interface. NIST explicitly says use of test vectors does not replace CAVP validation.

### [A09] Z3

Issuer: Z3 project. Role: independent finite arithmetic and bounded constraint solver.

- `https://github.com/Z3Prover/z3`
- `https://microsoft.github.io/z3guide/docs/theories/Bitvectors/`

### [A10] cvc5

Issuer: cvc5 project. Role: independently maintained SMT solver for the same bounded validation questions.

- `https://github.com/cvc5/cvc5`
- `https://cvc5.github.io/docs/latest/proofs/proofs.html`

Its available proof output does not automatically constitute a Lean-checked UORC certificate. An additional verified translation/checker would be a separately specified capability; version 1 does not trust an unverified proof-text import.

### [A11] WebAssembly reference interpreter and tests

Issuer: WebAssembly specification project. Role: independent reference execution and official feature test suites.

- `https://github.com/WebAssembly/spec/tree/main/interpreter`
- `https://github.com/WebAssembly/spec/tree/main/test`

The interpreter's README states its validation/execution/script behavior. Resolve its main or appropriate version to an immutable compatible source commit; do not run an unpinned rolling reference build in acceptance.

### [A12] Wasmtime

Issuer: Bytecode Alliance / Wasmtime project. Role: independent production WebAssembly runtime.

- `https://github.com/bytecodealliance/wasmtime`
- `https://docs.wasmtime.dev/`

### [A13] External byte comparison and hashing utilities

Issuer: GNU project. Role: whole-file comparison and independently executable hashing for transport/result validation.

- `https://www.gnu.org/software/diffutils/`
- `https://www.gnu.org/software/diffutils/manual/html_node/Invoking-cmp.html`
- `https://www.gnu.org/software/coreutils/`

The GNU manual endpoint was not successfully retrieved while drafting this brief. Setup MUST acquire and validate the actual official release documentation/binaries before using this authority row as executable evidence. This is not a claim that a GNU oracle has already run.

### [A14] Text compression corpus identity

Issuer: corpus/benchmark maintainer Matt Mahoney. Role: exact public corpus bytes and historical identity, not an assumed current leaderboard.

- `https://mattmahoney.net/dc/textdata.html`

### [A15] Silesia corpus

Issuer: Silesia corpus maintainer/site. Role: heterogeneous public corpus distribution and inventory.

- `https://sun.aei.polsl.pl/~sdeor/index.php?page=silesia`

### [A16] Zstandard comparator

Issuer: Zstandard project / Meta. Role: measurement-only compressor/decompressor.

- `https://github.com/facebook/zstd`
- `https://facebook.github.io/zstd/`

### [A17] Brotli comparator

Issuer: Google/Brotli project. Role: measurement-only compressor/decompressor.

- `https://github.com/google/brotli`

### [A18] XZ comparator

Issuer: Tukaani/XZ project. Role: measurement-only compressor/decompressor.

- `https://tukaani.org/xz/`

Review current official release/security information during acquisition, then freeze the exact chosen release and documented settings. No mutable executable download is allowed during tests.

### [A19] Normative requirement terminology

Publication: BCP 14, RFC 2119 and RFC 8174. Role: interpretation of uppercase requirement words.

- `https://www.rfc-editor.org/rfc/rfc2119.html`
- `https://www.rfc-editor.org/rfc/rfc8174.html`

### [A20] cmix comparator

Issuer: Byron Knoll / cmix project. Role: high-ratio measurement-only comparator.

- `https://www.byronknoll.com/cmix.html`

Acquire the release/source that page actually publishes, its license, and documented invocation. Do not infer present record status from the project's purpose or an older result table.

### E.1 Source limitations and evidence obligation

These sources establish interfaces, authority identities, and reference behavior used to design the contract. This document does not ship acquired oracle binaries, corpus archives, SDK images, proof objects, or successful UORC test results. The agent must acquire, lock, invoke, and report those actual artifacts during implementation. Missing evidence prevents acceptance; it must never be filled with fabricated digests, copied success logs, or assertions that the design itself proves the implementation.

---

**Final implementation rule:** construct the smallest exact reference the implemented search can actually justify; preserve every byte and every charged dependency; expose the exact scope of every claim; derive the application, proofs, diagnostics, tests, and documentation from one LexLean authority through the locked PrismPM SDK.
