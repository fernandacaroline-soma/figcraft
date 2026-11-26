"""Eureka G3, Frente V: text contrast of each screen of a drawn round in every brand of the brand
collection (`explore.measure` with `allBrands`), next to the screen's own brand.

Usage: python3 contraste_marcas.py WORKDIR SCREEN_ID...
Read-only: it only measures.
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


def contrast(sid, every):
    return [x for x in ex("explore.measure", {"id": sid, "allBrands": every})["findings"] if "contraste" in x["scenario"]]


for sid in ids:
    name = ex("node.get", {"id": sid, "fields": ["name"]})["name"]
    own, every = contrast(sid, False), contrast(sid, True)
    print(f"{name}: {len(own)} texto(s) abaixo de AA na marca da tela; {len(every)} em alguma marca")
    for x in every:
        print(f"  [{x['severity']}] {x['scenario']}")
