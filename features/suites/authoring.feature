Feature: authoring

  @AU-01 @build
  Scenario: Every value of the closed semantic-module schema has one notation and the notation reads back to the value: every alternative of the declaration, term, type, and proof unions, with and without its optional members, is written and read back equal, a node that no readable form carries exactly is written generically, the notation is its own canonical form, and every committed semantic module is written and read back equal to its data.
    Given the semantic-module schema and every committed semantic module
    When every alternative of the declaration, term, type, and proof unions is sampled with and without its optional members, and each sample and each module is written in the notation and read back
    Then each is read back equal, the notation written from what is read is the notation written, and a node no readable form carries exactly is written generically

  @AU-02 @build
  Scenario: A language-1.2 semantic module written in the authoring notation links to exactly the module its data form links to, with the same semantic ID and the same generated Lean and LaTeX byte for byte and source maps that relate each declaration to its text in the notation; fmt writes a module written as data in the notation, to the committed bytes, and leaves the notation unchanged, and the notation under language 1.1 is refused with LLP2003.
    Given committed language-1.2 examples and their copies written as data
    When each pair is checked, built, and format-checked, and the notation is put under language 1.1
    Then the semantic IDs and the generated Lean and LaTeX are equal, the source maps name the notation, fmt writes the data form as the committed notation and leaves the notation unchanged, and language 1.1 refuses it

  @AU-03 @build
  Scenario: Malformed notation fails closed and never panics: an unclosed delimiter, a keyword used as a name, a literal that is not canonical, an unknown kind, and every single-token deletion, duplication, and replacement of a committed module fail with a registered code at a position in the notation or link to a module that decodes, and a numeral with a redundant leading zero is LLL1003 outside a string and is not one inside it.
    Given a committed language-1.2 module in the notation
    When it is malformed in each way named, and each of its tokens is deleted, duplicated, and replaced in turn, and numerals with a redundant leading zero are written outside and inside a string
    Then each is refused with a registered code at a position or links to a module that decodes, nothing panics, and only the numeral outside a string is LLL1003

  @AU-04 @build
  Scenario: No language-1.2 source of the product is written as data: every committed language-1.2 source with a semantic or tooling module, in the compiler, the language libraries, the examples, and the positive fixtures, is written in the authoring notation and is canonical, the data form remains only under language 1.0 and 1.1 and in refused fixtures whose payload the notation cannot spell, and a refused fixture whose payload the notation can spell is a defect.
    Given every committed source of the repository that has a semantic module
    When its language and its form are read, and each refused fixture written as data is written in the notation and read back
    Then no language-1.2 source of the product is written as data, each is canonical, and a refused fixture that the notation can spell is not written as data

  @AU-05 @build
  Scenario: A language-1.2 tooling module written in the authoring notation links to exactly the module its data form links to, with the same semantic ID and the same generated Lean and LaTeX byte for byte and source maps that relate each declaration to its text in the notation; the notation and the canonical JSON of every committed tooling module are in bijection and the notation is its own canonical form; fmt writes a tooling module written as data as exactly the committed notation and leaves the notation unchanged; and malformed notation, and every single-token deletion and duplication of a committed tooling module, fails with a position or decodes, and nothing panics.
    Given every committed tooling module and the committed tooling fixtures, and their copies written as data
    When each module is written in the notation and read back, each fixture and its data form is linked, formatted, and rendered, and every token of a committed module is deleted and duplicated in turn
    Then each is read back equal, fmt writes the data form as the committed notation, the semantic IDs, Lean and LaTeX are equal, the source maps name the notation, and each mutation is refused with a position or decodes

  @AU-06 @build
  Scenario: The compiler-semantics ID does not depend on the form a module of the embedded tree is written in: a semantic or tooling module in the authoring notation contributes its data form, so the committed tree and the same tree with every such module written as data have one ID, a change to a module's meaning changes the ID in either form, and the embedded tree holds at least one module written in the notation.
    Given the embedded tree, and the same tree with every module written in the notation written as data
    When the tree digest of each is taken, and a declaration of a module is removed in either form
    Then the two trees have the compiler-semantics ID, and the changed tree has another in both forms
