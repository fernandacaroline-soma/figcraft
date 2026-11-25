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
    let tf = idv(&ex(s, "frame.create", json!({"x": 0, "y": 800, "width": 80, "height": 24, "name": "Tag"})));
    ex(s, "component.create", json!({"ids": [tf]}));
}

fn screen(name: &str) -> Value {
    json!({"name": name, "type": "incremental", "screen": {"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Bg", "children": [
        {"text": "Vestido midi", "size": "Font/Size/fs-xs", "color": "Colors/Text"},
        {"component": "Tag"},
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
    let state = f.iter().find(|x| x["area"] == "estados").expect(&text);
    assert!(state["scenario"].as_str().unwrap().contains("Desabilitado=On"), "{state}");
    // The NV grey (#bbbbbb on white, about 1.9:1) fails AA with the colour the brand's token resolves to.
    let low = f.iter().find(|x| x["scenario"].as_str().unwrap().contains("contraste")).expect(&text);
    assert_eq!(low["severity"], json!("alta"));
    // The 80 × 24 Tag is under 44 pt; the 160 × 48 Button is not.
    let small: Vec<&Value> = f.iter().filter(|x| x["scenario"].as_str().unwrap().contains(" pt (")).collect();
    assert_eq!(small.len(), 1, "{text}");
    assert!(small[0]["scenario"].as_str().unwrap().starts_with("Tag"));

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
