#!/bin/bash
# compile the compiler's generated modules and the 13 certification modules over the existing calculus/preservation oleans
W=$(cd "$(dirname "$0")" && pwd)
TC=$HOME/.elan/toolchains/leanprover--lean4---v4.32.1
export PATH=$TC/bin:$PATH
export LEAN_PATH=$W/out
cd $W
: > $W/summary.txt
step() { # module path
  m=$1; f=src/$(echo $m | tr . /).lean
  mkdir -p out/$(dirname $(echo $m | tr . /))
  s=$(date +%s)
  lean -R src -o out/$(echo $m | tr . /).olean $f > out/$m.log 2>&1
  echo "$m rc=$? seconds=$(( $(date +%s) - s ))" >> summary.txt
}
for m in LexLeanTarget.Gnaf LexLeanTarget.GnafFixtures LexLeanTarget.Certification LexLeanTarget.GnafCertificates LexLeanCertification.Trials LexLeanCertification.Tail LexLeanCertification.Factor LexLeanCertification.Schemas LexLeanCertification.Facts LexLeanCertification.PremiseSound LexLeanCertification.Positions LexLeanCertification.Lower LexLeanCertification.Frontier LexLeanCertification.Representation LexLeanCertification.Normality LexLeanCertification.Check LexLeanCertification.Soundness; do step $m; done
echo done >> summary.txt
