"""Eureka G3, Frente O: the design critique of the chosen proposal, on a round drawn by `rodada_n.py`.

Runs on the same app (WORKDIR/port), after `rodada_n.py` drew the board:
  1. `explore.choose` the screen given (a TEST pick: the real choice is the designer's);
  2. `explore.measure`: what the Eureka measures on the screen (states, contrast, touch, lint);
  3. `explore.critique` with the reviewer's findings (CRITIQUE_JSON, from the generator's critique
     script), the measured ones added by the engine;
  4. screenshots of the card with the critique (O1); then, as a test of the card only, one status
     and "I reviewed the critique" (O2), after which the Gatekeeper may be asked.

Read-only towards Figma and Notion: nothing is written back, and no DS component, token or rule changes.

Usage: python3 rodada_o.py WORKDIR SCREEN_ID CRITIQUE_JSON
"""
import json
import socket
import sys
import time

S, SID, CRIT = sys.argv[1], int(sys.argv[2]), sys.argv[3]
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=180).makefile("rw")
n = [0]


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r, ensure_ascii=False)[:1500]}")
    return r.get("result")


def ex(cmd, params, agent="Claude"):
    p = {"command": cmd, "params": params}
    if agent:
        p["agent"] = agent
    return call("engine.execute", p)


ex("explore.choose", {"id": SID}, agent=None)
measured = ex("explore.measure", {"id": SID}, agent=None)["findings"]
open(f"{S}/O0-medido.json", "w").write(json.dumps(measured, ensure_ascii=False, indent=1))
crit = json.load(open(CRIT))
crit["id"] = SID
c = ex("explore.critique", crit)["critique"]
# The Gatekeeper is refused until the designer reviews the critique.
n[0] += 1
f.write(json.dumps({"id": n[0], "method": "engine.execute", "params": {"command": "explore.review", "params": {"id": SID, "verdict": "aprovado"}}}) + "\n")
f.flush()
refused = json.loads(f.readline())

ex("select.set", {"ids": [SID]}, agent=None)
ex("view.zoomSelection", {}, agent=None)
time.sleep(3)
call("ui.screenshot", {"path": f"{S}/O1-cartao-critica.png"})
# Test of the card only: the real review is the designer's.
ex("explore.critiqueStatus", {"id": SID, "finding": 1, "status": "decisão da designer"}, agent=None)
ex("explore.critiqueReviewed", {"id": SID}, agent=None)
time.sleep(2)
call("ui.screenshot", {"path": f"{S}/O2-critica-revisada.png"})

sev = {}
for x in c["findings"]:
    sev[x["severity"]] = sev.get(x["severity"], 0) + 1
lines = [f"Crítica da proposta {SID}: {len(c['findings'])} achado(s) — " + ", ".join(f"{v} {k}" for k, v in sev.items()),
         f"  medidos pelo Eureka: {len(measured)}; do roteiro (revisor): {len(crit.get('findings', []))}"]
lines += [f"  {x['n']}. [{x['severity']}] {x['area']}: {x['scenario']} ({x['evidence']})" for x in c["findings"]]
why = json.dumps(refused.get("error"), ensure_ascii=False)[:300]
lines += [f"Gatekeeper antes da revisão da crítica: {'ACEITO (erro)' if refused.get('ok') else 'recusado'} — {why}"]
lines += ["Status e \"Revisei a crítica\" foram um teste do cartão; a revisão real é da designer.",
          "Nada gravado no Notion nem no Figma; nenhum componente, token ou regra do DS foi alterado."]
open(f"{S}/O-critica.txt", "w").write("\n".join(lines) + "\n")
print("\n".join(lines))
