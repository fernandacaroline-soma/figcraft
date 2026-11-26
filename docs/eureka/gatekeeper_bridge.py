"""Eureka G2, Frente G: bridge between the proposal card and the DS Gatekeeper (preview mode).

Decision of 24/11 (option a): the Gatekeeper reviews a proposal in PREVIEW MODE: read-only, no
record in Notion. The round is recorded only when the designer actually approves the change through
the official flow.

  export WORKDIR   writes WORKDIR/gk-request.json: the proposal being previewed (label, what each
                   layer gets, the hand-edit conflicts, the DS lint with sources) plus the context the
                   Gatekeeper needs (DS, collection, brand mode). Re-runs the dry run of the stored
                   proposal; nothing changes in the document.
  post WORKDIR VERDICT_JSON
                   attaches the Gatekeeper's verdict ({verdict, summary, items: [{message, source}]})
                   to the proposal being previewed (`proposal.review`, reviewer "DS Gatekeeper").

  export-choice WORKDIR SCREEN_ID
                   the same request for the proposal the designer CHOSE on an exploration board
                   (`explore.choose`): the proposal's description, the DS lint of its screen, the
                   context and the design critique the designer reviewed (Frente O; the Gatekeeper
                   is asked only after that review). Writes WORKDIR/gk-request.json.
  post-choice WORKDIR SCREEN_ID VERDICT_JSON
                   keeps the verdict on the chosen proposal (`explore.review`, reviewer "DS Gatekeeper").

The review itself runs in Claude (the `ds-orchestrator:ds-gatekeeper` agent, read-only), between the
two calls. The app only shows it; the designer decides.
"""
import json
import socket
import sys

cmd, S = sys.argv[1], sys.argv[2]
PORT = int(open(f"{S}/port").read())
f = socket.create_connection(("127.0.0.1", PORT), timeout=60).makefile("rw")
n = [0]


def call(method, params=None):
    n[0] += 1
    f.write(json.dumps({"id": n[0], "method": method, "params": params or {}}) + "\n")
    f.flush()
    r = json.loads(f.readline())
    if not r.get("ok", False):
        raise SystemExit(f"{method} failed: {json.dumps(r)[:600]}")
    return r.get("result")


if cmd == "export":
    state = json.load(open(f"{S}/proposal.json"))  # what was sent to proposal.preview
    params = dict(state)
    params["dryRun"] = True
    # A dry run of the same batch cannot run while the preview shows: end it, dry-run, show it again.
    call("engine.execute", {"command": "proposal.previewEnd", "params": {}})
    d = call("engine.execute", {"command": "command.batch", "params": params, "agent": "Claude"})
    p = call("engine.execute", {"command": "proposal.preview", "params": state, "agent": "Claude"})
    layers = []
    for k, title in (("created", "criada"), ("changed", "alterada"), ("removed", "removida")):
        for e in d.get(k, []):
            if k == "changed" and not e.get("summary"):
                continue  # only moved by layout
            layers.append({"layer": e.get("name"), "id": e.get("id"), "what": title, "changes": e.get("summary", [])})
    req = {
        "mode": "prévia — somente leitura, SEM registro no Notion (decisão da dona, 24/11, opção a)",
        "design_system": "DS Apps (Figma U3pOCU4CEBgYKHGVR1F3sK), coleção Tokens APP; marca na tela: Animale (CERTO)",
        "proposal": state.get("label"),
        "layers": layers,
        "hand_edit_conflicts": d.get("manualConflicts", []),
        "ds_lint": p.get("lint", []),
        "context": p.get("context", {}),  # kind of each layer, component of each instance, brand mode of each screen
        "rules_file": "docs/eureka/G2-F-regras.md",
        "answer_schema": {"verdict": "aprovado | com-ressalvas | bloqueado", "summary": "1-2 frases em pt-BR",
                          "items": [{"message": "o que corrigir, em pt-BR", "source": "regra oficial citada (página do Notion e seção)"}]},
    }
    json.dump(req, open(f"{S}/gk-request.json", "w"), indent=1, ensure_ascii=False)
    print(f"{S}/gk-request.json")
elif cmd == "post":
    v = json.load(open(sys.argv[3]))
    r = call("engine.execute", {"command": "proposal.review", "params": {
        "verdict": v["verdict"], "reviewer": "DS Gatekeeper", "summary": v.get("summary"), "items": v.get("items", [])}})
    call("ui.screenshot", {"path": f"{S}/G2-card-gatekeeper.png"})
    print(json.dumps(r, ensure_ascii=False)[:600])
elif cmd == "export-choice":
    sid = int(sys.argv[3])
    # Read what the proposal keeps before re-choosing (which keeps the chosen one's critique).
    kept = call("engine.execute", {"command": "node.get", "params": {"id": sid, "fields": ["pluginData"]}})["pluginData"]
    critique = json.loads(kept["eureka.critique"]) if "eureka.critique" in kept else None
    if not (critique or {}).get("reviewed"):
        raise SystemExit("the design critique comes first: keep it (explore.critique) and let the designer review it")
    c = call("engine.execute", {"command": "explore.choose", "params": {"id": sid}})
    info = c["chosen"]["proposal"] or {}
    modes = [sc.get("modes", {}) for sc in c["context"].get("screens", [])]
    req = {
        "mode": "prévia — somente leitura, SEM registro no Notion (decisão da dona, 24/11, opção a)",
        "design_system": "DS Apps (Figma U3pOCU4CEBgYKHGVR1F3sK), coleção Tokens APP",
        "brand_modes": modes,
        "board": info.get("board"), "problem": (info.get("context") or {}).get("problem"),
        "proposal": {k: info.get(k) for k in ("letter", "name", "type", "mechanism", "summary", "confidence")},
        "not_chosen": [o["name"] for o in c["others"]],
        "ds_lint": c["lint"],
        "context": c["context"],
        "design_critique": critique,
        "rules_file": "docs/eureka/G2-F-regras.md",
        "answer_schema": {"verdict": "aprovado | com-ressalvas | bloqueado", "summary": "1-2 frases em pt-BR",
                          "items": [{"message": "o que corrigir, em pt-BR", "source": "regra oficial citada (página do Notion e seção)"}]},
    }
    json.dump(req, open(f"{S}/gk-request.json", "w"), indent=1, ensure_ascii=False)
    print(f"{S}/gk-request.json")
elif cmd == "post-choice":
    sid, v = int(sys.argv[3]), json.load(open(sys.argv[4]))
    r = call("engine.execute", {"command": "explore.review", "params": {
        "id": sid, "verdict": v["verdict"], "reviewer": "DS Gatekeeper", "summary": v.get("summary"), "items": v.get("items", [])}})
    print(json.dumps(r, ensure_ascii=False)[:600])
else:
    raise SystemExit(__doc__)
