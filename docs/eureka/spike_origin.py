"""Eureka G2, Frente E demo: who changed what, per layer and per aspect.

Run after `spike_proposals.py --steps=import,generate` and `spike_loop.py` on the same WORKDIR:
Claude generated the card, the designer edited it, Claude's proposal was applied by a click.
Then one undo shows the stamps going back with the edit.

Usage: python3 spike_origin.py WORKDIR
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
LOG = []
LABEL = {"fills": "cor", "characters": "texto", "style": "estilo do texto", "layout": "auto layout", "corners": "cantos",
         "name": "nome", "parent": "lugar na árvore", "strokes": "contorno", "visible": "visível", "clip": "recorte"}


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r)[:600]}")
    return r.get("result")


def ex(cmd, params):
    return call("engine.execute", {"command": cmd, "params": params})


def say(line=""):
    print(line)
    LOG.append(line)


ids = json.load(open(f"{S}/state.json"))["ids"]


def table(title):
    say(title)
    for name in ("Product Card", "Nome", "Preço", "Button", "Label"):
        o = ex("origin.get", {"id": ids[name]})["aspects"]
        parts = []
        for k, v in sorted(o.items()):
            if k in ("parent", "clip", "child_layout", "blend"):
                continue
            who = (v.get("agent") or "IA") if v["by"] == "ai" else "designer"
            parts.append(f"{LABEL.get(k, k)}: {who}")
        say(f"  {name:13} {' · '.join(parts)}")


table("Quem mudou o quê (último a mexer em cada aspecto):")
ai = ex("origin.list", {"by": "ai"})
say(f"Camadas com algo feito pela IA: {', '.join(e['name'] for e in ai)}")
say()
ex("edit.undo", {})
table("Depois de 1 desfazer (a proposta aplicada volta, e a marca volta junto):")
ex("edit.redo", {})
open(f"{S}/O-origin.txt", "w").write("\n".join(LOG) + "\n")
