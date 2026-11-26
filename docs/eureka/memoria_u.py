"""Eureka G3, Frente U: the visual-memory entry of an approved layout, on a round drawn by `rodada_n.py`.

After the critique (`rodada_o.py`) and the Gatekeeper's verdict (`gatekeeper_bridge.py post-choice`)
on SCREEN_ID: a hand edit stands for the designer's refinement (an edit without an agent is hers in
the recorded origin), the layout is approved (`explore.approve`, as a TEST: the real approval is
the designer's) and `explore.memory` drafts the entry. Nothing is written to Notion: writing it is
the generator's, after the designer's explicit ok.

Usage: python3 memoria_u.py WORKDIR SCREEN_ID DATE
"""
import json
import socket
import sys

S, SID, DATE = sys.argv[1], int(sys.argv[2]), sys.argv[3]
f = socket.create_connection(("127.0.0.1", int(open(f"{S}/port").read())), timeout=120).makefile("rw")


def ex(cmd, params, agent=None):
    p = {"command": cmd, "params": params}
    if agent:
        p["agent"] = agent
    f.write(json.dumps({"id": 1, "method": "engine.execute", "params": p}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok"):
        raise SystemExit(f"{cmd}: {r}")
    return r["result"]


# The designer's refinement (a hand edit, no agent): the success message of the proposal shortened.
def walk(i):
    n = ex("node.get", {"id": i, "fields": ["name", "type", "children"]})
    yield n
    for c in n.get("children") or []:
        yield from walk(c["id"] if isinstance(c, dict) else c)


snack = next((n for n in walk(SID) if n["name"].startswith("Snackbar") and n["type"] == "INSTANCE"), None)
if snack:
    props = ex("node.get", {"id": snack["id"], "fields": ["componentProperties"]})["componentProperties"]
    key = next(k for k in props if k.split("#")[0].lstrip("✍️ ").strip().lower() == "texto")
    ex("node.set", {"id": snack["id"], "props": {"componentProperties": {key: "Na sacola · M"}}})
ex("explore.approve", {"id": SID, "by": "designer da NV (teste)", "date": DATE})
m = ex("explore.memory", {"id": SID})
open(f"{S}/U-memoria.md", "w").write(m["entry"])
print(m["entry"])
