"""Eureka E13: the Brand Resolver on a real round. A proposal's screens are designed for a brand with
`brand.apply`: the brand's mode of Tokens APP, the brand's profile from the gerador-propostas skill
(read as written) and its entry of the skill's brand registry. Read only towards the skill's
repository: the profile and the registry are copies kept in the cache, outside this repo.

Run after `rodada_n.py` (same WORKDIR, app open).

Usage: python3 marca_e13.py WORKDIR SCREEN_ID BRAND REGISTRY_YAML [PROFILE_MD]
Writes WORKDIR/E13-<brand>.png (the card) and prints what brand.apply returned (names and counts).
"""
import json
import re
import socket
import sys
import time

S, SCREEN, BRAND, REGISTRY = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
PROFILE = sys.argv[5] if len(sys.argv) > 5 else None
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=120).makefile("rw")
__import__("eureka_control").auth(f, PORT)
n = [0]


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r, ensure_ascii=False)[:800]}")
    return r.get("result")


def scalar(v):
    v = v.split(" #", 1)[0].strip()
    if v in ("null", "~", ""):
        return None
    if v.startswith("[") and v.endswith("]"):
        return [x.strip() for x in v[1:-1].split(",") if x.strip()]
    return v


def brand_entry(path, brand):
    """The brand's entry under `marcas:` of the registry (two-space indented YAML: keys, scalars,
    inline lists and one nested map). Enough for this file; not a general YAML reader."""
    lines = open(path, encoding="utf-8").read().splitlines()
    out, sub, inside = {}, None, False
    for line in lines:
        if not line.strip() or line.lstrip().startswith("#"):
            continue
        indent = len(line) - len(line.lstrip())
        m = re.match(r"\s*([^:]+):\s*(.*)$", line)
        if not m:
            continue
        key, value = m.group(1).strip(), m.group(2)
        if indent == 2:
            if inside:
                break
            inside = key == brand
            continue
        if not inside:
            continue
        if indent == 4:
            if value.strip() == "":
                sub = out.setdefault(key, {})
            else:
                out[key], sub = scalar(value), None
        elif indent == 6 and sub is not None:
            sub[key] = scalar(value)
    return out


registry = brand_entry(REGISTRY, BRAND)
params = {"ids": [SCREEN], "brand": BRAND, "registry": registry}
if PROFILE:
    params["path"] = PROFILE
r = call("engine.execute", {"command": "brand.apply", "params": params})
call("engine.execute", {"command": "select.set", "params": {"ids": [SCREEN]}})
time.sleep(3)
call("ui.screenshot", {"path": f"{S}/E13-{BRAND}.png"})
print(json.dumps({"brand": r["brand"], "mode": r["mode"], "basis": r["basis"], "profile": r["profile"],
                  "official": sorted(r["official"].keys()), "gaps": r["gaps"]}, ensure_ascii=False, indent=1))
