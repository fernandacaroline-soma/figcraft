"""Eureka G3, Frente N: a round of the proposals generator ("gerador-propostas") drawn by the Eureka.

The skill hands over its proposals as JSON (`rodada_nv01_propostas.json`): for each one, what it is
(name, type, mechanism, confidence, summary) and its screen described by DS components, variants,
texts and tokens, never by layer ids. Like the skill's own Figma recipe, each screen starts from the
journey's base screen and adds its modules; this script only expands that base (`$base`) into the
full description `explore.board` takes. The engine checks every component, variant, property and
token against the DS snapshot before drawing anything.

Read-only towards Figma and Notion: the DS comes from local snapshots (outside the repo), nothing is
written back, and no DS component, token or rule is changed.

Usage: python3 rodada_n.py WORKDIR TOKENS_JSON COMPONENTS_JSON PROPOSALS_JSON
"""
import copy
import json
import socket
import sys
import time

S, TOK, COMP, PROPS = sys.argv[1:5]
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


def ex(cmd, params, agent="Claude"):
    p = {"command": cmd, "params": params}
    if agent:
        p["agent"] = agent
    return call("engine.execute", p)


ex("file.new", {"name": "Eureka · Frente N · rodada gerada"}, agent=None)
ex("origin.track", {"on": True}, agent=None)
imp = ex("file.import", {"path": COMP}, agent=None)
ex("variable.import", {"variables": json.load(open(TOK))})
LIBRARY = json.dumps({"name": "DS Apps", "file": "U3pOCU4CEBgYKHGVR1F3sK", "snapshot": COMP.rsplit("/", 1)[-1]})
for page in imp.get("pages", []):
    for top in ex("node.get", {"id": page, "fields": ["children"]}, agent=None).get("children") or []:
        ex("node.set", {"id": top["id"], "props": {"pluginData": {"eureka.library": LIBRARY}}}, agent=None)

# What the skill can use, by the names a description takes (stands in for `search_design_system`).
catalog = ex("explore.catalog", {}, agent=None)
open(f"{S}/N0-catalogo.json", "w").write(json.dumps(catalog, ensure_ascii=False, indent=1))

SIZES = ["PP", "P", "M", "G", "GG"]


def sizes(picked=None):
    return {"frame": "Tamanhos", "layout": "horizontal", "gap": "Spacing/sp-xs", "children": [
        {"component": "Selector Size (Selector Button)", "name": f"Tamanho {s}",
         "variant": {"Selecionado": "On" if s == picked else "Off", "Desabilitado": "Off", "Tamanho": "Grande"}, "props": {"Texto": s}}
        for s in SIZES]}


def expand(item):
    """The skill's `{"$sizes": picked}` shorthand for the size row, anywhere in a description."""
    if isinstance(item, dict) and "$sizes" in item:
        return sizes(item["$sizes"])
    if isinstance(item, dict):
        return {k: expand(v) for k, v in item.items()}
    if isinstance(item, list):
        return [expand(v) for v in item]
    return item


def pdp(mods):
    """The PDP base screen with a proposal's modules in place."""
    content = mods.get("before_title", []) + [
        {"text": "Vestido midi de alça", "size": "Font/Size/fs-xs", "color": "Colors/Content/01"},
        {"text": "R$ 1.290,00", "size": "Font/Size/fs-xs", "color": "Colors/Content/Pure"},
    ]
    if not mods.get("no_sizes"):
        content.append(sizes())
    content += mods.get("after_sizes", [])
    content.append({"component": "Button", "name": "Button", "variant": {"Tipo": "Primário", "Tamanho": "Grande", "Desabilitado": "Off"},
                    "props": {"Texto": mods.get("button", "Adicionar à sacola"), "Ícone": False}, "width": "fill"})
    return {"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Surface/Background", "children": [
        {"image": "Galeria", "width": 375, "height": 360, "fill": "Colors/Surface/Container"},
        {"frame": "Conteúdo", "padding": {"h": "Spacing/Mobile/sp-marginside", "v": "Spacing/sp-md"}, "gap": "Spacing/sp-sm", "children": content},
    ] + mods.get("overlay", [])}


round_ = json.load(open(PROPS))
board = copy.deepcopy(round_)
for p in board["proposals"]:
    sc = p["screen"]
    p["screen"] = expand(pdp(sc) if sc.get("$base") == "pdp" else sc)
open(f"{S}/N1-descricao-expandida.json", "w").write(json.dumps(board, ensure_ascii=False, indent=1))

r = ex("explore.board", board)
ids = [p["id"] for p in r["proposals"]]
ex("variable.setMode", {"ids": ids, "collection": "Tokens APP", "mode": round_["brand"]})
lint = ex("ds.lint", {"ids": ids})
infos = [json.loads(ex("node.get", {"id": i, "fields": ["pluginData"]}, agent=None)["pluginData"]["eureka.proposal"]) for i in ids]

call("engine.execute", {"command": "select.set", "params": {"ids": [r["section"]]}})
call("engine.execute", {"command": "view.zoomSelection", "params": {}})
call("engine.execute", {"command": "file.export", "params": {"ids": [r["section"]], "format": "png", "scale": 2, "path": f"{S}/N2-rodada.png"}})
# The designer's pick through the card's button is `explore.choose`; here only shown, not made.
call("engine.execute", {"command": "select.set", "params": {"ids": [ids[0]]}})
time.sleep(3)
call("ui.screenshot", {"path": f"{S}/N3-cartao-escolher.png"})
call("engine.execute", {"command": "select.none", "params": {}})

lines = [f"Rodada gerada: {len(ids)} propostas — " + "; ".join(p["name"] for p in r["proposals"])]
for info in infos:
    lines.append(f"  {info['letter']} · componentes do DS: {', '.join(info['ds']['components'])} · {len(info['ds']['variables'])} token(s)")
lines += [f"Lint: {len(lint['findings'])} achado(s) da proposta; {lint.get('inherited', 0)} valor(es) herdado(s) dos componentes do DS"]
lines += [f"  · {x['message']}" for x in lint["findings"][:12]]
lines += ["Nada gravado no Notion nem no Figma; nenhum componente, token ou regra do DS foi alterado."]
open(f"{S}/N-rodada.txt", "w").write("\n".join(lines) + "\n")
print(json.dumps({"section": r["section"], "screens": ids}))
print("\n".join(lines))
