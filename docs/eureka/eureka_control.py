"""The Eureka scripts' side of the control channel's session token (E15).

`figcraft --control PORT` serves only clients that send its token first. The app takes it from
FIGCRAFT_CONTROL_TOKEN, or makes a random one and writes it to `figcraft-control-<PORT>.token` in
the user's temporary folder (only that user can read it). `auth(f, port)` sends it on a connection
made with `socket.create_connection(...).makefile("rw")`; `auth_socket(sock, port)` on a raw socket.
"""
import json
import os
import tempfile


def token(port):
    t = os.environ.get("FIGCRAFT_CONTROL_TOKEN", "").strip()
    if t:
        return t
    folders = [os.environ.get("TMPDIR"), tempfile.gettempdir(), "/tmp"]
    for folder in [d for d in folders if d]:
        path = os.path.join(folder, f"figcraft-control-{port}.token")
        if os.path.exists(path):
            with open(path, encoding="utf-8") as fh:
                return fh.read().strip()
    raise SystemExit(f"no control token for port {port}: start the app with --control {port}, or set FIGCRAFT_CONTROL_TOKEN")


def _check(line):
    r = json.loads(line or "{}")
    if not r.get("ok"):
        raise SystemExit(f"the app refused the control token: {r.get('error')}")


def auth(f, port):
    f.write(json.dumps({"id": 0, "method": "auth", "params": {"token": token(port)}}) + "\n")
    f.flush()
    _check(f.readline())


def auth_socket(sock, port):
    sock.sendall((json.dumps({"id": 0, "method": "auth", "params": {"token": token(port)}}) + "\n").encode())
    buf = b""
    while not buf.endswith(b"\n"):
        chunk = sock.recv(4096)
        if not chunk:
            break
        buf += chunk
    _check(buf.decode())
