#!/usr/bin/env python3
"""Write the FreeTSA certificate chain to DER from a committed corpus entry.

The corpus entries carry both certificates inside `timestamp.tsa_root` and
`timestamp.tsa_certificate` (base64), so a verifier needs no file from this
directory to check the chain. These are the live authority's certificates (the
corpus README records their SHA-256 and validity intervals), so they are the
pinned root and signing certificate the rebuilt corpus timestamps against.
"""
import base64
import json
import sys

SRC = sys.argv[1] if len(sys.argv) > 1 else "entries/alpha.json"
OUT_SIGNING = sys.argv[2] if len(sys.argv) > 2 else "tsa_certificate.der"
OUT_ROOT = sys.argv[3] if len(sys.argv) > 3 else "tsa_root.der"

with open(SRC) as handle:
    entry = json.load(handle)
anchor = entry["timestamp"]
for key, out in (("tsa_certificate", OUT_SIGNING), ("tsa_root", OUT_ROOT)):
    with open(out, "wb") as target:
        target.write(base64.b64decode(anchor[key]))
    print(f"wrote {out} from {SRC} timestamp.{key}")