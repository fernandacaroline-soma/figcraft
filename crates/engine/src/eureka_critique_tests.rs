//! Frente O: the design critique of the chosen proposal, between the designer's pick and the DS
//! Gatekeeper.

use serde_json::{Value, json};

use crate::components::tests::{ex, idv};
use crate::{NodeId, Session};

/// "Tokens APP" (Farm, NV): a white background and a content colour that is black on Farm and light
/// grey on NV; a "Button" set with `Desabilitado` = Off | On; a 80 × 24 "Tag" component.
fn library(s: &mut Session) {
    ex(s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["Farm", "NV"]}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"}));
    ex(
        s,
        "variable.create",
        json!({"collection": "Tokens APP", "name": "Colors/Text", "type": "color", "values": {"Farm": "#000000", "NV": "#bbbbbb"}}),
    );
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Font/Size/fs-xs", "type": "number", "value": 16}));
    let variants: Vec<NodeId> = ["Off", "On"]
        .iter()
        .enumerate()
        .map(|(k, v)| {
            let f = idv(&ex(
                s,
                "frame.create",
                json!({"x": k as f64 * 150.0, "y": 600, "width": 160, "height": 48, "name": format!("Tipo=Primário, Desabilitado={v}")}),
            ));
            idv(&ex(s, "component.create", json!({"ids": [f]})))
        })
        .collect();
    let set = idv(&ex(s, "component.combineVariants", json!({"ids": variants})));
    ex(s, "node.set", json!({"id": set, "props": {"name": "Button"}}));
    // An icon set: "error-circle" is an icon's name, not an error state of the screen.
    let icons: Vec<NodeId> = ["close", "error-circle"]
        .iter()
        .enumerate()
        .map(|(k, v)| {
            let f =
                idv(&ex(s, "frame.create", json!({"x": 400.0 + k as f64 * 40.0, "y": 600, "width": 48, "height": 48, "name": format!("Icon={v}")})));
            idv(&ex(s, "component.create", json!({"ids": [f]})))
        })
        .collect();
    let set = idv(&ex(s, "component.combineVariants", json!({"ids": icons})));
    ex(s, "node.set", json!({"id": set, "props": {"name": "Icon"}}));
    // A message whose "Status" is a kind (Brand) or a state (Erro): only Erro is a state to show.
    let notes: Vec<NodeId> = ["Brand", "Erro"]
        .iter()
        .enumerate()
        .map(|(k, v)| {
            let f = idv(&ex(
                s,
                "frame.create",
                json!({"x": 600.0 + k as f64 * 200.0, "y": 600, "width": 160, "height": 48, "name": format!("Status={v}")}),
            ));
            idv(&ex(s, "component.create", json!({"ids": [f]})))
        })
        .collect();
    let set = idv(&ex(s, "component.combineVariants", json!({"ids": notes})));
    ex(s, "node.set", json!({"id": set, "props": {"name": "Aviso"}}));
    let tf = idv(&ex(s, "frame.create", json!({"x": 0, "y": 800, "width": 80, "height": 24, "name": "Tag"})));
    ex(s, "component.create", json!({"ids": [tf]}));
}

fn screen(name: &str) -> Value {
    json!({"name": name, "type": "incremental", "screen": {"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Bg", "children": [
        {"text": "Vestido midi", "size": "Font/Size/fs-xs", "color": "Colors/Text"},
        {"component": "Tag"},
        {"component": "Tag"},
        {"component": "Icon", "variant": {"Icon": "close"}},
        {"component": "Aviso", "variant": {"Status": "Brand"}},
        {"component": "Button", "variant": {"Tipo": "Primário", "Desabilitado": "Off"}}
    ]}})
}

/// A board of two proposals on the brand `mode`: their screens' ids.
fn board(s: &mut Session, mode: &str) -> (NodeId, NodeId) {
    library(s);
    let r = ex(s, "explore.board", json!({"title": "Rodada", "proposals": [screen("Um"), screen("Dois")]}));
    let (a, b) = (idv(&r["proposals"][0]), idv(&r["proposals"][1]));
    ex(s, "variable.setMode", json!({"ids": [a.0, b.0], "collection": "Tokens APP", "mode": mode}));
    (a, b)
}

fn areas(findings: &Value) -> Vec<String> {
    findings.as_array().unwrap().iter().map(|f| f["area"].as_str().unwrap().to_string()).collect()
}

#[test]
fn the_eureka_measures_missing_states_contrast_in_the_brand_and_small_touch_targets() {
    let mut s = Session::new();
    let (a, _) = board(&mut s, "NV");
    let m = ex(&mut s, "explore.measure", json!({"id": a.0}));
    let f = m["findings"].as_array().unwrap();
    let text = serde_json::to_string(f).unwrap();
    assert!(f.iter().all(|x| x["evidence"].as_str().unwrap().starts_with("Eureka (medido)")), "{text}");
    // The Button's disabled state is in the DS but not on the screen.
    let states: Vec<&str> = f.iter().filter(|x| x["area"] == "estados").map(|x| x["scenario"].as_str().unwrap()).collect();
    assert_eq!(states.len(), 2, "the Button's and the message's, not the icon named error-circle: {text}");
    assert!(states.iter().any(|s| s.contains("não mostra Desabilitado=On")), "{text}");
    // Status: only Erro is asked for; Brand, shown, is a kind of message.
    assert!(states.iter().any(|s| s.ends_with("não mostra Status=Erro.")), "{text}");
    // The NV grey (#bbbbbb on white, about 1.9:1) fails AA with the colour the brand's token resolves to.
    let low = f.iter().find(|x| x["scenario"].as_str().unwrap().contains("contraste")).expect(&text);
    assert_eq!(low["severity"], json!("alta"));
    // The two 80 × 24 Tags are under 44 pt, in one finding; the 160 × 48 Button and the 48 × 48 icon
    // are not.
    let small: Vec<&Value> = f.iter().filter(|x| x["scenario"].as_str().unwrap().contains(" pt")).collect();
    assert_eq!(small.len(), 1, "{text}");
    assert!(small[0]["scenario"].as_str().unwrap().starts_with("Tag mede 80 × 24 pt (2 na tela)"), "{text}");

    // On Farm the same token is black: no contrast finding.
    let mut s = Session::new();
    let (a, _) = board(&mut s, "Farm");
    let m = ex(&mut s, "explore.measure", json!({"id": a.0}));
    assert!(!serde_json::to_string(&m).unwrap().contains("contraste"), "{m}");
}

#[test]
fn the_critique_comes_between_the_choice_and_the_gatekeeper() {
    let mut s = Session::new();
    let (a, b) = board(&mut s, "NV");
    let finding = json!({"area": "falhas", "scenario": "Sem estoque do tamanho escolhido.", "consequence": "A cliente só descobre na sacola.",
        "severity": "crítica", "recommendation": "Avisar no seletor.", "ids": [a.0]});
    // Only the chosen proposal is critiqued.
    assert!(s.execute("explore.critique", &json!({"id": a.0, "findings": [finding]})).is_err());
    ex(&mut s, "explore.choose", json!({"id": a.0}));
    assert!(s.execute("explore.critique", &json!({"id": b.0})).is_err(), "not the chosen one");
    // A wrong area or severity is refused with the ones that exist.
    let mut bad = finding.clone();
    bad["area"] = json!("visual");
    let e = s.execute("explore.critique", &json!({"id": a.0, "findings": [bad]})).unwrap_err().to_string();
    assert!(e.contains("casos de borda"), "{e}");
    let mut bad = finding.clone();
    bad["severity"] = json!("urgente");
    assert!(s.execute("explore.critique", &json!({"id": a.0, "findings": [bad]})).is_err());

    let r =
        ex(&mut s, "explore.critique", json!({"id": a.0, "reviewer": "Claude", "findings": [finding], "open": ["Mostrar o estoque por tamanho?"]}));
    let c = &r["critique"];
    assert_eq!(c["reviewed"], json!(false));
    assert_eq!(c["advisory"], json!(true));
    // Most severe first, numbered; the reviewer's finding is "inferência" when it gives no source.
    let f = c["findings"].as_array().unwrap();
    assert_eq!(f[0]["severity"], json!("crítica"));
    assert_eq!(f[0]["evidence"], json!("inferência"));
    assert_eq!(f[0]["n"], json!(1));
    assert!(areas(&c["findings"]).contains(&"estados".to_string()), "measured findings are added");
    assert!(f.iter().all(|x| x["status"] == "aberto"));

    // The Gatekeeper waits for the designer's review of the critique.
    assert!(s.execute("explore.review", &json!({"id": a.0, "verdict": "aprovado"})).is_err());
    ex(&mut s, "explore.critiqueStatus", json!({"id": a.0, "finding": 1, "status": "decisão da designer"}));
    assert!(s.execute("explore.critiqueStatus", &json!({"id": a.0, "finding": 1, "status": "feito"})).is_err());
    assert!(s.execute("explore.critiqueStatus", &json!({"id": a.0, "finding": 999, "status": "resolvido"})).is_err());
    ex(&mut s, "explore.critiqueReviewed", json!({"id": a.0}));
    ex(&mut s, "explore.review", json!({"id": a.0, "verdict": "com-ressalvas"}));
    let kept = |s: &Session, id: NodeId, key: &str| s.doc().unwrap().node(id).unwrap().plugin_data.get(key).cloned();
    let c: Value = serde_json::from_str(&kept(&s, a, "eureka.critique").unwrap()).unwrap();
    assert_eq!(c["findings"][0]["status"], json!("decisão da designer"));
    assert_eq!(c["reviewed"], json!(true));

    // Undo takes the verdict back, then the review of the critique.
    ex(&mut s, "edit.undo", json!({}));
    assert!(kept(&s, a, "eureka.review").is_none());
    ex(&mut s, "edit.undo", json!({}));
    assert!(!crate::cmd::eureka_critique::is_reviewed(s.doc().unwrap(), a));
    ex(&mut s, "edit.redo", json!({}));
    ex(&mut s, "edit.redo", json!({}));

    // A new critique replaces the old one and the verdict given on it.
    ex(&mut s, "explore.critique", json!({"id": a.0}));
    assert!(kept(&s, a, "eureka.review").is_none());
    assert!(!crate::cmd::eureka_critique::is_reviewed(s.doc().unwrap(), a));
    // Choosing another proposal drops the critique: it was of the other choice.
    ex(&mut s, "explore.choose", json!({"id": b.0}));
    assert!(kept(&s, a, "eureka.critique").is_none());
}

#[test]
fn contrast_is_measured_in_every_brand_when_asked() {
    // The screen is on Farm (black text, fine); the same token is light grey on NV.
    let mut s = Session::new();
    let (a, _) = board(&mut s, "Farm");
    let own = ex(&mut s, "explore.measure", json!({"id": a.0}));
    assert!(!serde_json::to_string(&own).unwrap().contains("contraste"), "the screen's brand passes: {own}");
    let every = ex(&mut s, "explore.measure", json!({"id": a.0, "allBrands": true}));
    let f: Vec<&Value> = every["findings"].as_array().unwrap().iter().filter(|x| x["scenario"].as_str().unwrap().contains("contraste")).collect();
    assert_eq!(f.len(), 1, "{every}");
    let scenario = f[0]["scenario"].as_str().unwrap();
    assert!(scenario.contains("em 1 de 2 marca(s): NV ("), "{scenario}");
    assert!(!scenario.contains("Farm"), "{scenario}");
    assert_eq!(f[0]["severity"], json!("alta"));
}

#[test]
fn a_disabled_text_and_a_text_over_an_unmeasured_background_are_not_flagged() {
    let mut s = Session::new();
    ex(&mut s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["NV"]}));
    ex(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"}));
    ex(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Pale", "type": "color", "value": "#dddddd"}));
    ex(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/White", "type": "color", "value": "#ffffff"}));
    // A size selector whose disabled variant has a pale label.
    let variants: Vec<NodeId> = ["Off", "On"]
        .iter()
        .enumerate()
        .map(|(k, v)| {
            let f = idv(&ex(
                &mut s,
                "frame.create",
                json!({"x": k as f64 * 100.0, "y": 600, "width": 48, "height": 48, "name": format!("Desabilitado={v}")}),
            ));
            let t = idv(&ex(&mut s, "text.create", json!({"text": "38", "x": 8, "y": 8, "parent": f.0})));
            ex(&mut s, "variable.bind", json!({"ids": [t.0], "field": "fill", "variable": "Colors/Pale"}));
            idv(&ex(&mut s, "component.create", json!({"ids": [f]})))
        })
        .collect();
    let set = idv(&ex(&mut s, "component.combineVariants", json!({"ids": variants})));
    ex(&mut s, "node.set", json!({"id": set, "props": {"name": "Tamanho"}}));
    let screen = |c: Value| json!({"name": "Tela", "type": "incremental", "screen": {"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Bg", "children": [c]}});
    let r = ex(
        &mut s,
        "explore.board",
        json!({"title": "R", "proposals": [
            screen(json!({"component": "Tamanho", "variant": {"Desabilitado": "On"}})),
            screen(json!({"text": "Sobre a foto", "color": "Colors/White"}))
        ]}),
    );
    for p in r["proposals"].as_array().unwrap() {
        let m = ex(&mut s, "explore.measure", json!({"id": p["id"], "allBrands": true}));
        assert!(!serde_json::to_string(&m).unwrap().contains("contraste"), "{m}");
    }
    // The same pale label in the enabled variant is measured.
    let r = ex(
        &mut s,
        "explore.board",
        json!({"title": "R2", "proposals": [
            screen(json!({"component": "Tamanho", "variant": {"Desabilitado": "Off"}})),
            screen(json!({"component": "Tamanho", "variant": {"Desabilitado": "Off"}}))
        ]}),
    );
    let m = ex(&mut s, "explore.measure", json!({"id": r["proposals"][0]["id"]}));
    assert!(serde_json::to_string(&m).unwrap().contains("contraste"), "{m}");
}

#[test]
fn a_low_contrast_text_inside_a_ds_component_names_it() {
    let mut s = Session::new();
    ex(&mut s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["NV"]}));
    ex(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"}));
    ex(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Pale", "type": "color", "value": "#dddddd"}));
    let f = idv(&ex(&mut s, "frame.create", json!({"x": 0, "y": 600, "width": 120, "height": 40, "name": "Preço"})));
    let t = idv(&ex(&mut s, "text.create", json!({"text": "10x sem juros", "x": 4, "y": 4, "parent": f.0})));
    ex(&mut s, "variable.bind", json!({"ids": [t.0], "field": "fill", "variable": "Colors/Pale"}));
    ex(&mut s, "component.create", json!({"ids": [f]}));
    let screen = json!({"name": "Tela", "type": "incremental", "screen": {"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Bg", "children": [{"component": "Preço"}]}});
    let r = ex(&mut s, "explore.board", json!({"title": "R", "proposals": [screen.clone(), screen]}));
    for every in [false, true] {
        let m = ex(&mut s, "explore.measure", json!({"id": r["proposals"][0]["id"], "allBrands": every}));
        let text = serde_json::to_string(&m).unwrap();
        assert!(text.contains("Vem do componente Preço do DS."), "{text}");
    }
}
