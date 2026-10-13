# Preservation library migration oracle

One-time, never a release input (SPEC.md §17.10). It converted the hand-written
Lean preservation library into native `lexlean/core-module/2` sources and
checks that the conversion lost nothing. The commit after the one that adds this
directory removes it; the record of what it established is
`tests/preservation-library/MIGRATION.md`.

Inputs: the Lean sources of repository commit `ba52c601`
(`language/preservation-1.2/library/LexLeanPreservation/*.lean` and
`language/preservation-1.2/modules/LexLeanTarget/*.lean`), the pinned Lean
`4.32.1`, Python 3.

1. `build.sh`: compile the calculus modules and the 13 library modules into
   `out/` (`src/` holds the sources laid out as `src/LexLeanTarget/...` and
   `src/LexLeanPreservation/...`).
2. `flags.lean`, run in a scratch directory with `LEAN_PATH=out`: writes
   `sparse.json` and `compiled.txt`, the constants Lean's code generator
   compiled and the sparse `casesOn` records.
3. `export.lean`, run with `LEAN_PATH=out` and `EXPORT_DIR=<dir>`: one JSON file
   per library module, each declaration with its kernel data, extension state,
   and the axioms `#print axioms` reports.
4. `canon.py <dir>` with `EXPORT=<export dir>` and `FLAGS=<flags dir>`: the
   canonical `lexlean/core-module/2` sources, which are what
   `language/preservation-1.2/library/src/` holds.
5. `equivalence.sh <generated> <work>`: the Lean `lexlean build` generated
   from those sources (`modules/` of the build tree) is compiled, exported with
   the same programs, and converted with the same script; the result must equal
   step 4's output byte for byte.
