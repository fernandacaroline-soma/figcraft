"""Eureka G3, Frente N: a round of the proposals generator ("gerador-propostas") drawn by the Eureka.

The skill hands over its proposals as JSON (`rodada_nv01_propostas.json`): for each one, what it is
(name, type, mechanism, confidence, summary) and its screen described by DS components, variants,
texts and tokens, never by layer ids. Like the skill's own Figma recipe, each screen starts from the
journey's base screen and adds its modules; this script only expands that base (`$base`) into the
full description `explore.board` takes. The engine checks every component, variant, property and
token against the DS snapshot before drawing anything.

Read-only towards Figma and Notion: the DS comes from local snapshots (outside the repo), nothing is
written back, and no DS component, token or rule is changed.

Usage: python3 rodada_n.py WORKDIR TOKENS_JSON COMPONENTS_JSON PROPOSALS_JSON [MORE_PROPOSALS_JSON...]
(more files add their proposals to the same board, such as the disruptive ones of Frente P).
"""
import copy
import json
import socket
import sys
import time

ARGS = [a for a in sys.argv[1:] if not a.startswith("--base=")]
# --base=template: the base screen is built from DS Apps' official PDP template parts (Frente T)
# instead of the hand-drawn one; needs a snapshot with them (components-app-frente-t-*).
BASE = next((a.split("=", 1)[1] for a in sys.argv[1:] if a.startswith("--base=")), "drawn")
S, TOK, COMP, PROPS = ARGS[:4]
MORE = ARGS[4:]
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=180).makefile("rw")
__import__("eureka_control").auth(f, PORT)
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
        content.append(sizes(mods.get("picked")))  # `picked`: the size shown selected
    content += mods.get("after_sizes", [])
    if not mods.get("no_button"):  # a proposal with its own action area (Action Button) leaves it out
        content.append({"component": "Button", "name": "Button", "variant": {"Tipo": "Primário", "Tamanho": "Grande", "Desabilitado": "Off"},
                        "props": {"Texto": mods.get("button", "Adicionar à sacola"), "Ícone": False}, "width": "fill"})
    return {"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Surface/Background", "children": [
        {"image": "Galeria", "width": 375, "height": 360, "fill": "Colors/Surface/Container"},
        {"frame": "Conteúdo", "padding": {"h": "Spacing/Mobile/sp-marginside", "v": "Spacing/sp-md"}, "gap": "Spacing/sp-sm", "children": content},
    ] + mods.get("overlay", [])}


def pdp_template(mods):
    """The PDP base screen from DS Apps' official template parts (page "Página de Produto"): the
    gallery, the product's main info, the size selection and the fixed action bar, with a proposal's
    modules in place. A long, scrolling page as in production; the fixed bar docks at the bottom.
    Texts inside the parts are the template's example content."""
    content = [{"component": ".Nested / PDP-product-gallery", "variant": {"Complete o look": "False"}, "name": "Galeria (template)"}]
    content += mods.get("before_title", [])
    content.append({"component": ".Nested / pdp-main-info", "variant": {"Sale": "False", "Indisponível": "false"}, "name": "Informações (template)"})
    if not mods.get("no_sizes"):
        content.append({"component": ".Nested / size-selection-PDP", "variant": {"Sem estoque": "False"}, "name": "Tamanhos (template)"})
    after = mods.get("after_sizes", [])
    if after:
        content.append({"frame": "Módulos da proposta", "padding": {"h": "Spacing/Mobile/sp-marginside", "v": "Spacing/sp-md"},
                        "gap": "Spacing/sp-sm", "width": "fill", "children": after})
    overlay = mods.get("overlay", [])
    if not mods.get("no_button"):
        overlay = [{"component": ".Nested / fixed-component", "variant": {"Sale": "False"}, "name": "Área fixa (template)", "dock": "bottom"}] + overlay
    return {"frame": "PDP", "width": 375, "height": 1600, "fill": "Colors/Surface/Background", "children": content + overlay}


round_ = json.load(open(PROPS))
for extra in MORE:
    added = json.load(open(extra))["proposals"]
    round_["proposals"] += added
    # Say where they came from: proposals added for a test are not the generator's output.
    round_["title"] += f" + {len(added)} de teste do Eureka"
if BASE == "template":
    round_["title"] += " · base: template oficial da PDP"
board = copy.deepcopy(round_)
for p in board["proposals"]:
    sc = p["screen"]
    base = pdp_template if BASE == "template" else pdp
    p["screen"] = expand(base(sc) if sc.get("$base") == "pdp" else sc)
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
