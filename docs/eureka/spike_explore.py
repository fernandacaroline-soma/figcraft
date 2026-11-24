"""Eureka G3, Frente I demo: an exploration board on the DS Apps tokens.

Problem (synthetic, for the demo): on the Animale app PDP, customers do not notice free shipping.
Three proposals in the generator's shape (gerador-propostas: incremental and disruptive, distinct in
mechanism), drawn side by side with Tokens APP only, then the designer picks one.

The proposals here are written by this script to show the board; in the real flow the generator
skill writes them (with evidence, benchmark, guardrails) and the Eureka draws them.

Usage: python3 spike_explore.py WORKDIR TOKENS_APP_JSON   (app running with --control; WORKDIR/port)
"""
import json
import socket
import sys

S, TOK = sys.argv[1], sys.argv[2]
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=120).makefile("rw")
n = [0]
COL = "Tokens APP"
MODE = "Animale (CERTO)"


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


def text(ref, parent, words, size, color):
    return [
        {"command": "text.create", "as": ref, "params": {"parent": {"$ref": parent}, "text": words, "x": 0, "y": 0, "name": words[:24],
            "props": {"fontSize": 12, "fill": V(color)}}},
        B(ref, "fontSize", size),
    ]


def badge(ref, parent, words):
    return [
        {"command": "frame.create", "as": ref, "params": {"parent": {"$ref": parent}, "x": 0, "y": 0, "width": 80, "height": 20, "name": "Selo",
            "props": {"fill": V("Colors/Feedback/Success/Surface"), "layout": "horizontal", "padding": 0, "sizingH": "hug", "sizingV": "hug"}}},
        B(ref, "paddingHorizontal", "Spacing/sp-xs"), B(ref, "paddingVertical", "Spacing/sp-xxs"), B(ref, "cornerRadius", "Border/Radius/Components/br-chips"),
        *text(ref + "T", ref, words, "Font/Size/fs-xxxs", "Colors/Feedback/Success/Content"),
    ]


def screen(variant):
    """One PDP screen; `variant` decides where free shipping shows."""
    c = [
        {"command": "frame.create", "as": "screen", "params": {"x": 0, "y": 0, "width": 360, "height": 640, "name": "PDP",
            "props": {"fill": V("Colors/Surface/Background"), "layout": "vertical", "padding": 0, "gap": 0}}},
        B("screen", "paddingHorizontal", "Spacing/sp-md"), B("screen", "paddingVertical", "Spacing/sp-md"), B("screen", "gap", "Spacing/sp-md"),
        {"command": "shape.rectangle", "as": "img", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 328, "height": 360, "name": "Imagem",
            "props": {"fill": V("Colors/Surface/Container")}}},
        {"command": "frame.create", "as": "info", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 328, "height": 60, "name": "Informações",
            "props": {"fills": [], "layout": "vertical", "gap": 0, "sizingV": "hug", "sizingH": "fill"}}},
        B("info", "gap", "Spacing/sp-xs"),
        *text("nome", "info", "Camisa de linho manga longa", "Font/Size/fs-xs", "Colors/Content/01"),
        {"command": "frame.create", "as": "linha", "params": {"parent": {"$ref": "info"}, "x": 0, "y": 0, "width": 328, "height": 24, "name": "Preço",
            "props": {"fills": [], "layout": "horizontal", "gap": 0, "crossAlign": "center", "sizingV": "hug", "sizingH": "hug"}}},
        B("linha", "gap", "Spacing/sp-xs"),
        *text("preco", "linha", "R$ 389,90", "Font/Size/fs-xs", "Colors/Content/Pure"),
    ]
    if variant == "preco":
        c += badge("selo", "linha", "FRETE GRÁTIS")
    if variant == "faixa":
        c += [
            {"command": "frame.create", "as": "faixa", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 328, "height": 32, "name": "Faixa de frete",
                "props": {"fill": V("Colors/Feedback/Success/Surface"), "layout": "horizontal", "padding": 0, "sizingH": "fill", "sizingV": "hug", "mainAlign": "center"}}},
            B("faixa", "paddingVertical", "Spacing/sp-xs"), B("faixa", "cornerRadius", "Border/Radius/Components/br-chips"),
            *text("faixaT", "faixa", "Frete grátis para todo o Brasil nesta peça", "Font/Size/fs-xxxs", "Colors/Feedback/Success/Content"),
        ]
    label = "Adicionar à sacola · frete grátis" if variant == "botao" else "Adicionar à sacola"
    c += [
        {"command": "frame.create", "as": "btn", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 328, "height": 48, "name": "Button",
            "props": {"fill": V("Colors/Surface/Brand/Background"), "layout": "horizontal", "mainAlign": "center", "crossAlign": "center", "sizingH": "fill"}}},
        B("btn", "cornerRadius", "Border/Radius/Components/br-button"),
        *text("btnT", "btn", label, "Font/Size/fs-xxs", "Colors/Content/Brand/On Brand"),
    ]
    return c


proposals = [
    {"name": "Selo junto ao preço", "type": "incremental", "mechanism": "sinalização no ponto de decisão de preço",
     "summary": "Um selo de frete grátis ao lado do preço, onde a cliente já olha para decidir.", "confidence": "baixa", "commands": screen("preco")},
    {"name": "Faixa acima da sacola", "type": "incremental", "mechanism": "reforço no momento da ação",
     "summary": "Uma faixa de frete logo acima do botão, lida na hora de adicionar à sacola.", "confidence": "baixa", "commands": screen("faixa")},
    {"name": "O botão conta o frete", "type": "disruptiva", "mechanism": "o benefício vira parte da ação principal",
     "summary": "O próprio botão anuncia o frete grátis: sem selo nem faixa, o benefício está no clique.", "confidence": "baixa", "commands": screen("botao")},
]

ex("file.new", {"name": "Eureka · exploração · Animale App"}, agent=None)
ex("origin.track", {"on": True}, agent=None)
ex("variable.import", {"variables": json.load(open(TOK))})
r = ex("explore.board", {"title": "PDP Animale App · frete grátis", "brand": "Animale", "platform": "app", "journey": "PDP",
                         "problem": "Na PDP do app, a cliente não percebe que a peça tem frete grátis.", "proposals": proposals})
ids = [p["id"] for p in r["proposals"]]
ex("variable.setMode", {"ids": ids, "collection": COL, "mode": MODE})  # the brand is chosen on each screen, never left to the default
ex("view.zoomFit", {})
call("ui.screenshot", {"path": f"{S}/E1-board.png"})
lint = ex("ds.lint", {"ids": ids})
c = ex("explore.choose", {"id": ids[1]}, agent=None)  # the designer picks B
call("ui.screenshot", {"path": f"{S}/E2-chosen.png"})
lines = [f"Quadro: {len(ids)} propostas — " + "; ".join(p["name"] for p in r["proposals"]),
         f"Lint do quadro: {len(lint['findings'])} achado(s)" + "".join(f"\n  · {x['message']}" for x in lint["findings"][:6]),
         f"Escolhida: {c['chosen']['name']} (as outras ficam no quadro como registro)"]
open(f"{S}/E-explore.txt", "w").write("\n".join(lines) + "\n")
json.dump({"board": r, "chosen": c}, open(f"{S}/E-explore.json", "w"), indent=1, ensure_ascii=False)
print("\n".join(lines))
