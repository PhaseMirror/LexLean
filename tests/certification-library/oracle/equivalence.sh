#!/bin/bash
# The migration's equivalence check for the certification library: the Lean the compiler generates from the native
# modules is compiled by the pinned Lean, its environment exported again by the same exporter, and the result,
# converted by the same conversion, must equal the committed native modules byte for byte.
# usage: equivalence.sh <generated modules dir, containing LexLeanCertification/*.lean> <work dir> <base out dir>
# <base out dir> holds the oleans of everything the library imports (calculus, preservation library, the compiler
# project's generated Gnaf, GnafFixtures, Certification, GnafCertificates) and the ORIGINAL library's oleans (probe).
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
generated="$1"; work="$2"; base="$3"
TC=$HOME/.elan/toolchains/leanprover--lean4---v4.32.1
export PATH=$TC/bin:$PATH
rm -rf "$work"; mkdir -p "$work/src" "$work/flags" "$work/export" "$work/out"
cp -r "$generated/LexLeanCertification" "$work/src/"
for d in LexLeanTarget LexLeanPreservation; do cp -r "$base/$d" "$work/out/"; done
cp -r "$base/LexLeanCore" "$work/out/" 2>/dev/null || true
export LEAN_PATH="$work/out"
mkdir -p "$work/out/LexLeanCertification"
: > "$work/summary.txt"
for m in Trials Tail Factor Schemas Facts PremiseSound Positions Lower Frontier Representation Normality Check Soundness; do
  s=$(date +%s)
  lean -R "$work/src" -o "$work/out/LexLeanCertification/$m.olean" "$work/src/LexLeanCertification/$m.lean" > "$work/out/$m.log" 2>&1
  echo "$m rc=$? seconds=$(( $(date +%s) - s ))" >> "$work/summary.txt"
done
(cd "$work/flags" && lean --run "$here/flags.lean" > flags.log 2>&1)
EXPORT_DIR="$work/export" lean --run "$here/export.lean" > "$work/export.log" 2>&1
FLAGS="$work/flags" EXPORT="$work/export" python3 "$here/canon.py" "$work/lextex" > "$work/canon.log" 2>&1
LEAN_PATH="$base" lean --run "$here/probe.lean" > "$work/probe-original.txt" 2>&1
LEAN_PATH="$work/out" lean --run "$here/probe.lean" | grep -v "LexLeanCore.Runtime" > "$work/probe-regenerated.txt" 2>&1
diff "$work/probe-original.txt" "$work/probe-regenerated.txt" > "$work/probe.diff" || true
echo finished >> "$work/summary.txt"
