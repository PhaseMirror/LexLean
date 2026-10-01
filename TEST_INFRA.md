# LexLean v0.4 Dual-Track E2E Test Infrastructure & Opaque-Box Test Specification

## 1. Overview & Architectural Vision

The LexLean v0.4 release expands LexLean from a closed-lexicon LaTeX-to-Lean 4 compiler into a general-purpose formal programming language, a verified production compiler targeting canonical Rust with Lean 4 semantic preservation proofs, and a typed model/reasoning runtime for PrismPM applications.

To guarantee maximum assurance without coupling test suites to mutable compiler internals, LexLean adopts an **Opaque-Box, Requirement-Driven Dual-Track E2E Testing Infrastructure**:
1. **Opaque-Box Discipline**: Tests interact solely with documented, stable external interfaces:
   - **Track 1 (CLI Process Boundary)**: External process invocation of the `lexlean` binary, evaluating exit codes, stdout/stderr streams, human-readable diagnostics, and canonical JSON command-result objects.
   - **Track 2 (Public Engine API Boundary)**: Programmatic invocation of the stable `lexlean::Engine` surface and public request/result DTOs (`BuildRequest`, `CheckRequest`, `LockRequest`, `VerifyRequest`, `FormatRequest`, `Selection`, `LexLeanError`, `Diagnostic`).
   - Neither track imports or relies on `#[doc(hidden)]` internal compiler modules (`ir`, `grammar`, `elaborate`, `backend`, `source`).
2. **Dual-Track Execution Model**:
   - **Implementation Track**: Sequential progression through Milestones M1 to M8 (Issues #16 through #36).
   - **E2E Testing Track**: Parallel, milestone-progressive test infrastructure providing immediate falsification, regression defense, and verification gates across all 4 tiers.

---

## 2. The 4-Tier Test Methodology

Every v0.4 feature is exercised across four distinct testing tiers:

```
+--------------------------------------------------------------------------+
| Tier 4: Real-World Application Scenarios                                  |
|   (Complete multi-file lifecycles, cross-version migrations, ecosystem)  |
+--------------------------------------------------------------------------+
| Tier 3: Cross-Feature Combinations                                       |
|   (Pairwise matrix: versions x diagnostics x commands x lock schemas)    |
+--------------------------------------------------------------------------+
| Tier 2: Boundary & Corner Cases                                          |
|   (Malformed schemas, limit overflows, corruption, fail-closed guards)  |
+--------------------------------------------------------------------------+
| Tier 1: Feature Coverage                                                 |
|   (>= 5 tests per feature: primary happy paths & core specifications)     |
+--------------------------------------------------------------------------+
```

### Tier 1: Feature Coverage (>=5 test cases per feature)
Verifies the standard nominal behavior (happy paths) of each feature as defined in `PROJECT.md`, `SPEC.md`, and the respective GitHub issue. For each capability, at least 5 distinct test scenarios must execute and verify:
- Accurate acceptance of valid inputs.
- Deterministic production of expected outputs.
- Verification of content-addressed output artifacts.
- Exact return codes and canonical JSON structures.

### Tier 2: Boundary & Corner Cases (>=5 test cases per feature)
Verifies resilience at domain boundaries, edge conditions, resource constraints, and error handling:
- Malformed syntax, unexpected encodings, whitespace anomalies, and boundary values.
- Empty inputs, single-atom files, and maximum allowed limit thresholds.
- Corrupted lockfiles, digest mismatches, and tampered metadata.
- Fail-closed security assertions: every error must report its exact registered diagnostic code (e.g., `LLC0103`, `LLR3001`, `LLS8002`).

### Tier 3: Cross-Feature Combinations (Pairwise Matrix)
Verifies orthogonal features interacting together without unexpected coupling or state leakage:
- **Language Version x Diagnostic Format**: `{1.0, 1.1, 1.2} x {Human, Json}`.
- **Language Version x CLI Command**: `{1.0, 1.1, 1.2} x {init, lock, check, build, verify, fmt, clean, explain}`.
- **Lockfile Schema x Project Language**: `{Lock v1, Lock v2, Missing Lock} x {Lang 1.0, Lang 1.1, Lang 1.2}`.
- **Lexicon Packages x Language Versions**: `{Builtin, Path, External Git} x {Lang 1.1, Lang 1.2}`.

### Tier 4: Real-World Application Scenarios
End-to-end integration scenarios mirroring real repository environments and workflows:
- **Complete Project Lifecycle**: Clean directory initialization (`init`), dependency locking (`lock`), type/semantic checking (`check`), artifact building (`build`), format validation (`fmt`), full verification (`verify`), and workspace cleansing (`clean`).
- **Ecosystem Migration Workflow**: Loading existing Language 1.0 and 1.1 projects (e.g. `examples/nat-add-zero`, `examples/semantic-1.1`, PrismPM SDK layouts), verifying byte-stability of existing artifacts, upgrading `lexlean.toml` to `language = "1.2"`, relocking with Lockfile v2, and ensuring smooth transition without regression.
- **Tampering & Drift Detection**: Automated detection of modified sources, altered lockfile digests, or unsynchronized configs under continuous verification gates.

---

## 3. Milestone Feature Inventory & E2E Test Matrix

| Milestone | Issues | Feature Scope | Tier 1 (Coverage) | Tier 2 (Boundary) | Tier 3 (Combinatorial) | Tier 4 (Scenario) |
|-----------|--------|---------------|-------------------|-------------------|------------------------|-------------------|
| **M1** | #16 | Language 1.2 & Migration Contract | Lang 1.0/1.1/1.2 declaration, version reporting, lock v1 read, lock v2 write, lexicon versioning | Invalid semver, `LLC0103` on unsupported version, lock digest mismatch `LLR3001`, empty version, corrupt lock | Version x Output format, Version x Subcommands, Lockfile v1/v2 x Project version | Complete 1.1->1.2 migration, multi-module project lifecycle, fail-closed lock tampering |
| **M2** | #17, #18, #19, #20 | Semantic Types, Closures, Totality, Collections | ADT declarations, lambda definitions, well-founded measures, map/set ops, graph traversal | Positivity violations, non-decreasing recursion (`LLF5001`), cyclic measure, duplicate keys, fuel bounds | Lambda x Collection ops, Recursive ADT x Well-founded recursion, Structural pattern match x Closure | Symbol table compiler, pure state-threading evaluator, graph reachability engine |
| **M3** | #35, #36, #21 | LCNF Authority, Production Eligibility, Realization Calculus | LCNF extraction, `ProductionEligible` check, computational root closure, target IR denotation | Unsafe/noncomputable code, escaping non-serializable closure, int overflow | Target profile x Int wrapping/checked, Formal theorem import x Executable root | Executable root compilation with formal theorem erasure and proof auditing |
| **M4** | #22, #23 | GNAF Universe & Lower-Bound Certification | Independent universe $U_{sys}$, machine contract $X$, cost model $M$, lower-bound certificates | Circular universe, incomplete universe evidence, suboptimal normal form | Cost metric x Machine contract, Scalar argmin x Pareto frontier | Proof-producing optimization attaining universal lower bound on verified algorithm |
| **M5** | #24, #25 | Canonical Rust Backend & Preservation Proofs | Rust AST rendering, hygiene pass, Lean simulation proofs, preservation certificate | Keyword collision, ownership mismatch, unproved lowering branch | Profile x Rust memory model, Optimization level x Preservation certificate | Multi-module Rust crate generation compiling cleanly under `cargo check` and passing proofs |
| **M6** | #26, #27, #28 | Differential Oracle, Self-Hosting, PrismPM Export | `lean4-prod` comparison, LexLean-authored compiler, PrismPM export bundle | Oracle mismatch fails closed, oracle offline, forged certificate | Oracle comparison x Target profile, PrismPM bundle x Content-addressed manifest | Differential validation against `lean4-prod` oracle; PrismPM fixture compilation |
| **M7** | #29, #30, #31, #32 | Models, Language Models, Reasoning Machines, Stdlib | Model contracts, tokenization pipelines, reasoning inference engines, stdlib composition | Tensor shape mismatch, context overflow, reasoning step/fuel exhaustion | Reasoning rules x Fuel limits, Model contract x Token pipeline, Stdlib x Multi-package | Reasoning machine deduction pipeline running bounded proof search in Prism fixture |
| **M8** | #33, #34, #15 | Verification Gate Extension, Release Closure, Signoff | `just vv` extension, clean-checkout reproducibility, documentation synchronization | Anti-vacuity gate failure, planted defect detection, clean-checkout failure | Multi-target CI matrix x Clean checkout reproducibility | Full repository verification from clean checkout across all v0.4 features |

---

## 4. Test Runner & Harness Architecture

The E2E test infrastructure is organized under:
- **Test Binary**: `crates/lexlean/tests/e2e.rs` (integrated with `cargo test -p lexlean --test e2e` and `cargo test --workspace --all-features`).
- **Shared Fixture Space**: `tests/e2e/fixtures/` containing test projects and input trees.
- **Harness Modules**:
  1. `harness::cli::CliRunner`: Executes the compiled `lexlean` binary in isolated child processes (`std::process::Command`), capturing stdout, stderr, exit code, and parsing JSON command results.
  2. `harness::api::ApiRunner`: Executes operations via the public `lexlean::Engine` Rust API.
  3. `harness::project::TestProject`: Provides a hermetic sandbox for creating, editing, locking, checking, and inspecting temporary projects.

```
                              +------------------------+
                              |   E2E Test Runner      |
                              | crates/lexlean/tests/  |
                              |         e2e.rs         |
                              +-----------+------------+
                                          |
                     +--------------------+--------------------+
                     |                                         |
          [Track 1: CLI Process]                     [Track 2: Public API]
                     |                                         |
          +----------v-----------+                  +----------v-----------+
          | harness::cli         |                  | harness::api         |
          | CliRunner            |                  | ApiRunner            |
          | (CARGO_BIN_EXE)      |                  | (lexlean::Engine)    |
          +----------+-----------+                  +----------+-----------+
                     |                                         |
                     +--------------------+--------------------+
                                          |
                              +-----------v------------+
                              | harness::project       |
                              | TestProject (Sandboxed |
                              | tempdir filesystem)    |
                              +------------------------+
```

### Running the E2E Test Suite
```bash
# Run all E2E tests
cargo test -p lexlean --test e2e

# Run only Tier 1 Feature tests
cargo test -p lexlean --test e2e tier1

# Run only Tier 2 Boundary tests
cargo test -p lexlean --test e2e tier2

# Run only Tier 3 Combinatorial tests
cargo test -p lexlean --test e2e tier3

# Run only Tier 4 Real-World Scenario tests
cargo test -p lexlean --test e2e tier4

# Run with verbose output and test output printing
cargo test -p lexlean --test e2e -- --nocapture
```

---

## 5. Canonical Input/Output Formats & Protocols

### 5.1 Project Configuration (`lexlean.toml`)
```toml
spec = "lexlean/project/1"
name = "example-project"
language = "1.2"               # Language version: "1.0", "1.1", or "1.2"
module_prefix = "Example"
source_roots = ["src"]
entrypoints = ["src/Main.lex.tex"]
build_root = ".lexlean"
lockfile = "lexlean.lock"
lean_workspace = "."
lean_toolchain = "leanprover/lean4:v4.32.1"

[[lexicon_source]]
package = "lexlean.std.nat"
kind = "builtin"

[limits]
max_file_bytes = 4194304
max_total_source_bytes = 67108864
max_primitive_atoms = 2000000
max_token_lattice_edges = 4000000
max_parse_states = 4000000
max_ir_nodes = 2000000
max_scope_depth = 1024
max_import_depth = 128
max_diagnostics = 256
max_child_output_bytes = 16777216
child_timeout_ms = 300000
```

### 5.2 Lockfile Schema Evolution
- **Schema v1 (`lexlean/lock/1`)**: Standard lockfile format used in Language 1.0 and 1.1 projects.
- **Schema v2 (`lexlean/lock/2`)**: Extended lockfile format introduced in Language 1.2 supporting enhanced compiler semantics, package registries, and target profiles while maintaining backward compatibility with v1.

### 5.3 Canonical JSON Command Result
When invoked with `--diagnostic-format json`, `lexlean` outputs a single line of canonical JSON to stdout:
```json
{
  "command": "check",
  "exit_code": 0,
  "diagnostics": [],
  "artifacts": [
    {
      "path": ".lexlean/build/...",
      "sha256": "..."
    }
  ]
}
```
When failure occurs (`exit_code != 0`), `diagnostics` contains the exact structured diagnostic records:
```json
{
  "command": "check",
  "exit_code": 1,
  "diagnostics": [
    {
      "code": "LLC0103",
      "severity": "error",
      "message": "unsupported language version `1.3`",
      "labels": [
        {
          "file": "lexlean.toml",
          "span": [45, 50],
          "message": "unsupported language version"
        }
      ]
    }
  ],
  "artifacts": []
}
```

---

## 6. Progressive Testability & Anti-Vacuity Invariants

1. **Progressive Testability**:
   - Tests for milestone $M_k$ execute only against features implemented in $M_1 \dots M_k$.
   - Tests assert fail-closed rejection for un-implemented or invalid versions.
2. **Anti-Vacuity Invariants**:
   - Every positive test must assert non-empty outputs and exact `exit_code == 0`.
   - Every negative test must assert exact registered diagnostic codes (e.g. `LLC0103`, `LLR3001`) and non-zero exit codes.
   - Tests must never accept dummy/facade results or ignore compiler diagnostics.
3. **Reproducibility & Determinism**:
   - Tests execute in isolated temporary directories (`tempfile::TempDir`).
   - Timestamps, system usernames, and platform-specific path prefixes are normalized before comparison.
