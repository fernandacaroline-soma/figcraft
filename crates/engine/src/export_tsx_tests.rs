//! `export.tsx` (Eureka Design, Frente J): TSX with the design system's tokens by name.

use serde_json::{Value, json};

use crate::codegen::ds_var_name;
use crate::{NodeId, Session, params::id_of};

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command}: {e}"))
}

/// A card in auto layout under a screen set to the "Animale (CERTO)" mode, its fill, gap and
/// padding bound to tokens: (session, screen, card, the price text with a loose 18px size).
fn screen() -> (Session, NodeId, NodeId, NodeId) {
    let mut s = Session::new();
    run(&mut s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["Farm", "Animale (CERTO)"]}));
    let var = |s: &mut Session, name: &str, ty: &str, value: Value| {
        run(s, "variable.create", json!({"collection": "Tokens APP", "name": name, "type": ty, "value": value}))["id"].as_str().unwrap().to_string()
    };
    let bg = var(&mut s, "Colors/Surface/Brand/Background", "color", json!("#0D0D0D"));
    let md = var(&mut s, "Spacing/sp-md", "float", json!(16));
    let ink = var(&mut s, "Colors/Content/Brand/On Brand", "color", json!("#FFFFFF"));
    let screen = id_of(&run(&mut s, "node.create", json!({"type": "frame", "name": "PDP Mobile", "width": 375, "height": 600}))["id"]).unwrap();
    let card =
        id_of(&run(&mut s, "node.create", json!({"type": "frame", "name": "Product Card", "width": 300, "height": 200, "parent": screen.0}))["id"])
            .unwrap();
    run(&mut s, "node.set", json!({"id": card.0, "props": {"layout": {"mode": "vertical", "gap": 16, "padding": [16, 16, 16, 16]}}}));
    let price = id_of(&run(&mut s, "text.create", json!({"text": "R$ 349,90", "x": 0, "y": 0, "parent": card.0}))["id"]).unwrap();
    run(&mut s, "node.set", json!({"id": price.0, "props": {"fontSize": 18}}));
    run(&mut s, "variable.bind", json!({"ids": [card.0], "field": "fill", "variable": bg}));
    for f in ["gap", "paddingTop", "paddingRight", "paddingBottom", "paddingLeft"] {
        run(&mut s, "variable.bind", json!({"ids": [card.0], "field": f, "variable": md}));
    }
    run(&mut s, "variable.bind", json!({"ids": [price.0], "field": "fill", "variable": ink}));
    run(&mut s, "variable.setMode", json!({"ids": [screen.0], "collection": "Tokens APP", "mode": "Animale (CERTO)"}));
    (s, screen, card, price)
}

#[test]
fn token_names_become_css_variables_by_one_rule() {
    assert_eq!(ds_var_name("Colors/Surface/Brand/Background"), "--colors-surface-brand-background");
    assert_eq!(ds_var_name("Spacing/sp-md"), "--spacing-sp-md");
    assert_eq!(ds_var_name("Colors/Content/Brand/On Brand"), "--colors-content-brand-on-brand");
    assert_eq!(ds_var_name("Ação / Botão 2"), "--acao-botao-2");
    assert_eq!(ds_var_name("  Font//Size  fs-xs  "), "--font-size-fs-xs");
    assert_eq!(ds_var_name("Border/Radius/Components/br-button"), "--border-radius-components-br-button");
    assert_eq!(ds_var_name("///"), "--token");
    assert!(ds_var_name(&"x".repeat(500)).len() <= 82);
}

#[test]
fn bound_values_are_tokens_and_loose_ones_are_warnings() {
    let (mut s, screen, _card, price) = screen();
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    let code = r["code"].as_str().unwrap();
    assert!(code.contains("var(--colors-surface-brand-background)"), "{code}");
    assert!(code.contains("var(--spacing-sp-md)"), "{code}");
    assert!(code.contains("var(--colors-content-brand-on-brand)"), "{code}");
    assert!(!code.contains("var(--colors-surface-brand-background,"), "no fallback values: {code}");
    assert!(!code.contains(":root"), "the values come from the design system: {code}");
    assert!(!code.contains("#0D0D0D") && !code.contains("rgb(13"), "no resolved token value: {code}");
    let w = r["warnings"].as_array().unwrap();
    assert!(w.iter().any(|w| w["id"] == json!(price.0) && w["field"] == "fontSize" && w["value"] == "18px"), "{r}");
    assert!(w.iter().all(|w| w["field"] != "fill" || w["id"] != json!(price.0)), "{r}");
    assert!(code.starts_with("/*") && code.contains("fontSize: 18px"), "the warnings head the file: {code}");
    assert!(code.contains("export function PDPMobile"), "{code}");
}

#[test]
fn the_brand_mode_is_data_brand_on_the_root() {
    let (mut s, screen, card, _) = screen();
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    assert_eq!(r["brand"]["brand"], "animale", "{r}");
    assert_eq!(r["brand"]["mode"], "Animale (CERTO)");
    let code = r["code"].as_str().unwrap();
    assert!(code.contains("data-brand=\"animale\""), "{code}");
    // A layer inside the screen takes the mode from above it.
    let inner = run(&mut s, "export.tsx", json!({"ids": [card.0]}));
    assert!(inner["code"].as_str().unwrap().contains("data-brand=\"animale\""));
    // No mode set anywhere: no attribute, and the header says so.
    let mut t = Session::new();
    let f = id_of(&run(&mut t, "node.create", json!({"type": "frame", "name": "Plain", "width": 10, "height": 10}))["id"]).unwrap();
    let plain = run(&mut t, "export.tsx", json!({"ids": [f.0]}));
    assert!(!plain["code"].as_str().unwrap().contains("data-brand"));
    assert_eq!(plain["brand"], Value::Null);
}

#[test]
fn export_defaults_to_the_selection_and_refuses_nothing_or_junk() {
    let (mut s, screen, ..) = screen();
    run(&mut s, "select.set", json!({"ids": [screen.0]}));
    assert!(run(&mut s, "export.tsx", json!({}))["code"].as_str().unwrap().contains("PDPMobile"));
    run(&mut s, "select.none", json!({}));
    assert!(s.execute("export.tsx", &json!({})).is_err());
    assert!(s.execute("export.tsx", &json!({"ids": [999_999]})).is_err());
    // inspect.code is unchanged: fallback values and a :root rule.
    let react = run(&mut s, "inspect.code", json!({"id": screen.0, "language": "react"}));
    let react = react["code"].as_str().unwrap();
    assert!(react.contains("var(--spacing-sp-md, 16px)"), "{react}");
}

#[test]
fn a_radius_token_that_is_zero_in_this_brand_is_still_written() {
    let (mut s, screen, card, _) = screen();
    let r =
        run(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Border/Radius/Components/br-card", "type": "float", "value": 0}));
    let br = r["id"].as_str().unwrap().to_string();
    run(&mut s, "variable.bind", json!({"ids": [card.0], "field": "cornerRadius", "variable": br}));
    let code = run(&mut s, "export.tsx", json!({"ids": [screen.0]}))["code"].as_str().unwrap().to_string();
    assert!(code.contains("border-radius: var(--border-radius-components-br-card)"), "{code}");
    // inspect.code still leaves a 0 radius out.
    let react = run(&mut s, "inspect.code", json!({"id": screen.0, "language": "react"}));
    assert!(!react["code"].as_str().unwrap().contains("border-radius"), "{react}");
}

/// Nine brands, as many as the app's token collection: synthetic names and values.
const BRANDS: [&str; 9] = ["Farm", "Animale (CERTO)", "NV", "Fábula", "Marca 5", "Marca 6", "Marca 7", "Marca 8", "Marca 9"];

fn nine_brands() -> Session {
    let mut s = Session::new();
    run(&mut s, "variable.createCollection", json!({"name": "Tokens APP", "modes": BRANDS}));
    let colors: serde_json::Map<String, Value> =
        BRANDS.iter().enumerate().map(|(i, b)| ((*b).to_string(), json!(format!("#0000{:02X}", i * 16)))).collect();
    run(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Surface/Brand/Background", "type": "color", "values": colors}));
    let radii: serde_json::Map<String, Value> = BRANDS.iter().enumerate().map(|(i, b)| ((*b).to_string(), json!(i))).collect();
    run(
        &mut s,
        "variable.create",
        json!({"collection": "Tokens APP", "name": "Border/Radius/Components/br-card", "type": "number", "values": radii}),
    );
    run(
        &mut s,
        "variable.create",
        json!({"collection": "Tokens APP", "name": "Colors/Button/Primary", "type": "color", "value": {"alias": "Colors/Surface/Brand/Background"}}),
    );
    s
}

#[test]
fn the_token_stylesheet_has_every_brand_of_the_collection() {
    let mut s = nine_brands();
    let r = run(&mut s, "export.tokensCss", json!({}));
    let css = r["css"].as_str().unwrap();
    for b in ["farm", "animale", "nv", "fabula", "marca-5", "marca-9"] {
        assert!(css.contains(&format!("[data-brand=\"{b}\"] {{")), "{b}: {css}");
    }
    assert_eq!(r["brands"].as_array().unwrap().len(), 9, "{r}");
    assert_eq!(r["brands"][2], json!({"collection": "Tokens APP", "mode": "NV", "brand": "nv"}));
    // The default mode is also :root, so a page without data-brand still reads the tokens.
    assert!(css.contains(":root,\n[data-brand=\"farm\"] {"), "{css}");
    // Names by the export.tsx rule; numbers in px; aliases by name; the value per brand.
    assert!(css.contains("--colors-surface-brand-background: #000020;"), "{css}");
    assert!(css.contains("--border-radius-components-br-card: 8px;"), "{css}");
    assert!(css.contains("--colors-button-primary: var(--colors-surface-brand-background);"), "{css}");
    assert_eq!(r["warnings"], json!([]));
    // Read only: nothing to undo, the variables unchanged.
    assert_eq!(run(&mut s, "variable.list", json!({})), run(&mut nine_brands(), "variable.list", json!({})));
}

#[test]
fn clashing_css_names_are_warnings() {
    let mut s = nine_brands();
    run(
        &mut s,
        "variable.create",
        json!({"collection": "Tokens APP", "name": "Colors Surface Brand Background", "type": "color", "value": "#FF0000"}),
    );
    run(&mut s, "variable.addMode", json!({"collection": "Tokens APP", "name": "NV (nova)"}));
    let w = run(&mut s, "export.tokensCss", json!({"collections": ["Tokens APP"]}))["warnings"].to_string();
    assert!(w.contains("--colors-surface-brand-background"), "{w}");
    assert!(w.contains("data-brand=\\\"nv\\\""), "{w}");
    assert!(s.execute("export.tokensCss", &json!({"collections": ["Nope"]})).is_err());
    assert!(s.execute("export.tokensCss", &json!({"collections": "Tokens APP"})).is_err());
}

#[test]
fn the_font_is_a_token_or_a_warning() {
    let (mut s, screen, _card, price) = screen();
    // The price's family and weight have no variable: two warnings, like the lint's loose values.
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    let w = &r["warnings"];
    for field in ["fontFamily", "fontWeight"] {
        assert!(w.as_array().unwrap().iter().any(|w| w["id"] == json!(price.0) && w["field"] == field), "{field}: {r}");
    }
    assert!(r["code"].as_str().unwrap().contains("fontWeight: 400"), "{r}");
    // Bound, they are tokens and the warnings go.
    let fam = run(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Font/Family/Primary", "type": "string", "value": "Inter"}))
        ["id"]
        .as_str()
        .unwrap()
        .to_string();
    let wt = run(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Font/Weight/Regular", "type": "number", "value": 400}))["id"]
        .as_str()
        .unwrap()
        .to_string();
    run(&mut s, "variable.bind", json!({"ids": [price.0], "field": "fontFamily", "variable": fam}));
    run(&mut s, "variable.bind", json!({"ids": [price.0], "field": "fontWeight", "variable": wt}));
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    let code = r["code"].as_str().unwrap();
    assert!(code.contains("font-family: var(--font-family-primary);"), "{code}");
    assert!(code.contains("font-weight: var(--font-weight-regular);"), "{code}");
    assert!(r["warnings"].as_array().unwrap().iter().all(|w| w["field"] != "fontFamily" && w["field"] != "fontWeight"), "{r}");
    // inspect.code keeps the weight a number.
    let react = run(&mut s, "inspect.code", json!({"id": screen.0, "language": "react"}));
    assert!(react["code"].as_str().unwrap().contains("font-weight: 400;"), "{react}");
}
