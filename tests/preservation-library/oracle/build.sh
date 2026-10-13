#!/bin/bash
export PATH=$HOME/.elan/bin:$PATH
cd "$(dirname "$0")"
LEAN=$(ls -d $HOME/.elan/toolchains/leanprover--lean4---v4.32.1)/bin/lean
export LEAN_PATH=$PWD/out
for m in LexLeanTarget.TargetSyntax LexLeanTarget.TargetSemantics LexLeanTarget.RustSyntax LexLeanTarget.RustSemantics LexLeanPreservation.Core LexLeanPreservation.Values LexLeanPreservation.Primitives LexLeanPreservation.Fixed LexLeanPreservation.Keys LexLeanPreservation.Templates LexLeanPreservation.Validate LexLeanPreservation.RustBase LexLeanPreservation.RustCorr LexLeanPreservation.RustLemmas LexLeanPreservation.RustMatches LexLeanPreservation.RustSound LexLeanPreservation.Compose; do
  f=src/$(echo $m | tr . /).lean
  mkdir -p out/$(dirname $(echo $m | tr . /))
  s=$(date +%s)
  $LEAN -R src -o out/$(echo $m | tr . /).olean $f > out/$m.log 2>&1
  echo "$m rc=$? seconds=$(( $(date +%s) - s ))" >> out/summary.txt
done
echo done >> out/summary.txt
