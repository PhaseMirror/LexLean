Feature: calculus

  The production realization calculus: its closed syntax, canonical identity, kernel-checked denotation, realization library, and Rust profile (§17.14).

  @TC-01 @build
  Scenario: Hand-constructed target programs have canonical bytes and a SHA-256 content identity, alpha-equivalent programs canonicalize to identical bytes and identity, and every committed fixture validates against the target schemas.
    Given every committed fixture under `compiler/fixtures/`
    When each is reread, renamed by an injective renaming of its locals, loaded, and run by the reference interpreter
    Then the committed bytes are canonical and schema-valid, every renaming has the same canonical bytes, identity, and outcome
    And a change that is not a renaming changes the identity

  @TC-02 @build
  Scenario: Every malformed target program or violated static rule fails closed with LLB6005, and an invalid program has neither a canonical form nor a rendering.
    Given a valid target program
    When it is made malformed, given an unbound local, an ill-typed body, a nonexhaustive or redundant match, a wrong arity, an undeclared function or ADT, an out-of-range or noncanonical literal, a closure over every parameter, or a primitive outside its signature
    Then each load fails with LLB6005 naming the violated rule
    And the invalid program also has no Rust rendering, while the valid one loads and renders

  @TC-03 @build
  Scenario: The calculus denotation is a kernel-checked LexLean definition, Lean's kernel reduces every kernel-reducible fixture to its expected outcome with its exact step count, no valid fixture is stuck, and a wrong expected outcome is rejected by Lean.
    Given the `compiler` project and its generated `TargetFixtures` module
    When the project is verified under pinned Lean, and a copy stating one wrong expected outcome is verified
    Then every kernel-reducible fixture has a theorem the kernel proves, the module equals its generator, and no fixture is stuck
    And the copy fails with LLV7002 on the wrong outcome

  @TC-04 @build
  Scenario: Lean's evaluator, running the verified calculus modules, reproduces every fixture's expected outcome and step count, including fixtures whose primitives the kernel cannot reduce.
    Given the verified `compiler` project
    When Lean evaluates every fixture's `Run` definition from the verified oleans
    Then every printed outcome equals the fixture's expected outcome, steps included
    And an altered expectation is detected

  @TC-05 @build
  Scenario: Every realization library template has a fixture whose outcome the kernel proves equal to the value LexLean's own collection primitive computes, committed instances equal their templates, and a mutated template is rejected by Lean.
    Given every library fixture and its oracle term
    When each committed instance is compared with its template, and a copy with a mutated `set_insert` instance is verified
    Then every template has a fixture, its instance equals the template, and an `Agrees` theorem states LexLean's own result
    And the mutated instance fails its `Agrees` theorem under pinned Lean

  @TC-06 @build
  Scenario: Every runtime construct of the production registry has exactly one realization row naming existing calculus elements, and the fixtures exercise every calculus type, literal, expression, shape, primitive, fixed width, and template.
    Given the production registry's runtime rows and the realization table
    When the table is checked against the registry, and every element the fixtures use is collected
    Then the table and the runtime rows are in bijection and every reference exists
    And no calculus element and no fixed width is left unexercised, and a missing or extra row is rejected

  @TC-07 @build
  Scenario: Every fixture with an observable outcome renders to safe Rust that rustc compiles with warnings denied and that prints exactly the denotation's value or overflow, and a planted renderer discrepancy is detected.
    Given every fixture whose outcome is a value without a closure or an overflow
    When each is rendered under `#![forbid(unsafe_code)]`, compiled by rustc with warnings denied, and run
    Then each prints exactly the denotation's observable outcome
    And a wrapping addition or a Euclidean quotient planted in a rendering changes the printed outcome
