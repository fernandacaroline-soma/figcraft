"""Eureka token snapshot tool (ADR-002, decisions D2 and D3).

Figma Variables are the source of truth. The token values live in a read-only cache OUTSIDE the
repository (`eureka-cache/`); the repository only keeps a manifest: names, types, scopes, modes,
alias targets and a hash of the values, never the values themselves.

The cache is a Figma local-variables export (`{meta: {variableCollections, variables}}`, the
shape `variable.import` reads) plus an `eureka_snapshot` block. It may also hold the file's text
styles as the Figma Plugin API reads them (`meta.textStyles`: `{id: {name, fontName, fontSize,
lineHeight, letterSpacing, boundVariables: {field: VARIABLE_ALIAS}}}`); the manifest keeps their
names and the names of the variables bound to them, and the hash covers their values. Aliases (D3) are kept as
`VARIABLE_ALIAS` values; when the export also carries `resolvedValuesByMode`, both are kept.

Usage:
  python3 tokens_snapshot.py manifest CACHE.json [OUT.json]   write the manifest (stdout without OUT)
  python3 tokens_snapshot.py check CACHE.json MANIFEST.json   exit 1 when the cache drifted
  python3 tokens_snapshot.py resolve CACHE.json MODE          print name -> resolved value for a mode
"""
import hashlib
import json
import sys

MAX_ALIAS_DEPTH = 16


def load(path):
    with open(path, encoding="utf-8") as f:
        d = json.load(f)
    meta = d.get("meta") or {}
    if not isinstance(meta.get("variables"), dict) or not isinstance(meta.get("variableCollections"), dict):
        raise SystemExit(f"{path}: not a Figma variables export (meta.variables / meta.variableCollections)")
    return d


def norm(v):
    """Comparable values: colours as 8-bit channels, other numbers at 3 decimals, recursively."""
    if isinstance(v, float):
        return round(v, 3) + 0.0
    if isinstance(v, int) and not isinstance(v, bool):
        return float(v)
    if isinstance(v, dict) and {"r", "g", "b"} <= set(v):
        # Colours as 8-bit channels: exports round the 0–1 floats differently.
        return {k: round(float(v.get(k, 1)) * 255) for k in ("r", "g", "b", "a")}
    if isinstance(v, dict):
        return {k: norm(x) for k, x in v.items()}
    if isinstance(v, list):
        return [norm(x) for x in v]
    return v


def values_hash(d):
    """A hash of every variable's values by mode (aliases by target name, numbers at 3 decimals),
    order independent."""
    meta = d["meta"]
    names = {vid: v.get("name") for vid, v in meta["variables"].items()}
    rows = []
    for v in meta["variables"].values():
        vals = {}
        for mode, val in sorted((v.get("valuesByMode") or {}).items()):
            if isinstance(val, dict) and val.get("type") == "VARIABLE_ALIAS":
                vals[mode] = {"alias": names.get(val.get("id"), val.get("id"))}
            else:
                vals[mode] = norm(val)
        rows.append([v.get("name"), vals])
    for s in (meta.get("textStyles") or {}).values():
        bound = {k: names.get((b or {}).get("id"), (b or {}).get("id")) for k, b in sorted((s.get("boundVariables") or {}).items())}
        vals = {k: norm(s.get(k)) for k in ("fontName", "fontSize", "lineHeight", "letterSpacing")}
        rows.append(["textStyle:" + str(s.get("name")), {"bound": bound, "values": vals}])
    rows.sort(key=lambda r: str(r[0]))
    return hashlib.sha256(json.dumps(rows, sort_keys=True, ensure_ascii=False).encode()).hexdigest()[:12]


def manifest(d):
    meta = d["meta"]
    names = {vid: v.get("name") for vid, v in meta["variables"].items()}
    cols = []
    for c in meta["variableCollections"].values():
        cols.append({"name": c.get("name"), "modes": [m.get("name") for m in c.get("modes", [])]})
    variables = []
    for v in meta["variables"].values():
        col = meta["variableCollections"].get(v.get("variableCollectionId"), {})
        aliases = sorted(
            {names.get(val.get("id"), val.get("id")) for val in (v.get("valuesByMode") or {}).values() if isinstance(val, dict) and val.get("type") == "VARIABLE_ALIAS"}
        )
        row = {"name": v.get("name"), "collection": col.get("name"), "type": v.get("resolvedType"), "scopes": sorted(v.get("scopes") or [])}
        if aliases:
            row["aliasOf"] = aliases
        variables.append(row)
    variables.sort(key=lambda r: (str(r["collection"]), str(r["name"])))
    styles = []
    for s in (meta.get("textStyles") or {}).values():
        bound = {k: names.get((b or {}).get("id"), (b or {}).get("id")) for k, b in (s.get("boundVariables") or {}).items()}
        styles.append({"name": s.get("name"), "bound": dict(sorted(bound.items()))})
    styles.sort(key=lambda r: str(r["name"]))
    snap = d.get("eureka_snapshot") or {}
    extra = {"textStyles": styles} if styles else {}
    return {
        "note": "Manifesto do snapshot de tokens. Sem valores: a fonte é o Figma (Variables) e o cache fica fora do repo.",
        "source": {k: snap.get(k) for k in ("source", "fileKey", "collection", "collectionKey", "exportedAt") if k in snap},
        "aliasesKept": any("aliasOf" in r for r in variables) or not snap.get("aliasesResolved", False),
        "collections": sorted(cols, key=lambda c: str(c["name"])),
        "count": len(variables),
        "valuesHash": values_hash(d),
        "variables": variables,
        **extra,
    }


def resolve(d, mode_name):
    """name -> value in mode `mode_name`, following aliases (cycles and depth are bounded)."""
    meta = d["meta"]
    cols = meta["variableCollections"]
    out = {}
    for vid, v in meta["variables"].items():
        val, cur, seen = None, v, set()
        mode = next((m["modeId"] for m in cols.get(v.get("variableCollectionId"), {}).get("modes", []) if m.get("name") == mode_name), None)
        for _ in range(MAX_ALIAS_DEPTH):
            if cur is None or cur.get("id") in seen:
                val = None
                break
            seen.add(cur.get("id"))
            col = cols.get(cur.get("variableCollectionId"), {})
            m = mode if mode in (cur.get("valuesByMode") or {}) else col.get("defaultModeId")
            val = (cur.get("valuesByMode") or {}).get(m)
            if isinstance(val, dict) and val.get("type") == "VARIABLE_ALIAS":
                cur = meta["variables"].get(val.get("id"))
                continue
            break
        out[v.get("name")] = val
    return out


def main(argv):
    if len(argv) < 3:
        raise SystemExit(__doc__)
    cmd, cache = argv[1], load(argv[2])
    if cmd == "manifest":
        text = json.dumps(manifest(cache), indent=1, ensure_ascii=False) + "\n"
        if len(argv) > 3:
            with open(argv[3], "w", encoding="utf-8") as f:
                f.write(text)
        else:
            sys.stdout.write(text)
    elif cmd == "check" and len(argv) > 3:
        with open(argv[3], encoding="utf-8") as f:
            want = json.load(f)
        got = manifest(cache)
        problems = []
        if got["valuesHash"] != want.get("valuesHash"):
            problems.append(f"values changed: {want.get('valuesHash')} -> {got['valuesHash']} (export again from Figma and refresh the manifest)")
        wn = {r["name"]: r for r in want.get("variables", [])}
        gn = {r["name"]: r for r in got["variables"]}
        for n in sorted(set(wn) - set(gn)):
            problems.append(f"missing in cache: {n}")
        for n in sorted(set(gn) - set(wn)):
            problems.append(f"not in manifest: {n}")
        for n in sorted(set(wn) & set(gn)):
            if wn[n] != gn[n]:
                problems.append(f"structure differs: {n}")
        ws = {r["name"]: r for r in want.get("textStyles", [])}
        gs = {r["name"]: r for r in got.get("textStyles", [])}
        for n in sorted(set(ws) ^ set(gs)):
            problems.append(f"text style {'missing in cache' if n in ws else 'not in manifest'}: {n}")
        for n in sorted(set(ws) & set(gs)):
            if ws[n] != gs[n]:
                problems.append(f"text style structure differs: {n}")
        print("ok" if not problems else "\n".join(problems))
        raise SystemExit(1 if problems else 0)
    elif cmd == "resolve" and len(argv) > 3:
        print(json.dumps(resolve(cache, argv[3]), indent=1, ensure_ascii=False))
    else:
        raise SystemExit(__doc__)


if __name__ == "__main__":
    main(sys.argv)
