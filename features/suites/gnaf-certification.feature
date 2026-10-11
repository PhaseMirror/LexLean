Feature: gnaf-certification

  GNAF certification over requests: proof-producing routes from premises to an optimum, a frontier, or an honest weaker status, and never an optimum from normality alone (§17.18).

  @GC-01 @build
  Scenario: Every committed GNAF certificate and certificate fixture is canonical, validates against the certificate schemas, states the framed SHA-256 identity of its certificate and the universe identity of the request it names, and is read back with its committed verdict; a malformed certificate or a certificate fixture stating another identity fails closed with LLB6007, and a certificate beyond the host's capacity fails closed with LLS8002.
    Given every committed certificate under `compiler/certificates/` and a set of malformed, misidentified, and over-capacity certificates
    When each is reread, schema-validated, and loaded
    Then every committed fixture is canonical, schema-valid, states its own identity and its request's universe identity, and loads with its committed verdict
    And each malformed or misidentified certificate fails with LLB6007 and each over-capacity one with LLS8002

  @GC-02 @build
  Scenario: The certification checker is a kernel-checked LexLean definition, Lean's kernel reduces the verdict of every committed certificate and its request to what the host transcription computes, and a copy stating a certified verdict for a certificate that omits one competitor's premise, for a lower bound its attainer does not meet, or for a locally normal system is rejected by Lean.
    Given the `compiler` project, its generated `Certification` and `GnafCertificates` modules, and a copy of the certificate module stating certified verdicts the checker does not give
    When the project is verified under pinned Lean and the copy is compiled against its built modules
    Then the kernel proves every committed verdict, every committed verdict equals the host's, and the modules equal their generators
    And Lean rejects the copy's theorem for each planted verdict as not definitionally equal to the checker's verdict

  @GC-03 @build
  Scenario: A universal lower-bound certificate proves a nontrivial global optimum or argmin from a premise for every system of the universe: the attainers meet the bound, and every other system is excluded, reduced, or shown to pay at least the bound from a prefix of the domain, so a system whose cost is unknown at the request's fuel is bounded without being evaluated in full; ties are all attainers, and an undercut, unattained, non-strict, omitted, weakened, extra, diverging, or inadmissible-attainer certificate leaves an honest non-global status naming the first open obligation.
    Given requests whose optimum a dispatching system attains, whose optimum is tied, and some of whose systems run out of fuel, and lower-bound certificates for them and their mutations
    When each certificate is checked
    Then the unmutated certificates certify the optimum, the argmin, and, over unresolved systems, an argmin that enumeration cannot answer
    And each mutation is refused or incomplete with the obligation it leaves open, and no mutation is certified

  @GC-04 @build
  Scenario: A frontier certificate proves a complete Pareto frontier: its members are admitted and none dominates another, and every other system is excluded, reduced, or strictly dominated by a member from the least evidence; a member omitted, a tie dropped, a dominated member, and a weakened premise are refused by naming the first obligation left open.
    Given GNAF-VEC-02 posed over the calculus, a tied frontier, and frontier certificates for them and their mutations
    When each certificate is checked
    Then the complete frontiers are certified, including the tied frontier with every tied member
    And each mutation leaves the obligation it breaks open

  @GC-05 @build
  Scenario: The transformation schemas collapse, specialize-small, and specialize-large apply only when the plans neither call the entry nor apply a closure and the domain makes the dispatch run the rewritten system's plan, and rewrite to a member of the universe of no greater size or preparation; a reduction certificate whose every other system is excluded or reduced certifies the optimum of the measured systems, an incomplete or ineligible reduction is refused; the module defines the observations a system keeps and the strict cost decrease of its rewriting over the request's domain, the kernel decides both for every reduction premise of the committed certificates, and the library proves both for every improving schema application.
    Given requests whose dispatches a domain specializes and a request whose domain does not, and reduction certificates for them and their mutations
    When each certificate is checked, the schemas' eligibility is read for every system, and the module's preservation relation is read for every reduction premise
    Then the reductions certify the measured optimum, only eligible systems reduce, and every committed reduction premise is a preserving application as the kernel decides
    And an incomplete or ineligible reduction is refused with the position it leaves open

  @GC-06 @build
  Scenario: Local normality is never optimality: no theorem defines optimal as GNAF-normal, the normality route's verdict is never certified, a GNAF-normal system that is not the argmin is locally normalized only, claiming it as the attainer of the bound it cannot meet is refused, a system a schema improves is not normal, and a certified optimum is GNAF-normal while the converse is false.
    Given a request on which a GNAF-normal system pays more than the optimum and normality, lower-bound, and reduction certificates for it
    When each certificate is checked and the library's normality theorems are read
    Then the normal system is locally normalized and never certified, and the optimum itself is only locally normalized by normality
    And the library proves that normality never certifies and that every certified optimum is GNAF-normal

  @GC-07 @build
  Scenario: A representation metric supports an execution-optimality claim only through a bridge: the system of least size is representation-minimal without one, an affine bridge from size to cost holding for every system certifies the least-size system as a global optimum when it attains the bound, and a missing, violated, unattained, non-minimal, or out-of-range bridge leaves a representation-minimal or correct-only status.
    Given a request of straight-line plans of different sizes and representation certificates with and without a bridge
    When each certificate is checked
    Then the certificate with a valid attained bridge certifies the least-size system as the optimum
    And a missing bridge establishes representation minimality only and every other defect leaves its obligation open

  @GC-08 @build
  Scenario: Every verdict is a closed, explicit claim status: refused for a certificate that does not bind its request's verifier, universe, objective, claim, and scope or does not cover its universe, a certified optimum, argmin, or frontier, or incomplete with the first open obligation and the strongest status it establishes, correct-only, locally normalized, or representation-minimal, never a fallback claim; a certificate for another universe, including the same problem on an enlarged machine, is refused.
    Given certificates that mismatch each part of their binding, cover too few or too many systems, name an incompatible route, are for a rejected request, or are for an enlarged machine
    When each is checked
    Then each is refused with the part of its binding it does not satisfy
    And every incomplete verdict names its first obligation and a status that claims no optimality

  @GC-09 @build
  Scenario: The certification library is kernel-checked under the pinned Lean with exactly its registered axioms and none of the forbidden forms: it proves that a certified verdict bounds the cost of every well-formed system of the request's grammar universe at the request's fuel and every larger one and that a bound is attained, that a certified frontier is exactly the Pareto frontier of the admitted systems, that normality never certifies, that every certified optimum is GNAF-normal, and that every improving schema application satisfies the module's own `Certification.preserves`, its rewriting keeping the system's observations over the request's domain at strictly lower cost, and its soundness theorem holds of every certified committed certificate.
    Given the library under `compiler/proofs/`, its registry, and the compiler project's generated modules
    When the library is staged beside the generated modules, compiled, replayed, and audited, instantiated at every certified committed certificate and at the module's preservation relation, and planted with an unsound lemma
    Then every theorem is accepted with exactly its registered axioms and every instance is accepted
    And a planted sorry is refused by the token audit, and a planted false lemma and a planted weakened checker are each refused by Lean

  @GC-10 @build
  Scenario: The certification model, certificate fixtures, certificate module, and schemas are generated, deterministic, and content-addressed: they equal their generators, the compiler project reaches them from its entrypoint, certificate identities are recomputed from content, and a planted change to a premise, a bound, or a route changes the verdict or the identity.
    Given the committed certification files and their generators
    When they are regenerated, reread, and mutated
    Then they equal their generators, the entrypoint reaches the modules, and every identity is the framed SHA-256 of the certificate's canonical bytes
    And a mutated premise, bound, or route has another identity and, where the mutation matters, another verdict
