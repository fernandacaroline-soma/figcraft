"""Eureka G2 · Frente F: design-system lint demo over FigCraft's control channel.
Builds the demo card (spike_proposals.py --steps=import,generate,manual: Claude generates with tokens, the
designer then sets the price to 18 and paints the button #7A4B2A and its label #FFFFFF by hand), runs
`ds.lint` and writes a report a designer reads. The lint only warns; nothing changes in the document.

Usage: python -X utf8 spike_lint.py WORKDIR TOKENS_JSON [--no-setup]
  WORKDIR       holds a file `port` with the app's control port; lint.json and lint.txt are written there.
  TOKENS_JSON   the Figma variables snapshot (kept outside the repository, ADR-002).
  --no-setup    lint the document already open (skip spike_proposals.py).
Rules and their sources: docs/eureka/G2-F-regras.md."""
import json
import os
import socket
import subprocess
import sys

args = [a for a in sys.argv[1:] if not a.startswith("--")]
S, TOK = args[0], args[1]

if "--no-setup" not in sys.argv:
    here = os.path.dirname(os.path.abspath(__file__))
    subprocess.run([sys.executable, "-X", "utf8", os.path.join(here, "spike_proposals.py"), S, TOK, "--steps=import,generate,manual"], check=True)

PORT = int(open(f"{S}/port").read())
sock = socket.create_connection(("127.0.0.1", PORT), timeout=120)
__import__("eureka_control").auth_socket(sock, PORT)
f = sock.makefile("rw", encoding="utf-8")


def call(method, params, i=[0]):
    i[0] += 1
    f.write(json.dumps({"id": i[0], "method": method, "params": params}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r)[:900]}")
    return r.get("result")


before = call("engine.execute", {"command": "document.checkpoint", "params": {}})["cursor"]
r = call("engine.execute", {"command": "ds.lint", "params": {}, "agent": "Claude"})
after = call("engine.execute", {"command": "document.changesSince", "params": {"cursor": before}})
with open(f"{S}/lint.json", "w", encoding="utf-8") as fh:
    json.dump(r, fh, indent=1, ensure_ascii=False)

TITLES = {
    "valor-solto": "Valor fixo onde existe token",
    "escopo": "Variable fora do escopo declarado",
    "marca-nao-escolhida": "Tela sem marca escolhida",
    "marca-no-componente": "Marca presa no componente original",
}
lines = ["Lint de DS — só avisos, nada foi alterado", ""]
by_rule = {}
for x in r["findings"]:
    by_rule.setdefault(x["rule"], []).append(x)
for rule, items in by_rule.items():
    lines.append(f"## {TITLES.get(rule, rule)} ({len(items)})")
    lines.append(f"   fonte: {items[0]['source']}")
    for x in items:
        lines.append(f"   ! {x['message']}  [id {x['id']}]")
    lines.append("")
lines.append(f"achados: {r['counted']} (listados {len(r['findings'])}), camadas vistas: {r['scanned']}, cortado: {'sim' if r['truncated'] else 'não'}")
lines.append(f"documento alterado pelo lint: {'sim' if after.get('total') else 'não'}")
text = "\n".join(lines) + "\n"
with open(f"{S}/lint.txt", "w", encoding="utf-8") as fh:
    fh.write(text)
print(text)
