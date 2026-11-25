"""Join the chunks of `ds_components_snapshot.js` into the components cache.

Usage: python3 ds_components_snapshot.py assemble OUT.json CHUNK.json...
Each CHUNK.json is one `use_figma` result ({total, chunks, chunk, data}); they may come in any order.
OUT.json goes to `eureka-cache/` (outside the repository), named with a hash of its content.
"""
import hashlib
import json
import sys


def assemble(out, parts):
    got, n = {}, None
    for p in parts:
        r = json.load(open(p, encoding="utf-8"))
        got[r["chunk"]] = r["data"]
        n = r["chunks"]
    if n is None or sorted(got) != list(range(n)):
        raise SystemExit(f"missing chunks: have {sorted(got)} of {n}")
    text = "".join(got[i] for i in range(n))
    json.loads(text)  # must be one valid JSON document
    open(out, "w", encoding="utf-8").write(text)
    print(out, hashlib.sha256(text.encode()).hexdigest()[:12])


if __name__ == "__main__":
    if len(sys.argv) < 4 or sys.argv[1] != "assemble":
        raise SystemExit(__doc__)
    assemble(sys.argv[2], sys.argv[3:])
