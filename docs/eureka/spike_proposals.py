"""Eureka Gate 0 spike: proposals over FigCraft's control channel.
Claude generates -> the designer edits -> Claude reads the delta -> Claude PROPOSES a refinement (dry run, diff,
preview) -> accept with a guard (one undo step) -> undo -> a stale proposal is refused.

Usage: python3 spike_proposals.py WORKDIR TOKENS_JSON [--pause] [--steps import,generate,...]
  WORKDIR       holds a file `port` with the app's control port; screenshots and JSON are written there.
  TOKENS_JSON   a Figma local-variables export `variable.import` reads (`meta.variableCollections` + `meta.variables`,
                values by mode); collection "Tokens WEB", modes Animale (default) and Farm.
  --pause       wait for Enter after the `manual` step so a person can also edit with the mouse.
Start the app first: target/debug/figcraft --control <port>
Steps (in order): import generate manual delta propose accept brand undo stale
Spacing/* variables are scoped GAP only in the DS: the script binds them to the gap and never changes a scope."""
import json
import socket
import sys

args = [a for a in sys.argv[1:] if not a.startswith("--")]
S, TOK = args[0], args[1]
PAUSE = "--pause" in sys.argv
STEPS = "import,generate,manual,delta,propose,accept,brand,undo,stale".split(",")
for a in sys.argv:
    if a.startswith("--steps="):
        STEPS = a.split("=", 1)[1].split(",")

PORT = int(open(f"{S}/port").read())
sock = socket.create_connection(("127.0.0.1", PORT), timeout=120)
f = sock.makefile("rw")
n = [0]
T = "Tokens WEB/"


def call(method, params=None, check=True):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if check and not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r)[:900]}")
    return r if not check else r.get("result")


def ex(cmd, params, agent="Claude"):
    return call("engine.execute", {"command": cmd, "params": params, "agent": agent} if agent else {"command": cmd, "params": params})


def try_ex(cmd, params, agent="Claude"):
    return call("engine.execute", {"command": cmd, "params": params, "agent": agent}, check=False)


def shot(name):
    call("ui.screenshot", {"path": f"{S}/{name}.png"})


def save(name, obj):
    with open(f"{S}/{name}", "w") as fh:
        json.dump(obj, fh, indent=1, ensure_ascii=False)


def V(name):
    return {"$var": T + name}


def bind(ids, field, name):
    return {"command": "variable.bind", "params": {"ids": ids, "field": field, "variable": name}}


def ids_by_name():
    out = {}

    def walk(o):
        if isinstance(o, dict):
            if "name" in o and "id" in o:
                out.setdefault(o["name"], o["id"])
            for v in o.values():
                walk(v)
        elif isinstance(o, list):
            for v in o:
                walk(v)

    walk(call("document.inspect", {"outline": True}))
    return out


def readable(diff):
    """The dry-run diff as text a designer reads: the `summary` lines, layout values apart, DS warnings."""
    lines = []
    for c in diff.get("created", []):
        lines.append(f"+ criado   {c['name']} ({c['type']})  [id {c['id']}]")
        lines += [f"      {s}" for s in c.get("summary", [])]
    for c in diff.get("removed", []):
        lines.append(f"- removido {c['name']}  [id {c['id']}]")
    for c in diff.get("changed", []):
        lines.append(f"~ alterado {c['name']}  [id {c['id']}]")
        lines += [f"      {s}" for s in c.get("summary", [])]
        if c.get("layout"):
            lines.append(f"      (layout, secundário: {', '.join(c['layout'])})")
    v = diff.get("variables")
    if v:
        lines.append(f"variables: {json.dumps(v, ensure_ascii=False)[:300]}")
    mc = diff.get("manualConflicts")
    if mc:
        lines.append("conflitos com edição manual (a sua edição vence, salvo escolha):")
        for c in mc:
            who = {"manual": "fica a sua", "proposal": "fica a do Claude", "needs-choice": "PRECISA DE ESCOLHA"}.get(c.get("kept"), c.get("kept"))
            lines.append(f"  ⚑ {c['name']} [id {c['id']}] — {', '.join(c.get('props', [])) or 'camada removida'} → {who}")
            lines += [f"      você:   {x}" for x in c.get("manual", [])]
            lines += [f"      Claude: {x}" for x in c.get("proposal", [])]
    if "dsWarnings" in diff:
        w = diff["dsWarnings"]
        lines.append("avisos de DS (valor solto, sem variable): " + ("nenhum" if not w else ""))
        for x in w:
            lines.append(f"  ! {x['name']} [id {x['id']}]: {x['prop']} = {json.dumps(x['value'], ensure_ascii=False)}")
    lines.append(f"total {diff.get('total')}  baseHash {diff.get('baseHash')}  dry-run {diff.get('elapsedMs', 0):.1f} ms")
    return "\n".join(lines)


state = {}
try:
    state = json.load(open(f"{S}/state.json"))
except OSError:
    pass


def refine_commands(ids, loose=False):
    """The proposal. `loose=True` is a careless draft (raw numbers) used to show the DS warnings."""
    selo_bind = [] if loose else [
        bind([{"$ref": "tag"}], "paddingHorizontal", "Spacing/sp-xs"),
        bind([{"$ref": "tag"}], "paddingVertical", "Spacing/sp-xxs"),
        bind([{"$ref": "tag"}], "cornerRadius", "Border/Radius/Components/br-chips"),
    ]
    txt_bind = [] if loose else [bind([{"$ref": "seloTexto"}], "fontSize", "Font/Size/fs-xxxs")]
    return [
        bind([ids["Button"]], "fill", "Colors/Surface/Brand/Background"),
        bind([ids["Label"]], "fill", "Colors/Content/Brand/On Brand"),
        bind([ids["Preço"]], "fontSize", "Font/Size/fs-xs"),  # the designer's loose 18 goes back to a token
        {"command": "frame.create", "as": "tag", "params": {"parent": ids["Product Card"], "index": 1, "x": 0, "y": 0, "width": 100, "height": 24,
            "name": "Selo Frete grátis", "props": {"fill": V("Colors/Feedback/Success/Surface"), "layout": "horizontal",
                "padding": [4, 8, 4, 8] if loose else 0, "cornerRadius": 4 if loose else 0, "sizingH": "hug", "sizingV": "hug"}}},
        *selo_bind,
        {"command": "text.create", "as": "seloTexto", "params": {"parent": {"$ref": "tag"}, "text": "FRETE GRÁTIS", "x": 0, "y": 0, "name": "Selo texto",
            "props": {"fontSize": 11 if loose else 12, "fill": V("Colors/Feedback/Success/Content")}}},
        *txt_bind,
    ]


for step in STEPS:
    print(f"== {step}")
    if step == "import":
        ex("file.new", {"name": "Eureka · propostas · Animale"}, agent=None)
        ex("origin.track", {"on": True}, agent=None)  # Eureka mode: record who changes what (G2-E)
        toks = json.load(open(TOK))
        r = ex("variable.import", {"variables": toks})
        save("1-import.json", r)
        print(json.dumps(r, ensure_ascii=False)[:400])
        shot("1-import")

    elif step == "generate":
        notes = []
        b = [
            {"command": "frame.create", "as": "screen", "params": {"x": 0, "y": 0, "width": 360, "height": 520, "name": "PDP Mobile",
                "props": {"fill": V("Colors/Surface/Background"), "layout": "vertical", "padding": 16, "gap": 16}}},
            {"command": "frame.create", "as": "card", "params": {"parent": {"$ref": "screen"}, "x": 0, "y": 0, "width": 328, "height": 100, "name": "Product Card",
                "props": {"fill": V("Colors/Surface/Brand/Component/Card"), "layout": "vertical", "padding": 12, "gap": 8, "sizingV": "hug"}}},
            {"command": "shape.rectangle", "as": "img", "params": {"parent": {"$ref": "card"}, "x": 0, "y": 0, "width": 304, "height": 240, "name": "Imagem",
                "props": {"fill": V("Colors/Surface/Brand/Terciary")}}},
            {"command": "text.create", "as": "nome", "params": {"parent": {"$ref": "card"}, "text": "Camisa de linho manga longa", "x": 0, "y": 0, "name": "Nome",
                "props": {"fontSize": 14, "fill": V("Colors/Content/01")}}},
            {"command": "text.create", "as": "preco", "params": {"parent": {"$ref": "card"}, "text": "R$ 389,90", "x": 0, "y": 0, "name": "Preço",
                "props": {"fontSize": 16, "fill": V("Colors/Content/Pure")}}},
            {"command": "frame.create", "as": "btn", "params": {"parent": {"$ref": "card"}, "x": 0, "y": 0, "width": 304, "height": 44, "name": "Button",
                "props": {"fill": V("Colors/Surface/Brand/Secondary"), "layout": "horizontal", "mainAlign": "center", "crossAlign": "center", "sizingH": "fill"}}},
            {"command": "text.create", "as": "label", "params": {"parent": {"$ref": "btn"}, "text": "Adicionar à sacola", "x": 0, "y": 0, "name": "Label",
                "props": {"fontSize": 13, "fill": V("Colors/Content/Brand/Highlight")}}},
        ]
        r = ex("command.batch", {"label": "Claude: gerar Product Card", "commands": b})
        ids = ids_by_name()
        # Extra bindings (gap, radius, font size) one by one: record what the engine refuses (e.g. by scope).
        extra = [
            ([ids["Product Card"]], "gap", "Spacing/Mobile/sp-productcard-gap"),
            ([ids["PDP Mobile"]], "gap", "Spacing/sp-md"),
            ([ids["Product Card"]], "paddingHorizontal", "Spacing/sp-sm"),
            ([ids["Product Card"]], "paddingVertical", "Spacing/sp-sm"),
            ([ids["PDP Mobile"]], "paddingHorizontal", "Spacing/sp-md"),
            ([ids["PDP Mobile"]], "paddingVertical", "Spacing/sp-md"),
            ([ids["Product Card"]], "cornerRadius", "Border/Radius/Components/br-card"),
            ([ids["Button"]], "cornerRadius", "Border/Radius/Components/br-button"),
            ([ids["Nome"]], "fontSize", "Font/Size/fs-xs"),
            ([ids["Preço"]], "fontSize", "Font/Size/fs-xs"),
            ([ids["Label"]], "fontSize", "Font/Size/fs-xxs"),
        ]
        for tids, field, name in extra:
            c = bind(tids, field, name)
            rr = try_ex(c["command"], c["params"])
            ok = rr.get("ok", False)
            notes.append({"field": field, "variable": name, "bound": ok, "error": None if ok else str(rr.get("error"))[:300]})
        save("2-generate-bindings.json", notes)
        state["ids"] = ids
        # The brand is chosen on the screen, never left to the collection's default mode (in the
        # DS file the default is Farm).
        ex("variable.setMode", {"ids": [ids["PDP Mobile"]], "collection": "Tokens WEB", "mode": "Animale"})
        state["cursor"] = ex("document.checkpoint", {})["cursor"]
        ex("proposal.turn", {})  # what the designer edits from here on is protected (ADR-009)
        ex("view.zoomFit", {})
        shot("2-generate")
        print(json.dumps(notes, ensure_ascii=False)[:900])

    elif step == "manual":
        ids = state["ids"]
        ex("node.set", {"ids": [ids["Preço"]], "props": {"characters": "R$ 349,90", "fontSize": 18}}, agent=None)
        ex("node.set", {"ids": [ids["Nome"]], "props": {"characters": "Camisa de linho off-white"}}, agent=None)
        ex("node.set", {"ids": [ids["Button"]], "props": {"fill": "#7A4B2A"}}, agent=None)
        ex("node.set", {"ids": [ids["Label"]], "props": {"fill": "#FFFFFF"}}, agent=None)
        shot("3-manual")
        if PAUSE:
            input("Edit in the app with the mouse if you like, then press Enter... ")
            shot("3-manual-mouse")

    elif step == "delta":
        r = ex("document.changesSince", {"cursor": state["cursor"], "advance": True})
        state["cursor"] = r["cursor"]
        save("4-delta.json", r)
        open(f"{S}/4-delta.txt", "w").write(readable(r) + "\n")
        print(readable(r))

    elif step == "propose":
        # A careless draft first: raw numbers on the new badge. Warnings only, nothing blocks.
        d = ex("command.batch", {"label": "Claude: rascunho com valores soltos", "dryRun": True, "commands": refine_commands(state["ids"], loose=True)})
        save("5a-draft-warnings.json", d["dsWarnings"])
        open(f"{S}/5a-draft-warnings.txt", "w").write(readable(d) + "\n")
        print("draft warnings:", json.dumps(d["dsWarnings"], ensure_ascii=False))
        cmds = refine_commands(state["ids"])
        # Default: every hand edit the proposal contradicts stays as the designer made it.
        c = ex("command.batch", {"label": "Claude: refinar (tokens + selo)", "dryRun": True, "commands": cmds})
        save("5b-conflicts.json", c["manualConflicts"])
        open(f"{S}/5b-conflicts.txt", "w").write(readable(c) + "\n")
        print(readable(c))
        # On the card the designer takes Claude's tokens for the two colours and keeps the price size.
        ids = state["ids"]
        state["override"] = [ids["Button"], ids["Label"]]
        r = ex("command.batch", {"label": "Claude: refinar (tokens + selo)", "dryRun": True, "override": state["override"], "commands": cmds})
        state["hash"] = r["baseHash"]
        save("5-propose-diff.json", r)
        open(f"{S}/5-propose-diff.txt", "w").write(readable(r) + "\n")
        print(readable(r))
        p = ex("proposal.preview", {"label": "Claude: refinar (tokens + selo)", "override": state["override"], "commands": cmds})
        ex("view.zoomFit", {})
        shot("5-propose-preview")
        ex("proposal.previewEnd", {})
        shot("5-propose-after-discard")  # same as step 3: the dry run left nothing behind

    elif step == "accept":
        cmds = refine_commands(state["ids"])
        r = ex("command.batch", {"label": "Claude: refinar (tokens + selo)", "ifUnchanged": state["hash"], "override": state["override"], "commands": cmds})
        save("6-accept.json", r)
        ids = ids_by_name()
        kept = {n: call("node.get", {"id": ids[n], "fields": ["name", "characters", "fontSize", "boundVariables"]}) for n in ("Nome", "Preço")}
        save("6-designer-text-kept.json", kept)
        ex("view.zoomFit", {})
        shot("6-accept")
        print(json.dumps(kept, ensure_ascii=False))

    elif step == "brand":
        ids = ids_by_name()
        r = ex("variable.setMode", {"ids": [ids["PDP Mobile"]], "collection": "Tokens WEB", "mode": "Farm"})
        shot("6b-brand-farm")
        ex("edit.undo", {}, agent=None)  # back to Animale, keeping the accepted refinement
        shot("6c-brand-back-animale")

    elif step == "undo":
        ex("edit.undo", {}, agent=None)
        ex("view.zoomFit", {})
        shot("7-undo")

    elif step == "stale":
        ids = ids_by_name()
        cmds = refine_commands(ids)
        r = ex("command.batch", {"label": "Claude: refinar (de novo)", "dryRun": True, "commands": cmds})
        ex("node.set", {"ids": [ids["Button"]], "props": {"fill": "#00AA55"}}, agent=None)  # the designer touches a layer the proposal touches
        rr = try_ex("command.batch", {"label": "Claude: refinar (de novo)", "ifUnchanged": r["baseHash"], "commands": cmds})
        save("8-stale.json", rr)
        shot("8-stale")
        print(json.dumps(rr, ensure_ascii=False)[:700])
        assert not rr.get("ok", True) and "stale" in json.dumps(rr), "expected a stale refusal"

    json.dump(state, open(f"{S}/state.json", "w"))
print("done")
