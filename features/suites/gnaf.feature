Feature: gnaf

  GNAF requests over the production realization calculus: the complete-system universe, machine contract, accounting, objective order, and claim, fixed before any optimizer (§17.15).

  @GN-01 @build
  Scenario: Every committed GNAF request and fixture is canonical and validates against the GNAF schemas, every malformed request or invalid reference or realized program fails closed with LLB6006, and fuel or a universe beyond the host's capacity fails closed with LLS8002.
    Given every committed request under `compiler/gnaf/` and a set of malformed, invalid, and over-capacity requests
    When each is reread, schema-validated, loaded, and answered
    Then every committed fixture is canonical, schema-valid, loads, and has its committed answer
    And each malformed or invalid request fails with LLB6006 and each over-capacity request with LLS8002, while a request at full fuel capacity is answered

  @GN-02 @build
  Scenario: The GNAF model is a kernel-checked LexLean definition, Lean's kernel reduces the answer of every committed request to the answer the host transcription computes, and a wrong answer or an answer computed from a universe with a system omitted is rejected by Lean.
    Given the `compiler` project and its generated `GnafFixtures` module
    When the project is verified under pinned Lean, and a copy stating a wrong answer and answers computed with a system omitted from the universe is verified
    Then the kernel proves every committed answer and the module equals its generator
    And the copy fails with LLV7002 on the wrong answer and on each omission

  @GN-03 @build
  Scenario: Candidate membership is the grammar's expansion fixed before any optimizer, and optimizer-defined, discovered, cached, and internal-plan universes and missing, self-referential, or optimizer-citing completeness evidence are rejected.
    Given a grammar of two plans and one dispatch threshold
    When its universe is expanded and requests carrying every other carrier and every other completeness evidence are answered
    Then the members are exactly the fixed and dispatching systems in expansion order, independent of any evaluation
    And every other carrier or completeness evidence is rejected, even a discovered carrier listing every member

  @GN-04 @build
  Scenario: Every action a system performs is charged by steps and every admitted preparation action by a positive constant or a common prepared boundary, hidden zero-cost, free, undeclared, duplicated, unaccounted, and unrealizable actions are rejected, and declared preparation charges enter every system's cost.
    Given a valid machine contract
    When each performed action is charged otherwise, each preparation action is charged by steps, zero, undeclared, or free off a common prepared boundary, an action is duplicated or removed, or communication, randomness, or scheduling is admitted
    Then each request is rejected naming the violated action
    And a positive preparation charge adds to every system's cost and common prepared state adds nothing

  @GN-05 @build
  Scenario: Scalar claims require the total step order and Pareto claims the componentwise steps-and-size order, a scalar claim over the partial order, a vector claim over the total order, an undecided claim class, and a scope beyond the grammar universe are rejected, and a frontier answer has incomparable members.
    Given the scalar and vector requests over the same universe
    When every claim class is posed under each objective and with each scope
    Then only scalar claims under the total order and Pareto claims under the componentwise order are accepted within the grammar universe
    And the frontier holds a faster larger system and a slower smaller system, neither dominating the other

  @GN-06 @build
  Scenario: Complete-system cost includes selection, the system argmin differs from the best internal plan and from the per-input plan envelope that no system attains, an inadmissible system is excluded, and an unresolved system makes the answer incomplete rather than being removed.
    Given short lists, long lists, and their union as domains
    When each is answered over the same universe of plans and dispatching systems
    Then the union's argmin is a dispatching system although the best single plan is cheaper on the long lists and recursion on the short ones
    And the sum of the per-domain optima is lower than any system's cost, an inadmissible plan is excluded, and a diverging plan makes the answer incomplete

  @GN-07 @build
  Scenario: The authority's GNAF-VEC-01, GNAF-VEC-02, GNAF-VEC-04, GNAF-VEC-17, GNAF-REJ-14, and GNAF-REJ-29 vectors are kernel-checked theorems of the model, and the authority is cited by revision and SHA-256 with a some-true ledger claim.
    Given the `Gnaf` model, `model/authorities.toml`, and `model/ledger.toml`
    When the model is verified and a copy stating a wrong frontier for GNAF-VEC-02 is verified
    Then the kernel proves every transcribed authority vector, and the authority row names the revision, immutable URL, and SHA-256 with a some-true ledger claim
    And the copy fails with LLV7002
