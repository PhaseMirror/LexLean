Feature: authoring

  @AU-01 @build
  Scenario: Every value of the closed semantic-module schema has one notation and the notation reads back to the value: every alternative of the declaration, term, type, and proof unions, with and without its optional members, is written and read back equal, a node that no readable form carries exactly is written generically, the notation is its own canonical form, and every committed semantic module is written and read back equal to its data.
    Given the semantic-module schema and every committed semantic module
    When every alternative of the declaration, term, type, and proof unions is sampled with and without its optional members, and each sample and each module is written in the notation and read back
    Then each is read back equal, the notation written from what is read is the notation written, and a node no readable form carries exactly is written generically

  @AU-02 @build
  Scenario: A language-1.2 semantic module written in the authoring notation links to exactly the module its data form links to, with the same semantic ID and the same generated Lean and LaTeX byte for byte and source maps that relate each declaration to its text in the notation; the conversion of a module written as data yields the notation, fmt leaves the notation unchanged, and the notation under language 1.1 is refused with LLP2003.
    Given committed language-1.2 examples and their copies converted from the data form to the notation
    When each pair is checked, built, and format-checked, and the notation is put under language 1.1
    Then the semantic IDs and the generated Lean and LaTeX are equal, the source maps name the notation, fmt leaves the notation unchanged and the data form as it is, and language 1.1 refuses it

  @AU-03 @build
  Scenario: Malformed notation fails closed and never panics: an unclosed delimiter, a keyword used as a name, a literal that is not canonical, an unknown kind, and every single-token deletion, duplication, and replacement of a committed module fail with a registered code at a position in the notation or link to a module that decodes, and a numeral with a redundant leading zero is LLL1003 outside a string and is not one inside it.
    Given a committed language-1.2 module in the notation
    When it is malformed in each way named, and each of its tokens is deleted, duplicated, and replaced in turn, and numerals with a redundant leading zero are written outside and inside a string
    Then each is refused with a registered code at a position or links to a module that decodes, nothing panics, and only the numeral outside a string is LLL1003
