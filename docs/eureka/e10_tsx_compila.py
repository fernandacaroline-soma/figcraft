"""Eureka E10: does the exported TSX compile?

Exports a screen as TSX + tokens.css with `figcraft-cli run` (`export.code`), makes a minimal
temporary React + TypeScript project, declares the packages the TSX imports with FAKE types
(generated from its imports, named exports typed as loose components), runs `npx tsc --noEmit`
there and writes a report: compiles or not, each error with its line and the line's text.

Two sources of the screen:
- synthetic (default): the script builds a document itself: a screen with a Code-Connect-linked
  Button, a text style, tokens bound to the layers. Runs anywhere, no real DS data.
- `--input FILE.figcraft --ids ID[,ID] [--map MAP.json]`: a real file (e.g. the template base with
  the map linked). The generated TSX/CSS stay in the temp dir; only the report is written, and it
  has error codes and messages, never the source text unless `--quote` is given.

Local installs only: `npm install` runs in the temp dir, never globally. Not part of CI.

Usage: python -X utf8 e10_tsx_compila.py [--cli PATH] [--input F --ids 1,2 [--map M]] [--out REPORT.json]
       [--keep] [--quote]
"""
import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

PACKAGE = "@acme/ds-apps"  # synthetic; a real map's own package names come from its imports


def synthetic_script():
    """The commands that build the sample document and export it. Names and values are invented."""
    def step(command, params, name=None):
        s = {"command": command, "params": params}
        if name:
            s["as"] = name
        return s

    ref = lambda n: {"$ref": n}  # noqa: E731
    steps = [
        step("variable.createCollection", {"name": "Tokens APP", "modes": ["Farm", "Animale (CERTO)"]}),
        step("variable.create", {"collection": "Tokens APP", "name": "Colors/Surface/Brand/Background", "type": "color", "value": "#0D0D0D"}, "bg"),
        step("variable.create", {"collection": "Tokens APP", "name": "Spacing/sp-md", "type": "float", "value": 16}, "md"),
        step("variable.create", {"collection": "Tokens APP", "name": "Colors/Content/Brand/On Brand", "type": "color", "value": "#FFFFFF"}, "ink"),
        step("node.create", {"type": "frame", "name": "PDP Mobile", "width": 375, "height": 600}, "screen"),
        step("node.create", {"type": "frame", "name": "Product Card", "width": 300, "height": 200, "parent": ref("screen")}, "card"),
        step("node.set", {"id": ref("card"), "props": {"layout": {"mode": "vertical", "gap": 16, "padding": [16, 16, 16, 16]}}}),
        step("text.create", {"text": "R$ 349,90", "x": 0, "y": 0, "parent": ref("card")}, "price"),
        step("node.set", {"id": ref("price"), "props": {"fontSize": 18}}),
        step("variable.bind", {"ids": [ref("card")], "field": "fill", "variable": ref("bg")}),
        *[step("variable.bind", {"ids": [ref("card")], "field": f, "variable": ref("md")})
          for f in ["gap", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft"]],
        step("variable.bind", {"ids": [ref("price")], "field": "fill", "variable": ref("ink")}),
        step("variable.setMode", {"ids": [ref("screen")], "collection": "Tokens APP", "mode": "Animale (CERTO)"}),
        step("style.create", {"type": "text", "name": "Body/Small", "value": {"fontFamily": "Inter", "fontStyle": "Regular", "fontSize": 12, "lineHeight": "150%", "letterSpacing": 0.5}}),
    ]
    # A design-system Button (a set, marked as from the library) linked to code through a map.
    for k, h in enumerate(["Primário", "Secundário"]):
        steps += [
            step("frame.create", {"x": 800 + k * 150, "y": 0, "width": 120, "height": 40, "name": f"Hierarquia={h}"}, f"vf{k}"),
            step("node.set", {"id": ref(f"vf{k}"), "props": {"fill": "#7A4B2A"}}),
            step("component.create", {"ids": [ref(f"vf{k}")]}, f"v{k}"),
        ]
    steps += [
        step("component.combineVariants", {"ids": [ref("v0"), ref("v1")]}, "set"),
        step("node.set", {"id": ref("set"), "props": {"name": "Button", "pluginData": {"eureka.library": "{\"name\":\"DS Apps\"}"}}}),
        step("instance.create", {"component": ref("set"), "x": 0, "y": 0, "parent": ref("card")}, "btn"),
        step("instance.setProperties", {"ids": [ref("btn")], "properties": {"Hierarquia": "Secundário"}}),
        step("codeConnect.import", {"json": {"version": 1, "links": [
            {"component": "Button", "language": "react", "importPath": PACKAGE, "name": "Button", "template": "<Button variant={Hierarquia} />"}]}}),
        step("export.code", {"ids": [ref("screen")]}, "code"),
    ]
    return steps


def real_script(ids, map_path):
    steps = []
    if map_path:
        steps.append({"command": "codeConnect.import", "params": {"path": map_path}})
    steps.append({"command": "export.code", "as": "code", "params": {"ids": ids}})
    return steps


def run_cli(cli, script, input_path, workdir):
    path = os.path.join(workdir, "script.json")
    with open(path, "w", encoding="utf-8", newline="") as fh:
        json.dump(script, fh, ensure_ascii=False)
    cmd = [cli, "run", path] + (["--in", input_path] if input_path else [])
    p = subprocess.run(cmd, capture_output=True, text=True, encoding="utf-8")
    if p.returncode != 0:
        raise SystemExit(f"figcraft-cli run failed ({p.returncode}):\n{p.stdout[-1500:]}\n{p.stderr[-1500:]}")
    lines = [json.loads(line) for line in p.stdout.splitlines() if line.startswith("{")]
    return lines[-1]["result"]


IMPORT = re.compile(r"""^\s*import\s+(?P<what>[^;]*?)\s+from\s+["'](?P<pkg>[^"'./][^"']*)["']""", re.M)


def fake_declarations(tsx):
    """`declare module` for every package imported, each named import a loose component."""
    mods = {}
    for m in IMPORT.finditer(tsx):
        what, pkg = m.group("what"), m.group("pkg")
        named = re.search(r"\{([^}]*)\}", what)
        names = mods.setdefault(pkg, set())
        for n in (named.group(1).split(",") if named else []):
            n = n.strip().split(" as ")[0].strip()
            if n:
                names.add(n)
    # An ambient file (no top-level import/export), so its `declare module` blocks are global.
    out = []
    for pkg, names in sorted(mods.items()):
        out.append(f'declare module "{pkg}" {{')
        out += [f'  export const {n}: import("react").ComponentType<Record<string, unknown>>;' for n in sorted(names)]
        out.append("}")
    out.append('declare module "*.css";')
    return "\n".join(out) + "\n", sorted(mods)


def write(path, text):
    with open(path, "w", encoding="utf-8", newline="") as fh:
        fh.write(text)


def make_project(root, files, ambient):
    write(os.path.join(root, "package.json"), json.dumps({
        "name": "e10-tsx-compila", "private": True, "version": "0.0.0",
        "devDependencies": {"react": "^18.3.1", "@types/react": "^18.3.12", "typescript": "^5.6.3"}}, indent=2))
    write(os.path.join(root, "tsconfig.json"), json.dumps({
        "compilerOptions": {"target": "ES2020", "module": "ESNext", "moduleResolution": "bundler", "jsx": "react-jsx",
                            "strict": True, "noEmit": True, "skipLibCheck": True, "esModuleInterop": True, "noUnusedLocals": False},
        "include": ["src"]}, indent=2))
    os.makedirs(os.path.join(root, "src"), exist_ok=True)
    for name, text in files.items():
        write(os.path.join(root, "src", name), text)
    write(os.path.join(root, "src", "fake-modules.d.ts"), ambient)


def npm_install(root):
    npm = "npm.cmd" if os.name == "nt" else "npm"
    i = subprocess.run([npm, "install", "--no-audit", "--no-fund", "--loglevel=error"], cwd=root, capture_output=True, text=True, encoding="utf-8")
    if i.returncode != 0:
        raise SystemExit("npm install failed:\n" + i.stdout[-1500:] + "\n" + i.stderr[-1500:])


def tsc(root):
    npx = "npx.cmd" if os.name == "nt" else "npx"
    p = subprocess.run([npx, "--no-install", "tsc", "--noEmit", "--pretty", "false"], cwd=root, capture_output=True, text=True, encoding="utf-8")
    return p.returncode, p.stdout


def split_css(tsx):
    """export.tsx's code holds the TSX and, after a `/* name.css */` line, the stylesheet the TSX
    imports. Returns (tsx part, [(css file name, css text)])."""
    marks = list(re.finditer(r"^/\* ([^*\n]+\.css) \*/\n", tsx, re.M))
    if not marks:
        return tsx, []
    out = tsx[:marks[0].start()].rstrip() + "\n"
    parts = []
    for i, m in enumerate(marks):
        end = marks[i + 1].start() if i + 1 < len(marks) else len(tsx)
        parts.append((m.group(1), tsx[m.end():end]))
    return out, parts


ERR = re.compile(r"^(?P<file>[^()\n]+)\((?P<line>\d+),(?P<col>\d+)\): error (?P<code>TS\d+): (?P<msg>.*)$", re.M)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--cli", default=os.environ.get("FIGCRAFT_CLI", "figcraft-cli"))
    ap.add_argument("--input")
    ap.add_argument("--ids")
    ap.add_argument("--map")
    ap.add_argument("--out")
    ap.add_argument("--keep", action="store_true")
    ap.add_argument("--quote", action="store_true", help="the report quotes the source line of each error")
    a = ap.parse_args()

    work = tempfile.mkdtemp(prefix="e10-")
    try:
        if a.input:
            ids = [int(x) for x in (a.ids or "").split(",") if x.strip()]
            if not ids:
                raise SystemExit("--input needs --ids")
            res = run_cli(a.cli, real_script(ids, a.map), a.input, work)
            source = "real"
        else:
            res = run_cli(a.cli, synthetic_script(), None, work)
            source = "synthetic"
        files = {f["name"]: f["text"] for f in res["files"]}
        tsx_name = next(n for n in files if n.endswith(".tsx"))
        tsx = files[tsx_name]
        # The TSX imports its own stylesheet by the component's name (`./pdp-mobile.css`); the
        # export writes the tokens to tokens.css. Both are in the project; a missing import file
        # is an error the report must show, so nothing is renamed here.
        ambient, pkgs = fake_declarations(tsx)
        proj = os.path.join(work, "app")
        os.makedirs(proj)
        make_project(proj, files, ambient)
        npm_install(proj)

        def check(label, name, text, extra):
            """tsc on one variant of the file; `extra` are more files for src/."""
            src = os.path.join(proj, "src")
            for f in os.listdir(src):
                if f.endswith((".tsx", ".css")):
                    os.remove(os.path.join(src, f))
            write(os.path.join(src, name), text)
            for n, t in extra.items():
                write(os.path.join(src, n), t)
            code, out = tsc(proj)
            lines = text.splitlines()
            errors = []
            for m in ERR.finditer(out):
                e = {"file": m.group("file").replace("\\", "/"), "line": int(m.group("line")), "col": int(m.group("col")),
                     "code": m.group("code"), "message": m.group("msg")}
                if (a.quote or source == "synthetic") and e["file"].endswith(name):
                    e["source"] = lines[e["line"] - 1].strip() if 0 < e["line"] <= len(lines) else ""
                errors.append(e)
            return {"variant": label, "compiles": code == 0, "tscExit": code, "errors": errors,
                    "tscOutput": out.strip() if not errors and code else None}

        # 1. the file exactly as export.code returns it (plus tokens.css).
        variants = [check("as exported", tsx_name, tsx, {k: v for k, v in files.items() if k.endswith(".css")})]
        # 2. the TSX part alone, the stylesheet split off into its own file (what a developer must do).
        tsx_only, css_parts = split_css(tsx)
        variants.append(check("css split off", tsx_name, tsx_only, {**{k: v for k, v in files.items() if k.endswith(".css")}, **dict(css_parts)}))
        report = {"source": source, "file": tsx_name, "compiles": variants[0]["compiles"], "packages": pkgs,
                  "cssInsideTsx": [n for n, _ in css_parts],
                  "imports": re.findall(r'^import .*$', tsx, re.M) if source == "synthetic" else None,
                  "variants": variants,
                  "tokensUndefined": [t["token"] for t in res.get("tokens", []) if not t.get("defined")],
                  "exportWarnings": len(res.get("warnings", []))}
        text = json.dumps(report, ensure_ascii=False, indent=1)
        if a.out:
            write(a.out, text + "\n")
        print(text)
        if a.keep:
            print(f"project kept at {work}", file=sys.stderr)
        return 0 if report["compiles"] else 1
    finally:
        if not a.keep:
            shutil.rmtree(work, ignore_errors=True)


if __name__ == "__main__":
    sys.exit(main())
