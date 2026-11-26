"""Eureka G1, Frente B demo: the app tells Claude when the document changes.

Claude makes a card (one batch), the designer edits two layers by hand, a dry run and a preview
change nothing; then `events.poll` with topic `document` lists exactly what happened, who did it,
and which layers changed.

Usage: python3 spike_events.py WORKDIR   (WORKDIR holds `port`; start the app with --control <port>)
"""
import json
import socket
import sys

S = sys.argv[1]
PORT = int(open(f"{S}/port").read())
sock = socket.create_connection(("127.0.0.1", PORT), timeout=60)
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


ex("file.new", {"name": "Eureka · eventos"})
cursor = call("events.poll", {"after": 0, "topics": ["document"], "timeoutMs": 0})["cursor"]

r = ex("command.batch", {"label": "Claude: card", "commands": [
    {"command": "frame.create", "as": "card", "params": {"x": 0, "y": 0, "width": 320, "height": 200, "name": "Card", "props": {"fill": "#F4F1EC", "layout": "vertical", "padding": 16, "gap": 8}}},
    {"command": "text.create", "as": "t", "params": {"parent": {"$ref": "card"}, "text": "Camisa de linho", "x": 0, "y": 0, "name": "Nome"}},
    {"command": "text.create", "as": "p", "params": {"parent": {"$ref": "card"}, "text": "R$ 389,90", "x": 0, "y": 0, "name": "Preço"}},
]}, agent="Claude")
ids = {}
for step in r:
    if isinstance(step, dict) and "id" in step:
        ids[len(ids)] = step["id"]
card, nome, preco = ids.get(0), ids.get(1), ids.get(2)

ex("node.set", {"ids": [preco], "props": {"characters": "R$ 349,90", "fontSize": 18}})
ex("node.set", {"ids": [nome], "props": {"characters": "Camisa de linho off-white"}})
# Neither a dry run nor a preview is an edit.
ex("command.batch", {"dryRun": True, "commands": [{"command": "node.set", "params": {"id": card, "props": {"name": "X"}}}]}, agent="Claude")
ex("proposal.preview", {"commands": [{"command": "node.set", "params": {"id": card, "props": {"name": "X"}}}]}, agent="Claude")
ex("proposal.previewEnd", {})

out = call("events.poll", {"after": cursor, "topics": ["document"], "timeoutMs": 0})
names = {card: "Card", nome: "Nome", preco: "Preço"}
lines = []
for e in out["events"]:
    d = e["data"]
    who = d.get("agent") or "designer"
    changed = ", ".join(names.get(i, f"#{i}") for i in d.get("changed", []))
    lines.append(f"#{e['seq']} {e['kind']}  por {who} ({d['source']})  comando {d['command']}  camadas: {changed}")
lines.append(f"{len(out['events'])} eventos (esperado 3: o card do Claude e as 2 edições à mão; dry-run e prévia não contam)")
text = "\n".join(lines)
open(f"{S}/events.txt", "w").write(text + "\n")
json.dump(out, open(f"{S}/events.json", "w"), indent=1, ensure_ascii=False)
print(text)
assert len(out["events"]) == 3, "expected exactly 3 document events"
