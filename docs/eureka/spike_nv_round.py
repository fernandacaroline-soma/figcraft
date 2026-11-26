"""Eureka G3, Frente I: a REAL round of the proposals generator on the exploration board.

Round 01 of 13/11/2026 in the Notion "Registro de propostas": NV · App · PDP, "baixa adição à
sacola, dúvida de tamanho e pouca inspiração" (status Propostas geradas, waiting for the designer's
choice). Its five proposals (A–E) are drawn here side by side with the DS Apps tokens in the NV
mode, each carrying the round's own name, mechanism, confidence and summary.

Read-only towards the round: nothing is written to Notion and the designer's Figma branch is not
touched. The screens approximate the round's Figma frames with tokens only: the Eureka does not
place DS Apps library components yet (Tag, Choice Tag, Bottom Sheet, Selector Size).

Usage: python3 spike_nv_round.py WORKDIR TOKENS_APP_WITH_NV_JSON
"""
import json
import socket
import sys

S, TOK = sys.argv[1], sys.argv[2]
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=120).makefile("rw")
__import__("eureka_control").auth(f, PORT)
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


V = lambda name: {"$var": f"{COL}/{name}"}  # noqa: E731
B = lambda ref, field, var: {"command": "variable.bind", "params": {"ids": [{"$ref": ref}], "field": field, "variable": var}}  # noqa: E731
k = [0]


def ref(prefix):
    k[0] += 1
    return f"{prefix}{k[0]}"


def text(parent, words, size="Font/Size/fs-xxxs", color="Colors/Content/01"):
    r = ref("t")
    return [
        {"command": "text.create", "as": r, "params": {"parent": {"$ref": parent}, "text": words, "x": 0, "y": 0, "name": words[:28],
            "props": {"fontSize": 12, "fill": V(color)}}},
        B(r, "fontSize", size),
    ]


def row(parent, name, gap="Spacing/sp-xs", fill=None):
    r = ref("r")
    props = {"fills": [], "layout": "horizontal", "gap": 0, "crossAlign": "center", "sizingH": "hug", "sizingV": "hug"}
    if fill:
        props["fill"] = V(fill)
    return r, [{"command": "frame.create", "as": r, "params": {"parent": {"$ref": parent}, "x": 0, "y": 0, "width": 10, "height": 10, "name": name, "props": props}},
               B(r, "gap", gap)]


def chip(parent, words, on=False):
    r = ref("c")
    return [
        {"command": "frame.create", "as": r, "params": {"parent": {"$ref": parent}, "x": 0, "y": 0, "width": 40, "height": 32, "name": f"Chip {words}",
            "props": {"fill": V("Colors/Surface/Brand/Background" if on else "Colors/Surface/Background"), "stroke": V("Colors/Surface/Brand/Stroke"), "strokeWeight": 1,
                      "layout": "horizontal", "padding": 0, "mainAlign": "center", "crossAlign": "center", "sizingH": "hug", "sizingV": "hug"}}},
        B(r, "paddingHorizontal", "Spacing/sp-sm"), B(r, "paddingVertical", "Spacing/sp-xs"), B(r, "cornerRadius", "Border/Radius/Components/br-chips"),
        *text(r, words, "Font/Size/fs-xxxs", "Colors/Content/Brand/On Brand" if on else "Colors/Content/01"),
    ]


def sizes(parent, picked=None):
    r, c = row(parent, "Tamanhos")
    for s in ("PP", "P", "M", "G", "GG"):
        c += chip(r, s, s == picked)
    return c


def button(parent, words):
    r = ref("b")
    return [
        {"command": "frame.create", "as": r, "params": {"parent": {"$ref": parent}, "x": 0, "y": 0, "width": 328, "height": 48, "name": "Button",
            "props": {"fill": V("Colors/Surface/Brand/Background"), "layout": "horizontal", "mainAlign": "center", "crossAlign": "center", "sizingH": "fill"}}},
        B(r, "cornerRadius", "Border/Radius/Components/br-button"),
        *text(r, words, "Font/Size/fs-xxs", "Colors/Content/Brand/On Brand"),
    ]


def screen(variant):
    c = [
        {"command": "frame.create", "as": "screen", "params": {"x": 0, "y": 0, "width": 360, "height": 760, "name": "PDP",
            "props": {"fill": V("Colors/Surface/Background"), "layout": "vertical", "padding": 0, "gap": 0}}},
        B("screen", "paddingHorizontal", "Spacing/sp-md"), B("screen", "paddingVertical", "Spacing/sp-md"), B("screen", "gap", "Spacing/sp-sm"),
        {"command": "shape.rectangle", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 328, "height": 300, "name": "Galeria",
            "props": {"fill": V("Colors/Surface/Container")}}},
    ]
    if variant == "D":
        c += text("screen", "Looks com esta peça", "Font/Size/fs-xxs", "Colors/Content/Pure")
        r, rc = row("screen", "Looks")
        c += rc
        for i in range(3):
            c += [{"command": "shape.rectangle", "params": {"parent": {"$ref": r}, "x": 0, "y": 0, "width": 96, "height": 120, "name": f"Look {i + 1}",
                    "props": {"fill": V("Colors/Surface/Container")}}}]
        c += text("screen", "Comprar o look completo  →", "Font/Size/fs-xxxs", "Colors/Content/Brand/Highlight")
    c += text("screen", "Vestido midi de alça", "Font/Size/fs-xs", "Colors/Content/01")
    c += text("screen", "R$ 1.290,00", "Font/Size/fs-xs", "Colors/Content/Pure")
    if variant != "E":
        c += sizes("screen")
    if variant == "A":
        c += text("screen", "A modelo veste P · 1,75 m · busto 84 cm, cintura 66 cm", "Font/Size/fs-xxxs", "Colors/Content/01")
    if variant == "B":
        c += text("screen", "Como veste", "Font/Size/fs-xxs", "Colors/Content/Pure")
        r, rc = row("screen", "Escala de caimento")
        c += rc + chip(r, "Justo") + chip(r, "Fiel", True) + chip(r, "Folgado")
        c += text("screen", "82% das clientes acharam fiel ao tamanho · ver avaliações", "Font/Size/fs-xxxs", "Colors/Content/01")
    if variant == "C":
        r, rc = row("screen", "Concierge NV", fill="Colors/Feedback/Success/Surface")
        c += rc + [B(r, "paddingHorizontal", "Spacing/sp-sm"), B(r, "paddingVertical", "Spacing/sp-xs")]
        c += text(r, "Em dúvida no tamanho ou no look? Fale com o Concierge NV", "Font/Size/fs-xxxs", "Colors/Feedback/Success/Content")
    c += button("screen", "Selecione o tamanho" if variant == "E" else "Adicionar à sacola")
    if variant == "E":
        r = ref("sheet")
        c += [{"command": "frame.create", "as": r, "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 328, "height": 120, "name": "Bottom Sheet · tamanhos",
                "props": {"fill": V("Colors/Surface/Background"), "stroke": V("Colors/Surface/Brand/Stroke"), "strokeWeight": 1, "layout": "vertical",
                          "padding": 0, "gap": 0, "sizingH": "fill", "sizingV": "hug"}}},
              B(r, "paddingHorizontal", "Spacing/sp-md"), B(r, "paddingVertical", "Spacing/sp-md"), B(r, "gap", "Spacing/sp-sm")]
        c += text(r, "Escolha o tamanho para adicionar à sacola", "Font/Size/fs-xxs", "Colors/Content/Pure")
        c += sizes(r, "M")
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

ex("file.new", {"name": "Eureka · rodada NV · PDP App"}, agent=None)
ex("origin.track", {"on": True}, agent=None)
ex("variable.import", {"variables": json.load(open(TOK))})
r = ex("explore.board", {"title": "NV App · PDP · Rodada 01 (13/11)", "brand": "NV", "platform": "app", "journey": "PDP",
                         "problem": f"Na PDP do app NV, baixa adição à sacola, dúvida de tamanho e pouca inspiração. Registro: {ROUND}",
                         "proposals": proposals})
ids = [p["id"] for p in r["proposals"]]
ex("variable.setMode", {"ids": ids, "collection": COL, "mode": MODE})
lint = ex("ds.lint", {"ids": ids})
call("engine.execute", {"command": "select.set", "params": {"ids": [r["section"]]}})
call("engine.execute", {"command": "view.zoomSelection", "params": {}})
call("engine.execute", {"command": "select.none", "params": {}})
import time  # noqa: E402

time.sleep(3)
call("ui.screenshot", {"path": f"{S}/N1-round.png"})
lines = [f"Rodada NV no quadro: {len(ids)} propostas — " + "; ".join(p["name"] for p in r["proposals"]),
         f"Lint: {len(lint['findings'])} achado(s)" + "".join(f"\n  · {x['message']}" for x in lint["findings"][:8]),
         "Nada gravado no Notion; o Figma da designer não foi tocado. Escolha: com a designer."]
open(f"{S}/N-round.txt", "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
