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
use crate::cmd::variables_io::{CssStyle, css_with};
use crate::codegen::ds_var_name;
use crate::codegen::{Loose, slug};
use crate::components::variant_props;
use crate::params::{get, targets};
use crate::{bad, spec};

/// Most layers one export takes.
const MAX_ROOTS: usize = 32;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "export.tokensCss",
            "Export the design-system token stylesheet",
            "{collections? [ids or names; default every collection of this file, not library copies], collection? (the brand collection, as export.tsx's), textStyles? (default true)}  the stylesheet export.tsx's names read: each variable as `--token-name` (the same rule); the modes of the brand collection (`collection`, else the one named Tokens APP or Tokens WEB, else the one with the most modes) on `[data-brand=\"<mode name without notes in brackets>\"]`, of another collection with modes on its own attribute (`Layout` → `[data-layout=\"mobile\"]`), the default mode also on `:root`; a collection with one mode on `:root` only; aliases as `var(--name)`, colour calculations as the colour they show in that mode, numbers in px unless used as opacity or font weight. Then (unless `textStyles` is false) each text style of this file (not library copies) as `--text-<style name, same rule>-<font-family|font-weight|font-size|line-height|letter-spacing>`, on `:root` and every `[data-brand]`: a field bound to a variable is `var(--token-name)` and follows the brand; a weight from a bound style name (a string variable such as \"Semi bold\") is `var(--<variable>-weight)`, a number per brand from the name it holds there; a field with no variable is the style's own value and a warning. Read only: the variables and styles are not changed. No dialog → {css, brands: [{collection, mode, brand}], attributes: [{collection, attribute (null: one mode, on :root), modes: [{mode, value}]}], textStyles: [{style, prefix, fields: {fontFamily|fontWeight|fontSize|lineHeight|letterSpacing: {value, variable?, from? (fontStyle)}}}], weights: [{variable, token, brands: {brand: weight}}], warnings: [text] (two variables, two modes, two collections' attributes or two text styles with the same CSS name; a text style field with no variable)}",
            false,
            |s, p| {
                let doc = s.doc()?;
                stylesheet(doc, p, "export.tokensCss")
            }
        ),
        spec!(
            "export.code",
            "Export code",
            "{ids? (default: the selection), dir? (folder to write the files into; else they come back as text), collection? (as export.tsx), collections?, textStyles? (as export.tokensCss)}  what the developers receive, for the designer: export.tsx of the layers and export.tokensCss of this file, as `<first layer's component name>.tsx` and `tokens.css` (written again on each export; the TSX is generated, not edited). Read only. No dialog → {files: [{name, path?, text?}], brand, tokens: [{token, defined (the token stylesheet declares it)}] (the variables the TSX reads, in first-use order), warnings (export.tsx's values with no token), notes (export.tsx's), cssWarnings (export.tokensCss's)}",
            false,
            |s, p| {
                let cmd = "export.code";
                let ids = targets(s, p);
                if ids.is_empty() {
                    return Err(bad(cmd, "select a layer or pass `ids`"));
                }
                let dir = match get(p, "dir") {
                    None | Some(Value::Null) => None,
                    Some(Value::String(d)) if !d.trim().is_empty() => Some(d.clone()),
                    Some(_) => return Err(bad(cmd, "dir: a folder path")),
                };
                let doc = s.doc()?;
                if let Some(id) = ids.iter().find(|id| !doc.contains(**id)) {
                    return Err(bad(cmd, format!("no such layer: {}", id.0)));
                }
                let brand = match get(p, "collection") {
                    None | Some(Value::Null) => None,
                    Some(Value::String(k)) => Some(doc.variables.collection(k).ok_or_else(|| bad(cmd, format!("no collection {k}")))?.id.clone()),
                    Some(_) => return Err(bad(cmd, "collection: a collection id or name")),
                };
                let sheet = stylesheet(doc, p, cmd)?;
                let tsx = export_with(doc, &ids, brand.as_deref());
                let name = ids.first().and_then(|id| crate::component_codegen::ds_type_name(doc, *id)).unwrap_or_else(|| "Screen".into());
                code_files(&tsx, &sheet, &name, dir.as_deref())
            }
        ),
        spec!(
            "export.tsx",
            "Export TSX with design-system tokens",
            "{ids? (default: the selection), collection? (the brand collection, id or name; default: the one named Tokens APP or Tokens WEB, this file's or a library's, else the one with the most modes; export.tokensCss keys the same one on data-brand)}  each layer as a React function component (TSX) with its stylesheet, for the developers: bound variables as `var(--token-name)` (name lower case, ASCII, `/` and spaces as `-`) with no fallback values, the brand collection's mode set on the layer or above it as `data-brand` on the root element and another collection's (with modes) as its attribute (`data-layout=\"mobile\"`), instances inlined, the other variants' values included; colours, spacing, radii, font families, font weights and font sizes with no variable bound are written as they are and listed as warnings (also in a comment at the top). A layer passed twice is exported once; two layers with the same name get numbered names (`Card`, `Card2`) and a note. An instance connected to code in React (codeConnect, e.g. from the design system's code map: codeConnect.import) is a call to its code component, imported from its package at the top; another design-system instance (`eureka.library`) is drawn as layers and noted. No dialog → {code, warnings: [{id, layer, field, value}], brand, components: [{id, layer, component, name, importPath, inside: [text] (what the instance changed inside it, which the call doesn't pass)}] (the calls), notes: [text] (a brand or mode from a library collection this file's export.tokensCss has no block for; a chosen collection with no mode set; renamed duplicates; a design-system instance with no code component; a call whose instance changed texts, variants or visibility inside it)}",
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
                let brand = match get(p, "collection") {
                    None | Some(Value::Null) => None,
                    Some(Value::String(k)) => Some(doc.variables.collection(k).ok_or_else(|| bad(cmd, format!("no collection {k}")))?.id.clone()),
                    Some(_) => return Err(bad(cmd, "collection: a collection id or name")),
                };
                Ok(export_with(doc, &ids, brand.as_deref()))
            }
        ),
    ]
}

/// `export.tokensCss` with its parameters `collections`, `collection` and `textStyles` read from `p`.
pub(crate) fn stylesheet(doc: &Document, p: &Value, cmd: &str) -> crate::Result<Value> {
    let colls: Vec<&Collection> = match get(p, "collections") {
        None | Some(Value::Null) => crate::library::local_collections(doc).collect(),
        Some(Value::Array(a)) => {
            let mut out: Vec<&Collection> = Vec::new();
            for x in a.iter().take(256) {
                let c = x.as_str().and_then(|k| doc.variables.collection(k)).ok_or_else(|| bad(cmd, format!("no collection {x}")))?;
                // One collection named twice (by id and by name, say) is one block.
                if !out.iter().any(|o| o.id == c.id) {
                    out.push(c);
                }
            }
            out
        }
        Some(_) => return Err(bad(cmd, "collections: [collection ids or names]")),
    };
    let with_styles = match get(p, "textStyles") {
        None | Some(Value::Null) => true,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(bad(cmd, "textStyles: true|false")),
    };
    // The brand collection the TSX keys `data-brand` on, the same in both files.
    let brand = match get(p, "collection") {
        None | Some(Value::Null) => None,
        Some(Value::String(k)) => Some(doc.variables.collection(k).ok_or_else(|| bad(cmd, format!("no collection {k}")))?.id.clone()),
        Some(_) => return Err(bad(cmd, "collection: a collection id or name")),
    };
    let mut out = tokens_css(doc, &colls, brand.as_deref());
    if with_styles {
        add_text_styles(doc, brand.as_deref(), &mut out);
    }
    Ok(out)
}

/// `export.code`'s result from `export.tsx`'s (`tsx`) and `export.tokensCss`'s (`sheet`): the two
/// files (`<name>.tsx`, `tokens.css`), written into `dir` when given, and the tokens the TSX reads.
pub(crate) fn code_files(tsx: &Value, sheet: &Value, name: &str, dir: Option<&str>) -> crate::Result<Value> {
    let code = tsx["code"].as_str().unwrap_or_default();
    let css = sheet["css"].as_str().unwrap_or_default();
    let mut tokens: Vec<Value> = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    // Only the code: the comments (the header explains `var(--token)`) are not reads.
    let mut bare = String::new();
    let mut rest = code;
    while let Some(i) = rest.find("/*") {
        bare.push_str(rest.get(..i).unwrap_or_default());
        rest = rest.get(i..).and_then(|r| r.find("*/").and_then(|j| r.get(j + 2..))).unwrap_or_default();
    }
    bare.push_str(rest);
    for part in bare.split("var(").skip(1) {
        let token = part.split([')', ',']).next().unwrap_or("").trim();
        if !token.starts_with("--") || seen.contains(&token) || seen.len() >= 4096 {
            continue;
        }
        seen.push(token);
        tokens.push(json!({"token": token, "defined": css.contains(&format!("{token}:"))}));
    }
    let mut files = Vec::new();
    for (file, text) in [(format!("{name}.tsx"), code), ("tokens.css".to_string(), css)] {
        match dir {
            Some(d) => {
                let path = std::path::Path::new(d).join(&file);
                crate::file_access::write(&path, text.as_bytes())?;
                files.push(json!({"name": file, "path": path.to_string_lossy()}));
            }
            None => files.push(json!({"name": file, "text": text})),
        }
    }
    Ok(json!({
        "files": files,
        "brand": tsx["brand"],
        "tokens": tokens,
        "warnings": tsx["warnings"],
        "notes": tsx["notes"],
        "cssWarnings": sheet["warnings"],
    }))
}

/// The token stylesheet of `colls`: `{css, brands, warnings}` (see `export.tokensCss`).
pub fn tokens_css(doc: &Document, colls: &[&Collection], brand: Option<&str>) -> Value {
    let mut warnings = Vec::new();
    let mut brands = Vec::new();
    let mut attributes = Vec::new();
    let mut seen: BTreeMap<String, String> = BTreeMap::new();
    let mut attrs_taken: BTreeMap<String, String> = BTreeMap::new();
    let brand_id = brand.map(str::to_string).or_else(|| brand_collection(doc).map(|c| c.id.clone()));
    for c in colls {
        let is_brand = brand_id.as_deref() == Some(c.id.as_str());
        // A collection with one mode has no attribute: its values are the same everywhere.
        if !is_brand && c.modes.len() < 2 {
            attributes.push(json!({"collection": c.name, "attribute": Value::Null, "modes": [c.modes.first().map(|m| m.name.clone())]}));
        }
        let attr = if is_brand { "data-brand".to_string() } else { collection_attr(c) };
        if (is_brand || c.modes.len() > 1)
            && let Some(first) = attrs_taken.insert(attr.clone(), c.name.clone())
        {
            warnings.push(format!("collections \"{first}\" and \"{}\" both key their modes on {attr}", c.name));
        }
        let mut taken: BTreeMap<String, &str> = BTreeMap::new();
        let mut values = Vec::new();
        for m in &c.modes {
            let b = brand_slug(&m.name);
            if (is_brand || c.modes.len() > 1)
                && let Some(first) = taken.insert(b.clone(), &m.name)
            {
                warnings.push(format!("{}: modes \"{first}\" and \"{}\" are both {attr}=\"{b}\"", c.name, m.name));
            }
            if is_brand {
                brands.push(json!({"collection": c.name, "mode": m.name, "brand": b}));
            } else {
                values.push(json!({"mode": m.name, "value": b}));
            }
        }
        if !is_brand && c.modes.len() > 1 {
            attributes.push(json!({"collection": c.name, "attribute": attr, "modes": values}));
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
    json!({"css": format!("{head}{}", css_with(doc, colls, CssStyle::Ds, brand_id.as_deref())), "brands": brands, "attributes": attributes, "warnings": warnings})
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
    /// The attributes (`data-layout`) of the other collections with modes that a style reads a
    /// token from: the styles' rule is declared on them too.
    pub attrs: Vec<String>,
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
pub fn text_style_tokens(doc: &Document, brand: Option<&str>) -> TextStyleTokens {
    let mut out = TextStyleTokens {
        decls: String::new(),
        weights_css: String::new(),
        attrs: Vec::new(),
        rows: Vec::new(),
        weights: Vec::new(),
        warnings: Vec::new(),
    };
    let brand_id = brand.map(str::to_string).or_else(|| brand_collection(doc).map(|c| c.id.clone()));
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
            for v in var.into_iter().chain(named) {
                if let Some(c) = doc.variables.collections.iter().find(|c| c.id == v.collection)
                    && brand_id.as_deref() != Some(c.id.as_str())
                    && c.modes.len() > 1
                {
                    let a = collection_attr(c);
                    if !out.attrs.contains(&a) {
                        out.attrs.push(a);
                    }
                }
            }
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
            // The collection's own attribute: the brand's on data-brand, another one's on its own
            // (`data-layout`), one mode on :root alone.
            let is_brand = brand_id.as_deref() == Some(c.id.as_str());
            let attr = if is_brand { "data-brand".to_string() } else { collection_attr(c) };
            let sel = match (is_brand || c.modes.len() > 1, i) {
                (false, _) => ":root".to_string(),
                (true, 0) => format!(":root,\n[{attr}=\"{b}\"]"),
                (true, _) => format!("[{attr}=\"{b}\"]"),
            };
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
fn add_text_styles(doc: &Document, brand: Option<&str>, out: &mut Value) {
    let t = text_style_tokens(doc, brand);
    if let Some(css) = out.get_mut("css").and_then(|c| c.as_str().map(str::to_string)) {
        let weights = if t.weights_css.is_empty() { String::new() } else { format!("/* Weights from style-name tokens */\n{}", t.weights_css) };
        let sels: String = t.attrs.iter().map(|a| format!(",\n[{a}]")).collect();
        let rule = if t.decls.is_empty() { String::new() } else { format!("/* Text styles */\n:root,\n[data-brand]{sels} {{\n{}}}\n", t.decls) };
        out["css"] = Value::String(format!("{css}{weights}{rule}"));
    }
    out["textStyles"] = Value::Array(t.rows);
    out["weights"] = Value::Array(t.weights);
    if let Some(w) = out.get_mut("warnings").and_then(Value::as_array_mut) {
        w.extend(t.warnings.into_iter().map(Value::String));
    }
}

/// The TSX for `ids` (up to [`MAX_ROOTS`]), the brand from the default brand collection:
/// `{code, warnings, brand, notes}`.
pub fn export(doc: &Document, ids: &[NodeId]) -> Value {
    export_with(doc, ids, None)
}

/// [`export`], the brand from collection `brand_coll` (an id) when given.
pub fn export_with(doc: &Document, ids: &[NodeId], brand_coll: Option<&str>) -> Value {
    let (mut parts, mut warnings, mut brands, mut notes) = (Vec::new(), Vec::new(), Vec::new(), Vec::<String>::new());
    let mut calls: Vec<Value> = Vec::new();
    let mut roots: Vec<NodeId> = Vec::new();
    for id in ids.iter().copied() {
        if roots.len() >= MAX_ROOTS {
            break;
        }
        if !roots.contains(&id) {
            roots.push(id);
        }
    }
    // Instances connected to code (codeConnect, from the design system's code map) are calls to
    // the code component, imported from its package; the design system's other instances are
    // drawn, and said. One file: each import once, at its top, whichever screens use it.
    let found: Vec<DsCalls> = roots.iter().map(|&id| ds_calls(doc, id)).collect();
    let mut imports: Vec<String> = Vec::new();
    let mut imported: Vec<String> = Vec::new();
    for ds in &found {
        for line in &ds.imports {
            if !imports.contains(line) {
                imports.push(line.clone());
            }
        }
        for name in ds.calls.iter().filter_map(|c| c["name"].as_str()) {
            if !imported.iter().any(|n| n == name) {
                imported.push(name.to_string());
            }
        }
    }
    let mut taken: Vec<String> = Vec::new();
    for (id, ds) in roots.iter().copied().zip(found) {
        let Some(name) = crate::component_codegen::ds_type_name(doc, id) else { continue };
        // Two layers of one name would be two functions and two stylesheets of one name, and a
        // layer named as an imported code component would clash with its import: the first free
        // `<name><n>` from 2, against every name given or imported ("Card", "Card", "Card2" →
        // Card, Card2, Card22).
        let free = |n: &str| !taken.iter().any(|t| t == n) && !imported.iter().any(|t| t == n);
        let suffix = if free(&name) {
            String::new()
        } else {
            (2..=MAX_ROOTS + 1).map(|k| k.to_string()).find(|k| free(&format!("{name}{k}"))).unwrap_or_default()
        };
        taken.push(format!("{name}{suffix}"));
        let mut own = Vec::new();
        if !suffix.is_empty() {
            if imported.contains(&name) {
                own.push(format!("{name} is also an imported code component: this layer ({}) is {name}{suffix}", id.0));
            } else {
                own.push(format!("two layers are named {name}: this one ({}) is {name}{suffix}", id.0));
            }
        }
        let (attr, brand, brand_notes) = brand_attr_in(doc, id, brand_coll);
        own.extend(brand_notes);
        let Some((code, loose)) = crate::component_codegen::react_ds(doc, id, &attr, &suffix) else { continue };
        for (layer, component) in &ds.drawn {
            own.push(format!("{layer}: {component} is a design-system component with no code component in the map (drawn as layers)"));
        }
        for (layer, name, changes) in &ds.inside {
            own.push(format!("{layer}: the call to {name} passes only its own properties; changed inside it: {}", changes.join("; ")));
        }
        calls.extend(ds.calls);
        parts.push(format!("{}{code}", header(&loose, brand.as_ref(), &own)));
        warnings.extend(loose.iter().map(|l| json!({"id": l.id.0, "layer": l.layer, "field": l.field, "value": l.value})));
        brands.push(brand.map_or(Value::Null, |(b, coll, mode)| json!({"brand": b, "collection": coll, "mode": mode})));
        for n in own {
            if !notes.contains(&n) {
                notes.push(n);
            }
        }
    }
    let brand = if brands.len() == 1 { brands.pop().unwrap_or(Value::Null) } else { Value::Array(brands) };
    let body = parts.join(
        "
",
    );
    let code = if imports.is_empty() || body.is_empty() {
        body
    } else {
        format!(
            "{}
{body}",
            imports.join(
                "
"
            )
        )
    };
    json!({"code": code, "warnings": warnings, "brand": brand, "notes": notes, "components": calls})
}

/// Most layers [`ds_calls`] walks.
const MAX_WALK: usize = 20_000;

/// What the design system's components become in the export of `root`.
#[derive(Default)]
struct DsCalls {
    /// Import lines of the code components called, once each in tree order.
    imports: Vec<String>,
    /// `{id, layer, component, name, importPath}` of each instance written as a call.
    calls: Vec<Value>,
    /// (layer, component) of the design system's instances with no code component: drawn.
    drawn: Vec<(String, String)>,
    /// (layer, code component, changes) of the calls whose instance changed something inside it.
    inside: Vec<(String, String, Vec<String>)>,
}

/// Most changes listed inside one call.
const MAX_INSIDE: usize = 12;

/// What instance `id` changed inside it, against the layers of its main component that each one
/// mirrors: texts, a nested instance's variant or other non-text values (texts are listed by their
/// layer), and visibility. A call passes only the instance's own properties, so these never reach
/// the code component: the developers need to see them.
fn changed_inside(doc: &Document, id: NodeId) -> Vec<String> {
    use figcraft_doc::NodeKind;
    let mut out = Vec::new();
    // The instance's own properties: the call passes them (a text or visibility they drive is not
    // a change inside it).
    let own: Vec<String> = crate::component_codegen::main_of(doc, id)
        .and_then(|m| crate::component_codegen::describe(doc, m))
        .map(|c| c.params.into_iter().map(|p| p.prop).collect())
        .unwrap_or_default();
    let passed = |n: &figcraft_doc::Node, src: &figcraft_doc::Node, field: &str| {
        let Some(prop) = n.prop_refs.get(field).or_else(|| src.prop_refs.get(field)) else { return false };
        let nearest =
            doc.ancestors(n.id).into_iter().take(MAX_WALK).find(|a| doc.node(*a).is_some_and(|x| matches!(x.kind, NodeKind::Instance { .. })));
        nearest == Some(id) && own.contains(prop)
    };
    let mut stack: Vec<NodeId> = doc.node(id).map(|n| n.children.iter().rev().copied().collect()).unwrap_or_default();
    let mut seen = 0usize;
    while let Some(d) = stack.pop() {
        seen += 1;
        if seen > MAX_WALK || out.len() >= MAX_INSIDE {
            break;
        }
        let Some(n) = doc.node(d) else { continue };
        let Some(src) = n.source.filter(|s| *s != d).and_then(|s| doc.node(s)) else {
            stack.extend(n.children.iter().rev().copied());
            continue;
        };
        if n.visible != src.visible && !passed(n, src, "visible") {
            out.push(format!("{}: {}", n.name, if n.visible { "shown" } else { "hidden" }));
            continue;
        }
        if !n.visible {
            continue;
        }
        match (&n.kind, &src.kind) {
            (NodeKind::Text(t), NodeKind::Text(o)) if t.characters != o.characters && !passed(n, src, "characters") => {
                let text: String = t.characters.chars().take(80).collect();
                out.push(format!("{}: \"{text}\"", n.name));
            }
            (NodeKind::Instance { component, props }, NodeKind::Instance { component: oc, props: op }) => {
                let mut changes = Vec::new();
                if component != oc {
                    let (now, was) = (doc.node(*component).map(|m| variant_props(&m.name)), doc.node(*oc).map(|m| variant_props(&m.name)));
                    for (k, v) in now.unwrap_or_default() {
                        if was.as_ref().and_then(|w| w.get(&k)) != Some(&v) {
                            changes.push(format!("{k}={v}"));
                        }
                    }
                }
                for (k, v) in props {
                    if !v.is_string() && op.get(k) != Some(v) {
                        changes.push(format!("{k}={}", v.as_bool().map_or_else(|| v.to_string(), |b| if b { "On".into() } else { "Off".into() })));
                    }
                }
                if !changes.is_empty() {
                    out.push(format!("{}: {}", n.name, changes.join(", ")));
                }
            }
            _ => {}
        }
        stack.extend(n.children.iter().rev().copied());
    }
    out
}

/// The visible instances under `root` (itself included only when it is not the root): one
/// connected to code in React is a call (its import, not its layers); another from a
/// design-system library (`eureka.library`) is drawn and listed; any other instance is drawn and
/// walked into, as the export inlines it.
fn ds_calls(doc: &Document, root: NodeId) -> DsCalls {
    use crate::codegen::CodeLang;
    let mut out = DsCalls::default();
    let mut stack: Vec<NodeId> = doc.node(root).map(|n| n.children.iter().rev().copied().collect()).unwrap_or_default();
    let mut seen = 0usize;
    while let Some(id) = stack.pop() {
        seen += 1;
        if seen > MAX_WALK {
            break;
        }
        let Some(n) = doc.node(id).filter(|n| n.visible) else { continue };
        if let figcraft_doc::NodeKind::Instance { component, .. } = n.kind {
            if let Some(link) = crate::code_connect::link(doc, id, CodeLang::React) {
                for line in crate::code_connect::call_import_lines(doc, id, CodeLang::React) {
                    if !line.is_empty() && !out.imports.contains(&line) {
                        out.imports.push(line);
                    }
                }
                let inside = changed_inside(doc, id);
                if !inside.is_empty() {
                    out.inside.push((n.name.clone(), link.name.clone(), inside.clone()));
                }
                out.calls.push(json!({"id": id.0, "layer": n.name, "component": doc.node(component).map(|m| m.name.clone()), "name": link.name, "importPath": link.import_path, "inside": inside}));
                continue;
            }
            if !crate::cmd::eureka::library_of(doc, component).is_null() {
                let set = crate::code_connect::owner(doc, component).and_then(|o| doc.node(o)).map_or_else(String::new, |o| o.name.clone());
                if !out.drawn.iter().any(|(l, c)| *l == n.name && *c == set) {
                    out.drawn.push((n.name.clone(), set));
                }
            }
        }
        stack.extend(n.children.iter().rev().copied());
    }
    out
}

/// The brand mode the variables of `id` resolve in, from the default brand collection (see
/// [`brand_attr_in`]): the root attribute (empty without a mode) and (brand, collection, mode).
pub fn brand_attr(doc: &Document, id: NodeId) -> (String, Option<(String, String, String)>) {
    let (attr, brand, _) = brand_attr_in(doc, id, None);
    (attr, brand)
}

/// The file's brand collection, the one `data-brand` keys: one named as [`BRAND_COLLECTIONS`] (this
/// file's before a library copy), else the collection with the most modes.
pub fn brand_collection(doc: &Document) -> Option<&Collection> {
    let all = &doc.variables.collections;
    let local = |c: &&Collection| !crate::library::is_remote_collection(doc, &c.id);
    // The design systems' own brand collections, by name, this file's or a library's.
    let named = |c: &&Collection| BRAND_COLLECTIONS.iter().any(|n| c.name.trim().eq_ignore_ascii_case(n));
    if let Some(c) = all.iter().filter(named).find(local).or_else(|| all.iter().find(named)) {
        return Some(c);
    }
    // Else the one with the most modes (a brand collection has one per brand); this file's own on
    // a tie, then the first.
    let max = all.iter().map(|c| c.modes.len()).max()?;
    let most = |c: &&Collection| c.modes.len() == max;
    all.iter().filter(most).find(local).or_else(|| all.iter().find(most))
}

/// The design systems' brand collections, by name: the one `data-brand` keys whenever a file has
/// it, here or from a library, whatever other collections it has.
pub const BRAND_COLLECTIONS: [&str; 2] = ["Tokens APP", "Tokens WEB"];

/// The attribute a collection that is not the brand collection keys its modes on:
/// `data-<collection name, as [`slug`]>` (`Layout` → `data-layout`), never `data-brand`.
pub fn collection_attr(c: &Collection) -> String {
    let s = slug(&c.name, "collection");
    if s == "brand" { "data-brand-collection".into() } else { format!("data-{s}") }
}

/// The selector of mode `i` (`m`) of collection `c` in the token stylesheet: the brand collection
/// (`brand`, an id, when given, else [`brand_collection`]) on `[data-brand="<brand>"]`, another collection with modes on
/// `[<collection_attr>="<mode>"]`, the default mode (the first) also on `:root`; a collection with
/// one mode on `:root` only.
pub(crate) fn ds_selector(doc: &Document, c: &Collection, i: usize, m: &figcraft_doc::Mode, brand: Option<&str>) -> String {
    let brand = match brand {
        Some(id) => c.id == id,
        None => brand_collection(doc).is_some_and(|b| b.id == c.id),
    };
    if !brand && c.modes.len() < 2 {
        return ":root".into();
    }
    let attr = if brand { "data-brand".to_string() } else { collection_attr(c) };
    let sel = format!("[{attr}=\"{}\"]", brand_slug(&m.name));
    if i == 0 { format!(":root,\n{sel}") } else { sel }
}

/// The modes of `id`, the mode set on the layer or above it (an instance also takes its main
/// component's): of the brand collection (`prefer`, an id, when given, else [`brand_collection`])
/// as `data-brand` (the mode's name without parenthesised notes, as [`slug`]: `Animale (CERTO)` →
/// `animale`), of every other collection with modes as its [`collection_attr`]. Returns the root
/// attributes, the brand (brand, collection, mode) and notes: a brand or mode from a library copy
/// (this file's token stylesheet has no block for it), a preferred collection with no mode set.
pub fn brand_attr_in(doc: &Document, id: NodeId, prefer: Option<&str>) -> (String, Option<(String, String, String)>, Vec<String>) {
    let brand_id = prefer.map(str::to_string).or_else(|| brand_collection(doc).map(|c| c.id.clone()));
    let modes = doc.modes_for(id);
    let mut attrs = String::new();
    let mut brand = None;
    let mut notes = Vec::new();
    // In the file's collection order: the same attributes every export.
    for c in &doc.variables.collections {
        let Some(m) = modes.get(&c.id).and_then(|mode| c.modes.iter().find(|m| m.id == *mode)) else { continue };
        let remote = crate::library::is_remote_collection(doc, &c.id);
        let value = brand_slug(&m.name);
        if brand_id.as_deref() == Some(c.id.as_str()) {
            attrs = format!(" data-brand=\"{value}\"{attrs}");
            brand = Some((value, c.name.clone(), m.name.clone()));
            if remote {
                notes.push(format!(
                    "the brand comes from {}, a library collection: this file's export.tokensCss has no [data-brand] block for it (export the library file's tokens)",
                    c.name
                ));
            }
        } else if c.modes.len() > 1 {
            let attr = collection_attr(c);
            attrs.push_str(&format!(" {attr}=\"{value}\""));
            if remote {
                notes.push(format!(
                    "{attr} comes from {}, a library collection: this file's export.tokensCss has no [{attr}] block for it (export the library file's tokens)",
                    c.name
                ));
            }
        }
    }
    if brand.is_none()
        && let Some(p) = prefer
    {
        let name = doc.variables.collections.iter().find(|c| c.id == p).map_or(p, |c| c.name.as_str());
        notes.push(format!("no mode of {name} is set on the layer or above it: no data-brand"));
    }
    (attrs, brand, notes)
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
fn header(loose: &[Loose], brand: Option<&(String, String, String)>, notes: &[String]) -> String {
    let clean = |s: &str| s.replace("*/", "* /").replace(['\n', '\r'], " ");
    let mut out = String::from(
        "/*\n * Generated by Eureka Design from the design document: do not edit by hand, export again.\n * Design-system tokens are CSS variables (var(--token)); their values come from the design system's stylesheet.\n",
    );
    match brand {
        Some((b, coll, mode)) => out.push_str(&format!(" * Brand: data-brand=\"{b}\" ({}: {})\n", clean(coll), clean(mode))),
        None => out.push_str(" * Brand: no mode set on the screen (the design system's default applies)\n"),
    }
    for n in notes {
        out.push_str(&format!(" * Note: {}\n", clean(n)));
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
