//! The way back from `export.tsx` (Eureka Design, task 7): calls to code components read as
//! instances of the design system's components, through the code map. Synthetic names only.

use serde_json::{Value, json};

use figcraft_doc::NodeKind;

use crate::{NodeId, Session, params::id_of};

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command}: {e}"))
}

fn id(v: &Value) -> NodeId {
    id_of(&v["id"]).unwrap()
}

fn children(s: &mut Session, of: NodeId) -> Vec<NodeId> {
    s.doc().unwrap().node(of).unwrap().children.clone()
}

fn ds(s: &mut Session, id: NodeId) {
    run(s, "node.set", json!({"id": id.0, "props": {"pluginData": {"eureka.library": "{\"name\":\"DS Teste\"}"}}}));
}

/// A design system with a "Button" set (Hierarquia: Primário, Secundário), two icons and a
/// "Snackbar" (a message bound to "Texto", an icon swap "Ícone", an action Button shown by
/// "Mostrar ação" and a "Detalhe" text no property drives), all linked through the code map; and
/// a screen using a Secundário Button and a Snackbar changed inside (the action's variant and the
/// detail's text). Returns (session, screen, card).
fn file() -> (Session, NodeId, NodeId) {
    let mut s = Session::new();
    let mut variants = Vec::new();
    for (k, h) in ["Primário", "Secundário"].iter().enumerate() {
        let f = id(&run(
            &mut s,
            "frame.create",
            json!({"x": 800.0 + k as f64 * 150.0, "y": 0, "width": 120, "height": 40, "name": format!("Hierarquia={h}")}),
        ));
        variants.push(id(&run(&mut s, "component.create", json!({"ids": [f.0]}))).0);
    }
    let button = id(&run(&mut s, "component.combineVariants", json!({"ids": variants})));
    run(&mut s, "node.set", json!({"id": button.0, "props": {"name": "Button"}}));
    ds(&mut s, button);
    let mut icons = Vec::new();
    for (k, n) in ["Icone Check", "Icone Fechar"].iter().enumerate() {
        let f = id(&run(&mut s, "frame.create", json!({"x": 800.0 + k as f64 * 40.0, "y": 100, "width": 24, "height": 24, "name": n})));
        let c = id(&run(&mut s, "component.create", json!({"ids": [f.0]})));
        ds(&mut s, c);
        icons.push(c);
    }
    let bar = id(&run(&mut s, "frame.create", json!({"x": 800, "y": 200, "width": 320, "height": 48, "name": "Snackbar"})));
    let msg = id(&run(&mut s, "text.create", json!({"text": "Mensagem", "x": 40, "y": 8, "parent": bar.0, "props": {"name": "Mensagem"}})));
    run(&mut s, "text.create", json!({"text": "Detalhe", "x": 40, "y": 28, "parent": bar.0, "props": {"name": "Detalhe"}}));
    let icon = id(&run(&mut s, "instance.create", json!({"component": icons[0].0, "x": 8, "y": 12, "parent": bar.0})));
    let action = id(&run(&mut s, "instance.create", json!({"component": button.0, "x": 190, "y": 4, "parent": bar.0})));
    run(&mut s, "node.set", json!({"id": action.0, "props": {"name": "Ação"}}));
    let bar = id(&run(&mut s, "component.create", json!({"ids": [bar.0]})));
    run(&mut s, "component.addProperty", json!({"id": bar.0, "name": "Texto", "type": "text", "layer": msg.0}));
    run(&mut s, "component.addProperty", json!({"id": bar.0, "name": "Ícone", "type": "instanceSwap", "default": icons[0].0, "layer": icon.0}));
    run(&mut s, "component.addProperty", json!({"id": bar.0, "name": "Mostrar ação", "type": "boolean", "layer": action.0}));
    ds(&mut s, bar);
    let map = json!({"version": 1, "links": [
        {"component": "Button", "language": "react", "importPath": "@teste/ds", "name": "Button", "template": "<Button variant={Hierarquia} />"},
        {"component": "Icone Check", "language": "react", "importPath": "@teste/ds", "name": "IconeCheck", "template": "<IconeCheck />"},
        {"component": "Icone Fechar", "language": "react", "importPath": "@teste/ds", "name": "IconeFechar", "template": "<IconeFechar />"},
        {"component": "Snackbar", "language": "react", "importPath": "@teste/ds", "name": "Snackbar",
         "template": "<Snackbar texto={Texto} icone={icone} acao=\"{Mostrar ação ? \"sim\" : \"nao\"}\" />"}
    ]});
    assert_eq!(run(&mut s, "codeConnect.import", json!({"json": map}))["linked"], 4);

    let screen = id(&run(&mut s, "frame.create", json!({"x": 0, "y": 0, "width": 375, "height": 600, "name": "Tela"})));
    let card = id(&run(&mut s, "node.create", json!({"type": "frame", "name": "Card", "width": 340, "height": 300, "parent": screen.0})));
    run(&mut s, "node.set", json!({"id": card.0, "props": {"layout": {"mode": "vertical", "gap": 8}}}));
    let b = id(&run(&mut s, "instance.create", json!({"component": button.0, "x": 0, "y": 0, "parent": card.0})));
    run(&mut s, "instance.setProperties", json!({"ids": [b.0], "properties": {"Hierarquia": "Secundário"}}));
    let sb = id(&run(&mut s, "instance.create", json!({"component": bar.0, "x": 0, "y": 0, "parent": card.0})));
    run(
        &mut s,
        "instance.setProperties",
        json!({"ids": [sb.0], "properties": {"Texto": "Pedido enviado", "Ícone": icons[1].0, "Mostrar ação": true}}),
    );
    let inner = children(&mut s, sb);
    let named = |s: &mut Session, n: &str| *inner.iter().find(|i| s.doc().unwrap().node(**i).unwrap().name == n).unwrap();
    let (detail, act) = (named(&mut s, "Detalhe"), named(&mut s, "Ação"));
    run(&mut s, "text.set", json!({"id": detail.0, "characters": "Chega amanhã"}));
    run(&mut s, "instance.setProperties", json!({"ids": [act.0], "properties": {"Hierarquia": "Secundário"}}));
    (s, screen, card)
}

/// The instances under `of` (not inside other instances): (main component's name, properties).
fn instances(s: &mut Session, of: NodeId) -> Vec<(String, Value)> {
    let doc = s.doc().unwrap();
    let mut out = Vec::new();
    for c in &doc.node(of).unwrap().children {
        let n = doc.node(*c).unwrap();
        if let NodeKind::Instance { component, props } = &n.kind {
            out.push((doc.node(*component).unwrap().name.clone(), json!(props)));
        }
    }
    out
}

/// The text of the layer called `name` inside instance number `k` under `of`.
fn inner_text(s: &mut Session, of: NodeId, k: usize, name: &str) -> String {
    let inst = children(s, of)[k];
    let doc = s.doc().unwrap();
    let layer = doc.node(inst).unwrap().children.iter().map(|i| doc.node(*i).unwrap()).find(|n| n.name == name).unwrap();
    layer.kind.text().unwrap().characters.clone()
}

/// The variant of the instance layer called `name` inside instance number `k` under `of`.
fn inner_variant(s: &mut Session, of: NodeId, k: usize, name: &str) -> String {
    let inst = children(s, of)[k];
    let doc = s.doc().unwrap();
    let layer = doc.node(inst).unwrap().children.iter().map(|i| doc.node(*i).unwrap()).find(|n| n.name == name).unwrap();
    let NodeKind::Instance { component, .. } = &layer.kind else { panic!("{name} is not an instance") };
    doc.node(*component).unwrap().name.clone()
}

#[test]
fn a_screens_design_system_calls_come_back_as_the_same_instances_and_variants() {
    let (mut s, screen, card) = file();
    let out = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    let code = out["code"].as_str().unwrap().to_string();
    assert!(code.contains("<Button variant=\"Secundário\" />"), "{code}");
    assert!(code.contains("<Snackbar texto=\"Pedido enviado\" icone={<IconeFechar />} acao=\"sim\" />"), "{code}");

    let read = run(&mut s, "codeConnect.readTsx", json!({"code": code, "components": out["components"]}));
    let calls = read["calls"].as_array().unwrap();
    assert_eq!(calls.len(), 2, "{read}");
    assert_eq!(read["counts"]["notInMap"], 0, "{read}");
    assert_eq!(read["notes"], json!([]), "{read}");
    assert_eq!(calls[0]["component"], "Button");
    assert_eq!(calls[0]["properties"], json!({"Hierarquia": "Secundário"}), "{read}");
    assert_eq!(calls[0]["importPath"], "@teste/ds");
    assert_eq!(calls[1]["component"], "Snackbar");
    assert_eq!(calls[1]["properties"]["Texto"], "Pedido enviado", "{read}");
    assert_eq!(calls[1]["properties"]["Mostrar ação"], true, "{read}");
    let inside = calls[1]["inside"].to_string();
    assert!(inside.contains("Chega amanhã") && inside.contains("Hierarquia=Secundário"), "{read}");

    // Placed back into a new frame: the same instances, variants and properties as the screen's.
    let back = id(&run(&mut s, "frame.create", json!({"x": 0, "y": 800, "width": 375, "height": 600, "name": "Volta"})));
    for c in calls {
        let r =
            run(&mut s, "codeConnect.placeCall", json!({"call": c["call"], "importPath": c["importPath"], "inside": c["inside"], "parent": back.0}));
        assert!(r["id"].is_u64(), "{r}");
        assert_eq!(r["notes"], json!([]), "{r}");
    }
    assert_eq!(instances(&mut s, back), instances(&mut s, card), "the same components, variants and properties");
    assert_eq!(instances(&mut s, back)[0].0, "Hierarquia=Secundário");
    // What the Snackbar changed inside it is back too.
    assert_eq!(inner_text(&mut s, back, 1, "Detalhe"), "Chega amanhã");
    assert_eq!(inner_variant(&mut s, back, 1, "Ação"), "Hierarquia=Secundário");
    assert_eq!(inner_text(&mut s, back, 1, "Mensagem"), "Pedido enviado");
    // And it exports to the same calls.
    let again = run(&mut s, "export.tsx", json!({"ids": [back.0]}));
    let again = again["code"].as_str().unwrap();
    for call in calls {
        assert!(again.contains(call["call"].as_str().unwrap()), "{again}");
    }
    // Without `components`, the header's notes give what changed inside (one Snackbar call).
    let from_header = run(&mut s, "codeConnect.readTsx", json!({"code": code}));
    assert_eq!(from_header["calls"][1]["inside"], calls[1]["inside"], "{from_header}");
}

#[test]
fn a_prop_the_component_does_not_have_is_a_note_and_code_names_are_read() {
    let (mut s, _, _) = file();
    let code = "import { Button, Snackbar } from \"@teste/ds\";\nexport function T() {\n  return (\n    <div>\n      <Button variant=\"secundario\" tamanho=\"Grande\" />\n      <Snackbar texto=\"Oi\" icone={<IconeCheck />} mostrarAcao={false} />\n    </div>\n  );\n}\n";
    let r = run(&mut s, "codeConnect.readTsx", json!({"code": code}));
    let calls = r["calls"].as_array().unwrap();
    // The variant written as code writes it is the variant; the unknown prop is a note, not a value.
    assert_eq!(calls[0]["properties"], json!({"Hierarquia": "Secundário"}), "{r}");
    let notes = calls[0]["notes"].to_string();
    assert!(notes.contains("unknown prop tamanho=Grande"), "{r}");
    assert!(r["notes"].to_string().contains("unknown prop tamanho"), "{r}");
    // `mostrarAcao` isn't in the template: read by the property's name in code ("Mostrar ação").
    assert_eq!(calls[1]["properties"]["Mostrar ação"], false, "{r}");
    assert_eq!(calls[1]["notes"], json!([]), "{r}");
    // Placed: the instance gets what was read, and the note.
    let p = run(&mut s, "codeConnect.placeCall", json!({"call": calls[0]["call"]}));
    assert!(p["notes"].to_string().contains("unknown prop tamanho"), "{p}");
    let doc = s.doc().unwrap();
    let NodeKind::Instance { component, .. } = &doc.node(id(&p)).unwrap().kind else { panic!() };
    assert_eq!(doc.node(*component).unwrap().name, "Hierarquia=Secundário");
}

#[test]
fn a_call_with_no_map_entry_is_a_note_and_nothing_is_made() {
    let (mut s, _, _) = file();
    let code = "import { Chip } from \"@teste/ds\";\n<div>\n  <Chip label=\"Novo\" />\n  <Button variant=\"Primário\" />\n</div>\n";
    let r = run(&mut s, "codeConnect.readTsx", json!({"code": code}));
    assert_eq!(r["counts"], json!({"calls": 2, "read": 1, "notInMap": 1, "notes": 1}), "{r}");
    assert!(r["calls"][0]["componentId"].is_null(), "{r}");
    assert!(r["notes"][0].as_str().unwrap().contains("<Chip>: no component of the document is linked to Chip"), "{r}");
    let before = s.doc().unwrap().nodes().count();
    let undo = s.ds().unwrap().history.undo.len();
    let p = run(&mut s, "codeConnect.placeCall", json!({"call": "<Chip label=\"Novo\" />"}));
    assert!(p["id"].is_null(), "{p}");
    assert!(p["notes"][0].as_str().unwrap().contains("Chip"), "{p}");
    assert_eq!(s.doc().unwrap().nodes().count(), before, "no component made up, no layers drawn");
    assert_eq!(s.ds().unwrap().history.undo.len(), undo);
    // A placed call is one undo step.
    run(&mut s, "codeConnect.placeCall", json!({"call": "<Button variant=\"Secundário\" />"}));
    assert_eq!(s.ds().unwrap().history.undo.len(), undo + 1);
    run(&mut s, "edit.undo", json!({}));
    assert_eq!(s.doc().unwrap().nodes().count(), before);
    // Junk is refused, never a panic.
    for junk in [json!({}), json!({"code": 4}), json!({"code": "<A", "components": 3}), json!({"path": ""})] {
        assert!(s.execute("codeConnect.readTsx", &junk).is_err(), "{junk}");
    }
    for junk in [json!({}), json!({"call": "nada"}), json!({"call": "<div />"}), json!({"call": "<A />", "inside": [1]})] {
        assert!(s.execute("codeConnect.placeCall", &junk).is_err(), "{junk}");
    }
}
