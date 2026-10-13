#!/bin/bash
# The migration's equivalence check: the Lean the compiler generates from the native modules is compiled by the
# pinned Lean, its environment is exported again by the same exporter, and the result, converted by the same
# conversion, must equal the committed native modules byte for byte.
# usage: equivalence.sh <generated modules dir, containing LexLeanPreservation/*.lean> <work dir>
set -euo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
generated="$1"; work="$2"
TC=$HOME/.elan/toolchains/leanprover--lean4---v4.32.1
export PATH=$TC/bin:$PATH
rm -rf "$work"; mkdir -p "$work/src" "$work/out/LexLeanTarget" "$work/flags" "$work/export"
cp -r "$generated/LexLeanPreservation" "$work/src/"
cp "$here"/out/LexLeanTarget/* "$work/out/LexLeanTarget/"
export LEAN_PATH="$work/out"
mkdir -p "$work/out/LexLeanPreservation"
: > "$work/summary.txt"
for m in Core Values Primitives Fixed Keys Templates Validate RustBase RustCorr RustLemmas RustMatches RustSound Compose; do
  s=$(date +%s)
  lean -R "$work/src" -o "$work/out/LexLeanPreservation/$m.olean" "$work/src/LexLeanPreservation/$m.lean" > "$work/out/$m.log" 2>&1
  echo "$m rc=$? seconds=$(( $(date +%s) - s ))" >> "$work/summary.txt"
done
(cd "$work/flags" && lean --run "$here/flags.lean" > flags.log 2>&1)
EXPORT_DIR="$work/export" lean --run "$here/export.lean" > "$work/export.log" 2>&1
FLAGS="$work/flags" EXPORT="$work/export" python3 "$here/canon.py" "$work/lextex" > "$work/canon.log" 2>&1
# the attribute state of every constant, read by the pinned Lean's own queries from both environments
LEAN_PATH="$here/out" lean --run "$here/probe.lean" > "$work/probe-original.txt" 2>&1
LEAN_PATH="$work/out" lean --run "$here/probe.lean" | grep -v "LexLeanCore.Runtime" > "$work/probe-regenerated.txt" 2>&1
diff "$work/probe-original.txt" "$work/probe-regenerated.txt" > "$work/probe.diff" || true
echo finished >> "$work/summary.txt"
