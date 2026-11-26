"""Eureka Frente R: the DS lint's rule 6 (`override-token`) on a drawn round (WORKDIR/port).

Simulates what the rule is for: on proposal G's screen, the primary button's background inside the
Action Button is bound to another token (as if someone swapped it by hand), then `ds.lint` runs.
Only the local document changes; nothing goes to Figma or the DS.

Usage: python3 regra6_demo.py WORKDIR SCREEN_ID
"""
import json
import socket
import sys

S, SID = sys.argv[1], int(sys.argv[2])
f = socket.create_connection(("127.0.0.1", int(open(f"{S}/port").read())), timeout=120).makefile("rw")
__import__("eureka_control").auth(f, int(open(f"{S}/port").read()))


def ex(cmd, params):
    f.write(json.dumps({"id": 1, "method": "engine.execute", "params": {"command": cmd, "params": params}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok"):
        raise SystemExit(f"{cmd}: {r}")
    return r["result"]


def walk(i):
    n = ex("node.get", {"id": i, "fields": ["name", "type", "children", "boundVariables"]})
    yield n
    for c in n.get("children") or []:
        yield from walk(c["id"] if isinstance(c, dict) else c)


def rule6():
    return [x for x in ex("ds.lint", {"ids": [SID]})["findings"] if x["rule"] == "override-token"]


before = rule6()
# The first layer bound to a colour token inside the "Botão Primário" of the screen's Action Button.
inside, target = False, None
for n in walk(SID):
    if "Botão Primário" in n["name"]:
        inside = True
    if inside and "fill" in (n.get("boundVariables") or {}):
        target = n
        break
if target is None:
    raise SystemExit("no bound layer found inside Botão Primário")
current = target["boundVariables"]["fill"]
other = next(v for v in ex("variable.list", {})["variables"] if v["name"].startswith("Colors/Feedback") and v["id"] != current)
ex("variable.bind", {"ids": [target["id"]], "field": "fill", "variable": other["id"]})
after = rule6()
print(f"Regra 6 na tela {SID}: {len(before)} achado(s) antes; {len(after)} depois de trocar o token de \"{target['name']}\".")
for x in after:
    print(f"  · {x['message']}")
