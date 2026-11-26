"""E15: check that this computer reaches the app through the control channel's session token.

Start the app with `--control PORT` (after `git pull` and a build), then run:
    python3 docs/eureka/checar_chave.py PORT
Paste the output in docs/eureka/FILA-OUTRO-PC.md (Recados). It prints no secret.
"""
import json
import socket
import sys

sys.path.insert(0, __file__.rsplit("/", 1)[0])
import eureka_control  # noqa: E402

PORT = int(sys.argv[1])


def first_reply(lines):
    f = socket.create_connection(("127.0.0.1", PORT), timeout=30).makefile("rw")
    out = []
    for obj in lines:
        f.write(json.dumps(obj) + "\n")
        f.flush()
        line = f.readline()
        out.append(json.loads(line) if line else None)
    return out


wrong = first_reply([{"id": 1, "method": "auth", "params": {"token": "errada"}}])[0]
none = first_reply([{"id": 1, "method": "ui.inspect", "params": {}}])[0]
right, inspect = first_reply([
    {"id": 1, "method": "auth", "params": {"token": eureka_control.token(PORT)}},
    {"id": 2, "method": "ui.inspect", "params": {}},
])
ok = (wrong and not wrong["ok"]) and (none and not none["ok"]) and right["ok"] and inspect["ok"]
print(f"chave errada recusada: {bool(wrong and not wrong['ok'])}")
print(f"sem chave recusado: {bool(none and not none['ok'])}")
print(f"chave certa: {right['ok']} · {right.get('result', {}).get('app')} · protocolo {right.get('result', {}).get('protocol')}")
print(f"comando depois da chave: {inspect['ok']}")
print("RESULTADO: OK" if ok else "RESULTADO: FALHOU")
