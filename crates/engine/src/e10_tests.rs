//! E10, part A, the engine tests the usability script asked for (points 1, 2): editing a library
//! component's instance by hand keeps it an instance, and a board's whole Eureka state survives
//! save and reopen. Synthetic data only. Automated tests are not a usability validation.

use serde_json::{Value, json};

use crate::cmd::eureka::{LIBRARY_KEY, ORIGIN_KEY, library_of};
use crate::cmd::eureka_brand::{BRAND_KEY, brand_of};
use crate::cmd::eureka_critique::CRITIQUE_KEY;
use crate::cmd::eureka_explore::{PROPOSAL_KEY, REVIEW_KEY};
use crate::components::tests::{ex, idv, node};
use crate::{NodeId, Session};
use figcraft_doc::NodeKind;

/// A "Button" set (State=Default / State=Hover, each with a "Label" text bound to a text property),
/// marked as coming from a library, and one instance of it on a screen.
struct Kit {
    set: NodeId,
    default: NodeId,
    hover: NodeId,
    inst: NodeId,
}

fn kit(s: &mut Session) -> Kit {
    ex(s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["NV"]}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"}));
    let mut variants = Vec::new();
    for (k, state) in ["Default", "Hover"].into_iter().enumerate() {
        let f = idv(&ex(s, "frame.create", json!({"x": k as f64 * 200.0, "y": 0, "width": 120, "height": 40, "name": format!("State={state}")})));
        ex(s, "text.create", json!({"text": "Comprar", "x": 8, "y": 8, "parent": f, "props": {"name": "Label"}}));
        variants.push(idv(&ex(s, "component.create", json!({"ids": [f]}))));
    }
    let set = idv(&ex(s, "component.combineVariants", json!({"ids": variants.clone()})));
    ex(s, "component.addProperty", json!({"id": set, "name": "Rotulo", "type": "text", "default": "Comprar"}));
    for v in &variants {
        let label = node(s, *v).children[0];
        ex(s, "component.bindProperty", json!({"layer": label, "property": "Rotulo"}));
    }
    // What the library import records on the set.
    ex(s, "node.set", json!({"id": set, "props": {"pluginData": {LIBRARY_KEY: "{\"name\":\"DS Teste\",\"file\":\"abc\"}"}}}));
    let screen = idv(&ex(s, "frame.create", json!({"x": 0, "y": 300, "width": 375, "height": 812, "name": "Tela"})));
    let inst = idv(&ex(s, "instance.create", json!({"component": variants[0], "x": 10, "y": 10, "parent": screen})));
    Kit { set, default: variants[0], hover: variants[1], inst }
}

fn variable_count(s: &Session) -> (usize, usize) {
    let v = &s.doc().unwrap().variables;
    (v.collections.len(), v.variables.len())
}

/// What "still an official instance" means: an instance, of a variant of the same set, whose
/// library is still known (the set carries `eureka.library`). `Err` says what broke.
fn still_official(s: &Session, k: &Kit) -> Result<NodeId, String> {
    let doc = s.doc().map_err(|e| e.to_string())?;
    let n = doc.node(k.inst).ok_or("the instance is gone")?;
    let NodeKind::Instance { component, .. } = &n.kind else { return Err("no longer an instance".into()) };
    let parent = doc.node(*component).and_then(|c| c.parent);
    if parent != Some(k.set) {
        return Err(format!("component {component} is not in the original set"));
    }
    if library_of(doc, *component).is_null() {
        return Err("library marker lost".into());
    }
    Ok(*component)
}

fn text_of(s: &Session, inst: NodeId) -> String {
    let doc = s.doc().unwrap();
    let label = doc.node(inst).unwrap().children[0];
    doc.node(label).unwrap().kind.text().map(|t| t.characters.clone()).unwrap_or_default()
}

fn by_of(s: &mut Session, id: NodeId) -> Value {
    ex(s, "origin.get", json!({"id": id.0}))["aspects"].clone()
}

#[test]
fn hand_editing_a_library_instance_keeps_it_official_with_no_new_token_and_human_origin() {
    let mut s = Session::new();
    let k = kit(&mut s);
    ex(&mut s, "origin.track", json!({"on": true}));
    let tokens = variable_count(&s);
    assert_eq!(still_official(&s, &k), Ok(k.default));

    // The same commands the component panel runs, with no agent: the designer.
    ex(&mut s, "instance.setProperties", json!({"ids": [k.inst.0], "properties": {"Rotulo": "Comprar agora"}}));
    ex(&mut s, "instance.setProperties", json!({"ids": [k.inst.0], "properties": {"State": "Hover"}}));

    assert_eq!(still_official(&s, &k), Ok(k.hover), "same set, other variant");
    assert_eq!(text_of(&s, k.inst), "Comprar agora");
    assert_eq!(variable_count(&s), tokens, "no token or collection was created");
    // The main components are untouched: the edit is the instance's.
    assert_eq!(text_of_main(&s, k.default), "Comprar");
    // Origin: every aspect the edit stamped on the instance and its label is the designer's.
    let doc = s.doc().unwrap();
    let mut stamped = 0;
    for id in doc.subtree(k.inst) {
        let o = by_of(&mut s, id);
        for (aspect, v) in o.as_object().into_iter().flatten() {
            assert_eq!(v["by"], json!("human"), "{id} {aspect}: {v}");
            stamped += 1;
        }
    }
    assert!(stamped > 0, "the edit left a human origin to read");
    assert!(ex(&mut s, "origin.list", json!({"by": "ai"})).as_array().unwrap().is_empty(), "nothing is credited to an agent");
    let human = ex(&mut s, "origin.list", json!({"by": "human"}));
    assert!(human.as_array().unwrap().iter().any(|l| l["id"] == json!(k.inst.0) || l["name"] == json!("Label")), "{human}");
}

fn text_of_main(s: &Session, main: NodeId) -> String {
    text_of(s, main)
}

/// The proof the test above can fail: a detach (what the regression would be) is caught by the
/// same check, and so is a text edit that never reaches the instance's own origin as human.
#[test]
fn the_check_catches_a_detached_instance() {
    let mut s = Session::new();
    let k = kit(&mut s);
    assert_eq!(still_official(&s, &k), Ok(k.default));
    ex(&mut s, "component.detach", json!({"ids": [k.inst.0]}));
    let err = still_official(&s, &k).unwrap_err();
    assert!(err.contains("no longer an instance") || err.contains("gone"), "{err}");
}

/// The proof for a swap to a component outside the set (also not "the same instance").
#[test]
fn the_check_catches_a_swap_to_another_component() {
    let mut s = Session::new();
    let k = kit(&mut s);
    let f = idv(&ex(&mut s, "frame.create", json!({"x": 0, "y": 900, "width": 50, "height": 20, "name": "Other"})));
    ex(&mut s, "text.create", json!({"text": "x", "x": 1, "y": 1, "parent": f}));
    let other = idv(&ex(&mut s, "component.create", json!({"ids": [f]})));
    ex(&mut s, "instance.swap", json!({"ids": [k.inst.0], "component": other.0}));
    assert!(still_official(&s, &k).unwrap_err().contains("not in the original set"));
}

// ---------------------------------------------------------------------------------------------
// Point 2: save, reopen, nothing lost.
// ---------------------------------------------------------------------------------------------

fn plugin_keys(s: &Session, id: NodeId, keys: &[&str]) -> Vec<Option<String>> {
    let n = s.doc().unwrap().node(id).unwrap();
    keys.iter().map(|k| n.plugin_data.get(*k).cloned()).collect()
}

fn mode_names(s: &Session, id: NodeId) -> Vec<(String, String)> {
    let doc = s.doc().unwrap();
    let mut out: Vec<(String, String)> = doc
        .node(id)
        .unwrap()
        .var_modes
        .iter()
        .filter_map(|(cid, mid)| {
            let c = doc.variables.collection_by_id(cid)?;
            Some((c.name.clone(), c.modes.iter().find(|m| &m.id == mid)?.name.clone()))
        })
        .collect();
    out.sort();
    out
}

fn screen(name: &str) -> Value {
    json!({"name": name, "type": "incremental", "screen": {"frame": "PDP", "width": 100, "height": 100, "fill": "Colors/Bg",
        "children": [{"type": "text", "text": "Comprar", "x": 8, "y": 8}]}})
}

#[test]
fn proposal_critique_review_brand_and_origin_survive_save_and_reopen() {
    const KEYS: [&str; 5] = [PROPOSAL_KEY, CRITIQUE_KEY, REVIEW_KEY, BRAND_KEY, ORIGIN_KEY];
    let mut s = Session::new();
    ex(&mut s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["NV", "Farm"]}));
    ex(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"}));
    ex(&mut s, "origin.track", json!({"on": true}));
    // The agent draws the board; the designer decides (no agent).
    let board = s
        .execute_as(
            Some("Claude"),
            "explore.board",
            &json!({"title": "Rodada", "brand": "NV", "platform": "app", "journey": "PDP", "proposals": [screen("Um"), screen("Dois")]}),
        )
        .unwrap();
    let (a, b) = (idv(&board["proposals"][0]), idv(&board["proposals"][1]));
    ex(&mut s, "explore.choose", json!({"id": a.0}));
    ex(&mut s, "explore.critique", json!({"id": a.0}));
    ex(&mut s, "explore.critiqueReviewed", json!({"id": a.0}));
    ex(
        &mut s,
        "explore.review",
        json!({"id": a.0, "verdict": "com-ressalvas", "reviewer": "Revisor Teste", "summary": "Ajustar o texto.",
        "items": [{"message": "Texto curto demais.", "source": "regra de teste"}]}),
    );
    ex(&mut s, "brand.apply", json!({"ids": [a.0, b.0], "brand": "NV"}));
    // A manual edit after all that, on the text of the chosen screen.
    let text = node(&s, a).children[0];
    ex(&mut s, "node.set", json!({"id": text.0, "props": {"opacity": 0.5}}));
    ex(&mut s, "text.set", json!({"id": text.0, "characters": "Comprar agora"}));

    // Before: every key is there, the agent's and the designer's origin differ, the brand is on.
    let layers = [a, b, text];
    for id in [a, b] {
        assert!(plugin_keys(&s, id, &KEYS[..1])[0].is_some() && plugin_keys(&s, id, &KEYS[3..4])[0].is_some());
        assert_eq!(mode_names(&s, id), vec![("Tokens APP".to_string(), "NV".to_string())]);
    }
    let chosen = plugin_keys(&s, a, &KEYS);
    assert!(chosen.iter().all(Option::is_some), "proposal, critique, review, brand and origin on the chosen one: {chosen:?}");
    assert_eq!(by_of(&mut s, text)["characters"]["by"], json!("human"));
    assert_eq!(by_of(&mut s, text)["opacity"]["by"], json!("human"));
    assert_eq!(by_of(&mut s, a)["name"]["by"], json!("ai"), "the agent's frame stays the agent's");
    let before: Vec<_> = layers.iter().map(|l| (plugin_keys(&s, *l, &KEYS), mode_names(&s, *l), by_of(&mut s, *l))).collect();
    let brand_before = brand_of(s.doc().unwrap(), a);

    // Save, close, reopen in a fresh session.
    let dir = std::env::temp_dir().join(format!("figcraft-e10-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("rodada.figcraft").to_string_lossy().to_string();
    ex(&mut s, "file.saveAs", json!({"path": path}));
    let mut t = Session::empty();
    ex(&mut t, "file.open", json!({"path": path}));
    std::fs::remove_dir_all(&dir).ok();

    let after: Vec<_> = layers.iter().map(|l| (plugin_keys(&t, *l, &KEYS), mode_names(&t, *l), by_of(&mut t, *l))).collect();
    for (i, (x, y)) in before.iter().zip(&after).enumerate() {
        assert_eq!(x.0, y.0, "layer {i}: plugin data (proposal, critique, review, brand, origin)");
        assert_eq!(x.1, y.1, "layer {i}: brand mode");
        assert_eq!(x.2, y.2, "layer {i}: origin per aspect");
    }
    assert_eq!(brand_of(t.doc().unwrap(), a), brand_before);
    assert_eq!(by_of(&mut t, text)["characters"]["by"], json!("human"));
    // And the state still works after reopening: the review is still the chosen one's verdict.
    let kept: Value = serde_json::from_str(t.doc().unwrap().node(a).unwrap().plugin_data[REVIEW_KEY].as_str()).unwrap();
    assert_eq!(kept["verdict"], json!("com-ressalvas"));
}
