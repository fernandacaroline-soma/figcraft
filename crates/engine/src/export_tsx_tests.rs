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
