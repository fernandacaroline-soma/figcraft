//! `export.tsx` (Eureka Design, ADR-003): an approved screen as TSX the developers can use, with
//! the design system's tokens by name, never their values. The document is the source; the TSX is
//! generated output, exported again rather than edited by hand.
//!
//! It is the React code of `inspect.code` in design-system token mode
//! ([`crate::codegen::Tokens`]):
//! - a bound variable is `var(--token-name)` ([`crate::codegen::ds_var_name`]), with no fallback
//!   and no `:root` rule (the values come from the design system's own stylesheet);
//! - a colour, spacing, radius, font family, font weight or font size with no variable bound is written as it is and listed
//!   among the warnings, in the result and in a comment at the top of the file;
//! - the brand mode set on the screen (or above it) is a `data-brand` attribute on the root
//!   element ([`brand_attr`]), which the design system's stylesheet keys the variables on;
//! - instances are inlined with their overrides (no Code Connect yet).
//!
//! `export.tokensCss` writes the stylesheet those names read: every mode of the collections as a
//! `[data-brand]` block of custom properties, from the variables in the document (imported from
//! the design system's snapshot; nothing here changes them). The file's text styles follow as
//! `--text-<style>-<font-family|font-weight|font-size|line-height|letter-spacing>`
//! ([`text_style_tokens`]): a field bound to a variable reads that token, so it changes with the
//! brand (a weight bound as a style name reads a number written per brand from that name); a
//! field with no variable is the style's own value, the same in every brand, and a warning.

use std::collections::BTreeMap;

use figcraft_doc::{Collection, Document, LetterSpacing, LineHeight, NodeId, StyleKind, trim_num};
use serde_json::{Value, json};

use crate::cmd::CommandSpec;
use crate::cmd::variables_io::{CssStyle, css};
use crate::codegen::ds_var_name;
use crate::codegen::{Loose, slug};
use crate::params::{get, targets};
use crate::{bad, spec};

/// Most layers one export takes.
const MAX_ROOTS: usize = 32;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "export.tokensCss",
            "Export the design-system token stylesheet",
            "{collections? [ids or names; default every collection of this file, not library copies], textStyles? (default true)}  the stylesheet export.tsx's names read: each variable as `--token-name` (the same rule), every mode of each collection on `[data-brand=\"<mode name without notes in brackets>\"]` and the default mode also on `:root`; aliases as `var(--name)`, colour calculations as the colour they show in that mode, numbers in px unless used as opacity or font weight. Then (unless `textStyles` is false) each text style of this file (not library copies) as `--text-<style name, same rule>-<font-family|font-weight|font-size|line-height|letter-spacing>`, on `:root` and every `[data-brand]`: a field bound to a variable is `var(--token-name)` and follows the brand; a weight from a bound style name (a string variable such as \"Semi bold\") is `var(--<variable>-weight)`, a number per brand from the name it holds there; a field with no variable is the style's own value and a warning. Read only: the variables and styles are not changed. No dialog → {css, brands: [{collection, mode, brand}], textStyles: [{style, prefix, fields: {fontFamily|fontWeight|fontSize|lineHeight|letterSpacing: {value, variable?, from? (fontStyle)}}}], weights: [{variable, token, brands: {brand: weight}}], warnings: [text] (two variables, two modes or two text styles with the same CSS name; a text style field with no variable)}",
            false,
            |s, p| {
                let cmd = "export.tokensCss";
                let doc = s.doc()?;
                let colls: Vec<&Collection> = match get(p, "collections") {
                    None | Some(Value::Null) => crate::library::local_collections(doc).collect(),
                    Some(Value::Array(a)) => a
                        .iter()
                        .map(|x| x.as_str().and_then(|k| doc.variables.collection(k)).ok_or_else(|| bad(cmd, format!("no collection {x}"))))
                        .collect::<crate::Result<_>>()?,
                    Some(_) => return Err(bad(cmd, "collections: [collection ids or names]")),
                };
                let with_styles = match get(p, "textStyles") {
                    None | Some(Value::Null) => true,
                    Some(Value::Bool(b)) => *b,
                    Some(_) => return Err(bad(cmd, "textStyles: true|false")),
                };
                let mut out = tokens_css(doc, &colls);
                if with_styles {
                    add_text_styles(doc, &mut out);
                }
                Ok(out)
            }
        ),
        spec!(
            "export.tsx",
            "Export TSX with design-system tokens",
            "{ids? (default: the selection)}  each layer as a React function component (TSX) with its stylesheet, for the developers: bound variables as `var(--token-name)` (name lower case, ASCII, `/` and spaces as `-`) with no fallback values, the brand mode set on the layer or above it as `data-brand` on the root element, instances inlined; colours, spacing, radii, font families, font weights and font sizes with no variable bound are written as they are and listed as warnings (also in a comment at the top). No dialog → {code, warnings: [{id, layer, field, value}], brand}",
            false,
            |s, p| {
                let cmd = "export.tsx";
                let ids = targets(s, p);
                if ids.is_empty() {
                    return Err(bad(cmd, "select a layer or pass `ids`"));
                }
                let doc = s.doc()?;
                if let Some(id) = ids.iter().find(|id| !doc.contains(**id)) {
                    return Err(bad(cmd, format!("no such layer: {}", id.0)));
                }
                Ok(export(doc, &ids))
            }
        ),
    ]
}

/// The token stylesheet of `colls`: `{css, brands, warnings}` (see `export.tokensCss`).
pub fn tokens_css(doc: &Document, colls: &[&Collection]) -> Value {
    let mut warnings = Vec::new();
    let mut brands = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    for c in colls {
        let mut taken: BTreeMap<String, &str> = BTreeMap::new();
        for m in &c.modes {
            let b = brand_slug(&m.name);
            if let Some(first) = taken.insert(b.clone(), &m.name) {
                warnings.push(format!("{}: modes \"{first}\" and \"{}\" are both data-brand=\"{b}\"", c.name, m.name));
            }
            brands.push(json!({"collection": c.name, "mode": m.name, "brand": b}));
        }
        for v in doc.variables.variables.iter().filter(|v| v.collection == c.id) {
            let n = ds_var_name(&v.name);
            if let Some(first) = seen.insert(n.clone(), v.name.clone()) {
                warnings.push(format!("\"{first}\" and \"{}\" are both {n}", v.name));
            }
        }
    }
    let head = "/*
 * Generated by Eureka Design: the design system's tokens by brand, for the TSX export.
 * Values as the document holds them (imported from the design system); export again, do not edit.
 */
";
    json!({"css": format!("{head}{}", css(doc, colls, CssStyle::Ds)), "brands": brands, "warnings": warnings})
}

/// Text style fields, in the order they are written: (field key in a style's `bound`, CSS suffix).
const TEXT_FIELDS: [(&str, &str); 5] = [
    ("fontFamily", "font-family"),
    ("fontWeight", "font-weight"),
    ("fontSize", "font-size"),
    ("lineHeight", "line-height"),
    ("letterSpacing", "letter-spacing"),
];

/// Most text styles one stylesheet takes.
const MAX_TEXT_STYLES: usize = 2000;

/// The text style tokens of a file: what `export.tokensCss` adds after the variables.
pub struct TextStyleTokens {
    /// The `--text-<style>-<field>` declarations, without a selector.
    pub decls: String,
    /// Per-brand blocks of the weights read from style-name variables (`--<variable>-weight`).
    pub weights_css: String,
    /// `textStyles` rows: `{style, prefix, fields}`.
    pub rows: Vec<Value>,
    /// `weights` rows: `{variable, token, brands: {brand: weight}}`.
    pub weights: Vec<Value>,
    pub warnings: Vec<String>,
}

/// The text styles of this file (not library copies) as tokens, for `export.tokensCss`: one
/// `--text-<style>-<field>` per field. A bound field is `var(--token-name)`. A weight that comes
/// from a bound style name (a string variable such as "Semi bold", as design systems that change
/// the face by brand bind it) is `var(--<variable>-weight)`, a number written for every mode of the
/// variable's collection from the name it holds there (the weight the app gives that name). A
/// field with no variable is the style's own value and a warning.
pub fn text_style_tokens(doc: &Document) -> TextStyleTokens {
    let mut out = TextStyleTokens { decls: String::new(), weights_css: String::new(), rows: Vec::new(), weights: Vec::new(), warnings: Vec::new() };
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut named_weights: Vec<&figcraft_doc::Variable> = Vec::new();
    for st in crate::library::local_styles(doc).take(MAX_TEXT_STYLES) {
        let StyleKind::Text { style: t, .. } = &st.kind else { continue };
        let prefix = format!("--text-{}", slug(&st.name, "style"));
        if let Some(first) = seen.insert(prefix.clone(), st.name.clone()) {
            out.warnings.push(format!("text styles \"{first}\" and \"{}\" are both {prefix}-*", st.name));
            continue;
        }
        let mut fields = serde_json::Map::new();
        for (key, suffix) in TEXT_FIELDS {
            let var = st.bound.get(key).and_then(|id| doc.variables.get_by_id(id));
            let named = (key == "fontWeight")
                .then(|| st.bound.get("fontStyle").and_then(|id| doc.variables.get_by_id(id)))
                .flatten()
                .filter(|v| v.kind == figcraft_doc::VariableType::String);
            let (value, row) = match (var, named) {
                (Some(v), _) => {
                    let value = format!("var({})", ds_var_name(&v.name));
                    (value.clone(), json!({"value": value, "variable": v.name}))
                }
                (None, Some(v)) => {
                    let value = format!("var({}-weight)", ds_var_name(&v.name));
                    if !named_weights.iter().any(|w| w.id == v.id) {
                        named_weights.push(v);
                    }
                    (value.clone(), json!({"value": value, "variable": v.name, "from": "fontStyle"}))
                }
                (None, None) => {
                    let own = own_value(t, key);
                    out.warnings.push(format!("{}: {suffix} {own} has no variable (the same in every brand)", st.name));
                    (own.clone(), json!({"value": own}))
                }
            };
            out.decls.push_str(&format!("  {prefix}-{suffix}: {value};\n"));
            fields.insert(key.to_string(), row);
        }
        out.rows.push(json!({"style": st.name, "prefix": prefix, "fields": fields}));
    }
    for v in named_weights {
        let token = format!("{}-weight", ds_var_name(&v.name));
        if doc.variables.variables.iter().any(|o| ds_var_name(&o.name) == token) {
            out.warnings.push(format!("{token} (the weight of \"{}\") is also a variable's name", v.name));
        }
        let Some(c) = doc.variables.collections.iter().find(|c| c.id == v.collection) else { continue };
        let mut brands = serde_json::Map::new();
        for (i, m) in c.modes.iter().enumerate() {
            let modes = BTreeMap::from([(c.id.clone(), m.id.clone())]);
            let Some(figcraft_doc::VariableValue::String(name)) = doc.variables.resolve(&v.id, &modes) else {
                out.warnings.push(format!("\"{}\" has no style name in {}: no {token} there", v.name, m.name));
                continue;
            };
            let w = figcraft_doc::style_weight(&name);
            let b = brand_slug(&m.name);
            let sel = if i == 0 { format!(":root,\n[data-brand=\"{b}\"]") } else { format!("[data-brand=\"{b}\"]") };
            out.weights_css.push_str(&format!("{sel} {{\n  {token}: {w};\n}}\n"));
            brands.insert(b, json!(w));
        }
        out.weights.push(json!({"variable": v.name, "token": token, "brands": brands}));
    }
    out
}

/// A text style's own value of `key`, in CSS units.
fn own_value(t: &figcraft_doc::TextStyle, key: &str) -> String {
    match key {
        "fontFamily" => format!("\"{}\"", t.font_family.replace(['"', '\\', '\n', '\r'], " ")),
        "fontWeight" => t.weight().to_string(),
        "fontSize" => format!("{}px", trim_num(t.font_size)),
        "lineHeight" => match t.line_height {
            LineHeight::Auto => "normal".to_string(),
            LineHeight::Pixels(x) => format!("{}px", trim_num(x)),
            LineHeight::Percent(x) => trim_num(x / 100.0),
        },
        _ => match t.letter_spacing {
            LetterSpacing::Pixels(x) => format!("{}px", trim_num(x)),
            LetterSpacing::Percent(x) => format!("{}em", trim_num(x / 100.0)),
        },
    }
}

/// `out` (a `tokens_css` result) with the text style tokens: the weights by brand, the styles'
/// rule, `textStyles`, `weights` and the warnings. The styles' rule is on `:root` and every
/// `[data-brand]`: a custom property's `var()` is resolved where it is declared, so declared on
/// `:root` alone it would keep the default brand's values inside a `data-brand` element.
fn add_text_styles(doc: &Document, out: &mut Value) {
    let t = text_style_tokens(doc);
    if let Some(css) = out.get_mut("css").and_then(|c| c.as_str().map(str::to_string)) {
        let weights = if t.weights_css.is_empty() { String::new() } else { format!("/* Weights from style-name tokens */\n{}", t.weights_css) };
        let rule = if t.decls.is_empty() { String::new() } else { format!("/* Text styles */\n:root,\n[data-brand] {{\n{}}}\n", t.decls) };
        out["css"] = Value::String(format!("{css}{weights}{rule}"));
    }
    out["textStyles"] = Value::Array(t.rows);
    out["weights"] = Value::Array(t.weights);
    if let Some(w) = out.get_mut("warnings").and_then(Value::as_array_mut) {
        w.extend(t.warnings.into_iter().map(Value::String));
    }
}

/// The TSX for `ids` (up to [`MAX_ROOTS`]): `{code, warnings, brand}`.
pub fn export(doc: &Document, ids: &[NodeId]) -> Value {
    let (mut parts, mut warnings, mut brands) = (Vec::new(), Vec::new(), Vec::new());
    for id in ids.iter().copied().take(MAX_ROOTS) {
        let (attr, brand) = brand_attr(doc, id);
        let Some((code, loose)) = crate::component_codegen::react_ds(doc, id, &attr) else { continue };
        parts.push(format!("{}{code}", header(&loose, brand.as_ref())));
        warnings.extend(loose.iter().map(|l| json!({"id": l.id.0, "layer": l.layer, "field": l.field, "value": l.value})));
        brands.push(brand.map_or(Value::Null, |(b, coll, mode)| json!({"brand": b, "collection": coll, "mode": mode})));
    }
    let brand = if brands.len() == 1 { brands.pop().unwrap_or(Value::Null) } else { Value::Array(brands) };
    json!({"code": parts.join("\n"), "warnings": warnings, "brand": brand})
}

/// The brand mode the variables of `id` resolve in: the first variable collection with a mode set
/// on the layer or above it (an instance also takes its main component's). `data-brand` is the
/// mode's name without parenthesised notes, as [`slug`]: `Animale (CERTO)` → `animale`.
/// Returns the root attribute (empty without a mode) and (brand, collection, mode).
pub fn brand_attr(doc: &Document, id: NodeId) -> (String, Option<(String, String, String)>) {
    for (coll, mode) in doc.modes_for(id) {
        let Some(c) = doc.variables.collection(&coll) else { continue };
        let Some(m) = c.modes.iter().find(|m| m.id == mode) else { continue };
        let brand = brand_slug(&m.name);
        return (format!(" data-brand=\"{brand}\""), Some((brand, c.name.clone(), m.name.clone())));
    }
    (String::new(), None)
}

/// A mode's `data-brand` value: its name without parenthesised notes, as [`slug`]
/// (`Animale (CERTO)` → `animale`).
pub fn brand_slug(mode: &str) -> String {
    slug(&strip_notes(mode), "brand")
}

/// `name` without its parenthesised parts: `Animale (CERTO)` → `Animale `.
fn strip_notes(name: &str) -> String {
    let (mut out, mut depth) = (String::new(), 0usize);
    for c in name.chars() {
        match c {
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            c if depth == 0 => out.push(c),
            _ => {}
        }
    }
    if out.trim().is_empty() { name.to_string() } else { out }
}

/// The comment at the top of an exported file: where it comes from, the brand and the values
/// with no token.
fn header(loose: &[Loose], brand: Option<&(String, String, String)>) -> String {
    let clean = |s: &str| s.replace("*/", "* /").replace(['\n', '\r'], " ");
    let mut out = String::from(
        "/*\n * Generated by Eureka Design from the design document: do not edit by hand, export again.\n * Design-system tokens are CSS variables (var(--token)); their values come from the design system's stylesheet.\n",
    );
    match brand {
        Some((b, coll, mode)) => out.push_str(&format!(" * Brand: data-brand=\"{b}\" ({}: {})\n", clean(coll), clean(mode))),
        None => out.push_str(" * Brand: no mode set on the screen (the design system's default applies)\n"),
    }
    if loose.is_empty() {
        out.push_str(" * Warnings: none (every colour, spacing, radius and font is a token)\n");
    } else {
        out.push_str(&format!(" * Warnings: {} value(s) with no token (design-system debt):\n", loose.len()));
        for l in loose {
            out.push_str(&format!(" *   - {} · {}: {}\n", clean(&l.layer), l.field, clean(&l.value)));
        }
    }
    out.push_str(" */\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brand_names_drop_notes_and_accents() {
        assert_eq!(slug(&strip_notes("Animale (CERTO)"), "brand"), "animale");
        assert_eq!(slug(&strip_notes("Farm"), "brand"), "farm");
        assert_eq!(slug(&strip_notes("Fábula Rio (novo)"), "brand"), "fabula-rio");
        assert_eq!(slug(&strip_notes("(só nota)"), "brand"), "so-nota");
        assert_eq!(slug(&strip_notes("((("), "brand"), "brand");
    }
}
