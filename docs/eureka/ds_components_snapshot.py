"""Join the chunks of `ds_components_snapshot.js` into the components cache.

Usage: python3 ds_components_snapshot.py assemble OUT.json CHUNK.json...
Each CHUNK.json is one `use_figma` result ({packed?, total, chunks, chunk, data}); they may come in any
order. A packed result (LZW over the UTF-8 bytes, base64: see `pack` in the script) is unpacked here.
OUT.json goes to `eureka-cache/` (outside the repository), named with a hash of its content.
"""
import hashlib
import json
import sys

PACK_ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/'
PACK_LIMIT = 1 << 20


def unpack(t):
    val = {ch: i for i, ch in enumerate(PACK_ALPHABET)}
    bits = ''.join(format(val[ch], '06b') for ch in t)
    pos = 0; i = 0; codes = []
    while True:
        width = len(format(min(256 + i, PACK_LIMIT), 'b'))
        if pos + width > len(bits): break
        codes.append(int(bits[pos:pos + width], 2)); pos += width; i += 1
    d = {j: bytes([j]) for j in range(256)}; nxt = 256; w = d[codes[0]]; out = [w]
    for c in codes[1:]:
        e = d[c] if c in d else w + w[:1]
        out.append(e)
        if nxt < PACK_LIMIT: d[nxt] = w + e[:1]
        nxt += 1; w = e
    return b''.join(out).decode('utf-8')


def assemble(out, parts):
    got, n, packed = {}, None, False
    for p in parts:
        r = json.load(open(p, encoding="utf-8"))
        got[r["chunk"]] = r["data"]
        n, packed = r["chunks"], r.get("packed", False)
    if n is None or sorted(got) != list(range(n)):
        raise SystemExit(f"missing chunks: have {sorted(got)} of {n}")
    text = "".join(got[i] for i in range(n))
    if packed:
        text = unpack(text)
    json.loads(text)  # must be one valid JSON document
    open(out, "w", encoding="utf-8").write(text)
    print(out, hashlib.sha256(text.encode()).hexdigest()[:12])


if __name__ == "__main__":
    if len(sys.argv) < 4 or sys.argv[1] != "assemble":
        raise SystemExit(__doc__)
    assemble(sys.argv[2], sys.argv[3:])
