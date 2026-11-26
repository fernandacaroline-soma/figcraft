//! `explore.exportCode` (Eureka Design, task 5): the chosen, reviewed proposal as code with its
//! summary for the developers.

use serde_json::{Value, json};

use crate::components::tests::{ex, idv};
use crate::{NodeId, Session};

/// A board of two token-only proposals on NV: their screens' ids.
fn board(s: &mut Session) -> (NodeId, NodeId) {
    ex(s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["Farm", "NV"]}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Text", "type": "color", "value": "#000000"}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Font/Size/fs-xs", "type": "number", "value": 16}));
    let screen = |name: &str| {
        json!({"name": name, "type": "disruptiva", "summary": "Tamanho antes do preço.", "screen": {"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Bg", "children": [
            {"text": "Vestido midi", "size": "Font/Size/fs-xs", "color": "Colors/Text"}
        ]}})
    };
    let r = ex(s, "explore.board", json!({"title": "Rodada NV", "proposals": [screen("Um"), screen("Dois")]}));
    let (a, b) = (idv(&r["proposals"][0]), idv(&r["proposals"][1]));
    ex(s, "variable.setMode", json!({"ids": [a.0, b.0], "collection": "Tokens APP", "mode": "NV"}));
    (a, b)
}

fn finding(severity: &str, scenario: &str) -> Value {
    json!({"area": "falhas", "scenario": scenario, "consequence": "A cliente desiste.", "severity": severity, "recommendation": "Avisar antes."})
}

#[test]
fn the_chosen_proposal_goes_out_as_code_with_its_verdict_and_open_findings() {
    let mut s = Session::new();
    let (a, b) = board(&mut s);
    // Not before the choice, the critique's review and the verdict.
    assert!(s.execute("explore.exportCode", &json!({"id": a.0})).is_err());
    ex(&mut s, "explore.choose", json!({"id": a.0}));
    assert!(s.execute("explore.exportCode", &json!({"id": a.0})).is_err(), "no verdict yet");
    ex(
        &mut s,
        "explore.critique",
        json!({"id": a.0, "reviewer": "Claude", "findings": [finding("baixa", "Sem foto do avesso."), finding("crítica", "Sem estoque do tamanho.")]}),
    );
    ex(&mut s, "explore.critiqueStatus", json!({"id": a.0, "finding": 1, "status": "resolvido"}));
    ex(&mut s, "explore.critiqueReviewed", json!({"id": a.0}));
    ex(
        &mut s,
        "explore.review",
        json!({"id": a.0, "verdict": "com-ressalvas", "reviewer": "DS Gatekeeper", "summary": "Pode seguir com ajustes.",
        "items": [{"message": "Padding sem token.", "source": "Regras DS Apps, Tokens"}]}),
    );
    assert!(s.execute("explore.exportCode", &json!({"id": b.0})).is_err(), "not the chosen one");

    let r = ex(&mut s, "explore.exportCode", json!({"id": a.0}));
    let tsx = r["files"][0]["text"].as_str().unwrap();
    assert!(tsx.starts_with("/*\n * Eureka: the proposal the designer chose"), "{tsx}");
    assert!(tsx.contains(" * Proposal A · Um (Disruptiva), board \"Rodada NV\"\n *   Tamanho antes do preço.\n"), "{tsx}");
    assert!(
        tsx.contains(" * Verdict (DS Gatekeeper): com-ressalvas · Pode seguir com ajustes.\n *   - Padding sem token. [Regras DS Apps, Tokens]\n"),
        "{tsx}"
    );
    // The resolved finding is left out; the open ones, most severe first.
    assert!(tsx.contains("open:\n"), "{tsx}");
    assert!(!tsx.contains("Sem estoque do tamanho."), "resolved: {tsx}");
    assert!(tsx.contains(" *   - [baixa] falhas: Sem foto do avesso.\n"), "{tsx}");
    // Then the TSX export.code gives, with the brand and the tokens.
    assert!(tsx.contains("data-brand=\"nv\"") && tsx.contains("var(--colors-bg)"), "{tsx}");
    assert_eq!(r["files"][1]["name"], "tokens.css");
    assert_eq!(r["review"]["verdict"], "com-ressalvas");
    assert!(r["critique"]["open"].as_array().unwrap().iter().all(|f| f["status"] != "resolvido"), "{r}");
    // Read only.
    let again = ex(&mut s, "explore.exportCode", json!({"id": a.0}));
    assert_eq!(again["files"], r["files"]);
    assert!(s.execute("explore.exportCode", &json!({"id": a.0, "dir": 3})).is_err());
    assert!(s.execute("explore.exportCode", &json!({})).is_err());
}

#[test]
fn comment_closers_in_the_reviewers_text_do_not_end_the_header() {
    let mut s = Session::new();
    let (a, _) = board(&mut s);
    ex(&mut s, "explore.choose", json!({"id": a.0}));
    ex(&mut s, "explore.critique", json!({"id": a.0, "findings": [finding("alta", "Texto */ fecha o comentário")]}));
    ex(&mut s, "explore.critiqueReviewed", json!({"id": a.0}));
    ex(&mut s, "explore.review", json!({"id": a.0, "verdict": "aprovado", "summary": "ok */ export default 1; /*"}));
    let r = ex(&mut s, "explore.exportCode", json!({"id": a.0}));
    let tsx = r["files"][0]["text"].as_str().unwrap();
    let header_end = tsx.find(" */\n").unwrap();
    assert!(!tsx[..header_end].contains("*/"), "{tsx}");
    assert!(!tsx.contains("Exported although"), "only a blocked verdict says so: {tsx}");
}

#[test]
fn a_blocked_verdict_still_exports_and_says_the_designer_decides() {
    let mut s = Session::new();
    let (a, _) = board(&mut s);
    ex(&mut s, "explore.choose", json!({"id": a.0}));
    ex(&mut s, "explore.critique", json!({"id": a.0, "findings": [finding("alta", "Contraste baixo")]}));
    ex(&mut s, "explore.critiqueReviewed", json!({"id": a.0}));
    ex(&mut s, "explore.review", json!({"id": a.0, "verdict": "bloqueado", "summary": "Token errado."}));
    let r = ex(&mut s, "explore.exportCode", json!({"id": a.0}));
    let tsx = r["files"][0]["text"].as_str().unwrap();
    let header_end = tsx.find(" */\n").unwrap();
    assert!(tsx[..header_end].contains("Exported although the verdict is \"bloqueado\""), "{tsx}");
    assert!(tsx[..header_end].contains("the designer chose this proposal and decides"), "{tsx}");
}

/// A design-system "Button" (a component an import marked as from DS Apps) on a screen.
fn ds_button(s: &mut Session) -> NodeId {
    let f = idv(&ex(s, "frame.create", json!({"x": 800, "y": 0, "width": 120, "height": 40, "name": "Button"})));
    let c = idv(&ex(s, "component.create", json!({"ids": [f.0]})));
    ex(s, "node.set", json!({"id": c.0, "props": {"pluginData": {"eureka.library": "{\"name\":\"DS Apps\"}"}}}));
    let screen = idv(&ex(s, "frame.create", json!({"x": 0, "y": 0, "width": 375, "height": 200, "name": "PDP"})));
    ex(s, "instance.create", json!({"component": c.0, "x": 0, "y": 0, "parent": screen.0}));
    screen
}

fn button_map() -> Value {
    json!({"version": 1, "links": [{"component": "Button", "language": "react", "importPath": "@acme/ds-apps", "name": "Button", "template": "<Button />"}]})
}

#[test]
fn the_code_map_is_linked_once_and_the_file_remembers_it() {
    // Before: the card says no map, and which DS components would come out drawn.
    let mut s = Session::new();
    let screen = ds_button(&mut s);
    let r = ex(&mut s, "explore.codeMap", json!({}));
    assert_eq!(r["map"], Value::Null, "{r}");
    assert_eq!((r["designSystem"].clone(), r["linked"].clone(), r["unlinked"].clone()), (json!(1), json!(0), json!(["Button"])), "{r}");
    let code = ex(&mut s, "export.tsx", json!({"ids": [screen.0]}))["code"].as_str().unwrap().to_string();
    assert!(!code.contains("import { Button }"), "{code}");

    // The picked file (the browser hands its contents): checked, linked, remembered.
    use base64::Engine as _;
    let data = base64::engine::general_purpose::STANDARD.encode(button_map().to_string());
    let r = ex(&mut s, "explore.codeMap", json!({"data": data, "name": "ds-apps.connect.json"}));
    assert_eq!(r["map"], json!({"name": "ds-apps.connect.json", "linked": 1, "missing": 0}), "{r}");
    assert_eq!((r["linked"].clone(), r["unlinked"].clone()), (json!(1), json!([])), "{r}");
    assert_eq!(r["check"]["mapped"], 1, "{r}");
    let code = ex(&mut s, "export.tsx", json!({"ids": [screen.0]}))["code"].as_str().unwrap().to_string();
    assert!(code.contains("import { Button } from \"@acme/ds-apps\";"), "the export uses it with no other step: {code}");
    assert_eq!(ex(&mut s, "explore.codeMap", json!({}))["map"]["name"], "ds-apps.connect.json");

    // One undo step takes back the links and the remembered map.
    ex(&mut s, "edit.undo", json!({}));
    let r = ex(&mut s, "explore.codeMap", json!({}));
    assert_eq!((r["map"].clone(), r["unlinked"].clone()), (Value::Null, json!(["Button"])), "{r}");
}

#[test]
fn a_bad_code_map_is_refused_and_nothing_is_kept() {
    let mut s = Session::new();
    ds_button(&mut s);
    for bad in
        [json!({"json": "not json"}), json!({"data": "%%%"}), json!({"json": {"version": 2, "links": []}}), json!({"json": 3}), json!({"path": 3})]
    {
        assert!(s.execute("explore.codeMap", &bad).is_err(), "{bad}");
    }
    assert_eq!(ex(&mut s, "explore.codeMap", json!({}))["map"], Value::Null);
}
