#!/bin/bash
# Rebuild the committed lexeme corpus (WP-2d) with signed tree heads.
#
# The seeds are minted here and not retained, so this run produces a different
# but equally valid corpus: different signatures, different FreeTSA tokens,
# different roots, and a new verdicts.json. It is a recorded event, not a
# reproduction of the previous corpus.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LEXLEAN="${1:-$ROOT/target/debug/lexlean}"
WORK="$ROOT/.corpus-rebuild"
CORP="$ROOT/lexeme/corpus"
TOOLCHAIN="leanprover/lean4:v4.32.1"
TSA="https://freetsa.org/tsr"

rm -rf "$WORK"
mkdir -p "$WORK/ledger" "$WORK/entries" "$WORK/tsa"
python3 "$ROOT/scripts/certs-from-entry.py" "$CORP/entries/alpha.json" "$WORK/tsa/tsa_certificate.der" "$WORK/tsa/tsa_root.der"

# Three entries, because a one-entry ledger has no consistency proof and no
# second head, and §33.5's fork argument is about a tree that already has
# neighbours. The gamma source is the committed one: `def Parity.negate` inside
# `namespace Corpus.Gamma` canonicalizes to `Corpus.negate`, which is what the
# committed entry records.
declare -A TITLES=(
  [alpha]="Corpus alpha: successor and its witness"
  [beta]="Corpus beta: double and its witness"
  [gamma]="Corpus gamma: parity and its witness"
)
declare -A SRCROOT=( [alpha]="alpha" [beta]="beta" [gamma]="gamma" )

for name in alpha beta gamma; do
  src="$CORP/sources/${SRCROOT[$name]}"
  "$LEXLEAN" lexeme hash \
    --source "$src"/*.lean \
    --source-root "$src" \
    --title "${TITLES[$name]}" \
    --toolchain "$TOOLCHAIN" \
    --out "$WORK/entries/$name.json"
  # A fresh software seed per entry; the log key is one seed shared by all
  # heads, so the key that signs entries is not the key that signs heads.
  head -c 32 /dev/urandom > "$WORK/$name.seed"
  "$LEXLEAN" lexeme sign --entry "$WORK/entries/$name.json" --seed "$WORK/$name.seed" --out "$WORK/entries/$name.json"
  "$LEXLEAN" lexeme stamp --entry "$WORK/entries/$name.json" --request "$WORK/$name.tsq" --tsa "$TSA"
  curl -sS -m 30 -H 'Content-Type: application/timestamp-query' --data-binary @"$WORK/$name.tsq" "$TSA" -o "$WORK/$name.tsr"
  "$LEXLEAN" lexeme stamp --entry "$WORK/entries/$name.json" --tsr "$WORK/$name.tsr" \
    --certificate "$WORK/tsa/tsa_certificate.der" --root "$WORK/tsa/tsa_root.der" --tsa "$TSA" --out "$WORK/entries/$name.json"
  head -c 32 /dev/urandom > "$WORK/log.seed"
  "$LEXLEAN" lexeme append --ledger "$WORK/ledger" --entry "$WORK/entries/$name.json" --log-key "$WORK/log.seed" --out "$WORK/entries/$name.included.json"
done

# The recorded verdicts: the oracle the LG-14 case compares against.
python3 - "$WORK" "$CORP/verdicts.json" <<'PY'
import json, subprocess, sys
from pathlib import Path

work = Path(sys.argv[1])
out = Path(sys.argv[2])
lexlean = str(work.parent / "target" / "debug" / "lexlean")
ledger = work / "ledger"
entries = []
for name in ("alpha", "beta", "gamma"):
    entry = work / "entries" / f"{name}.included.json"
    result = subprocess.run(
        [lexlean, "--diagnostic-format", "json", "lexeme", "verify",
         "--entry", str(entry), "--ledger", str(ledger)],
        capture_output=True, text=True, check=True,
    )
    payload = json.loads(result.stdout)
    entries.append({
        "entry": f"entries/{name}.json",
        "title": payload["title"],
        "verified": payload["verified"],
        "checks": payload["checks"],
        "verdict": payload["verdict"],
    })
recorded = {
    "spec": "lexlean/corpus-verdicts/1",
    "entries": entries,
}
text = json.dumps(recorded, separators=(",", ":"), sort_keys=True) + "\n"
out.write_text(text)
print(f"wrote {out} ({len(text)} bytes) over {len(entries)} entries")
PY

# Publish the rebuilt artifacts into the committed corpus directory. The
# verdicts are written straight to the committed path by the script above, so
# they are the oracle for these bytes and not a copy of a scratch file.
cp "$WORK/entries/alpha.included.json"   "$CORP/entries/alpha.json"
cp "$WORK/entries/beta.included.json"    "$CORP/entries/beta.json"
cp "$WORK/entries/gamma.included.json"   "$CORP/entries/gamma.json"
cp "$WORK/ledger/log.json"   "$CORP/ledger/log.json"
cp "$WORK/ledger/heads.json" "$CORP/ledger/heads.json"
cp "$WORK/ledger/roots.txt"  "$CORP/ledger/roots.txt"
echo "corpus rebuilt: $(wc -l < "$CORP/ledger/heads.json") heads, $(wc -l < "$CORP/ledger/log.json") entries"