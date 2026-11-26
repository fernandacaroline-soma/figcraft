//! Frente N: a proposal's screen described by its components, variants, texts and tokens.

use serde_json::{Value, json};

use crate::components::tests::{ex, idv};
use crate::{NodeId, Session};
use figcraft_doc::NodeKind;

struct Lib {
    primary: NodeId,
}

/// "Tokens APP" with a colour, a spacing and a font size; a "Button" set (`✍️ Tipo` =
/// Primário | Secundário); a "Tag" component with a `✍️ Texto` text property; a "Sheet" component
/// with a nested Tag named "🖊️ Rótulo" and a "Conteúdo" slot holding a placeholder.
fn library(s: &mut Session) -> Lib {
    ex(s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["NV"]}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Surface", "type": "color", "value": "#f4f1ec"}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Spacing/sp-xs", "type": "number", "value": 8}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Font/Size/fs-xs", "type": "number", "value": 16}));
    let variants: Vec<NodeId> = ["Primário", "Secundário"]
        .iter()
        .enumerate()
        .map(|(k, t)| {
            let f = idv(&ex(s, "frame.create", json!({"x": k as f64 * 150.0, "y": 600, "width": 100, "height": 40, "name": format!("✍️ Tipo={t}")})));
            idv(&ex(s, "component.create", json!({"ids": [f]})))
        })
        .collect();
    let set = idv(&ex(s, "component.combineVariants", json!({"ids": variants.clone()})));
    ex(s, "node.set", json!({"id": set, "props": {"name": "Button"}}));

    let tf = idv(&ex(s, "frame.create", json!({"x": 0, "y": 800, "width": 80, "height": 24, "name": "Tag"})));
    let label = idv(&ex(s, "text.create", json!({"text": "Tag", "x": 4, "y": 4, "parent": tf, "props": {"name": "Label"}})));
    let tag = idv(&ex(s, "component.create", json!({"ids": [tf]})));
    ex(s, "component.addProperty", json!({"id": tag, "name": "✍️ Texto", "type": "text", "layer": label}));
    // A boolean of the same short name, as DS Apps has ("✍️ Subtítulo" and "👁️ Subtítulo").
    ex(s, "component.addProperty", json!({"id": label, "name": "👁️ Texto", "type": "boolean", "default": true}));

    let sf = idv(&ex(s, "frame.create", json!({"x": 0, "y": 900, "width": 375, "height": 200, "name": "Sheet"})));
    let inner = idv(&ex(s, "instance.create", json!({"component": tag, "x": 10, "y": 10, "parent": sf})));
    ex(s, "node.set", json!({"id": inner, "props": {"name": "🖊️ Rótulo"}}));
    let content = idv(&ex(s, "frame.create", json!({"x": 10, "y": 50, "width": 355, "height": 120, "parent": sf, "name": "Conteúdo"})));
    ex(s, "shape.rectangle", json!({"x": 0, "y": 0, "width": 50, "height": 20, "parent": content, "props": {"name": "Placeholder"}}));
    ex(s, "component.create", json!({"ids": [sf]}));
    ex(s, "component.makeSlot", json!({"id": content}));
    Lib { primary: variants[0] }
}

fn screen() -> Value {
    json!({"frame": "PDP", "width": 375, "height": 812, "fill": "Tokens APP/Colors/Surface", "gap": "Spacing/sp-xs", "children": [
        {"image": "Galeria", "height": 300, "fill": "Colors/Surface"},
        {"text": "Vestido midi", "size": "Font/Size/fs-xs", "color": "Colors/Surface"},
        {"component": "Tag", "props": {"Texto": "Fiel ao tamanho", "texto ": true}},
        {"component": "Button", "variant": {"Tipo": "Primário"}, "width": "fill", "name": "Adicionar"},
        {"component": "Sheet", "dock": "bottom", "layers": {"Rótulo": {"texto": "Escolha o tamanho"}}, "slot": [
            {"frame": "Tamanhos", "layout": "horizontal", "gap": "Spacing/sp-xs", "children": [
                {"component": "Tag", "props": {"Texto": "P"}}, {"component": "Tag", "props": {"Texto": "M"}}
            ]}
        ]}
    ]})
}

fn board(s: &mut Session, screens: Vec<Value>) -> Result<Value, crate::EngineError> {
    let proposals: Vec<Value> =
        screens.into_iter().enumerate().map(|(i, sc)| json!({"name": format!("P{i}"), "type": "incremental", "screen": sc})).collect();
    s.execute("explore.board", &json!({"title": "Rodada", "proposals": proposals}))
}

fn find(s: &Session, root: NodeId, name: &str) -> Option<NodeId> {
    let doc = s.doc().ok()?;
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        let n = doc.node(id)?;
        if n.name == name {
            return Some(id);
        }
        stack.extend(n.children.iter().copied());
    }
    None
}

fn text_prop(s: &Session, id: NodeId) -> Option<String> {
    match &s.doc().ok()?.node(id)?.kind {
        NodeKind::Instance { props, .. } => {
            props.iter().find(|(k, _)| crate::cmd::eureka_screen::short(k) == "texto").and_then(|(_, v)| v.as_str()).map(str::to_string)
        }
        _ => None,
    }
}

#[test]
fn a_described_screen_is_drawn_with_the_librarys_components_and_tokens() {
    let mut s = Session::new();
    let lib = library(&mut s);
    let r = board(&mut s, vec![screen(), json!({"frame": "Vazia", "width": 375, "height": 812})]).unwrap();
    let root = crate::params::id_of(&r["proposals"][0]["id"]).unwrap();
    let doc = s.doc().unwrap();

    // The button is the asked variant, filling the width.
    let button = find(&s, root, "Adicionar").unwrap();
    assert!(matches!(doc.node(button).unwrap().kind, NodeKind::Instance { component, .. } if component == lib.primary));
    // Texts go by the property's short name, at the top and in a nested instance.
    let tags: Vec<String> = doc.node(root).unwrap().children.iter().filter_map(|c| text_prop(&s, *c)).collect();
    assert_eq!(tags, vec!["Fiel ao tamanho".to_string()]);
    let sheet = find(&s, root, "Sheet").unwrap();
    assert_eq!(text_prop(&s, find(&s, sheet, "🖊️ Rótulo").unwrap()).as_deref(), Some("Escolha o tamanho"));
    // The slot holds the proposal's content instead of its placeholder.
    let sizes = find(&s, sheet, "Tamanhos").unwrap();
    assert_eq!(doc.node(sizes).unwrap().children.len(), 2);
    // (an instance's inherited layer is hidden, not deleted, and the reviewer leaves hidden layers out)
    assert!(find(&s, sheet, "Placeholder").is_none_or(|ph| !doc.node(ph).unwrap().visible));
    // Docked to the bottom of the screen, out of the flow.
    let (sh, root_h) = (doc.node(sheet).unwrap(), doc.node(root).unwrap().height);
    let y = sh.transform.as_coeffs()[5];
    assert!((y + sh.height - root_h).abs() < 0.5, "sheet at {y} + {} in {root_h}", sh.height);
    // Colours and sizes are bound, not copied: the lint finds no fixed value of the proposal.
    let lint = ex(&mut s, "ds.lint", json!({"ids": [root]}));
    let raw: Vec<&Value> = lint["findings"].as_array().unwrap().iter().filter(|f| f["rule"] == "valor-solto").collect();
    assert!(raw.is_empty(), "{raw:?}");
    // The proposal records what it used, for the designer and the reviewer.
    let info: Value = serde_json::from_str(s.doc().unwrap().node(root).unwrap().plugin_data.get("eureka.proposal").unwrap()).unwrap();
    assert_eq!(info["ds"]["components"], json!(["Tag", "Button", "Sheet"]));
    assert!(info["ds"]["variables"].as_array().unwrap().iter().any(|v| v == "Spacing/sp-xs"));
}

#[test]
fn a_misspelt_name_refuses_the_board_with_what_exists_and_draws_nothing() {
    let mut s = Session::new();
    library(&mut s);
    let before = s.doc().unwrap().clone();
    let ok = json!({"frame": "Ok", "width": 375, "height": 812});
    for (bad, says) in [
        (json!({"component": "Botão"}), "no component or component set named `Botão`"),
        (json!({"component": "Button", "variant": {"Tipo": "Terciário"}}), "Primário, Secundário"),
        (json!({"component": "Button", "variant": {"Tamanho": "G"}}), "it has: tipo"),
        (json!({"component": "Tag", "props": {"Titulo": "x"}}), "it has: texto"),
        (json!({"component": "Tag", "props": {"Texto": 3}}), "true or false"),
        (json!({"component": "Tag", "slot": []}), "has no slot"),
        (json!({"component": "Sheet", "layers": {"Ícone": {"Texto": "x"}}}), "has no layer `Ícone`"),
        (json!({"text": "x", "color": "Colors/Inexistente"}), "no such variable"),
    ] {
        let sc = json!({"frame": "PDP", "width": 375, "height": 812, "children": [bad]});
        let err = board(&mut s, vec![ok.clone(), sc]).unwrap_err().to_string();
        assert!(err.contains("proposal B") && err.contains(says), "{err}");
        assert_eq!(*s.doc().unwrap(), before, "nothing drawn");
    }
}

#[test]
fn the_catalog_lists_components_by_the_names_a_description_uses() {
    let mut s = Session::new();
    library(&mut s);
    let c = ex(&mut s, "explore.catalog", json!({}));
    let list = c["components"].as_array().unwrap();
    let button = list.iter().find(|x| x["name"] == "Button").unwrap();
    assert_eq!(button["variants"]["tipo"], json!(["Primário", "Secundário"]));
    let tag = list.iter().find(|x| x["name"] == "Tag").unwrap();
    assert_eq!(tag["props"]["texto"], json!({"text": "Tag", "boolean": true}));
    assert_eq!(list.iter().find(|x| x["name"] == "Sheet").unwrap()["slots"], 1);
    assert!(c["variables"].as_array().unwrap().iter().any(|v| v == "Tokens APP/Spacing/sp-xs"));
    let only = ex(&mut s, "explore.catalog", json!({"query": "butt"}));
    assert_eq!(only["components"].as_array().unwrap().len(), 1);
}

#[test]
fn a_token_name_in_two_collections_means_the_official_one() {
    let mut s = Session::new();
    library(&mut s);
    // A library's own copy of the tokens, with the same names.
    ex(&mut s, "variable.createCollection", json!({"name": "Tokens", "modes": ["Default"]}));
    ex(&mut s, "variable.create", json!({"collection": "Tokens", "name": "Colors/Surface", "type": "color", "value": "#000000"}));
    let out = board(&mut s, vec![json!({"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Surface", "children": []})]).unwrap();
    let screen = idv(&out["proposals"][0]);
    let doc = s.doc().unwrap();
    let bound = doc.node(screen).unwrap().bound.get("fills/0/color").cloned().unwrap();
    let var = doc.variables.get_by_id(&bound).unwrap();
    assert_eq!(doc.variables.collection(&var.collection).unwrap().name, "Tokens APP");
    // A name only the other collection has still resolves there; an unknown one is still refused.
    assert!(board(&mut s, vec![json!({"frame": "PDP", "width": 375, "height": 812, "fill": "Colors/Nope", "children": []})]).is_err());
}
