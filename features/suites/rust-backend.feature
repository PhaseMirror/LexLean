Feature: rust-backend

  The canonical Rust backend: the closed Rust AST, its checks, packages, and provenance (§17.16).

  @RB-01 @build
  Scenario: Every Rust rendering is built as a closed AST whose every construct corresponds to an element of the target program it realizes, every correspondence row is exercised, and a construct the program does not justify is refused.
    Given every fixture program in each profile that renders it, and every committed package
    When each is lowered to the closed Rust AST and its constructs are compared with the correspondence table
    Then every construct realizes an element the program uses and every table row is exercised
    And a construct whose element is removed from the program's set, or that has no row, is refused

  @RB-02 @build
  Scenario: An exported name that is not a lowercase snake-case identifier, is a Rust keyword, imitates a generated name, or is declared by the runtime, a name exported twice, and an unavailable crate name each fail with LLB6005, and a crate binding one name twice in a function is refused.
    Given the committed negative manifests whose exported names or crate name collide
    When each is packaged, and a lowered crate is planted with one binding repeated
    Then each fails with LLB6005 and its stated identifier collision
    And the planted crate is refused for binding a name twice

  @RB-03 @build
  Scenario: An export that copies a parameter whose type is not Copy fails with LLB6005, and a crate that moves a value twice or reads it after moving it is refused.
    Given the negative manifest copying a list parameter, and lowered crates
    When the manifest is packaged and each crate is planted with an extra move of a match scrutinee
    Then the manifest fails with LLB6005 and its stated ownership mismatch
    And the planted crates are refused for moving a value twice and for reading it after the move

  @RB-04 @build
  Scenario: A package whose boundary holds a function value, or whose rust-core program needs the heap, fails with LLB6005, and a rust-core crate naming any heap type, runtime function, or construct is refused.
    Given the negative manifests with a function-valued boundary or a rust-core program that needs the heap
    When each is packaged, and rust-core crates are planted with a string type and a heap runtime function
    Then each fails with LLB6005 and its stated reason
    And the planted crates are refused as hidden allocation

  @RB-05 @build
  Scenario: A function that can overflow returns R<T> and every call to it propagates, any other returns its value, an export whose declared errors differ from its function's fails with LLB6005, and a crate that drops, invents, or misreturns a failure is refused.
    Given every fixture program and the negative manifests misstating errors
    When fallibility is analysed, the manifests are packaged, and crates are planted with a dropped, an invented, and a misreturned failure
    Then every entry whose outcome is overflow is typed fallible and each manifest fails with LLB6005 and its stated arithmetic mismatch
    And every planted crate is refused

  @RB-06 @build
  Scenario: Every committed package builds offline under its declared gates, rustc warnings and Clippy's default lints denied with three documented exceptions, and its exported function, called from a separate crate, prints exactly the denotation's observable outcome; a planted lint and a planted semantic mutation are detected.
    Given every committed package with an observable outcome and a harness crate calling its export
    When the packages and harnesses are built offline in one workspace, the packages are linted, and each harness runs
    Then every package passes its gates and every harness prints the denotation's observable outcome
    And a package with a clone of a Copy value fails its lint gate and a wrapping subtraction changes the printed outcome

  @RB-07 @build
  Scenario: Packages are deterministic and content-addressed: two renderings written under two roots are byte-identical to each other and to the committed package, every manifest and provenance validates against its schema, and the provenance binds the SHA-256 of each file, the program identity, the runtime, the sources, and the language-1.2 compiler-semantics ID.
    Given every committed package and its manifest
    When each manifest is validated and packaged twice, and the files are written under two roots
    Then the renderings are identical to each other and to the committed bytes, and every manifest and provenance is schema-valid
    And the provenance binds each file's SHA-256, the program identity, the runtime, the sources, and the compiler-semantics ID
