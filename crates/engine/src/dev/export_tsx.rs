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
//! the design system's snapshot; nothing here changes them).

use std::collections::BTreeMap;

use figcraft_doc::{Collection, Document, NodeId};
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
            "{collections? [ids or names; default every collection of this file, not library copies]}  the stylesheet export.tsx's names read: each variable as `--token-name` (the same rule), every mode of each collection on `[data-brand=\"<mode name without notes in brackets>\"]` and the default mode also on `:root`; aliases as `var(--name)`, colour calculations as the colour they show in that mode, numbers in px unless used as opacity or font weight. Read only: the variables are not changed. No dialog → {css, brands: [{collection, mode, brand}], warnings: [text] (two variables or two modes with the same CSS name)}",
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
                Ok(tokens_css(doc, &colls))
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
