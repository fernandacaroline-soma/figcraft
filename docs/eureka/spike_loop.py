"""Eureka G1 checkpoint demo: the whole loop, with Claude reacting on its own.

  1. Claude generated the screen (run `spike_proposals.py WORKDIR TOKENS --steps=import,generate` first).
  2. The designer edits in the app (with --live, by hand with the mouse; otherwise this script makes
     the same edits as a person would, without an agent name).
  3. Claude notices by itself: it waits on `events.poll` (topic `document`), keeps only the
     designer's events (`source: human`) and waits for a quiet moment.
  4. Claude proposes: raw values the designer typed go back to the nearest design-system token,
     plus the "Frete grátis" badge. A dry run gives the card: what changes, the hand-edit conflicts
     (the designer's edit wins by default), the DS warnings.
  5. The designer chooses on the card (here: Claude's tokens for the colours, keep the hand-picked
     price size), sees the preview and applies. One undo step.

Usage: python3 spike_loop.py WORKDIR TOKENS_JSON [--live]
"""
import json
import os
import socket
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import tokens_snapshot  # noqa: E402

args = [a for a in sys.argv[1:] if not a.startswith("--")]
S, TOK = args[0], args[1]
LIVE = "--live" in sys.argv
PORT = int(open(f"{S}/port").read())
sock = socket.create_connection(("127.0.0.1", PORT), timeout=120)
f = sock.makefile("rw")
n = [0]
LOG = []


def say(line=""):
    print(line)
    LOG.append(line)


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r)[:900]}")
    return r.get("result")


def ex(cmd, params, agent="Claude"):
    p = {"command": cmd, "params": params}
    if agent:
        p["agent"] = agent
    return call("engine.execute", p)


def shot(name):
    call("ui.screenshot", {"path": f"{S}/{name}.png"})


def rgb(c):
    if isinstance(c, str):  # "#RRGGBB" or "#RRGGBBAA"
        h = c.lstrip("#")
        return tuple(int(h[i:i + 2], 16) / 255 for i in (0, 2, 4))
    return tuple(float(c.get(k, 0)) for k in ("r", "g", "b"))


def hexs(c):
    return "#" + "".join(f"{round(x * 255):02X}" for x in rgb(c))


state = json.load(open(f"{S}/state.json"))
ids = state["ids"]
names = {v: k for k, v in ids.items()}
tokens = tokens_snapshot.resolve(tokens_snapshot.load(TOK), "Animale")
colour_tokens = {k: v for k, v in tokens.items() if isinstance(v, dict) and "r" in v and v.get("a", 1) > 0.99}
size_tokens = {k: v for k, v in tokens.items() if k.startswith("Font/Size/") and isinstance(v, (int, float))}


def nearest_colour(c, prefix):
    cands = {k: v for k, v in colour_tokens.items() if k.startswith(prefix)} or colour_tokens
    return min(cands, key=lambda k: sum((a - b) ** 2 for a, b in zip(rgb(c), rgb(cands[k]))))


# --- 2. the designer edits -------------------------------------------------------------------
cursor = call("events.poll", {"after": 0, "topics": ["document"], "timeoutMs": 0})["cursor"]
if LIVE:
    say("Edite o card no app (preço, cores…). O Claude está esperando.")
else:
    ex("node.set", {"ids": [ids["Preço"]], "props": {"characters": "R$ 349,90", "fontSize": 18}}, agent=None)
    ex("node.set", {"ids": [ids["Button"]], "props": {"fill": "#7A4B2A"}}, agent=None)
    ex("node.set", {"ids": [ids["Label"]], "props": {"fill": "#FFFFFF"}}, agent=None)
    shot("L1-designer")

# --- 3. Claude notices -----------------------------------------------------------------------
seen = []
while True:
    r = call("events.poll", {"after": cursor, "topics": ["document"], "timeoutMs": 20000 if not seen else 3000})
    cursor = r["cursor"]
    mine = [e for e in r["events"] if e["data"].get("source") == "human"]
    seen += mine
    if seen and not mine:
        break  # the designer paused
    if not r["events"] and not seen and not LIVE:
        raise SystemExit("no edit seen")
changed = sorted({i for e in seen for i in e["data"].get("changed", [])})
say("Claude percebeu sozinho (events.poll, tópico document):")
for e in seen:
    d = e["data"]
    say(f"  · {d['command']} por designer → {', '.join(names.get(i, f'#{i}') for i in d['changed'])}")

# --- 4. Claude proposes ----------------------------------------------------------------------
cmds, why = [], []
for i in changed:
    if i not in names:
        continue
    node = call("node.get", {"id": i, "fields": ["name", "type", "fills", "fontSize", "boundVariables"]})
    bound = node.get("boundVariables") or {}
    fills = [p for p in (node.get("fills") or []) if isinstance(p, dict) and p.get("type", "").lower() == "solid"]
    if fills and "fill" not in bound and "color" in fills[0]:
        prefix = "Colors/Content/" if node.get("type", "").lower() == "text" else "Colors/Surface/"
        tok = nearest_colour(fills[0]["color"], prefix)
        cmds.append({"command": "variable.bind", "params": {"ids": [i], "field": "fill", "variable": tok}})
        why.append(f"{node['name']}: cor solta {hexs(fills[0]['color'])} → {tok}")
    size = node.get("fontSize")
    if isinstance(size, (int, float)) and "fontSize" not in bound and size_tokens:
        tok = min(size_tokens, key=lambda k: abs(size_tokens[k] - size))
        cmds.append({"command": "variable.bind", "params": {"ids": [i], "field": "fontSize", "variable": tok}})
        why.append(f"{node['name']}: tamanho solto {size:g} → {tok} ({size_tokens[tok]:g})")
cmds += [
    {"command": "frame.create", "as": "tag", "params": {"parent": ids["Product Card"], "index": 1, "x": 0, "y": 0, "width": 100, "height": 24,
        "name": "Selo Frete grátis", "props": {"fill": {"$var": "Tokens WEB/Colors/Feedback/Success/Surface"}, "layout": "horizontal", "padding": 0, "sizingH": "hug", "sizingV": "hug"}}},
    {"command": "variable.bind", "params": {"ids": [{"$ref": "tag"}], "field": "paddingHorizontal", "variable": "Spacing/sp-xs"}},
    {"command": "variable.bind", "params": {"ids": [{"$ref": "tag"}], "field": "paddingVertical", "variable": "Spacing/sp-xxs"}},
    {"command": "variable.bind", "params": {"ids": [{"$ref": "tag"}], "field": "cornerRadius", "variable": "Border/Radius/Components/br-chips"}},
    {"command": "text.create", "as": "t", "params": {"parent": {"$ref": "tag"}, "text": "FRETE GRÁTIS", "x": 0, "y": 0, "name": "Selo texto",
        "props": {"fontSize": 12, "fill": {"$var": "Tokens WEB/Colors/Feedback/Success/Content"}}}},
    {"command": "variable.bind", "params": {"ids": [{"$ref": "t"}], "field": "fontSize", "variable": "Font/Size/fs-xxxs"}},
]
why.append("novo: selo Frete grátis, todo em tokens")
say()
say("Proposta do Claude:")
for w in why:
    say(f"  · {w}")


def card(d):
    say(f"Cartão: {d['total']} camadas mudam · avisos de DS: {len(d.get('dsWarnings', [])) or 'nenhum'} · bloqueado: {'sim' if d.get('blocked') else 'não'}")
    for c in d.get("manualConflicts", []):
        who = {"manual": "fica a sua", "proposal": "fica a do Claude", "needs-choice": "PRECISA DE ESCOLHA"}.get(c["kept"], c["kept"])
        say(f"  ⚑ {c['name']} — {', '.join(c.get('props', []))} → {who}")
        for x in c.get("manual", []):
            say(f"      você:   {x}")
        for x in c.get("proposal", []):
            say(f"      Claude: {x}")


label = "Claude: voltar aos tokens + selo"
d = ex("command.batch", {"label": label, "dryRun": True, "commands": cmds})
say()
say("Padrão (sua edição vence):")
card(d)

# --- 5. the designer chooses, previews, applies ----------------------------------------------
take = [c["id"] for c in d.get("manualConflicts", []) if c["name"] in ("Button", "Label")]
say()
say("Escolha no cartão: cores → fica a do Claude; tamanho do preço → fica a sua")
p = ex("proposal.preview", {"label": label, "override": take, "commands": cmds})
card(p)
shot("L2-preview")
a = ex("proposal.apply", {})
say()
say(f"Aplicado: {a.get('applied')} (1 passo de desfazer)")
after = call("events.poll", {"after": cursor, "topics": ["document"], "timeoutMs": 0})
for e in after["events"]:
    say(f"  evento: {e['data']['command']} por {e['data'].get('agent') or 'designer'}")
shot("L3-applied")
preco = call("node.get", {"id": ids["Preço"], "fields": ["fontSize", "boundVariables"]})
say(f"Preço: tamanho {preco.get('fontSize'):g}, vínculos {preco.get('boundVariables') or {}} (a escolha da designer ficou)")
open(f"{S}/L-loop.txt", "w").write("\n".join(LOG) + "\n")
