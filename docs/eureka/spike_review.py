"""Eureka G2, Frente G demo: the proposal card with the DS lint and a reviewer's verdict.

Run after `spike_proposals.py WORKDIR TOKENS --app --steps=import,generate,manual`. Claude previews a
refinement with one careless raw value (the badge's padding and radius), the card lists the DS
lint of what the proposal makes, and a reviewer attaches a verdict with `proposal.review`.

The verdict here is written by this script from the lint findings, standing in for the DS
Gatekeeper skill: wiring the real skill (and whether it records each round in Notion) is the
next step and needs the owner's decision.

Usage: python3 spike_review.py WORKDIR
"""
import json
import socket
import sys

S = sys.argv[1]
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=60).makefile("rw")
n = [0]


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r)[:600]}")
    return r.get("result")


def ex(cmd, params, agent="Claude"):
    p = {"command": cmd, "params": params}
    if agent:
        p["agent"] = agent
    return call("engine.execute", p)


ids = json.load(open(f"{S}/state.json"))["ids"]
V = lambda name: {"$var": "Tokens APP/" + name}  # noqa: E731
cmds = [
    {"command": "variable.bind", "params": {"ids": [ids["Button"]], "field": "fill", "variable": "Colors/Surface/Brand/Background"}},
    {"command": "frame.create", "as": "tag", "params": {"parent": ids["Product Card"], "index": 1, "x": 0, "y": 0, "width": 100, "height": 24,
        "name": "Selo Frete grátis", "props": {"fill": V("Colors/Feedback/Success/Surface"), "layout": "horizontal", "padding": [4, 8, 4, 8],
            "cornerRadius": 4, "sizingH": "hug", "sizingV": "hug"}}},
    {"command": "text.create", "as": "t", "params": {"parent": {"$ref": "tag"}, "text": "FRETE GRÁTIS", "x": 0, "y": 0, "name": "Selo texto",
        "props": {"fontSize": 12, "fill": V("Colors/Feedback/Success/Content")}}},
    {"command": "variable.bind", "params": {"ids": [{"$ref": "t"}], "field": "fontSize", "variable": "Font/Size/fs-xxxs"}},
]
p = ex("proposal.preview", {"label": "Claude: botão no token + selo", "override": [ids["Button"]], "commands": cmds})
lint = p.get("lint", [])
lines = [f"Lint da proposta: {len(lint)} achado(s)"] + [f"  · {x['message']}  [{x['rule']}]" for x in lint]
verdict = "com-ressalvas" if lint else "aprovado"
items = [{"message": x["message"], "source": x["source"]} for x in lint[:5]]
r = ex("proposal.review", {"verdict": verdict, "reviewer": "DS Gatekeeper (demo)",
                           "summary": "O botão volta ao token da marca. O selo novo tem padding e raio fixos onde o DS Apps tem tokens.", "items": items})
lines.append(f"Parecer: {r['review']['verdict']} por {r['review']['reviewer']} (só orienta)")
call("ui.screenshot", {"path": f"{S}/G1-card.png"})
open(f"{S}/G-review.txt", "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
