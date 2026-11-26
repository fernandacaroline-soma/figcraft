"""Eureka G3, Frente Y: a code map for DS Apps' PDP template parts, and what it changes in the export.

Run after `rodada_n.py --base=template` (same WORKDIR, app still open). It:
1. exports the screen as TSX without a map (the "before");
2. drafts a Code Connect map for every DS component in the file: `export.codeMap` lists them as
   `missing`; a first pass with empty templates lists each one's properties (`unmapped`); the draft
   reads every property by its name in code (`<PdpMainInfo sale={sale} … />`);
3. checks the draft (`export.codeMap`), links it (`codeConnect.import`) and exports again (the "after").

The code names, the package and the prop names are a PROPOSAL drawn from the DS names (same name as
the DS, as the team uses): Tecnologia confirms them. The map is written to the cache, outside the
repo; the report in WORKDIR has only names and counts. Read-only towards Figma and the DS.

Usage: python3 mapa_y.py WORKDIR SCREEN_ID MAP_PATH [PACKAGE]
"""
import json
import re
import socket
import sys
import unicodedata

S, SCREEN, MAP = sys.argv[1:4]
PACKAGE = sys.argv[4] if len(sys.argv) > 4 else "@azzas/ds-apps"
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=180).makefile("rw")
n = [0]


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r, ensure_ascii=False)[:1500]}")
    return r.get("result")


def ex(cmd, params):
    return call("engine.execute", {"command": cmd, "params": params})


def words(name):
    plain = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode()
    plain = re.sub(r"\(.*?\)", " ", plain)  # "Choice Tag (Chips)": the name, not the alias
    plain = re.sub(r"^\.?nested\s*/\s*", "", plain.strip(), flags=re.I)  # ".Nested / x": a part of a template
    return [w for w in re.split(r"[^A-Za-z0-9]+", plain) if w]


def pascal(name):
    out = "".join(w[:1].upper() + w[1:].lower() if w.isupper() and len(w) > 1 else w[:1].upper() + w[1:] for w in words(name))
    return out if out[:1].isalpha() else f"Ds{out}"


def export():
    r = ex("export.tsx", {"ids": [SCREEN]})
    return {"warnings": len(r["warnings"]), "calls": [c["name"] for c in r["components"]],
            "inside": [(c["name"], c.get("inside") or []) for c in r["components"] if c.get("inside")],
            "drawn": [x for x in r["notes"] if "no code component" in x]}


before = export()

# Every DS component in the file, then each one's properties.
empty = ex("export.codeMap", {"json": {"version": 1, "links": []}})
names = sorted({m["component"] if isinstance(m, dict) else m for m in empty["missing"]})
draft = {"version": 1, "links": [{"component": c, "language": "react", "importPath": PACKAGE, "name": pascal(c), "template": f"<{pascal(c)} />"} for c in names]}
# A placeholder reads the property by its name in code (`param`): the DS names carry emoji and ids
# ("🤏 Tamanho", "✍️ Texto#7:26") that a template can't spell. The prop takes that same name.
props = {}
for m in ex("export.codeMap", {"json": draft})["mapped"]:
    props[m["component"]] = [q["param"] for q in ex("inspect.componentCode", {"id": m["id"]})["properties"]]
for link in draft["links"]:
    ps = props.get(link["component"], [])
    attrs = " ".join(f"{p}={{{p}}}" for p in ps)
    link["template"] = f"<{link['name']} {attrs} />" if attrs else f"<{link['name']} />"
json.dump(draft, open(MAP, "w"), ensure_ascii=False, indent=1)

check = ex("export.codeMap", {"path": MAP})
ex("codeConnect.import", {"path": MAP})
after = export()

lines = [f"Mapa: {len(draft['links'])} componente(s) do DS, pacote {PACKAGE} (proposta; a Tecnologia confirma)."]
lines += [f"  · {l['component']} → {l['name']} ({len(props.get(l['component'], []))} prop(s))" for l in draft["links"]]
c = check["counts"]
lines += [f"export.codeMap: {c.get('mapped', 0)} ligado(s), {c.get('missing', 0)} sem entrada, {c.get('invalid', 0)} inválido(s), "
          f"{sum(len(m['unknown']) for m in check['mapped'])} chave(s) desconhecida(s), {sum(len(m['unmapped']) for m in check['mapped'])} prop(s) sem leitura."]
lines += [f"Antes: {before['warnings']} valor(es) sem token, {len(before['drawn'])} componente(s) do DS desenhado(s) como camadas, {len(before['calls'])} chamada(s)."]
lines += [f"Depois: {after['warnings']} valor(es) sem token, {len(after['drawn'])} desenhado(s), {len(after['calls'])} chamada(s): {', '.join(after['calls'])}."]
lines += [f"  · ainda desenhado: {x}" for x in after["drawn"]]
lines += [f"  · {name} não leva o que a proposta trocou dentro dele (vai como nota no cabeçalho): {'; '.join(ch)}" for name, ch in after["inside"]]
lines += ["Figma só lido; nenhum componente, token ou regra do DS alterado; o mapa fica no cache, fora do repositório."]
open(f"{S}/Y-mapa.txt", "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
