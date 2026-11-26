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

/// The nine brands with two text styles: "Heading/H1" with its size, line height and weight bound
/// to per-brand tokens and its family loose; "Body/Small" with nothing bound. Returns the session.
fn nine_brands_with_text_styles() -> Session {
    let mut s = nine_brands();
    let per_brand = |s: &mut Session, name: &str, f: &dyn Fn(usize) -> f64| {
        let values: serde_json::Map<String, Value> = BRANDS.iter().enumerate().map(|(i, b)| ((*b).to_string(), json!(f(i)))).collect();
        run(s, "variable.create", json!({"collection": "Tokens APP", "name": name, "type": "number", "values": values}))["id"]
            .as_str()
            .unwrap()
            .to_string()
    };
    let size = per_brand(&mut s, "Font/Size/fs-lg", &|i| 20.0 + i as f64);
    let lh = per_brand(&mut s, "Font/Line Height/lh-lg", &|i| 28.0 + i as f64);
    let wt = per_brand(&mut s, "Font/Weight/Bold", &|i| if i == 2 { 600.0 } else { 700.0 });
    let h1 = json!({"fontFamily": "Inter", "fontStyle": "Bold", "fontSize": 24, "lineHeight": 32, "letterSpacing": "-2%"});
    let h1 = run(&mut s, "style.create", json!({"type": "text", "name": "Heading/H1", "value": h1}))["id"].as_str().unwrap().to_string();
    for (field, var) in [("fontSize", &size), ("lineHeight", &lh), ("fontWeight", &wt)] {
        run(&mut s, "style.bind", json!({"style": h1, "field": field, "variable": var}));
    }
    let body = json!({"fontFamily": "Inter", "fontStyle": "Regular", "fontSize": 12, "lineHeight": "150%", "letterSpacing": 0.5});
    run(&mut s, "style.create", json!({"type": "text", "name": "Body/Small", "value": body}));
    s
}

#[test]
fn text_styles_are_tokens_that_follow_the_brand() {
    let mut s = nine_brands_with_text_styles();
    let r = run(&mut s, "export.tokensCss", json!({}));
    let css = r["css"].as_str().unwrap();
    // Bound fields read the brand's token; the rule is on every data-brand too, so var() resolves
    // inside a brand element and not only at :root.
    assert!(css.contains("/* Text styles */\n:root,\n[data-brand] {"), "{css}");
    assert!(css.contains("  --text-heading-h1-font-size: var(--font-size-fs-lg);"), "{css}");
    assert!(css.contains("  --text-heading-h1-line-height: var(--font-line-height-lh-lg);"), "{css}");
    assert!(css.contains("  --text-heading-h1-font-weight: var(--font-weight-bold);"), "{css}");
    // The tokens themselves per brand: sizes in px, the weight with no unit (NV is 600).
    assert!(css.contains("[data-brand=\"nv\"] {"), "{css}");
    assert!(css.contains("--font-size-fs-lg: 22px;"), "{css}");
    assert!(css.contains("--font-weight-bold: 600;"), "{css}");
    // Loose fields are the style's own values, in CSS units.
    assert!(css.contains("  --text-heading-h1-font-family: \"Inter\";"), "{css}");
    assert!(css.contains("  --text-heading-h1-letter-spacing: -0.02em;"), "{css}");
    assert!(css.contains("  --text-body-small-font-weight: 400;"), "{css}");
    assert!(css.contains("  --text-body-small-line-height: 1.5;"), "{css}");
    assert!(css.contains("  --text-body-small-letter-spacing: 0.5px;"), "{css}");
    assert!(css.contains("  --text-body-small-font-size: 12px;"), "{css}");
    // Every loose field is a warning: 2 in Heading/H1, 5 in Body/Small.
    let w: Vec<&str> = r["warnings"].as_array().unwrap().iter().filter_map(Value::as_str).collect();
    assert_eq!(w.len(), 7, "{w:?}");
    assert!(w.contains(&"Heading/H1: font-family \"Inter\" has no variable (the same in every brand)"), "{w:?}");
    // The rows name the variables by their design-system names.
    let h1 = r["textStyles"].as_array().unwrap().iter().find(|t| t["style"] == "Heading/H1").unwrap().clone();
    assert_eq!(h1["prefix"], "--text-heading-h1");
    assert_eq!(h1["fields"]["fontSize"], json!({"value": "var(--font-size-fs-lg)", "variable": "Font/Size/fs-lg"}));
    assert_eq!(h1["fields"]["fontFamily"], json!({"value": "\"Inter\""}));
    // Read only.
    let mut fresh = nine_brands_with_text_styles();
    assert_eq!(run(&mut s, "style.list", json!({})), run(&mut fresh, "style.list", json!({})));
    assert_eq!(run(&mut s, "variable.list", json!({})), run(&mut fresh, "variable.list", json!({})));
}

#[test]
fn text_styles_can_be_left_out_and_clashes_are_warnings() {
    let mut s = nine_brands_with_text_styles();
    let r = run(&mut s, "export.tokensCss", json!({"textStyles": false}));
    assert!(!r["css"].as_str().unwrap().contains("--text-"), "{r}");
    assert_eq!(r["warnings"], json!([]));
    assert!(s.execute("export.tokensCss", &json!({"textStyles": "yes"})).is_err());
    // Two styles with one CSS name: the second is left out and named in a warning.
    let v = json!({"fontFamily": "Inter", "fontSize": 10});
    run(&mut s, "style.create", json!({"type": "text", "name": "Body / Small", "value": v}));
    let r = run(&mut s, "export.tokensCss", json!({}));
    let w = r["warnings"].to_string();
    assert!(w.contains("are both --text-body-small-*"), "{w}");
    assert_eq!(r["textStyles"].as_array().unwrap().len(), 2, "{r}");
    // The weight from a bound style name, as the design system binds it: a string variable with a
    // face name per brand. Each brand gets the number that name means; the style reads it.
    let names: serde_json::Map<String, Value> = BRANDS
        .iter()
        .enumerate()
        .map(|(i, b)| {
            (
                (*b).to_string(),
                json!(match i {
                    1 => "Semi bold",
                    2 => "light",
                    3 => "Book",
                    _ => "Medium",
                }),
            )
        })
        .collect();
    let fs = run(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Font/Weight/font-bold", "type": "string", "values": names}))
        ["id"]
        .as_str()
        .unwrap()
        .to_string();
    let st = run(&mut s, "style.create", json!({"type": "text", "name": "Label", "value": {"fontFamily": "Inter", "fontStyle": "Medium"}}))["id"]
        .as_str()
        .unwrap()
        .to_string();
    run(&mut s, "style.bind", json!({"style": st, "field": "fontStyle", "variable": fs}));
    let r = run(&mut s, "export.tokensCss", json!({}));
    let css = r["css"].as_str().unwrap();
    assert!(css.contains("  --text-label-font-weight: var(--font-weight-font-bold-weight);"), "{css}");
    assert!(css.contains(":root,\n[data-brand=\"farm\"] {\n  --font-weight-font-bold-weight: 500;\n}"), "{css}");
    assert!(css.contains("[data-brand=\"animale\"] {\n  --font-weight-font-bold-weight: 600;\n}"), "{css}");
    assert!(css.contains("[data-brand=\"nv\"] {\n  --font-weight-font-bold-weight: 300;\n}"), "{css}");
    assert!(css.contains("[data-brand=\"fabula\"] {\n  --font-weight-font-bold-weight: 400;\n}"), "{css}");
    // The weights come before the rule that reads them, and the style name is a token, not a warning.
    assert!(css.find("--font-weight-font-bold-weight: 500").unwrap() < css.find("/* Text styles */").unwrap(), "{css}");
    let label = r["textStyles"].as_array().unwrap().iter().find(|t| t["style"] == "Label").unwrap().clone();
    assert_eq!(label["fields"]["fontWeight"]["from"], "fontStyle");
    assert!(!r["warnings"].to_string().contains("Label: font-weight"), "{r}");
    assert_eq!(r["weights"][0]["brands"]["nv"], 300, "{r}");
    assert_eq!(r["weights"][0]["brands"].as_object().unwrap().len(), 9, "{r}");
}

// ------------------------------------------------------------ review fixes (G3)

/// A button set with two variants: the default's fill is a token, the other's a loose colour.
#[test]
fn the_other_variants_loose_values_are_warnings_too() {
    let (mut s, ..) = screen();
    let bg = s.doc().unwrap().variables.get("Colors/Surface/Brand/Background").unwrap().id.clone();
    let mut variants = Vec::new();
    for (k, state) in ["Default", "Hover"].iter().enumerate() {
        let f = id_of(
            &run(
                &mut s,
                "frame.create",
                json!({"x": 500.0 + k as f64 * 150.0, "y": 0, "width": 100, "height": 40, "name": format!("State={state}")}),
            )["id"],
        )
        .unwrap();
        variants.push(id_of(&run(&mut s, "component.create", json!({"ids": [f.0]}))["id"]).unwrap());
    }
    run(&mut s, "variable.bind", json!({"ids": [variants[0].0], "field": "fill", "variable": bg}));
    run(&mut s, "node.set", json!({"id": variants[1].0, "props": {"fill": "#7A4B2A"}}));
    let set = id_of(&run(&mut s, "component.combineVariants", json!({"ids": [variants[0].0, variants[1].0]}))["id"]).unwrap();
    let r = run(&mut s, "export.tsx", json!({"ids": [set.0]}));
    let w = r["warnings"].as_array().unwrap();
    assert!(w.iter().any(|w| w["id"] == json!(variants[1].0) && w["field"] == "fill" && w["value"].as_str().unwrap().contains("#7A4B2A")), "{r}");
    let code = r["code"].as_str().unwrap();
    assert!(code.contains("State=Hover · fill"), "the header lists it: {code}");
    // Once each, even when the same variant is passed again.
    let n = w.iter().filter(|w| w["id"] == json!(variants[1].0)).count();
    assert_eq!(n, 1, "{r}");
}

/// A screen with modes set in two collections: the brand comes from the file's brand collection
/// (the one with a mode per brand), not from whichever collection sorts first.
#[test]
fn the_brand_comes_from_the_brand_collection() {
    let (mut s, screen, ..) = screen();
    // "A Theme" sorts before "Tokens APP" by name; the brand collection is the one with the most modes.
    run(&mut s, "variable.createCollection", json!({"name": "A Theme", "modes": ["Light"]}));
    run(&mut s, "variable.create", json!({"collection": "A Theme", "name": "ink", "type": "color", "value": "#000000"}));
    run(&mut s, "variable.setMode", json!({"ids": [screen.0], "collection": "A Theme", "mode": "Light"}));
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    assert_eq!(r["brand"]["collection"], "Tokens APP", "{r}");
    assert_eq!(r["brand"]["brand"], "animale", "{r}");
    // One mode: the same values everywhere, so no attribute and nothing to say.
    let code = r["code"].as_str().unwrap();
    assert!(!code.contains("data-a-theme"), "{code}");
    assert_eq!(r["notes"], json!([]), "{r}");
    // Asked for by name, the other collection is the brand.
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0], "collection": "A Theme"}));
    assert_eq!(r["brand"]["brand"], "light", "{r}");
    assert!(s.execute("export.tsx", &json!({"ids": [screen.0], "collection": "Nope"})).is_err());
    assert!(s.execute("export.tsx", &json!({"ids": [screen.0], "collection": 3})).is_err());
}

#[test]
fn a_brand_from_a_library_collection_is_a_note() {
    let d = std::env::temp_dir().join(format!("figcraft-g3-brandlib-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let lib_path = d.join("ds.figcraft").to_string_lossy().into_owned();
    let mut lib = Session::new();
    run(&mut lib, "variable.createCollection", json!({"name": "Brands", "modes": ["Farm", "NV"]}));
    run(
        &mut lib,
        "variable.create",
        json!({"collection": "Brands", "name": "Colors/Brand", "type": "color", "values": {"Farm": "#111111", "NV": "#222222"}}),
    );
    let lid = run(&mut lib, "library.publish", json!({}))["id"].as_str().unwrap().to_string();
    run(&mut lib, "file.saveAs", json!({"path": lib_path}));

    let mut s = Session::new();
    run(&mut s, "library.add", json!({"path": lib_path}));
    let f = id_of(&run(&mut s, "frame.create", json!({"x": 0, "y": 0, "width": 100, "height": 100, "name": "Screen"}))["id"]).unwrap();
    run(&mut s, "library.bindVariable", json!({"library": lid, "variable": "Colors/Brand", "field": "fill", "ids": [f.0]}));
    run(&mut s, "variable.setMode", json!({"ids": [f.0], "collection": "Brands", "mode": "NV"}));
    let r = run(&mut s, "export.tsx", json!({"ids": [f.0]}));
    assert_eq!(r["brand"]["brand"], "nv", "{r}");
    assert!(r["notes"].to_string().contains("a library collection"), "{r}");
    assert!(r["code"].as_str().unwrap().contains(" * Note: the brand comes from Brands"), "{r}");
    // The file's own token stylesheet indeed has no block for it.
    let css = run(&mut s, "export.tokensCss", json!({}));
    assert!(!css["css"].as_str().unwrap().contains("data-brand=\"nv\""), "{css}");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_radius_token_on_one_corner_is_not_lost() {
    let (mut s, screen, card, _) = screen();
    let r =
        run(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Border/Radius/Components/br-card", "type": "float", "value": 0}));
    let br = r["id"].as_str().unwrap().to_string();
    // All four corners 0 in this brand, only the top left one bound.
    run(&mut s, "variable.bind", json!({"ids": [card.0], "field": "topLeftRadius", "variable": br}));
    let code = run(&mut s, "export.tsx", json!({"ids": [screen.0]}))["code"].as_str().unwrap().to_string();
    assert!(code.contains("border-radius: var(--border-radius-components-br-card) 0px 0px 0px"), "{code}");
    // Not 0 here either: still the corner's own token.
    run(&mut s, "node.set", json!({"id": card.0, "props": {"cornerRadius": 8}}));
    run(&mut s, "variable.bind", json!({"ids": [card.0], "field": "topLeftRadius", "variable": br}));
    let code = run(&mut s, "export.tsx", json!({"ids": [screen.0]}))["code"].as_str().unwrap().to_string();
    assert!(code.contains("border-radius: var(--border-radius-components-br-card)"), "{code}");
}

#[test]
fn layers_with_one_name_get_numbered_names() {
    let (mut s, screen, ..) = screen();
    let other = id_of(&run(&mut s, "frame.create", json!({"x": 600, "y": 0, "width": 375, "height": 600, "name": "PDP Mobile"}))["id"]).unwrap();
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0, other.0, screen.0]}));
    let code = r["code"].as_str().unwrap();
    assert_eq!(code.matches("export function PDPMobile()").count(), 1, "{code}");
    assert_eq!(code.matches("export function PDPMobile2()").count(), 1, "{code}");
    assert_eq!(code.matches("import \"./pdp-mobile.css\";").count(), 1, "{code}");
    assert_eq!(code.matches("import \"./pdp-mobile-2.css\";").count(), 1, "{code}");
    assert!(r["notes"].to_string().contains("two layers are named PDPMobile"), "{r}");
    assert_eq!(r["brand"].as_array().map(Vec::len), Some(2), "the repeated id is exported once: {r}");
}

#[test]
fn a_collection_named_twice_is_one_block() {
    let mut s = nine_brands();
    let id = s.doc().unwrap().variables.collection("Tokens APP").unwrap().id.clone();
    let once = run(&mut s, "export.tokensCss", json!({"collections": ["Tokens APP"]}));
    let twice = run(&mut s, "export.tokensCss", json!({"collections": ["Tokens APP", id]}));
    assert_eq!(once["css"], twice["css"]);
    assert_eq!(once["brands"], twice["brands"]);
    assert!(twice["warnings"].as_array().unwrap().iter().all(|w| !w.as_str().unwrap().contains("are both")), "{twice}");
}

#[test]
fn export_code_writes_both_files_and_says_what_is_a_token() {
    let (mut s, screen, _card, price) = screen();
    // As text: the same TSX as export.tsx and the same stylesheet as export.tokensCss.
    let r = run(&mut s, "export.code", json!({"ids": [screen.0]}));
    let tsx = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    let css = run(&mut s, "export.tokensCss", json!({}));
    assert_eq!(r["files"][0]["name"], "PDPMobile.tsx", "{r}");
    assert_eq!(r["files"][0]["text"], tsx["code"]);
    assert_eq!(r["files"][1]["name"], "tokens.css");
    assert_eq!(r["files"][1]["text"], css["css"]);
    let tokens: Vec<&str> = r["tokens"].as_array().unwrap().iter().map(|t| t["token"].as_str().unwrap()).collect();
    assert!(tokens.contains(&"--colors-surface-brand-background") && tokens.contains(&"--spacing-sp-md"), "{r}");
    assert_eq!(tokens.iter().filter(|t| **t == "--spacing-sp-md").count(), 1, "once each: {r}");
    assert!(r["tokens"].as_array().unwrap().iter().all(|t| t["defined"] == true), "the file's stylesheet declares them: {r}");
    assert!(r["warnings"].as_array().unwrap().iter().any(|w| w["id"] == json!(price.0) && w["field"] == "fontSize"), "{r}");
    assert_eq!(r["brand"]["brand"], "animale");
    // Into a folder: two files on disk.
    let d = std::env::temp_dir().join(format!("figcraft-g3-code-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let r = run(&mut s, "export.code", json!({"ids": [screen.0], "dir": d.to_string_lossy()}));
    assert_eq!(std::fs::read_to_string(d.join("PDPMobile.tsx")).unwrap(), tsx["code"].as_str().unwrap());
    assert_eq!(std::fs::read_to_string(d.join("tokens.css")).unwrap(), css["css"].as_str().unwrap());
    assert!(r["files"][0]["path"].as_str().unwrap().ends_with("PDPMobile.tsx"), "{r}");
    let _ = std::fs::remove_dir_all(&d);
    // Read only, and refuses nothing or junk.
    run(&mut s, "select.none", json!({}));
    assert!(s.execute("export.code", &json!({})).is_err());
    assert!(s.execute("export.code", &json!({"ids": [screen.0], "dir": 5})).is_err());
    assert!(s.execute("export.code", &json!({"ids": [screen.0], "textStyles": "yes"})).is_err());
}

#[test]
fn a_token_the_stylesheet_does_not_declare_is_flagged() {
    let (mut s, screen, ..) = screen();
    // Only the text styles (an empty collection list): the TSX's tokens are not in this tokens.css.
    let r = run(&mut s, "export.code", json!({"ids": [screen.0], "collections": []}));
    assert!(r["tokens"].as_array().unwrap().iter().all(|t| t["defined"] == false), "{r}");
}

// ------------------------------------------------------------ Web (Tokens WEB, Layout, Checkout)

/// A Web-like file: "Tokens WEB" with the nine brands, "Layout" with Desktop and Mobile, and a
/// one-mode collection (as the Checkout file's): synthetic names and values.
fn web() -> (Session, NodeId) {
    let mut s = nine_brands();
    run(&mut s, "variable.renameCollection", json!({"collection": "Tokens APP", "name": "Tokens WEB"}));
    run(&mut s, "variable.createCollection", json!({"name": "Layout", "modes": ["Desktop", "Mobile"]}));
    run(&mut s, "variable.create", json!({"collection": "Layout", "name": "Grid/Gutter", "type": "number", "values": {"Desktop": 24, "Mobile": 8}}));
    run(&mut s, "variable.create", json!({"collection": "Layout", "name": "Grid/Margin", "type": "number", "values": {"Desktop": 80, "Mobile": 16}}));
    run(&mut s, "variable.createCollection", json!({"name": "Variable collection"}));
    run(&mut s, "variable.create", json!({"collection": "Variable collection", "name": "2 meios de pagamento", "type": "boolean", "value": true}));
    let page = id_of(&run(&mut s, "node.create", json!({"type": "frame", "name": "PDP Web", "width": 1440, "height": 900}))["id"]).unwrap();
    let margin = s.doc().unwrap().variables.get("Grid/Margin").unwrap().id.clone();
    run(&mut s, "node.set", json!({"id": page.0, "props": {"layout": {"mode": "vertical", "padding": [0, 80, 0, 80]}}}));
    for f in ["paddingLeft", "paddingRight"] {
        run(&mut s, "variable.bind", json!({"ids": [page.0], "field": f, "variable": margin}));
    }
    run(&mut s, "variable.setMode", json!({"ids": [page.0], "collection": "Tokens WEB", "mode": "NV"}));
    run(&mut s, "variable.setMode", json!({"ids": [page.0], "collection": "Layout", "mode": "Mobile"}));
    (s, page)
}

#[test]
fn layout_modes_are_their_own_attribute_not_a_brand() {
    let (mut s, _) = web();
    let r = run(&mut s, "export.tokensCss", json!({}));
    let css = r["css"].as_str().unwrap();
    // Brands on data-brand, the nine of them.
    assert!(
        css.contains(
            ":root,
[data-brand=\"farm\"] {"
        ),
        "{css}"
    );
    assert!(css.contains("[data-brand=\"nv\"] {"), "{css}");
    assert_eq!(r["brands"].as_array().unwrap().len(), 9, "{r}");
    assert!(r["brands"].as_array().unwrap().iter().all(|b| b["collection"] == "Tokens WEB"), "{r}");
    // Layout on data-layout, Desktop the default.
    assert!(
        css.contains(
            ":root,
[data-layout=\"desktop\"] {
  --grid-gutter: 24px;
  --grid-margin: 80px;
}"
        ),
        "{css}"
    );
    assert!(
        css.contains(
            "[data-layout=\"mobile\"] {
  --grid-gutter: 8px;
  --grid-margin: 16px;
}"
        ),
        "{css}"
    );
    assert!(!css.contains("data-brand=\"mobile\"") && !css.contains("data-brand=\"desktop\""), "{css}");
    // One mode: :root only.
    assert!(
        css.contains(
            "/* Variable collection · Mode 1 */
:root {
  --2-meios-de-pagamento: true;
}"
        ),
        "{css}"
    );
    let attrs = r["attributes"].as_array().unwrap();
    let layout = attrs.iter().find(|a| a["collection"] == "Layout").unwrap();
    assert_eq!(layout["attribute"], "data-layout");
    assert_eq!(layout["modes"], json!([{"mode": "Desktop", "value": "desktop"}, {"mode": "Mobile", "value": "mobile"}]));
    assert_eq!(attrs.iter().find(|a| a["collection"] == "Variable collection").unwrap()["attribute"], Value::Null);
    assert_eq!(r["warnings"], json!([]), "{r}");
}

#[test]
fn the_screen_says_its_brand_and_its_layout() {
    let (mut s, page) = web();
    let r = run(&mut s, "export.tsx", json!({"ids": [page.0]}));
    let code = r["code"].as_str().unwrap();
    assert!(code.contains("data-brand=\"nv\" data-layout=\"mobile\""), "{code}");
    assert!(code.contains("var(--grid-margin)"), "{code}");
    assert_eq!(r["brand"]["collection"], "Tokens WEB", "{r}");
    assert_eq!(r["notes"], json!([]), "{r}");
    // The stylesheet has a block for every attribute the TSX sets.
    let css = run(&mut s, "export.tokensCss", json!({}))["css"].as_str().unwrap().to_string();
    for sel in ["[data-brand=\"nv\"]", "[data-layout=\"mobile\"]"] {
        assert!(css.contains(sel), "{sel}: {css}");
    }
}

#[test]
fn two_collections_on_one_attribute_are_a_warning() {
    let (mut s, _) = web();
    run(&mut s, "variable.createCollection", json!({"name": "LAYOUT", "modes": ["A", "B"]}));
    run(&mut s, "variable.create", json!({"collection": "LAYOUT", "name": "x", "type": "number", "value": 1}));
    let r = run(&mut s, "export.tokensCss", json!({}));
    assert!(r["warnings"].to_string().contains("both key their modes on data-layout"), "{r}");
}

#[test]
fn app_and_web_name_a_token_the_same() {
    // The developers get one convention: a token with the same name in Tokens APP and Tokens WEB
    // is the same CSS variable.
    let (mut s, _) = web();
    run(&mut s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["Farm", "NV"]}));
    run(
        &mut s,
        "variable.create",
        json!({"collection": "Tokens APP", "name": "Colors/Surface/Brand/Background", "type": "color", "value": "#111111"}),
    );
    let r = run(&mut s, "export.tokensCss", json!({}));
    assert!(r["warnings"].to_string().contains("are both --colors-surface-brand-background"), "the clash is reported: {r}");
    let web = run(&mut s, "export.tokensCss", json!({"collections": ["Tokens WEB"]}));
    let app = run(&mut s, "export.tokensCss", json!({"collections": ["Tokens APP"]}));
    assert!(web["css"].as_str().unwrap().contains("--colors-surface-brand-background:"));
    assert!(app["css"].as_str().unwrap().contains("--colors-surface-brand-background:"));
}

// ------------------------------------------------------------ design-system components (task 2)

/// A screen with a design-system "Button" (a set, linked to code through the map) and a
/// design-system "Chip" with no code component, both instances; the Button's own fill is a
/// loose colour (the component's, not the screen's).
fn screen_with_ds_components() -> (Session, NodeId) {
    let (mut s, screen, card, _) = screen();
    let mut variants = Vec::new();
    for (k, h) in ["Primário", "Secundário"].iter().enumerate() {
        let f = id_of(
            &run(
                &mut s,
                "frame.create",
                json!({"x": 800.0 + k as f64 * 150.0, "y": 0, "width": 120, "height": 40, "name": format!("Hierarquia={h}")}),
            )["id"],
        )
        .unwrap();
        run(&mut s, "node.set", json!({"id": f.0, "props": {"fill": "#7A4B2A"}}));
        variants.push(id_of(&run(&mut s, "component.create", json!({"ids": [f.0]}))["id"]).unwrap().0);
    }
    let set = id_of(&run(&mut s, "component.combineVariants", json!({"ids": variants}))["id"]).unwrap();
    run(&mut s, "node.set", json!({"id": set.0, "props": {"name": "Button", "pluginData": {"eureka.library": "{\"name\":\"DS Apps\"}"}}}));
    let chip = id_of(&run(&mut s, "frame.create", json!({"x": 800, "y": 200, "width": 60, "height": 24, "name": "Chip"}))["id"]).unwrap();
    let chip = id_of(&run(&mut s, "component.create", json!({"ids": [chip.0]}))["id"]).unwrap();
    run(&mut s, "node.set", json!({"id": chip.0, "props": {"pluginData": {"eureka.library": "{\"name\":\"DS Apps\"}"}}}));
    let b = id_of(&run(&mut s, "instance.create", json!({"component": set.0, "x": 0, "y": 0, "parent": card.0}))["id"]).unwrap();
    run(&mut s, "instance.setProperties", json!({"ids": [b.0], "properties": {"Hierarquia": "Secundário"}}));
    run(&mut s, "instance.create", json!({"component": chip.0, "x": 0, "y": 0, "parent": card.0}));
    let map = json!({"version": 1, "links": [
        {"component": "Button", "language": "react", "importPath": "@acme/ds-apps", "name": "Button", "template": "<Button variant={Hierarquia} />"}
    ]});
    let r = run(&mut s, "codeConnect.import", json!({"json": map}));
    assert_eq!(r["linked"], 1, "{r}");
    (s, screen)
}

#[test]
fn a_design_system_component_in_the_map_is_a_call_to_its_code_component() {
    let (mut s, screen) = screen_with_ds_components();
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    let code = r["code"].as_str().unwrap();
    assert!(
        code.starts_with(
            "import { Button } from \"@acme/ds-apps\";
/*"
        ),
        "the import heads the file: {code}"
    );
    assert!(code.contains("import \"./pdp-mobile.css\";"), "{code}");
    assert!(code.contains("<Button variant=\"Secundário\" />"), "the instance's own value: {code}");
    assert!(!code.contains("#7A4B2A"), "the component's insides are the code component's: {code}");
    assert!(r["warnings"].as_array().unwrap().iter().all(|w| w["value"] != "#7A4B2A"), "{r}");
    assert_eq!(r["components"][0]["name"], "Button", "{r}");
    assert_eq!(r["components"][0]["importPath"], "@acme/ds-apps", "{r}");
    // The Chip has no code component: drawn, and said.
    assert!(r["notes"].to_string().contains("Chip is a design-system component with no code component in the map"), "{r}");
    assert!(code.contains(" * Note: Chip: Chip is a design-system component"), "{code}");
}

#[test]
fn without_a_map_the_design_systems_instances_are_drawn_and_noted() {
    let (mut s, screen) = screen_with_ds_components();
    let set = s.doc().unwrap().nodes().find(|n| n.name == "Button" && matches!(n.kind, figcraft_doc::NodeKind::ComponentSet { .. })).unwrap().id;
    run(&mut s, "codeConnect.remove", json!({"id": set.0}));
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0]}));
    let code = r["code"].as_str().unwrap();
    assert!(!code.contains("import { Button }"), "{code}");
    assert_eq!(r["components"], json!([]), "{r}");
    assert!(r["notes"].to_string().contains("Button is a design-system component with no code component in the map"), "{r}");
}

/// The Button set of [`screen_with_ds_components`] and a second screen using it.
fn two_screens_with_one_button() -> (Session, NodeId, NodeId) {
    let (mut s, screen) = screen_with_ds_components();
    let set = s.doc().unwrap().nodes().find(|n| n.name == "Button" && matches!(n.kind, figcraft_doc::NodeKind::ComponentSet { .. })).unwrap().id;
    let other = id_of(&run(&mut s, "frame.create", json!({"x": 0, "y": 1200, "width": 360, "height": 200, "name": "Checkout"}))["id"]).unwrap();
    run(&mut s, "instance.create", json!({"component": set.0, "x": 0, "y": 0, "parent": other.0}));
    (s, screen, other)
}

#[test]
fn two_screens_with_one_code_component_import_it_once_at_the_top() {
    let (mut s, screen, other) = two_screens_with_one_button();
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0, other.0]}));
    let code = r["code"].as_str().unwrap();
    assert_eq!(code.matches("import { Button } from \"@acme/ds-apps\";").count(), 1, "{code}");
    assert!(code.starts_with("import { Button }"), "{code}");
    assert_eq!(code.matches("<Button").count(), 2, "both screens call it: {code}");
    assert_eq!(r["components"].as_array().unwrap().len(), 2, "{r}");
}

#[test]
fn a_layer_named_as_an_imported_component_gets_a_number() {
    let (mut s, screen, _) = two_screens_with_one_button();
    let root = id_of(&run(&mut s, "frame.create", json!({"x": 0, "y": 1600, "width": 100, "height": 40, "name": "Button"}))["id"]).unwrap();
    let r = run(&mut s, "export.tsx", json!({"ids": [screen.0, root.0]}));
    let code = r["code"].as_str().unwrap();
    assert!(code.contains("function Button2("), "the layer is not the imported Button: {code}");
    assert!(!code.contains("function Button("), "{code}");
    assert!(r["notes"].to_string().contains("Button is also an imported code component"), "{r}");
}

/// A file whose brand collection, "Tokens APP", comes from a library, with a local "Layout"
/// (Desktop, Mobile) and a local one-mode theme: the screen on NV and Mobile.
fn library_brand_and_local_layout(tag: &str) -> (Session, NodeId, std::path::PathBuf) {
    let d = std::env::temp_dir().join(format!("figcraft-g3-libbrand-{tag}-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let lib_path = d.join("ds.figcraft").to_string_lossy().into_owned();
    let mut lib = Session::new();
    run(&mut lib, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["Farm", "NV"]}));
    run(
        &mut lib,
        "variable.create",
        json!({"collection": "Tokens APP", "name": "Colors/Brand", "type": "color", "values": {"Farm": "#111111", "NV": "#222222"}}),
    );
    let lid = run(&mut lib, "library.publish", json!({}))["id"].as_str().unwrap().to_string();
    run(&mut lib, "file.saveAs", json!({"path": lib_path}));

    let mut s = Session::new();
    run(&mut s, "library.add", json!({"path": lib_path}));
    run(&mut s, "variable.createCollection", json!({"name": "Layout", "modes": ["Desktop", "Mobile"]}));
    run(&mut s, "variable.create", json!({"collection": "Layout", "name": "Grid/Gutter", "type": "number", "values": {"Desktop": 24, "Mobile": 8}}));
    run(&mut s, "variable.createCollection", json!({"name": "Tema"}));
    run(&mut s, "variable.create", json!({"collection": "Tema", "name": "ink", "type": "color", "value": "#000000"}));
    let f = id_of(&run(&mut s, "frame.create", json!({"x": 0, "y": 0, "width": 100, "height": 100, "name": "Screen"}))["id"]).unwrap();
    run(&mut s, "library.bindVariable", json!({"library": lid, "variable": "Colors/Brand", "field": "fill", "ids": [f.0]}));
    run(&mut s, "variable.setMode", json!({"ids": [f.0], "collection": "Tokens APP", "mode": "NV"}));
    run(&mut s, "variable.setMode", json!({"ids": [f.0], "collection": "Layout", "mode": "Mobile"}));
    (s, f, d)
}

#[test]
fn the_named_brand_collection_wins_even_from_a_library() {
    let (mut s, f, d) = library_brand_and_local_layout("named");
    let r = run(&mut s, "export.tsx", json!({"ids": [f.0]}));
    let code = r["code"].as_str().unwrap();
    // The library's Tokens APP is the brand; the local Layout keeps its own attribute.
    assert!(code.contains("data-brand=\"nv\" data-layout=\"mobile\""), "{code}");
    assert!(!code.contains("data-brand=\"mobile\"") && !code.contains("data-tokens-app"), "{code}");
    assert_eq!(r["brand"]["collection"], "Tokens APP", "{r}");
    // The file's stylesheet keys Layout on data-layout, not on data-brand.
    let css = run(&mut s, "export.tokensCss", json!({}));
    let text = css["css"].as_str().unwrap();
    assert!(text.contains("[data-layout=\"mobile\"]") && !text.contains("data-brand=\"mobile\""), "{css}");
    assert_eq!(css["brands"], json!([]), "the brand collection is the library's: {css}");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_chosen_brand_collection_is_the_same_in_the_tsx_and_the_stylesheet() {
    let (mut s, f, d) = library_brand_and_local_layout("chosen");
    // Asked for "Layout" as the brand (an odd choice, but both files must agree).
    let r = run(&mut s, "export.code", json!({"ids": [f.0], "collection": "Layout"}));
    let tsx = r["files"][0]["text"].as_str().unwrap();
    let css = r["files"][1]["text"].as_str().unwrap();
    assert!(tsx.contains("data-brand=\"mobile\""), "{tsx}");
    assert!(css.contains("[data-brand=\"mobile\"]"), "{css}");
    assert!(!css.contains("[data-layout="), "{css}");
    let sheet = run(&mut s, "export.tokensCss", json!({"collection": "Layout"}));
    assert_eq!(sheet["css"].as_str().unwrap(), css);
    assert!(s.execute("export.tokensCss", &json!({"collection": 4})).is_err());
    assert!(s.execute("export.tokensCss", &json!({"collection": "Nope"})).is_err());
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn a_local_collection_with_more_modes_does_not_take_the_brand_from_tokens_app() {
    let (mut s, page) = web();
    run(&mut s, "variable.createCollection", json!({"name": "Densidade", "modes": ["A", "B", "C", "D", "E", "F", "G", "H", "I", "J", "K", "L"]}));
    run(&mut s, "variable.create", json!({"collection": "Densidade", "name": "d", "type": "number", "value": 1}));
    let r = run(&mut s, "export.tsx", json!({"ids": [page.0]}));
    assert_eq!(r["brand"]["collection"], "Tokens WEB", "named, though Densidade has more modes: {r}");
}

#[test]
fn numbered_names_never_meet_a_layer_already_named_so() {
    let (mut s, screen, ..) = screen();
    let mk = |s: &mut Session, name: &str, x: f64| {
        id_of(&run(s, "frame.create", json!({"x": x, "y": 0, "width": 10, "height": 10, "name": name}))["id"]).unwrap()
    };
    run(&mut s, "node.set", json!({"id": screen.0, "props": {"name": "Card"}}));
    let (b, c) = (mk(&mut s, "Card", 600.0), mk(&mut s, "Card2", 700.0));
    let code = run(&mut s, "export.tsx", json!({"ids": [screen.0, b.0, c.0]}))["code"].as_str().unwrap().to_string();
    for f in ["export function Card()", "export function Card2()", "export function Card22()"] {
        assert_eq!(code.matches(f).count(), 1, "{f}: {code}");
    }
}
