"""Eureka G3, Frente P: what `explore.measure` finds on each screen of a drawn round (WORKDIR/port).

Usage: python3 medicao_p.py WORKDIR SCREEN_ID... > report.txt
Read-only: it only measures; nothing is kept on the proposals.
"""
import json
import socket
import sys

S, ids = sys.argv[1], [int(x) for x in sys.argv[2:]]
f = socket.create_connection(("127.0.0.1", int(open(f"{S}/port").read())), timeout=120).makefile("rw")


def ex(cmd, params):
    f.write(json.dumps({"id": 1, "method": "engine.execute", "params": {"command": cmd, "params": params}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok"):
        raise SystemExit(f"{cmd}: {r}")
    return r["result"]


total = {}
for sid in ids:
    name = ex("node.get", {"id": sid, "fields": ["name"]})["name"]
    found = ex("explore.measure", {"id": sid})["findings"]
    print(f"{name}: {len(found)} achado(s) medido(s)")
    for x in found:
        total[x["area"]] = total.get(x["area"], 0) + 1
        print(f"  [{x['severity']}] {x['area']}: {x['scenario']}")
print("Total por área: " + ", ".join(f"{k} {v}" for k, v in sorted(total.items())))
