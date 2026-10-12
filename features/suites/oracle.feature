Feature: oracle

  The differential validation oracle: lean4-prod pinned as validation evidence only, its corpus and tables, its comparison of semantics and contracts, its planted and known differences, its failures, and its isolation from every identity (§17.19).

  @OR-01 @build
  Scenario: The differential oracle is lean4-prod, pinned in oracle/lean4-prod.toml by its remote, its exact commit, and the SHA-256 of the canonical frame of every file the integration uses, with the role validation-evidence and nothing of it vendored because its repository ships no licence text; a checkout is acquired only if those files hash to the pin and, in a git checkout, HEAD is the pinned commit, and a pin with another role or a missing, unknown, vendored, or malformed field, a checkout with one byte changed, one file added, removed, or renamed, an empty checkout, a pin whose Rust or Lean toolchain is not the repository's own, and an authority row or ledger claim that cites the oracle are each refused.
    Given the pin file, the digest frame, and a checkout of the oracle
    When each way a pin or a checkout can differ from the pin is presented to the acquisition
    Then each is refused, the toolchains are the repository's, the digest equals an independent transcription of its frame, and no authority or claim cites the oracle

  @OR-02 @build
  Scenario: The differential corpus is every production root of examples/production, examples/production-coverage, examples/models, and examples/reasoning, a minimal probe root for each runtime construct of the production registry that those roots do not realize in a root the oracle accepts, and the committed production-refusal fixtures with their marking removed; every construct of the registry is realized in the closure of a corpus root the oracle accepts and whose crate compiles or is a row of the known-non-overlap table, versioned by the digest of its rows, a formal-only or erased construct as such and a runtime construct with a probe whose refusal or failure to compile is re-asserted, and a construct in neither, a row for a construct a compiled root realizes, a row whose probe the oracle no longer answers in the stated way, and a row naming no construct of the registry are each refused.
    Given the production registry, the corpus, the table of known non-overlap, and the oracle
    When every construct is classified against the roots the oracle accepts and the table, and each kind of malformed table is presented
    Then every construct is realized in the closure of a compiled root or is a row whose answer still holds, and each malformed table is refused

  @OR-03 @build
  Scenario: On every corpus root the oracle accepts, the Rust it generates is built under the pinned rustc and called on the seeded inputs of the RB-06 and SP-11 differentials and on boundary inputs derived from the types and literals of the root, among them every constructor of every parameter type, the bounds of each integer width and their neighbours, and the delimiters of its strings, and its observable outcome, an overflow included, equals the interpreter's and LexLean's own rendering's, its signature, failure typing, and parameter passing equal the root's contract except where a known difference lists it, and no call of a root eligible for rust-core allocates or panics; the acceptance or refusal of every other root equals the verdict the tables record, and the oracle's refusal of each root that LexLean rejects is the kind its row states, so that a refusal for another reason is a disagreement, and the whole run is validated.
    Given every corpus root, the oracle, the pinned rustc, and the seeded and boundary inputs
    When the oracle's Rust and LexLean's rendering are each built, called on every input, and compared with the interpreter
    Then every outcome, contract, and resource invariant agrees except as the known differences list, and the run is validated

  @OR-04 @build
  Scenario: A planted arithmetic discrepancy (a checked operation made wrapping at a bound of its width, a division, negation, or addition changed, a shift past the width, a replaced operation, or an off-by-one literal or limit), a planted control discrepancy (swapped branches, an inverted comparison, a dropped recursive arm, a changed constant in a match arm, an append in the other order), a planted resource discrepancy (an allocation in a rust-core root), and a planted ABI discrepancy (swapped parameters, a changed parameter width, a dropped failure typing) in the oracle's generated Rust are each detected as a disagreement that records the root, the inputs, and both sides' outcomes or contracts, however many plants share a root, since each is built and judged under a key of its own, while the unplanted crates, a rewrite of each plant's site that means the same, and an edit that changes no behavior are not discrepancies.
    Given the oracle's generated Rust for roots with arithmetic, branches, and parameters
    When each discrepancy is planted in a copy of a crate, beside a rewrite of the same site that means the same, and the differential is run
    Then each plant is a disagreement carrying both sides' evidence, each rewrite that means the same is not, and the unplanted run is validated

  @OR-05 @build
  Scenario: Known deliberate differences are the rows of oracle/known-differences.toml, each with its kind, name, reason, and exact roots, and its rows and those of the known-non-overlap table are versioned by content, the version of each naming the digest of its rows in oracle/table-versions.toml; a difference of the run that no row names is a disagreement, a listed difference is accepted, and a row that lists a root on which the difference no longer arises, a row with no reason or no root, a repeated row, an unknown field, a row changed without a new version, a version raised without its digest recorded, and a version that changed nothing are each refused, so that every difference LexLean and the oracle have is stated, tested, and re-asserted on every run.
    Given the committed table of known differences and the differences of the run
    When rows are removed, added, and changed, versions are raised and not, and malformed tables are presented
    Then an unlisted difference is a disagreement, a listed one is accepted, a stale row, a row changed under its version, and a malformed table are refused

  @OR-06 @build
  Scenario: The differential compares outcomes, contracts, and resources and never bytes: a rendering that is deliberately better than LexLean's and semantically equivalent to it, a loop for a recursion and a closed form for an iteration, byte-different from LexLean's rendering and from the oracle's, is accepted on every input, and a rendering that differs in one result on one input, that takes a parameter of another width, that does more work than LexLean's as its own runtime counts it, or that allocates more as an allocator it cannot reach counts it is refused.
    Given LexLean's and the oracle's renderings of a root and a better equivalent rendering of it
    When each is built and called on every seeded input and its outcomes and contract are compared
    Then the better rendering is accepted despite its bytes and a rendering wrong on one input is refused

  @OR-07 @build
  Scenario: An oracle that was not acquired, whose checkout is not the pin, that did not build, that crashed, was killed, overran its time (and with it everything it started), wrote nothing or an IR defining nothing, or whose generator failed without refusing a program, a fetch that fails leaving nothing that a later run takes for a checkout, a cache that is not the pin discarded and fetched once more, a transient failure of the network tried again twice and a permanent one not, and a comparison over no root or no case, are each unavailable and never validated, which fails the gate on the normative host and is reported as not validated elsewhere, while the oracle's own structured refusals are answers and a tool that is not meant to overrun its time is given a generous one; and a disagreement fails closed with a record of both sides' evidence for every difference.
    Given oracle tools that fail in each way, checkouts that are not the pin, and runs that compared nothing
    When each is presented to the acquisition, the verdict, and the validation, and a disagreement is recorded
    Then each failure is unavailable, a structured refusal is an answer, nothing is validated, and a disagreement is recorded with both sides' evidence

  @OR-08 @build
  Scenario: The oracle is validation evidence and an input to no identity: no file the shipped crate, the language data, the schemas, the compiler project, the committed examples' records, or the authority and ledger files hold names the oracle, its pinned commit, or its digest, so that planting any of them in any of those places fails audit-oracle-isolation; no input of the compiler-semantics ID names it, an oracle field in a build manifest, a preservation record, an attestation, a snapshot, a compiler input, or a GNAF certificate fails its closed schema, and no module of the conformance crate but the oracle's own uses it, so that removing it changes none of them.
    Given the shipped crate, the language data, the schemas, the compiler project, the committed records, and the audit
    When the oracle's name, commit, and digest are planted in each and an oracle field in each record
    Then the audit and the schemas refuse each, and nothing an identity is computed from names the oracle
