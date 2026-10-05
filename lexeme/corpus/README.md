# The committed lexeme corpus

This directory is the corpus `SPEC.md` §33.10's `LG-14` and `LG-15` rows are
checked against. It is a §33.7 ledger directory plus the artifacts a third party
needs to re-check it without this repository's history.

```text
lexeme/corpus/
├── README.md          this file
├── entries/           each entry as published, with the inclusion proof of §33.5
│   ├── alpha.json
│   ├── beta.json
│   └── gamma.json
├── ledger/            the §33.7 ledger directory
│   ├── log.json       one canonical JSON entry per line, as appended (no inclusion)
│   ├── heads.json     one published head per append
│   └── roots.txt      one root hash per published head, in order
├── sources/           the Lean text each entry hashes
└── verdicts.json      the recorded verdict of every entry: the `LG-14` oracle
```

## What the three entries are

| Entry | Source | Declarations | Leaf | `genTime` |
| --- | --- | --- | --- | --- |
| `alpha` | `sources/alpha/Successor.lean` | 2 | 0 of 3 | `20261005030258Z` |
| `beta` | `sources/beta/Double.lean` | 2 | 1 of 3 | `20261005030525Z` |
| `gamma` | `sources/gamma/Parity.lean` | 3 | 2 of 3 | `20261005030526Z` |

Three entries, because a one-entry ledger has no consistency proof and no second
head, and §33.5's second-preimage argument is about a tree that already has
neighbours.

## How it was acquired

One pass, on 2026-10-05, with `lexlean lexeme` from this tree:

```bash
lexlean lexeme hash  --source lexeme/corpus/sources/alpha/Successor.lean \
                     --source-root lexeme/corpus/sources/alpha \
                     --title "Corpus alpha: successor and its witness" \
                     --toolchain leanprover/lean4:v4.32.1 --out alpha.json
lexlean lexeme sign  --entry alpha.json --seed alpha.seed --out alpha.json
lexlean lexeme stamp --entry alpha.json --request alpha.tsq --tsa https://freetsa.org/tsr
curl -H 'Content-Type: application/timestamp-query' --data-binary @alpha.tsq \
     https://freetsa.org/tsr -o alpha.tsr
lexlean lexeme stamp --entry alpha.json --tsr alpha.tsr \
                     --certificate tsa_certificate.der --root tsa_root.der \
                     --tsa https://freetsa.org/tsr --out alpha.json
lexlean lexeme append --ledger ledger --entry alpha.json --out alpha.included.json
```

The time anchors are genuine FreeTSA tokens, and each was verified in process
against the pinned root before it was recorded (`lexeme lexeme stamp` refuses a
token that does not verify, `crates/lexlean/src/lexeme/cli.rs:467-487`):

| Certificate | SHA-256 of the DER | Validity |
| --- | --- | --- |
| FreeTSA Root CA (pinned root) | `a6379e7cecc05faa3cbf076013d745e327bbbaa38c0b9af22469d4701d18aabc` | 2016-03-13 to 2041-03-07 |
| FreeTSA TSA (signing certificate) | `32e841a95cc1164101ffde41298ef2fc75c1c4372ef095e88a6bbd47dfb191fc` | 2026-02-15 to 2040-02-02 |

Both certificates are carried inside every entry (`timestamp.tsa_root`,
`timestamp.tsa_certificate`), so a verifier needs no file from this directory to
check the chain.

## What the corpus does not establish

- **The signing seeds were not retained.** Each entry is signed by a software
  Ed25519 key generated for this pass (`--seed` reads a 32-byte file, and the PIN
  and seed handling are §33.3's, not a claim of custody). Re-acquiring the corpus
  therefore produces different signatures, different roots, and different
  verdicts. The corpus is committed once, like the RFC 3161 fixtures under
  `tests/fixtures/rfc3161/`, and `verdicts.json` is the oracle for *these* bytes.
  Reproducing it needs the seeds and therefore needs this directory.
- **The entries are not claims about their subject matter.** Each verdict states
  existence at a stated time, authorship by a stated key, and integrity since
  (§33.6), and nothing else.
- **The heads are signed.** `SPEC.md:6634` requires every published head to be
  signed by the log key, and `LLG1009` registers the refusal for one that is
  not (`model/errors.toml:186-189`). `heads.json` now carries each head's own
  §33.3 `Signature` (`key`, `public_key`, `value`) alongside `tree_size`,
  `root_hash`, and `consistency_proof`, and `Ledger::check_stored_head`
  verifies it. That is work package 2 of `docs/LexLean MVP Plan.md`, landed
  2026-10-05.
- **The entries carry no §34 stratification layer.** `pirtm` is absent from all
  three, so nothing here exercises `contractivity`, `zeno_finton`, or
  `snapaddr`.

## How to re-check it

```bash
cargo build -p lexlean
for entry in lexeme/corpus/entries/*.json; do
  ./target/debug/lexlean lexeme verify --entry "$entry" --ledger lexeme/corpus/ledger
done
```

Each command prints the §33.6 verdict, and
`./target/debug/lexlean lexeme --diagnostic-format json verify …` prints the five
checks separately. `conformance_lg_14` compares those checks against
`verdicts.json` and re-runs one of them in a process that cannot open a socket.