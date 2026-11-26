"""Eureka E15: a mock agent connector, driving a running app through the agent contract
(docs/eureka/E15-contrato-conectores.md). It shows each part of the contract on the real app:

1. isolation: no token, then a wrong token, are refused and the connection ends;
2. the handshake: the right token, and the protocol version the connector needs;
3. context: what the design system offers (`explore.catalog`);
4. work: a board of two proposals, as the agent ("Mock agent");
5. errors: a wrong request is refused and changes nothing (`document.changesSince`);
6. progress: what the work changed since it started (`document.checkpoint`);
7. human approval: the agent may not choose the proposal; the person does (no agent).

Usage: python3 conector_mock.py WORKDIR   (the app runs with --control PORT; WORKDIR/port has PORT)
Writes WORKDIR/E15-conector.txt.
"""
import json
import socket
import sys

S = sys.argv[1]
PORT = int(open(f"{S}/port").read())
sys.path.insert(0, __file__.rsplit("/", 1)[0])
import eureka_control  # noqa: E402

AGENT = "Mock agent"
NEEDS_PROTOCOL = 1
lines = []


def say(text):
    lines.append(text)
    print(text)


def open_conn():
    return socket.create_connection(("127.0.0.1", PORT), timeout=60).makefile("rw")


def send(f, obj):
    f.write(json.dumps(obj) + "\n")
    f.flush()
    line = f.readline()
    return json.loads(line) if line else None


# 1. Isolation.
f = open_conn()
r = send(f, {"id": 1, "method": "engine.execute", "params": {"command": "explore.catalog", "params": {}}})
closed = f.readline() == ""
say(f"1. Sem chave: {r['error'][:40]}… · conexão encerrada: {closed}")
f = open_conn()
r = send(f, {"id": 1, "method": "auth", "params": {"token": "chute"}})
say(f"   Chave errada: {r['error']} · conexão encerrada: {f.readline() == ''}")

# 2. Handshake.
f = open_conn()
r = send(f, {"id": 1, "method": "auth", "params": {"token": eureka_control.token(PORT)}})
proto = r["result"]["protocol"]
say(f"2. Chave certa: {r['result']['app']}, protocolo {proto} (o conector pede {NEEDS_PROTOCOL}: {'ok' if proto >= NEEDS_PROTOCOL else 'para aqui'})")
if proto < NEEDS_PROTOCOL:
    raise SystemExit("protocol too old")
n = [1]


def ex(cmd, params, agent=AGENT):
    n[0] += 1
    p = {"command": cmd, "params": params}
    if agent:
        p["agent"] = agent
    return send(f, {"id": n[0], "method": "engine.execute", "params": p})


# 3. Context and 4. work.
ex("file.new", {"name": "Eureka · E15 · conector de teste"}, agent=None)
ex("variable.createCollection", {"name": "Tokens APP", "modes": ["NV"]})
ex("variable.create", {"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"})
cat = ex("explore.catalog", {})["result"]
say(f"3. Contexto: o catálogo lista {len(cat.get('variables', []))} variável(is) e {len(cat.get('components', []))} componente(s)")
screen = lambda name: {"name": name, "type": "incremental", "screen": {"frame": "PDP", "width": 120, "height": 120, "fill": "Colors/Bg", "children": []}}
start = ex("document.checkpoint", {})["result"]["cursor"]
board = ex("explore.board", {"title": "Rodada do conector", "proposals": [screen("Um"), screen("Dois")]})["result"]
a = board["proposals"][0]["id"]
say(f"4. Trabalho: quadro com {len(board['proposals'])} propostas, feito como \"{AGENT}\"")

# 5. Errors: a wrong request changes nothing.
mark = ex("document.checkpoint", {})["result"]["cursor"]
r = ex("explore.board", {"title": "Errada", "proposals": [screen("Só uma")]})
changed = ex("document.changesSince", {"cursor": mark})["result"]["total"]
say(f"5. Erro: {r['error'][:70]}… · camadas mudadas depois do erro: {changed}")

# 6. Progress: what the work changed since the connector started it.
done = ex("document.changesSince", {"cursor": start})["result"]
say(f"6. Progresso: {len(done['created'])} camada(s) criada(s) desde o início do trabalho")

# 7. Human approval.
r = ex("explore.choose", {"id": a})
say(f"7. O agente tenta escolher: {r['error']}")
r = ex("explore.choose", {"id": a}, agent=None)
say(f"   A pessoa escolhe (sem agente, como o botão do cartão): ok = {r['ok']} (teste: a escolha real é da designer)")

open(f"{S}/E15-conector.txt", "w").write("\n".join(lines) + "\n")
