"""Eureka G3, Frente M: the NV round (Rodada 01, 13/11/2026) built with DS Apps library components.

Same five proposals as `spike_nv_round.py`, but the button, the size selector, the fit chips and the
size bottom sheet are instances of the official DS Apps components (Button, Selector Size, Choice
Tag (Chips), Bottom Sheet), read from a local snapshot of the DS Apps file (never from the repo).
Only layout glue (screen, content column, gallery placeholders, texts) is drawn, and it is bound to
Tokens APP variables. The screens run in the NV mode.

Read-only towards Figma and Notion: the snapshot is a cache made by a read-only script, nothing is
written back, and no DS component, token or rule is changed.

Usage: python3 spike_nv_round_ds.py WORKDIR TOKENS_APP_WITH_NV_JSON COMPONENTS_JSON
"""
import json
import socket
import sys
import time

S, TOK, COMP = sys.argv[1], sys.argv[2], sys.argv[3]
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=180).makefile("rw")
n = [0]
COL, MODE = "Tokens APP", "NV"
ROUND = "https://app.notion.com/p/417690e9a7e28145a5b5cf56a2214e4e"


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r)[:900]}")
    return r.get("result")


def ex(cmd, params, agent="Claude"):
    p = {"command": cmd, "params": params}
    if agent:
        p["agent"] = agent
    return call("engine.execute", p)


# --- the DS Apps components in the snapshot -------------------------------------------------------
ex("file.new", {"name": "Eureka · rodada NV · PDP App · DS Apps"}, agent=None)
ex("origin.track", {"on": True}, agent=None)
imp = ex("file.import", {"path": COMP}, agent=None)
ex("variable.import", {"variables": json.load(open(TOK))})

# Where the components came from, for the reviewer's "official component" check (`eureka.library`).
LIBRARY = json.dumps({"name": "DS Apps", "file": "U3pOCU4CEBgYKHGVR1F3sK", "snapshot": COMP.rsplit("/", 1)[-1]})
for page in imp.get("pages", []):
    for top in ex("node.get", {"id": page, "fields": ["children"]}, agent=None).get("children") or []:
        ex("node.set", {"id": top["id"], "props": {"pluginData": {"eureka.library": LIBRARY}}}, agent=None)


def component_set(name):
    for i in ex("select.byName", {"query": name, "page": "all"}, agent=None)["selection"]:
        g = ex("node.get", {"id": i, "fields": ["name", "type", "children"]}, agent=None)
        if g.get("type") == "COMPONENT_SET" and g.get("name") == name:
            return g
    raise SystemExit(f"component set {name!r} is not in the snapshot")


def variant(set_name, **want):
    """The variant whose name has every `prop=value` (prop names without their emoji prefix)."""
    for c in component_set(set_name)["children"]:
        pairs = {}
        for part in c["name"].split(", "):
            k, _, v = part.partition("=")
            pairs[k.split(" ", 1)[-1].strip()] = v.strip()
        if all(pairs.get(k) == v for k, v in want.items()):
            return c["id"]
    raise SystemExit(f"no {set_name} variant with {want}")


BUTTON = variant("Button", Tipo="Primário", Tamanho="Grande", Desabilitado="Off")
SIZE_OFF = variant("Selector Size (Selector Button)", Selecionado="Off", Desabilitado="Off", Tamanho="Grande")
SIZE_ON = variant("Selector Size (Selector Button)", Selecionado="On", Desabilitado="Off", Tamanho="Grande")
CHIP_OFF = variant("Choice Tag (Chips)", Tamanho="Médio", Selecionado="Off", Desabilitado="Off")
CHIP_ON = variant("Choice Tag (Chips)", Tamanho="Médio", Selecionado="On", Desabilitado="Off")
SHEET = variant("Bottom Sheet", **{"Modal Sheet": "Off"})
ex("select.none", {}, agent=None)

# --- drawing helpers (layout glue only, bound to Tokens APP) ---------------------------------------
V = lambda name: {"$var": f"{COL}/{name}"}  # noqa: E731
B = lambda ref, field, var: {"command": "variable.bind", "params": {"ids": [{"$ref": ref}], "field": field, "variable": var}}  # noqa: E731
k = [0]


def ref(prefix):
    k[0] += 1
    return f"{prefix}{k[0]}"


def text(parent, words, size="Font/Size/fs-xxxs", color="Colors/Content/01", wrap=False):
    r = ref("t")
    c = [
        {"command": "text.create", "as": r, "params": {"parent": {"$ref": parent}, "text": words, "x": 0, "y": 0, "name": words[:28],
            "props": {"fontSize": 12, "fill": V(color)}}},
        B(r, "fontSize", size),
    ]
    if wrap:
        c.append({"command": "node.set", "params": {"id": {"$ref": r}, "props": {"sizingH": "fill"}}})
    return c


def row(parent, name, gap="Spacing/sp-xs", fill=None):
    r = ref("r")
    props = {"fill": "none", "layout": "horizontal", "gap": 0, "padding": 0, "sizingH": "fill", "sizingV": "hug", "crossAlign": "center"}
    if fill:
        props["fill"] = V(fill)
    return r, [{"command": "frame.create", "as": r, "params": {"parent": {"$ref": parent}, "x": 0, "y": 0, "width": 10, "height": 10, "name": name, "props": props}},
               B(r, "gap", gap)]


def instance(parent, component, name, props=None, fill_width=False):
    r = ref("i")
    c = [{"command": "instance.create", "as": r, "params": {"component": component, "parent": {"$ref": parent}}},
         {"command": "node.set", "params": {"id": {"$ref": r}, "props": {"name": name, **({"sizingH": "fill"} if fill_width else {})}}}]
    if props:
        c.append({"command": "node.set", "params": {"id": {"$ref": r}, "props": {"componentProperties": props}}})
    return c


SIZES = ["PP", "P", "M", "G", "GG"]


def sizes(parent, picked=None):
    r, c = row(parent, "Tamanhos")
    for s in SIZES:
        c += instance(r, SIZE_ON if s == picked else SIZE_OFF, f"Tamanho {s}", {"✍️ Texto#4292:0": s})
    return c


def chip(parent, words, on=False):
    return instance(parent, CHIP_ON if on else CHIP_OFF, f"Chip {words}", {"✍️ Texto#21866:6": words})


def button(parent, words):
    return instance(parent, BUTTON, "Button", {"✍️ Texto#7:26": words, "👁️ Ícone": False}, fill_width=True)


def screen(variant_letter):
    c = [
        {"command": "frame.create", "as": "screen", "params": {"x": 0, "y": 0, "width": 375, "height": 812, "name": "PDP",
            "props": {"fill": V("Colors/Surface/Background"), "layout": "vertical", "padding": 0, "gap": 0, "clip": True}}},
        {"command": "shape.rectangle", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 375, "height": 360, "name": "Galeria",
            "props": {"fill": V("Colors/Surface/Container")}}},
        {"command": "frame.create", "as": "content", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 375, "height": 10, "name": "Conteúdo",
            "props": {"fill": "none", "layout": "vertical", "padding": 0, "gap": 0, "sizingH": "fill", "sizingV": "hug"}}},
        B("content", "paddingHorizontal", "Spacing/Mobile/sp-marginside"), B("content", "paddingVertical", "Spacing/sp-md"), B("content", "gap", "Spacing/sp-sm"),
    ]
    if variant_letter == "D":
        c += text("content", "Looks com esta peça", "Font/Size/fs-xxs", "Colors/Content/Pure")
        r, rc = row("content", "Looks")
        c += rc
        for i in range(3):
            c += [{"command": "shape.rectangle", "params": {"parent": {"$ref": r}, "x": 0, "y": 0, "width": 96, "height": 120, "name": f"Look {i + 1}",
                    "props": {"fill": V("Colors/Surface/Container")}}}]
        c += text("content", "Comprar o look completo  →", "Font/Size/fs-xxxs", "Colors/Content/Brand/Highlight")
    c += text("content", "Vestido midi de alça", "Font/Size/fs-xs", "Colors/Content/01")
    c += text("content", "R$ 1.290,00", "Font/Size/fs-xs", "Colors/Content/Pure")
    if variant_letter != "E":
        c += sizes("content")
    if variant_letter == "A":
        c += text("content", "A modelo veste P · 1,75 m · busto 84 cm, cintura 66 cm", "Font/Size/fs-xxxs", "Colors/Content/01")
    if variant_letter == "B":
        c += text("content", "Como veste", "Font/Size/fs-xxs", "Colors/Content/Pure")
        r, rc = row("content", "Escala de caimento")
        c += rc + chip(r, "Justo") + chip(r, "Fiel", True) + chip(r, "Folgado")
        c += text("content", "82% das clientes acharam fiel ao tamanho · ver avaliações", "Font/Size/fs-xxxs", "Colors/Content/01")
    if variant_letter == "C":
        # No DS Apps component was checked for this banner yet: drawn with tokens and named as such.
        r, rc = row("content", "Concierge NV (sem componente verificado)", fill="Colors/Feedback/Success/Surface")
        c += rc + [B(r, "paddingHorizontal", "Spacing/sp-sm"), B(r, "paddingVertical", "Spacing/sp-xs")]
        c += text(r, "Em dúvida no tamanho ou no look? Fale com o Concierge NV", "Font/Size/fs-xxxs", "Colors/Feedback/Success/Content", wrap=True)
    c += button("content", "Selecione o tamanho" if variant_letter == "E" else "Adicionar à sacola")
    if variant_letter == "E":
        c += instance("screen", SHEET, "Bottom Sheet · tamanhos",
                      {"✍️ Título": "Escolha o tamanho", "👁️ Subtítulo": False, "👁️ Voltar": False})
    return c


# From the round's own record (Notion, Rodada 01 — 13/11/2026).
ROUND_PROPOSALS = [
    ("A", "Tamanho na hora de escolher", "incremental", "informação no momento certo", "média",
     "Módulo logo abaixo do seletor com a modelo veste 36, altura e medidas da peça."),
    ("B", "Como veste", "incremental", "reduzir risco percebido", "baixa",
     "Escala justo / fiel / folgado e % das clientes que acharam fiel, com link para avaliações de caimento."),
    ("C", "Concierge NV para tamanho e styling", "incremental", "personalização", "média",
     "O Concierge NV, que já existe na PDP, vira ponto de ajuda no tamanho e no look."),
    ("D", "Galeria editorial com look comprável", "incremental", "conteúdo e inspiração", "baixa",
     "Carrossel \"Looks com esta peça\" logo após a galeria, com \"Comprar o look completo\"."),
    ("E", "Botão fixo que resolve o tamanho", "incremental", "reduzir passos", "baixa",
     "Botão fixo com estado \"Selecione o tamanho\"; ao tocar, abre o seletor de tamanhos."),
]
proposals = [{"name": name, "type": kind, "mechanism": mech, "confidence": conf, "summary": summary, "commands": screen(letter)}
             for letter, name, kind, mech, conf, summary in ROUND_PROPOSALS]

r = ex("explore.board", {"title": "NV App · PDP · Rodada 01 (13/11) · DS Apps", "brand": "NV", "platform": "app", "journey": "PDP",
                         "problem": f"Na PDP do app NV, baixa adição à sacola, dúvida de tamanho e pouca inspiração. Registro: {ROUND}",
                         "proposals": proposals})
ids = [p["id"] for p in r["proposals"]]


# Proposal E: the sizes go into the Bottom Sheet's slot (an instance's own content), M picked.
def find(node, name, depth=0):
    g = ex("node.get", {"id": node, "fields": ["name", "children"]}, agent=None)
    if g.get("name") == name:
        return node
    if depth < 6:
        for ch in g.get("children") or []:
            hit = find(ch["id"], name, depth + 1)
            if hit is not None:
                return hit
    return None


sheet = find(ids[4], "Bottom Sheet · tamanhos")
slot = find(sheet, "🖊️ Slot") if sheet is not None else None
if slot is not None:
    placeholder = find(slot, "🔁 Slot-Container")
    if placeholder is not None:
        ex("edit.delete", {"ids": [placeholder]})
    fr = ex("frame.create", {"parent": slot, "x": 0, "y": 0, "width": 10, "height": 10, "name": "Tamanhos",
                             "props": {"fill": "none", "layout": "horizontal", "padding": 0, "gap": 0, "sizingH": "fill", "sizingV": "hug"}})["id"]
    ex("variable.bind", {"ids": [fr], "field": "gap", "variable": "Spacing/sp-xs"})
    ex("variable.bind", {"ids": [fr], "field": "paddingHorizontal", "variable": "Spacing/Mobile/sp-marginside"})
    for s in SIZES:
        i = ex("instance.create", {"component": SIZE_ON if s == "M" else SIZE_OFF, "parent": fr})["id"]
        ex("node.set", {"id": i, "props": {"name": f"Tamanho {s}", "componentProperties": {"✍️ Texto#4292:0": s}}})
    h = ex("node.get", {"id": sheet, "fields": ["height"]}, agent=None).get("height", 0)
    ex("node.set", {"id": sheet, "props": {"absolute": True, "x": 0, "y": 812 - h}})
    primary = find(sheet, "🖊️ Botão Primário")
    if primary is not None:
        ex("node.set", {"id": primary, "props": {"componentProperties": {"✍️ Texto#7:26": "Adicionar à sacola"}}})

ex("variable.setMode", {"ids": ids, "collection": COL, "mode": MODE})
lint = ex("ds.lint", {"ids": ids})


def count_instances(node, depth=0):
    g = ex("node.get", {"id": node, "fields": ["type", "children"]}, agent=None)
    if g.get("type") == "INSTANCE":
        return 1
    return sum(count_instances(ch["id"], depth + 1) for ch in (g.get("children") or [])) if depth < 8 else 0


inst = [count_instances(i) for i in ids]
call("engine.execute", {"command": "select.set", "params": {"ids": [r["section"]]}})
call("engine.execute", {"command": "view.zoomSelection", "params": {}})
call("engine.execute", {"command": "select.none", "params": {}})
time.sleep(3)
call("ui.screenshot", {"path": f"{S}/M1-round-ds.png"})
call("engine.execute", {"command": "select.set", "params": {"ids": [r["section"]]}})
call("engine.execute", {"command": "file.export", "params": {"ids": [r["section"]], "format": "png", "scale": 2, "path": f"{S}/M2-round-ds-export.png"}})
call("engine.execute", {"command": "select.none", "params": {}})
lines = [f"Rodada NV com componentes DS Apps: {len(ids)} propostas — " + "; ".join(p["name"] for p in r["proposals"]),
         "Instâncias oficiais por tela (A–E): " + ", ".join(str(x) for x in inst),
         f"Import do snapshot: {len(imp.get('warnings', []))} aviso(s)" + "".join(f"\n  · {w}" for w in imp.get("warnings", [])),
         f"Lint: {len(lint['findings'])} achado(s) da proposta; {lint.get('inherited', 0)} valor(es) fixo(s) herdado(s) dos componentes do DS (não contam para a tela)" + "".join(f"\n  · {x['message']}" for x in lint["findings"][:12]),
         "Nada gravado no Notion nem no Figma; nenhum componente, token ou regra do DS foi alterado."]
open(f"{S}/M-round-ds.txt", "w").write("\n".join(lines) + "\n")
print(json.dumps({"section": r["section"], "screens": ids}))
print("\n".join(lines))
