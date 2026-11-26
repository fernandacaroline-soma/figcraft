"""Spike (Tarefa 6 da FILA): TSX -> Eureka -> TSX.

Le um TSX simples, do tipo que o `export.tsx` gera (JSX de <div>/<p> com className e o CSS no
fim do arquivo), e o reconstroi como camadas no FigCraft com comandos. As variaveis sao importadas
de um snapshot do DS, fora do repositorio. Depois roda o `export.tsx` de novo e compara a volta com
o original, linha a linha.

Nao e um importador de producao: so mede o que a ida e volta preserva e o que perde.

Uso:
  python -X utf8 docs/eureka/spike_tsx_volta.py --tsx <arquivo.tsx> --snapshot <variables.json> \
      --cli <figcraft-cli> --out <pasta de trabalho fora do repo>

Gera em --out: script.json (comandos), run.json (saida do cli), volta.tsx e relatorio.json.
"""
import argparse
import difflib
import json
import re
import subprocess
import sys
import unicodedata
from pathlib import Path


def slug(name, fallback="token"):
    """O mesmo nome de variavel CSS do export (`ds_var_name`, sem o `--`)."""
    s = unicodedata.normalize("NFKD", name).encode("ascii", "ignore").decode().lower()
    s = re.sub(r"[^a-z0-9]+", "-", s).strip("-")[:80].strip("-")
    return s or fallback


def brand_slug(mode):
    no_notes = re.sub(r"\([^)]*\)", "", mode)
    return slug(no_notes if no_notes.strip() else mode, "brand")


# ------------------------------------------------------------------ leitura do TSX


def split_tsx(text):
    """(jsx, css): o que o componente retorna e a folha de estilo do fim do arquivo."""
    body = text[text.find("return (") + len("return ("):]
    jsx = body[: body.find("\n  );")]
    css_start = text.find(".css */")
    css = text[css_start + len(".css */"):] if css_start >= 0 else ""
    return jsx, css


TAG = re.compile(r"<(/?)(div|p|span)\b([^>]*?)(/?)>|([^<]+)")
ATTR = re.compile(r'([a-zA-Z-]+)="([^"]*)"')


def parse_jsx(jsx):
    """A arvore: [{tag, attrs, text, children}]."""
    root = {"tag": "root", "attrs": {}, "text": "", "children": []}
    stack = [root]
    for m in TAG.finditer(jsx):
        closing, tag, attrs, selfclose, text = m.groups()
        if text is not None:
            t = text.strip()
            if t and stack[-1]["tag"] != "root":
                stack[-1]["text"] += t
            continue
        if closing:
            if len(stack) > 1:
                stack.pop()
            continue
        node = {"tag": tag, "attrs": dict(ATTR.findall(attrs)), "text": "", "children": []}
        stack[-1]["children"].append(node)
        if not selfclose:
            stack.append(node)
    return root["children"]


def parse_css(css):
    rules = {}
    for m in re.finditer(r"\.([a-zA-Z0-9_-]+)\s*\{([^}]*)\}", css):
        decls = {}
        for d in m.group(2).split(";"):
            if ":" in d:
                k, v = d.split(":", 1)
                decls[k.strip()] = v.strip()
        rules[m.group(1)] = decls
    return rules


# ------------------------------------------------------------------ TSX -> comandos


class Builder:
    def __init__(self, variables, modes):
        self.steps = []
        self.next_id = 2  # documento novo: a pagina e 1, as camadas vem depois, em ordem
        self.by_css = {slug(v["name"]): v["name"] for v in variables}
        self.modes = modes  # brand slug -> (colecao, modo)
        self.lost = []  # o que a volta nao conseguiu ler

    def step(self, command, **params):
        self.steps.append({"command": command, "params": params})

    def token(self, value):
        m = re.fullmatch(r"var\(--([a-z0-9-]+)\)", value.strip())
        return self.by_css.get(m.group(1)) if m else None

    def px(self, value):
        m = re.fullmatch(r"(-?[0-9.]+)px", value.strip())
        return float(m.group(1)) if m else None

    def bind_or_set(self, nid, field, value, prop=None, what=""):
        var = self.token(value)
        if var:
            self.step("variable.bind", ids=[nid], field=field, variable=var)
            return
        if value.startswith("var("):
            self.lost.append(f"{what} {field}: {value} (token que o snapshot nao tem)")
            return
        if prop is not None:
            self.step("node.set", id=nid, props={prop: value if not value.endswith("px") else self.px(value)})

    def build(self, node, parent, parent_layout, rules):
        nid = self.next_id
        self.next_id += 1
        cls = node["attrs"].get("className", "")
        decls = rules.get(cls, {})
        name = cls  # o nome da camada nao volta: so a classe
        if node["tag"] == "p":
            params = {"text": node["text"] or " ", "x": 0, "y": 0, "props": {"name": name}}
            if parent:
                params["parent"] = parent
            self.step("text.create", **params)
        else:
            params = {"type": "frame", "name": name, "width": self.px(decls.get("width", "100px")) or 100,
                      "height": self.px(decls.get("height", "100px")) or 100}
            if parent:
                params["parent"] = parent
            self.step("node.create", **params)
        layout = None
        props = {}
        if decls.get("display") == "flex":
            layout = "vertical" if decls.get("flex-direction") == "column" else "horizontal"
            lay = {"mode": layout}
            gap = decls.get("gap", "0px")
            if self.px(gap) is not None:
                lay["gap"] = self.px(gap)
            pad = decls.get("padding", "").split()
            if len(pad) == 4 and all(self.px(p) is not None for p in pad):
                lay["padding"] = [self.px(p) for p in pad]
            props["layout"] = lay
            align = {"flex-start": "min", "center": "center", "flex-end": "max", "space-between": "spaceBetween"}
            if decls.get("justify-content") in align:
                props["mainAlign"] = align[decls["justify-content"]]
            if decls.get("align-items") in align:
                props["crossAlign"] = align[decls["align-items"]]
        if "height" not in decls and node["tag"] != "p" and layout:
            props["sizingV" if layout == "vertical" else "sizingH"] = "hug"
        if decls.get("align-self") == "stretch" and parent_layout:
            props["sizingH" if parent_layout == "vertical" else "sizingV"] = "fill"
        if decls.get("position") == "absolute":
            props["x"], props["y"] = self.px(decls.get("left", "0px")), self.px(decls.get("top", "0px"))
        if props:
            self.step("node.set", id=nid, props=props)
        what = f"{name}:"
        # Ligacoes de espaco (o export escreve var() tambem no gap e no padding)
        if layout:
            if self.token(decls.get("gap", "")):
                self.step("variable.bind", ids=[nid], field="gap", variable=self.token(decls["gap"]))
            pad = decls.get("padding", "").split()
            for side, v in zip(["paddingTop", "paddingRight", "paddingBottom", "paddingLeft"], pad):
                if self.token(v):
                    self.step("variable.bind", ids=[nid], field=side, variable=self.token(v))
        for css_prop, field in [("background", "fill"), ("color", "fill")]:
            if css_prop in decls:
                self.bind_or_set(nid, field, decls[css_prop], "fill", what)
        if "border-radius" in decls:
            corners = decls["border-radius"].split()
            if len(corners) == 1:
                self.bind_or_set(nid, "cornerRadius", corners[0], "cornerRadius", what)
            else:
                self.lost.append(f"{what} border-radius por canto: {decls['border-radius']}")
        if "font-size" in decls:
            self.bind_or_set(nid, "fontSize", decls["font-size"], "fontSize", what)
        if "font-family" in decls:
            fam = decls["font-family"].strip('"')
            if self.token(decls["font-family"]):
                self.step("variable.bind", ids=[nid], field="fontFamily", variable=self.token(decls["font-family"]))
            else:
                self.step("node.set", id=nid, props={"fontFamily": fam})
        for k in decls:
            if k not in KNOWN:
                self.lost.append(f"{what} {k}: {decls[k]} (propriedade que a volta nao le)")
        for brand_attr, brand in node["attrs"].items():
            if brand_attr == "data-brand" and brand in self.modes:
                coll, mode = self.modes[brand]
                self.step("variable.setMode", ids=[nid], collection=coll, mode=mode)
            elif brand_attr.startswith("data-"):
                self.lost.append(f"{what} {brand_attr}={brand} (modo de outra colecao)")
        for child in node["children"]:
            self.build(child, nid, layout, rules)


KNOWN = {"position", "left", "top", "width", "height", "display", "flex-direction", "gap", "padding",
         "justify-content", "align-items", "align-self", "background", "color", "border-radius",
         "font-size", "font-family", "font-weight"}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--tsx", required=True)
    ap.add_argument("--snapshot", required=True)
    ap.add_argument("--cli", required=True)
    ap.add_argument("--out", required=True)
    a = ap.parse_args()
    out = Path(a.out)
    out.mkdir(parents=True, exist_ok=True)
    original = Path(a.tsx).read_text(encoding="utf8")
    snap = json.loads(Path(a.snapshot).read_text(encoding="utf8"))
    meta = snap.get("meta", snap)
    variables = list(meta["variables"].values() if isinstance(meta["variables"], dict) else meta["variables"])
    colls = meta["variableCollections"]
    colls = list(colls.values() if isinstance(colls, dict) else colls)
    modes = {brand_slug(m["name"]): (c["name"], m["name"]) for c in colls for m in c["modes"]}

    jsx, css = split_tsx(original)
    tree = parse_jsx(jsx)
    rules = parse_css(css)
    b = Builder(variables, modes)
    b.step("variable.import", variables=str(Path(a.snapshot).resolve()))
    for n in tree:
        b.build(n, None, None, rules)
    b.step("export.tsx", ids=[2])
    (out / "script.json").write_text(json.dumps(b.steps, ensure_ascii=False, indent=1), encoding="utf8")
    run = subprocess.run([a.cli, "run", str(out / "script.json")], capture_output=True, text=True, encoding="utf8")
    (out / "run.json").write_text(run.stdout, encoding="utf8")
    results = [json.loads(line) for line in run.stdout.splitlines() if line.strip().startswith("{")]
    errors = [r for r in results if r.get("ok") is False or "error" in r]
    back = next((r["result"]["code"] for r in reversed(results) if isinstance(r.get("result"), dict) and "code" in r["result"]), "")
    (out / "volta.tsx").write_text(back, encoding="utf8")

    def body(t):
        return t[t.find("import "):].splitlines() if "import " in t else t.splitlines()

    diff = [d for d in difflib.unified_diff(body(original), body(back), "original", "volta", lineterm="", n=0)
            if d.startswith(("+", "-")) and not d.startswith(("+++", "---"))]
    same = sum(1 for _ in difflib.SequenceMatcher(None, body(original), body(back)).get_matching_blocks())
    report = {
        "tsx": a.tsx, "camadas": b.next_id - 2, "comandos": len(b.steps), "erros": errors[:20],
        "linhas_original": len(body(original)), "linhas_volta": len(body(back)),
        "linhas_iguais": sum(blk.size for blk in difflib.SequenceMatcher(None, body(original), body(back)).get_matching_blocks()),
        "diferencas": diff[:200], "perdas_lidas": b.lost, "blocos_iguais": same,
    }
    (out / "relatorio.json").write_text(json.dumps(report, ensure_ascii=False, indent=1), encoding="utf8")
    print(json.dumps({k: report[k] for k in ("camadas", "comandos", "linhas_original", "linhas_volta", "linhas_iguais")}, ensure_ascii=False))
    print(f"{len(errors)} erro(s) de comando; {len(diff)} linha(s) diferentes; {len(b.lost)} perda(s) na leitura")
    for d in diff[:60]:
        print(d)
    return 0 if not errors else 1


if __name__ == "__main__":
    sys.exit(main())
