Feature: lexeme

  The lexeme ledger and the PIRTM stratification layer above it: canonical
  identity, authorship, external time, an append-only transparency log, and
  the prime-indexed stratum receipt that sits above all of it (§33, §34).

  @LG-01 @build
  Scenario: Canonicalization discards comments and layout and orders declarations by fully qualified name, so two sources differing only in those respects have one canonical form and one content digest.
    Given two Lean sources differing only in comments, layout, and declaration order
    When each is canonicalized and its content digest taken
    Then the two sources share one canonical form and one digest
    And a difference in any declaration body changes the digest

  @LG-02 @build
  Scenario: The canonical form is the §21.1 frame encoding under the `lexlean-lexeme-v1` domain, so the same source hashes identically in two distinct build directories and across two runs.
    Given the same Lean source in two distinct build directories
    When it is canonicalized twice and the §21.1 frames digested
    Then both digests are identical and equal the `lexlean-lexeme-v1` frame digest
    And a run in a third directory reproduces it again

  @LG-03 @build
  Scenario: The content-hash record names the source paths, the canonicalization identifier, the toolchain, and the sorted declaration names, and every one of those fields changes the digest when changed.
    Given a content-hash record naming its sources, canonicalization identifier, toolchain, and sorted declarations
    When each field in turn is altered and the record re-digested
    Then every field is named in the record
    And every one of the four alterations changes the digest

  @LG-04 @build
  Scenario: An entry carries a detached Ed25519 signature over its 32-byte content digest, and verification refuses a digest altered by one bit or a signature checked against a public key other than the one recorded.
    Given an entry carrying a detached Ed25519 signature over its 32-byte content digest
    When the signature is verified, then the digest is flipped by one bit and the signature checked against another key
    Then the untampered signature verifies under the recorded key
    And a one-bit digest change and a foreign public key are each refused with LLG1003

  @LG-05 @build
  Scenario: A hardware-bound key records its device, slot, algorithm, and pin policy, and signing fails when the device is absent rather than falling back to a software key.
    Given a hardware-bound key recording its device, slot, algorithm, and pin policy
    When signing is attempted with the device absent
    Then the key record names the device, slot, algorithm, and pin policy
    And signing fails with LLG1004 rather than falling back to a software key

  @LG-06 @build
  Scenario: An entry's RFC 3161 token is parsed for its `TSTInfo`, and verification refuses a token whose message imprint names another algorithm, another digest, or another artifact.
    Given an entry carrying an RFC 3161 token
    When the token is parsed for its TSTInfo and the imprint is checked against the entry's artifact
    Then the imprint names the entry's algorithm, digest, and artifact
    And an imprint naming another algorithm, digest, or artifact is refused with LLG1005

  @LG-07 @build
  Scenario: The token's signature verifies under the certificate the entry carries, that certificate verifies under the pinned TSA root in the same entry, and the root's validity interval contains the token's `genTime`.
    Given a token, the TSA certificate it names, and a pinned TSA root
    When the token's signature is verified under the certificate and the certificate under the root
    Then both signatures verify and the root's validity contains the token's genTime
    And a broken chain or an interval excluding genTime is refused with LLG1006

  @LG-08 @build
  Scenario: Leaf and internal hashes use the RFC 6962 domain separation `0x00` and `0x01`, so a leaf digest is never an internal node digest and the tree admits no second preimage through the structure.
    Given leaf and internal hashes of a two-leaf tree
    When both node kinds are hashed under their RFC 6962 domain prefixes
    Then the leaf uses 0x00 and the internal node 0x01, and no leaf digest equals an internal digest
    And the tree admits no second preimage through its structure

  @LG-09 @build
  Scenario: Appending an entry returns the new tree size, the appended leaf index, and an O(log n) audit path, and the stored head of every prefix equals the root recomputed from that prefix's entries.
    Given a ledger of committed entries
    When each is appended and the tree size, leaf index, and audit path returned
    Then every append reports the new size, the index, and an O(log n) audit path
    And the stored head of every prefix equals the root recomputed from that prefix

  @LG-10 @build
  Scenario: An inclusion proof is accepted exactly when the recomputed root equals the published root, and is refused for a wrong-length path, a reordered path, or an index at or beyond the tree size.
    Given an inclusion proof for a committed leaf
    When it is verified, then re-verified with a wrong-length path, a reordered path, and an index at the tree size
    Then the proof is accepted exactly when the recomputed root equals the published root
    And each of the three defects is refused with LLG1007

  @LG-11 @build
  Scenario: Two published tree heads verify as consistent exactly when the consistency proof from the smaller size recomputes the larger root from the smaller root, so a forked history is detectable from the heads alone.
    Given two published tree heads of sizes n and m greater than n
    When the consistency proof from n to m is verified against the smaller root
    Then the heads are consistent exactly when that proof recomputes the larger root
    And a forked history is refused with LLG1008

  @LG-12 @build
  Scenario: `lexlean lexeme verify` reports canonical hash, signature, timestamp, and inclusion separately and emits a verdict asserting only existence at a stated time, authorship by a stated key, and integrity since.
    Given a committed ledger entry
    When `lexlean lexeme verify` is run over it
    Then canonical hash, signature, timestamp, and inclusion are reported separately
    And the verdict asserts only existence at a stated time, authorship by a stated key, and integrity since

  @LG-13 @build
  Scenario: Every entry is validated against `schemas/lexeme-entry.schema.json` before it is appended, and an entry failing validation is refused with a registered diagnostic code.
    Given an entry and the closed `lexlean/lexeme/1` schema
    When the entry is validated against the schema and then appended
    Then a schema-valid entry is appended
    And an entry failing validation is refused with a registered diagnostic code

  @LG-14 @build
  Scenario: Re-verifying a committed ledger directory reproduces every verdict and every published root without network access.
    Given a committed ledger directory
    When it is re-verified with the network unavailable
    Then every verdict and every published root is reproduced
    And no network access is required

  @LG-15 @build
  Scenario: The browser verifier and the CLI reach the same verdict for every entry of the committed corpus, checked by running both against the recorded roots.
    Given the committed ledger corpus and its recorded roots
    When the browser verifier and the CLI are each run over every entry
    Then both reach the same verdict for every entry
    And the verdicts are equal against the recorded roots

  @LG-16 @build
  Scenario: The first ledger entry is the ledger's own Lean specification, its artifacts carry the digests of the CLI and the browser verifier, and its entry is signed, timestamped, and included like any other.
    Given the ledger's own Lean specification
    When the ledger is bootstrapped
    Then the first entry is that specification and its artifacts carry the digests of the CLI and the browser verifier
    And its entry is signed, timestamped, and included like any other

  @LP-01 @build
  Scenario: The snapaddr is the SHA-256 of the §34.1 frame encoding under `lexlean-pirtm-v1` and is recomputable from the entry's own fields.
    Given a canonicalized lexeme with its prime index and atom count
    When the §34.1 frames are built under `lexlean-pirtm-v1` and digested
    Then the snapaddr is the digest of those frames and is recomputable from the entry's own fields
    And a verifier holding only the entry reaches the same `snapaddr:<64hex>`

  @LP-02 @build
  Scenario: The prime index is the least prime not below the atom count, is itself prime, and a lexeme with no declarations is refused.
    Given lexemes with one, two, and three declarations and one with none
    When the least prime not below each atom count is computed and checked for primality
    Then each prime index is the least prime not below the count and is itself prime
    And a lexeme with no declarations is refused with LLG1011

  @LP-03 @build
  Scenario: Two sources differing only in comments, layout, and declaration order have one snapaddr, and changing any atom body changes it.
    Given two sources differing only in comments, layout, and declaration order
    When both are snapaddressed, and then one atom body is altered
    Then the two differing sources share one snapaddr
    And altering any atom body changes it

  @LP-04 @build
  Scenario: The reference adjacency matrix is the non-negative integer matrix of whole-token references over each atom's body proper in canonical atom order, and is recomputable from the canonical form.
    Given a stratum whose atoms reference one another
    When the whole-token reference relation is built in canonical atom order
    Then the adjacency matrix is the non-negative integer reference matrix
    And recomputing it from the canonical form gives the same matrix

  @LP-05 @build
  Scenario: Each contraction factor is the exact rational `1/(1+t)` for its atom's body proper token count, with numerator one.
    Given a stratum with a known body token count per atom
    When a contraction factor is formed for each atom
    Then each factor is the exact rational 1/(1+t) with numerator one
    And an empty body yields the factor 1/1

  @LP-06 @build
  Scenario: The norm is the maximum absolute column sum of `A·diag(λ)` computed in exact rationals and reported as a reduced pair, and no float appears in a receipt.
    Given a stratum's adjacency matrix and contraction factors
    When the gains A·diag(λ) are formed and the maximum absolute column sum taken
    Then the norm is an exact reduced rational pair equal to the maximum column sum
    And no floating-point value appears anywhere in the receipt

  @LP-07 @build
  Scenario: A receipt is accepted exactly when the norm is below one, a norm of exactly one is refused, and a refusal states the exact value.
    Given strata whose norms fall below, at, and above one
    When a receipt is formed for each
    Then a receipt is accepted exactly when the norm is below one
    And a norm of exactly one is refused with LLG1014 and states the exact reduced value

  @LP-08 @build
  Scenario: The receipt hash is LexLean's own domain-separated hash over the named frames, and an entry states that it is not the PIRTM `seal_hash`.
    Given an accepted and a refused receipt
    When the receipt frames are built under `lexlean-contractivity-v1` and digested
    Then the receipt hash is LexLean's own domain-separated digest of the named frames
    And the entry states that it is not the PIRTM `seal_hash`

  @LP-09 @build
  Scenario: The theorem anchor must name an atom of the stratum, so an anchor naming an absent declaration is refused.
    Given a receipt anchored to a declaration of the stratum and one anchored to an absent name
    When both anchors are checked against the stratum's atoms
    Then an anchor naming an atom is accepted
    And an anchor naming an absent declaration is refused with LLG1012

  @LP-10 @build
  Scenario: The Zeno-Finton gain is the exact rational `2^(-index)`, strictly decreasing, positive at every finite index, and never zero.
    Given entries at leaf indices zero through eight
    When the Zeno-Finton gain is formed for each
    Then each gain is the exact rational 2^(-index), strictly decreasing
    And every gain is positive at a finite index and none is zero

  @LP-11 @build
  Scenario: The Zeno-Finton signal never satisfies a §33.4 check, never appears in the §33.6 verdict, and an entry carrying only it is untimestamped.
    Given an entry carrying a Zeno-Finton gain and no RFC 3161 token
    When it is verified and the gain is offered in place of a timestamp
    Then the entry is reported untimestamped and the gain appears in no verdict
    And offering the gain as a time anchor is refused with LLG1015

  @LP-12 @build
  Scenario: Dropping every §34 field leaves every §33 check and the §33.6 verdict bit-identical, and an entry omitting the layer is a valid §33 entry.
    Given an entry carrying the full §34 layer beside its §33 fields
    When it is verified, and then again with every §34 field dropped
    Then every §33 check and the §33.6 verdict are bit-identical both times
    And an entry omitting the layer entirely is a valid §33 entry
