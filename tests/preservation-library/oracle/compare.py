"""Compare two directories of native library sources declaration by declaration: every module's data must be
equal, except that the regenerated environment also privately imports `Lean` (the generic runtime's import,
which SPEC.md §17.10 allows). Prints one line per module and exits nonzero on any other difference.
usage: compare.py <original dir> <regenerated dir>"""
import glob, json, os, re, sys


def load(path):
    text = open(path).read()
    return json.loads(re.search(r"\\coredata\{(.*)\}\n\\end\{coremodule\}", text, re.S).group(1))


def main():
    original, regenerated = sys.argv[1], sys.argv[2]
    bad = 0
    decls = nodes = 0
    for path in sorted(glob.glob(os.path.join(original, "*.lex.tex"))):
        name = os.path.basename(path)
        a = load(path)
        b = load(os.path.join(regenerated, name))
        b["imports"] = [i for i in b["imports"] if i != "Lean"]
        same = a == b
        decls += len(a["declarations"])
        nodes += len(a["nodes"])
        print(f"{name[:-8]:<11} declarations={len(a['declarations']):<5} nodes={len(a['nodes']):<7} {'equal' if same else 'DIFFERS'}")
        if not same:
            bad += 1
            for key in sorted(set(a) | set(b)):
                if a.get(key) != b.get(key):
                    print("  differing member:", key)
    print(f"total declarations={decls} nodes={nodes} differing modules={bad}")
    sys.exit(1 if bad else 0)


main()
