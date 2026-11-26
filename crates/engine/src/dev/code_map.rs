//! `export.codeMap` (Eureka Design): checks a map from the design system's components to the code
//! components the developers have, before the TSX export uses it. The map is a code connect file
//! (`figcraft.connect.json`, [`crate::code_connect`]: `{version: 1, links: [{component, language,
//! importPath, name, template}]}`) kept outside the repository (in `eureka-cache/`), one entry per
//! component set of the design system. Nothing is linked or changed: `codeConnect.import` does that
//! once the map checks out.
//!
//! What it reports, for the design system's components in the document (main components and sets
//! an import marked as coming from a design-system library, `eureka.library`; every component with
//! `all`):
//! - mapped: the code component, and the template's placeholders that aren't properties
//!   (`unknown`) and the properties no placeholder reads (`unmapped`, so a variant the code never
//!   receives shows);
//! - missing: components of the design system with no entry;
//! - notInDocument: entries naming no component of the document (the map may cover more of the
//!   library than one file holds: information, not an error);
//! - invalid: entries that would not link (bad name, import path or template).

use std::collections::BTreeMap;

use figcraft_doc::{CodeLink, Document, NodeId};
use serde_json::{Value, json};

use crate::cmd::CommandSpec;
use crate::code_connect::{check, language, owner, template_names};
use crate::codegen::CodeLang;
use crate::component_codegen::{Component, describe, main_of};
use crate::params::get;
use crate::{Result, Session, bad, spec};

/// Largest map file read (bytes).
const MAX_FILE: u64 = 4 * 1024 * 1024;
/// Most entries a map holds.
const MAX_ENTRIES: usize = 5_000;
/// Most rows each list reports.
const MAX_ROWS: usize = 2_000;
/// Most (entry, component) checks one report makes: entries × components of one name.
const MAX_CHECKS: usize = 20_000;

pub fn specs() -> Vec<CommandSpec> {
    vec![spec!(
        "export.codeMap",
        "Check the design-system code map",
        "{json (the map: a code connect file, {version: 1, links: [{component, language, importPath, name, template}]}, as an object or JSON text) | path (its file, e.g. in eureka-cache/), language? (react|leptos|vue|swiftui|compose|arkui|flutter, default react), all? (every component of the document, not only those an import marked as from a design-system library)}  checks the map against the document, read only (codeConnect.import links it) → {language, mapped: [{component, library, name, importPath, unknown (placeholders that aren't properties), unmapped (properties no placeholder reads)}], missing: [{component, library}] (design-system components with no entry), notInDocument: [component] (entries naming no component here), invalid: [{entry, component, error}] (also a second entry for one component and language), counts: {designSystem, mapped, missing, notInDocument, invalid}, truncated (stopped after 20000 checks)}",
        false,
        code_map
    )]
}

fn code_map(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "export.codeMap";
    let parse_json = |b: &[u8]| serde_json::from_slice::<Value>(b).map_err(|e| bad(cmd, format!("not JSON: {e}")));
    let path = match get(p, "path") {
        None | Some(Value::Null) => None,
        Some(Value::String(t)) if !t.trim().is_empty() => Some(t.clone()),
        Some(_) => return Err(bad(cmd, "path: the map file")),
    };
    let map = match (get(p, "json"), path) {
        (Some(v @ Value::Object(_)), _) => v.clone(),
        (Some(Value::String(t)), _) => parse_json(t.as_bytes())?,
        (Some(_), _) => return Err(bad(cmd, "json: the map (an object or JSON text)")),
        (None, Some(path)) => parse_json(&crate::cmd::file::read_capped(&path, MAX_FILE)?)?,
        (None, None) => return Err(bad(cmd, "pass `json` or `path`")),
    };
    let lang = match get(p, "language") {
        None | Some(Value::Null) => CodeLang::React,
        Some(Value::String(l)) => language(l).ok_or_else(|| bad(cmd, "language: react|leptos|vue|swiftui|compose|arkui|flutter"))?,
        Some(_) => return Err(bad(cmd, "language: react|leptos|vue|swiftui|compose|arkui|flutter")),
    };
    let all = match get(p, "all") {
        None | Some(Value::Null) => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(bad(cmd, "all: true|false")),
    };
    if map.get("version").is_some_and(|x| x.as_u64() != Some(1)) {
        return Err(bad(cmd, "version: only 1 is supported"));
    }
    let entries = map.get("links").and_then(Value::as_array).ok_or_else(|| bad(cmd, "expected {version: 1, links: [...]}"))?;
    if entries.len() > MAX_ENTRIES {
        return Err(bad(cmd, format!("at most {MAX_ENTRIES} links")));
    }
    Ok(report(s.doc()?, entries, lang, all))
}

/// The design system's components in `doc` by name (or every component with `all`): link owners
/// (sets, and components outside a set) with the library an import recorded on them.
fn ds_components(doc: &Document, all: bool) -> BTreeMap<String, Vec<(NodeId, Value)>> {
    let mut out: BTreeMap<String, Vec<(NodeId, Value)>> = BTreeMap::new();
    for n in doc.nodes().filter(|n| owner(doc, n.id) == Some(n.id)) {
        let library = main_of(doc, n.id).map_or(Value::Null, |m| crate::cmd::eureka::library_of(doc, m));
        if all || !library.is_null() {
            out.entry(n.name.clone()).or_default().push((n.id, library));
        }
    }
    out
}

fn report(doc: &Document, entries: &[Value], lang: CodeLang, all: bool) -> Value {
    let ds = ds_components(doc, all);
    let mut described: BTreeMap<NodeId, Option<Component>> = BTreeMap::new();
    let (mut mapped, mut invalid, mut not_in_doc) = (Vec::new(), Vec::new(), Vec::<String>::new());
    let mut covered: Vec<&str> = Vec::new();
    let mut checks = 0usize;
    let mut cut = false;
    // (component, language) of the entries read: a second one for the same pair is a duplicate.
    let mut seen: Vec<(String, String)> = Vec::new();
    for (i, e) in entries.iter().enumerate() {
        let field = |k: &str| e.get(k).and_then(Value::as_str).map(str::trim);
        let Some(component) = field("component") else {
            invalid.push(json!({"entry": i + 1, "component": Value::Null, "error": "needs a component name"}));
            continue;
        };
        // Other languages' entries belong to other exports.
        if field("language").and_then(language) != Some(lang) {
            if field("language").and_then(language).is_none() {
                invalid.push(json!({"entry": i + 1, "component": component, "error": "language: react|leptos|vue|swiftui|compose|arkui|flutter"}));
            }
            continue;
        }
        let (Some(name), Some(template)) = (field("name"), e.get("template").and_then(Value::as_str)) else {
            invalid.push(json!({"entry": i + 1, "component": component, "error": "needs name and template strings"}));
            continue;
        };
        let pair = (component.to_string(), lang.id().to_string());
        if seen.contains(&pair) {
            invalid.push(json!({"entry": i + 1, "component": component, "error": "a second entry for this component and language (codeConnect.import keeps the last)"}));
            continue;
        }
        seen.push(pair);
        let link = CodeLink {
            language: lang.id().into(),
            import_path: field("importPath").unwrap_or_default().to_string(),
            name: name.into(),
            template: template.into(),
        };
        let Some((key, owners)) = ds.get_key_value(component) else {
            if !not_in_doc.iter().any(|c| c == component) {
                not_in_doc.push(component.to_string());
            }
            continue;
        };
        covered.push(key.as_str());
        for (id, library) in owners {
            checks += 1;
            if checks > MAX_CHECKS {
                cut = true;
                break;
            }
            let Some(c) = described.entry(*id).or_insert_with(|| main_of(doc, *id).and_then(|m| describe(doc, m))).as_ref() else { continue };
            match check(c, &link, lang) {
                Ok(unknown) => {
                    let used = template_names(&link.template);
                    let unmapped: Vec<&str> =
                        c.params.iter().filter(|p| !used.iter().any(|u| *u == p.prop || *u == p.ident)).map(|p| p.prop.as_str()).collect();
                    mapped.push(json!({
                        "component": component, "id": id.0, "library": library, "name": link.name,
                        "importPath": link.import_path, "unknown": unknown, "unmapped": unmapped,
                    }));
                }
                Err(err) => invalid.push(json!({"entry": i + 1, "component": component, "error": err})),
            }
        }
    }
    let missing: Vec<Value> = ds
        .iter()
        .filter(|(name, _)| !covered.contains(&name.as_str()))
        .flat_map(|(name, owners)| owners.iter().map(move |(id, library)| json!({"component": name, "id": id.0, "library": library})))
        .collect();
    let designs: usize = ds.values().map(Vec::len).sum();
    let counts = json!({"designSystem": designs, "mapped": mapped.len(), "missing": missing.len(), "notInDocument": not_in_doc.len(), "invalid": invalid.len()});
    let cap = |mut v: Vec<Value>| {
        v.truncate(MAX_ROWS);
        v
    };
    not_in_doc.truncate(MAX_ROWS);
    json!({"language": lang.id(), "mapped": cap(mapped), "missing": cap(missing), "notInDocument": not_in_doc, "invalid": cap(invalid), "counts": counts, "truncated": cut})
}
