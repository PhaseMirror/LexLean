# Certification library migration oracle

One-time, never a release input (SPEC.md §17.10). It converted the hand-written
Lean certification library into native `lexlean/core-module/2` sources and
checks that the conversion lost nothing. The commit after the one that adds this
directory removes it; the record of what it established is
`tests/certification-library/MIGRATION.md`. It is the same oracle as the
preservation library's (commit `d54dbf85`), with the module list, prefix, and
titles of the certification library.

Inputs: the Lean sources of repository commit `ba52c601`
(`compiler/proofs/LexLeanCertification/*.lean`), the compiler project's generated
`LexLeanTarget.{Gnaf,GnafFixtures,Certification,GnafCertificates}` of the same
commit (`compiler/expected/build/modules/`), the calculus modules and the
preservation library (compiled `LexLeanTarget` and `LexLeanPreservation`
oleans), the pinned Lean `4.32.1`, Python 3.

1. Lay out `src/LexLeanCertification/` (the library's Lean files) and
   `src/LexLeanTarget/` (the four generated modules), and `out/` with the
   compiled calculus and preservation modules; `build.sh` compiles the four
   generated modules and the 13 library modules into `out/`.
2. `flags.lean`, run in a scratch directory with `LEAN_PATH=out`: writes
   `sparse.json` and `compiled.txt`, the constants Lean's code generator
   compiled and the sparse `casesOn` records.
3. `export.lean`, run with `LEAN_PATH=out` and `EXPORT_DIR=<dir>`: one JSON file
   per library module, each declaration with its kernel data, extension state,
   and the axioms Lean's audit computes.
4. `canon.py` with `EXPORT=<export dir>` and `FLAGS=<flags dir>`: the canonical
   `lexlean/core-module/2` sources, which are what `compiler/proofs/src/` holds.
5. `equivalence.sh <generated> <work> <out>`: the Lean `lexlean build` generated
   from those sources (`modules/` of the build tree) is compiled, exported with
   the same programs, and converted with the same script; `compare.py` compares
   the result with step 4's output declaration by declaration, and `probe.lean`
   compares the attribute state of both environments.
