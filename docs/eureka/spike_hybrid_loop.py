"""Eureka Sprint 0 spike: hybrid loop over FigCraft's control channel.
Claude generates with DS tokens -> designer edits -> Claude reads state -> Claude refines.

Usage: python3 spike_hybrid_loop.py WORKDIR STEP TOKENS_JSON
  WORKDIR holds a file `port` with the app's control port; screenshots are written there.
  STEP is generate | manual | read | refine (run in that order).
  TOKENS_JSON is the DS token export (e.g. project/tokens.json of the "DS Animale Web" artifact).
Start the app first: cargo run -p figcraft -- --control <port>"""
import json, socket, sys

S = sys.argv[1]
PORT = int(open(f"{S}/port").read())
TOK = sys.argv[3]
sock = socket.create_connection(("127.0.0.1", PORT), timeout=120)
__import__("eureka_control").auth_socket(sock, PORT)
f = sock.makefile("rw")
n = [0]


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r)[:600]}")
    return r.get("result")


def ex(cmd, params, agent=None):
    p = {"command": cmd, "params": params}
    if agent:
        p["agent"] = agent
    return call("engine.execute", p)


def shot(name):
    call("ui.screenshot", {"path": f"{S}/{name}.png"})


def px(v):
    return float(str(v).replace("px", "") or 0)


step = sys.argv[2]
toks = json.load(open(TOK))

if step == "generate":
    ex("file.new", {"name": "Eureka spike · PDP Animale"})
    # 1. Tokens: read from the DS source file, no manual copy. One collection, one brand mode.
    cmds = [{"command": "variable.createCollection", "params": {"name": "Tokens WEB", "modes": ["Animale"]}}]
    for t in toks["color"]["tokens"]:
        cmds.append({"command": "variable.create", "params": {"collection": "Tokens WEB", "name": t["name"], "type": "color", "value": t["value"]["animale"]}})
    for t in toks["spacing"]["tokens"]:
        cmds.append({"command": "variable.create", "params": {"collection": "Tokens WEB", "name": t["name"], "type": "number", "value": px(t["value"])}})
    ex("command.batch", {"label": "Importar tokens DS N Marcas (Animale)", "commands": cmds}, agent="Claude")
    # 2. Claude generates the card, bound to tokens, as one undo step.
    V = lambda name: {"$var": name}
    b = [
        {"command": "frame.create", "as": "screen", "params": {"x": 0, "y": 0, "width": 360, "height": 520, "name": "PDP · Mobile",
            "props": {"fill": V("colors-surface-background"), "layout": "vertical", "padding": 16, "gap": 16}}},
        {"command": "frame.create", "as": "card", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 328, "height": 100, "name": "Product Card",
            "props": {"fill": V("colors-surface-brand-component-card"), "layout": "vertical", "padding": 12, "gap": 8, "sizingV": "hug"}}},
        {"command": "shape.rectangle", "as": "img", "params": {"parent": {"$ref": "card"}, "x": 0, "y": 0, "width": 304, "height": 300, "name": "Imagem",
            "props": {"fill": V("colors-surface-brand-terciary")}}},
        {"command": "text.create", "as": "nome", "params": {"parent": {"$ref": "card"}, "text": "Camisa de linho manga longa", "x": 0, "y": 0, "name": "Nome",
            "props": {"fontSize": 14, "fill": V("colors-content-01")}}},
        {"command": "text.create", "as": "preco", "params": {"parent": {"$ref": "card"}, "text": "R$ 389,90", "x": 0, "y": 0, "name": "Preço",
            "props": {"fontSize": 16, "fill": V("colors-content-pure")}}},
        {"command": "frame.create", "as": "btn", "params": {"parent": {"$ref": "card"}, "x": 0, "y": 0, "width": 304, "height": 44, "name": "Button",
            "props": {"fill": V("colors-surface-brand-secondary"), "stroke": V("colors-surface-brand-background"), "strokeWeight": 1,
                      "layout": "horizontal", "mainAlign": "center", "crossAlign": "center", "sizingH": "fill"}}},
        {"command": "text.create", "params": {"parent": {"$ref": "btn"}, "text": "Adicionar à sacola", "x": 0, "y": 0, "name": "Label",
            "props": {"fontSize": 13, "fill": V("colors-content-brand-highlight")}}},
    ]
    r = ex("command.batch", {"label": "Claude: gerar Product Card", "commands": b}, agent="Claude")
    ex("view.zoomFit", {})
    shot("spike-1-generated")
    print(json.dumps(r)[:400])

elif step == "manual":
    # The designer's own edits (what the Design panel would send): new price, a hand-picked colour.
    doc = call("document.inspect", {"outline": True})
    ids = {}
    def walk(o):
        if isinstance(o, dict):
            if "name" in o and "id" in o:
                ids.setdefault(o["name"], o["id"])
            for v in o.values():
                walk(v)
        elif isinstance(o, list):
            for v in o:
                walk(v)
    walk(doc)
    ex("node.set", {"ids": [ids["Preço"]], "props": {"characters": "R$ 349,90", "fontSize": 18}})
    ex("node.set", {"ids": [ids["Nome"]], "props": {"characters": "Camisa de linho off-white"}})
    ex("node.set", {"ids": [ids["Button"]], "props": {"fill": "#7A4B2A"}})
    ex("node.set", {"ids": [ids["Label"]], "props": {"fill": "#FFFFFF"}})
    shot("spike-2-manual")
    json.dump(ids, open(f"{S}/ids.json", "w"))
    print("manual edits done", ids)

elif step == "read":
    ids = json.load(open(f"{S}/ids.json"))
    out = {}
    for name in ("Nome", "Preço", "Button", "Label"):
        out[name] = call("node.get", {"id": ids[name], "fields": ["name", "characters", "fills", "boundVariables", "fontSize"]})
    json.dump(out, open(f"{S}/spike-3-state.json", "w"), indent=1, ensure_ascii=False)
    print(json.dumps(out, ensure_ascii=False)[:2500])

elif step == "refine":
    ids = json.load(open(f"{S}/ids.json"))
    V = lambda name: {"$var": name}
    # Claude keeps the designer's text edits and flags/fixes only the off-token colours, plus adds the requested tag.
    b = [
        {"command": "node.set", "params": {"ids": [ids["Button"]], "props": {"fill": V("colors-surface-brand-background")}}},
        {"command": "node.set", "params": {"ids": [ids["Label"]], "props": {"fill": V("colors-content-brand-on-brand")}}},
        {"command": "frame.create", "as": "tag", "params": {"parent": ids["Product Card"], "index": 1, "x": 0, "y": 0, "width": 100, "height": 24, "name": "Tag · Frete grátis",
            "props": {"fill": V("colors-feedback-success-surface"), "layout": "horizontal", "padding": [4, 8, 4, 8], "sizingH": "hug", "sizingV": "hug"}}},
        {"command": "text.create", "params": {"parent": {"$ref": "tag"}, "text": "FRETE GRÁTIS", "x": 0, "y": 0, "name": "Label",
            "props": {"fontSize": 11, "fill": V("colors-feedback-success-content")}}},
    ]
    ex("command.batch", {"label": "Claude: refinar (tokens + selo)", "commands": b}, agent="Claude")
    shot("spike-4-refined")
    print("refined")
