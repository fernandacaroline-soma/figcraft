//! `export.codeMap` (Eureka Design, task 3): the design system's components against a code map.

use serde_json::{Value, json};

use crate::{NodeId, Session, params::id_of};

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command}: {e}"))
}

fn id(v: &Value) -> NodeId {
    id_of(&v["id"]).unwrap()
}

/// A file with two design-system components (a "Button" set: Hierarquia × Tamanho, and a "Chip")
/// marked as imported from a library, and a local "Card": synthetic names.
fn file() -> (Session, NodeId) {
    let mut s = Session::new();
    let mut variants = Vec::new();
    for (k, (h, t)) in [("Primário", "Grande"), ("Secundário", "Grande"), ("Primário", "Pequeno")].iter().enumerate() {
        let f = id(&run(
            &mut s,
            "frame.create",
            json!({"x": k as f64 * 150.0, "y": 0, "width": 120, "height": 40, "name": format!("Hierarquia={h}, Tamanho={t}")}),
        ));
        variants.push(id(&run(&mut s, "component.create", json!({"ids": [f.0]}))).0);
    }
    let set = id(&run(&mut s, "component.combineVariants", json!({"ids": variants})));
    run(&mut s, "node.set", json!({"id": set.0, "props": {"name": "Button", "pluginData": {"eureka.library": "{\"name\":\"DS Apps\"}"}}}));
    let chip = id(&run(&mut s, "frame.create", json!({"x": 0, "y": 200, "width": 60, "height": 24, "name": "Chip"})));
    let chip = id(&run(&mut s, "component.create", json!({"ids": [chip.0]})));
    run(&mut s, "node.set", json!({"id": chip.0, "props": {"pluginData": {"eureka.library": "{\"name\":\"DS Apps\"}"}}}));
    let card = id(&run(&mut s, "frame.create", json!({"x": 0, "y": 300, "width": 200, "height": 100, "name": "Card"})));
    run(&mut s, "component.create", json!({"ids": [card.0]}));
    (s, set)
}

fn map() -> Value {
    json!({"version": 1, "links": [
        {"component": "Button", "language": "react", "importPath": "@acme/ds-apps", "name": "Button", "template": "<Button variant={Hierarquia} label={texto} />"},
        {"component": "Selector Size", "language": "react", "importPath": "@acme/ds-apps", "name": "SelectorSize", "template": "<SelectorSize />"},
        {"component": "Chip", "language": "react", "importPath": "@acme/ds-apps", "name": "1Chip", "template": "<Chip />"},
        {"component": "Button", "language": "swiftui", "name": "DSButton", "template": "DSButton()"},
        {"component": "Button", "language": "cobol", "name": "X", "template": "X"}
    ]})
}

#[test]
fn the_map_is_checked_against_the_design_systems_components() {
    let (mut s, set) = file();
    let before = s.doc().unwrap().node(set).unwrap().code_links.clone();
    let r = run(&mut s, "export.codeMap", json!({"json": map()}));
    // Button: mapped, with a placeholder that isn't a property and a property the code never gets.
    let b = r["mapped"].as_array().unwrap().iter().find(|m| m["component"] == "Button").unwrap().clone();
    assert_eq!(b["name"], "Button");
    assert_eq!(b["importPath"], "@acme/ds-apps");
    assert_eq!(b["library"]["name"], "DS Apps", "{r}");
    assert_eq!(b["unknown"], json!(["texto"]), "{r}");
    assert_eq!(b["unmapped"], json!(["Tamanho"]), "{r}");
    // Chip: an entry that would not link (its code name starts with a digit), so it is invalid and
    // still missing; the local Card is not the design system's.
    assert!(r["invalid"].as_array().unwrap().iter().any(|e| e["component"] == "Chip" && e["error"].as_str().unwrap().starts_with("name")), "{r}");
    assert!(r["invalid"].as_array().unwrap().iter().any(|e| e["error"].as_str().unwrap().starts_with("language")), "{r}");
    assert_eq!(r["notInDocument"], json!(["Selector Size"]), "{r}");
    assert!(r["missing"].as_array().unwrap().iter().all(|m| m["component"] != "Card"), "{r}");
    assert_eq!(r["counts"]["designSystem"], 2, "{r}");
    assert_eq!(r["counts"]["mapped"], 1, "{r}");
    // Read only: nothing linked.
    assert_eq!(s.doc().unwrap().node(set).unwrap().code_links, before);
    // Another language reads its own entries.
    let swift = run(&mut s, "export.codeMap", json!({"json": map(), "language": "swiftui"}));
    assert_eq!(swift["mapped"][0]["name"], "DSButton", "{swift}");
    assert_eq!(swift["mapped"][0]["unmapped"], json!(["Hierarquia", "Tamanho"]), "{swift}");
}

#[test]
fn a_design_system_component_with_no_entry_is_missing() {
    let (mut s, _) = file();
    let only_button = json!({"version": 1, "links": [{"component": "Button", "language": "react", "name": "Button", "template": "<Button variant={Hierarquia} size={Tamanho} />"}]});
    let r = run(&mut s, "export.codeMap", json!({"json": only_button}));
    assert_eq!(r["missing"].as_array().unwrap().iter().map(|m| m["component"].clone()).collect::<Vec<_>>(), vec![json!("Chip")], "{r}");
    assert_eq!(r["mapped"][0]["unmapped"], json!([]), "every property reaches the code: {r}");
    // With `all`, the file's own components count too.
    let r = run(&mut s, "export.codeMap", json!({"json": only_button, "all": true}));
    assert!(r["missing"].as_array().unwrap().iter().any(|m| m["component"] == "Card"), "{r}");
}

#[test]
fn the_map_comes_from_a_file_or_text_and_junk_is_refused() {
    let (mut s, _) = file();
    let d = std::env::temp_dir().join(format!("figcraft-codemap-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let path = d.join("ds-apps.connect.json");
    std::fs::write(&path, map().to_string()).unwrap();
    let from_file = run(&mut s, "export.codeMap", json!({"path": path.to_string_lossy()}));
    let from_text = run(&mut s, "export.codeMap", json!({"json": map().to_string()}));
    assert_eq!(from_file, from_text);
    let _ = std::fs::remove_dir_all(&d);
    for junk in [
        json!({}),
        json!({"json": 5}),
        json!({"json": "{not json"}),
        json!({"json": {"version": 2, "links": []}}),
        json!({"json": {"links": 3}}),
        json!({"json": map(), "language": "cobol"}),
        json!({"json": map(), "all": "yes"}),
        json!({"path": 7}),
    ] {
        assert!(s.execute("export.codeMap", &junk).is_err(), "{junk}");
    }
}
