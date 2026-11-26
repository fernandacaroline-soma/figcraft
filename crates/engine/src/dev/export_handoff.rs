//! `explore.exportCode` (Eureka Design, task 5): the proposal the designer chose on an exploration
//! board, once its design critique and the Gatekeeper's verdict are in, as the code the developers
//! receive: `export.code` of its screen (`<Screen>.tsx` and `tokens.css`), the TSX headed by a
//! summary of the proposal, the verdict and the critique's open findings. Read only.

use figcraft_doc::{Document, NodeId};
use serde_json::{Value, json};

use crate::cmd::CommandSpec;
use crate::cmd::eureka_critique::CRITIQUE_KEY;
use crate::cmd::eureka_explore::{PROPOSAL_KEY, REVIEW_KEY};
use crate::params::{get, id_of};
use crate::{Result, Session, bad, spec};

/// Most review items and critique findings the summary lists (the rest are counted).
const MAX_LISTED: usize = 20;
/// Largest code map read (bytes).
const MAX_MAP: u64 = 4 * 1024 * 1024;
/// Longest text the summary keeps per field.
const MAX_FIELD: usize = 400;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "explore.exportCode",
            "Export the chosen proposal as code",
            "{id (the chosen proposal's screen), dir? (folder to write the files into; else they come back as text), collection? (as export.tsx)}  once the designer has chosen the proposal (explore.choose) and the reviewer's verdict is in (explore.review), export.code of its screen with a summary for the developers at the top of the TSX: the proposal (letter, name, type, board, summary), the verdict (reviewer, verdict, summary, items) and the design critique (its findings still open, by severity). Read only. No dialog → export.code's result and {proposal, review, critique: {findings, open}}",
            false,
            export_chosen
        ),
        spec!(
            "explore.codeMap",
            "Link the design system's code map",
            "{path? (the map file), data? (its contents, base64, where the platform has no paths), name? (the map's file name, with data or json), json? (the map: a code connect file, as export.codeMap and codeConnect.import take it)}  checks the map (export.codeMap) and links it (codeConnect.import), so `explore.exportCode` writes the design system's instances as calls to their code components; the file remembers the map's name and counts (on its first page). Without a map, says what is linked → {map: {name, linked, missing} | null, designSystem (components of a design-system library here), unlinked: [component] (with no React code component), check? (export.codeMap's counts), warnings?}",
            true,
            link_code_map
        ),
    ]
}

/// Where the file keeps the code map it linked (on its first page): `{name, linked, missing}`.
pub const CODE_MAP_KEY: &str = "eureka.codeMap";
/// Most unlinked component names listed.
const MAX_UNLINKED: usize = 50;

/// The code map this file linked, if any.
pub fn linked_code_map(doc: &Document) -> Option<Value> {
    doc.pages.iter().find_map(|p| plugin_json(doc, *p, CODE_MAP_KEY))
}

/// `explore.codeMap`.
fn link_code_map(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.codeMap";
    let text = |k: &str| match get(p, k) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(t)) if !t.trim().is_empty() => Ok(Some(t.clone())),
        Some(_) => Err(bad(CMD, format!("{k}: a string"))),
    };
    let (path, data, name) = (text("path")?, text("data")?, text("name")?);
    let map = match (get(p, "json"), path.as_deref(), data) {
        (Some(v @ (Value::Object(_) | Value::String(_))), _, _) => Some((v.clone(), name.unwrap_or_else(|| "code map".into()))),
        (Some(_), _, _) => return Err(bad(CMD, "json: the map (an object or JSON text)")),
        (None, Some(path), _) => {
            let bytes = crate::cmd::file::read_capped(path, MAX_MAP)?;
            let file = std::path::Path::new(path).file_name().map_or_else(|| path.to_string(), |f| f.to_string_lossy().into_owned());
            Some((json!(String::from_utf8_lossy(&bytes)), file))
        }
        (None, None, Some(d)) => {
            use base64::Engine as _;
            let bytes = base64::engine::general_purpose::STANDARD.decode(d.trim()).map_err(|e| bad(CMD, format!("data: {e}")))?;
            if bytes.len() as u64 > MAX_MAP {
                return Err(bad(CMD, "the map is too large"));
            }
            Some((json!(String::from_utf8_lossy(&bytes)), name.unwrap_or_else(|| "code map".into())))
        }
        (None, None, None) => None,
    };
    let Some((json, name)) = map else { return Ok(status(s.doc()?, None, None)) };
    let check = crate::code_map::code_map(s, &json!({"json": json}))?;
    let imported = crate::code_connect::import(s, &json!({"json": json}))?;
    let st = status(s.doc()?, None, None);
    let kept = json!({"name": name.chars().take(MAX_FIELD).collect::<String>(), "linked": st["linked"], "missing": st["unlinked"].as_array().map_or(0, Vec::len)});
    let doc = s.ds_mut()?.doc_mut();
    let pages = doc.pages.clone();
    for (k, page) in pages.iter().enumerate() {
        let Some(n) = doc.node_mut(*page) else { continue };
        n.plugin_data.remove(CODE_MAP_KEY);
        if k == 0 {
            n.plugin_data.insert(CODE_MAP_KEY.to_string(), kept.to_string());
        }
    }
    Ok(status(s.doc()?, Some(check["counts"].clone()), Some(imported["warnings"].clone())))
}

/// What is linked: the map the file remembers, and its design-system components with and without
/// a React code component.
fn status(doc: &Document, check: Option<Value>, warnings: Option<Value>) -> Value {
    let ds = crate::code_map::ds_components(doc, false);
    let unlinked: Vec<&String> = ds
        .iter()
        .filter(|(_, ids)| ids.iter().all(|(id, _)| crate::code_connect::link(doc, *id, crate::codegen::CodeLang::React).is_none()))
        .map(|(name, _)| name)
        .collect();
    let mut out = json!({
        "map": linked_code_map(doc),
        "designSystem": ds.len(),
        "linked": ds.len() - unlinked.len(),
        "unlinked": unlinked.iter().take(MAX_UNLINKED).collect::<Vec<_>>(),
    });
    if let Some(c) = check {
        out["check"] = c;
    }
    if let Some(w) = warnings {
        out["warnings"] = w;
    }
    out
}

fn plugin_json(doc: &Document, id: NodeId, key: &str) -> Option<Value> {
    serde_json::from_str::<Value>(doc.node(id)?.plugin_data.get(key)?).ok().filter(Value::is_object)
}

fn export_chosen(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.exportCode";
    let id = get(p, "id").and_then(id_of).ok_or_else(|| bad(CMD, "`id` is required (the chosen proposal's screen)"))?;
    let dir = match get(p, "dir") {
        None | Some(Value::Null) => None,
        Some(Value::String(d)) if !d.trim().is_empty() => Some(d.clone()),
        Some(_) => return Err(bad(CMD, "dir: a folder path")),
    };
    let doc = s.doc()?;
    let Some(proposal) = plugin_json(doc, id, PROPOSAL_KEY) else { return Err(bad(CMD, "that layer is not a proposal on an exploration board")) };
    if proposal.get("status").and_then(Value::as_str) != Some("escolhida") {
        return Err(bad(CMD, "only the chosen proposal is exported: pick it with `explore.choose` first"));
    }
    let Some(review) = plugin_json(doc, id, REVIEW_KEY) else {
        return Err(bad(CMD, "the reviewer's verdict comes first: keep it with `explore.review`"));
    };
    let critique = plugin_json(doc, id, CRITIQUE_KEY).unwrap_or(Value::Null);
    let brand = match get(p, "collection") {
        None | Some(Value::Null) => None,
        Some(Value::String(k)) => Some(doc.variables.collection(k).ok_or_else(|| bad(CMD, format!("no collection {k}")))?.id.clone()),
        Some(_) => return Err(bad(CMD, "collection: a collection id or name")),
    };
    let sheet = crate::export_tsx::stylesheet(doc, &json!({}), CMD)?;
    let mut tsx = crate::export_tsx::export_with(doc, &[id], brand.as_deref());
    let code = tsx["code"].as_str().unwrap_or_default().to_string();
    tsx["code"] = json!(format!("{}{code}", summary(&proposal, &review, &critique)));
    let name = crate::component_codegen::ds_type_name(doc, id).unwrap_or_else(|| "Screen".into());
    let mut out = crate::export_tsx::code_files(&tsx, &sheet, &name, dir.as_deref())?;
    let findings = critique.get("findings").and_then(Value::as_array).cloned().unwrap_or_default();
    let open: Vec<Value> = findings.iter().filter(|f| is_open(f)).cloned().collect();
    if let Some(o) = out.as_object_mut() {
        o.insert("proposal".into(), proposal);
        o.insert("review".into(), review);
        o.insert("critique".into(), json!({"findings": findings.len(), "open": open}));
    }
    Ok(out)
}

/// A critique finding still to deal with: not resolved, not left to the designer's decision.
fn is_open(f: &Value) -> bool {
    !matches!(f.get("status").and_then(Value::as_str), Some("resolvido" | "decisão da designer"))
}

/// The comment the TSX starts with: what the developers need to know about the chosen proposal.
fn summary(proposal: &Value, review: &Value, critique: &Value) -> String {
    let t = |v: &Value, k: &str| -> String {
        let s: String = v.get(k).and_then(Value::as_str).unwrap_or_default().chars().take(MAX_FIELD).collect();
        s.replace("*/", "* /").replace(['\n', '\r'], " ")
    };
    let mut out = String::from("/*\n * Eureka: the proposal the designer chose, reviewed, for the developers.\n");
    out.push_str(&format!(
        " * Proposal {} · {} ({}), board \"{}\"\n",
        t(proposal, "letter"),
        t(proposal, "name"),
        t(proposal, "type"),
        t(proposal, "board")
    ));
    if !t(proposal, "summary").is_empty() {
        out.push_str(&format!(" *   {}\n", t(proposal, "summary")));
    }
    out.push_str(&format!(" * Verdict ({}): {}", t(review, "reviewer"), t(review, "verdict")));
    if !t(review, "summary").is_empty() {
        out.push_str(&format!(" · {}", t(review, "summary")));
    }
    out.push('\n');
    // A blocked verdict does not stop the export: the Gatekeeper advises, the designer decides.
    if t(review, "verdict").trim().eq_ignore_ascii_case("bloqueado") {
        out.push_str(" *   Exported although the verdict is \"bloqueado\": the verdict is advice; the designer chose this proposal and decides.\n");
    }
    let items = review.get("items").and_then(Value::as_array).cloned().unwrap_or_default();
    for i in items.iter().take(MAX_LISTED) {
        let source = t(i, "source");
        let source = if source.is_empty() { String::new() } else { format!(" [{source}]") };
        out.push_str(&format!(" *   - {}{source}\n", t(i, "message")));
    }
    if items.len() > MAX_LISTED {
        out.push_str(&format!(" *   … and {} more\n", items.len() - MAX_LISTED));
    }
    let findings = critique.get("findings").and_then(Value::as_array).cloned().unwrap_or_default();
    let mut open: Vec<&Value> = findings.iter().filter(|f| is_open(f)).collect();
    let rank =
        |f: &Value| crate::cmd::eureka_critique::SEVERITIES.iter().position(|s| Some(*s) == f.get("severity").and_then(Value::as_str)).unwrap_or(9);
    open.sort_by_key(|f| rank(f));
    match (critique.is_null(), open.len()) {
        (true, _) => out.push_str(" * Design critique: none kept\n"),
        (false, 0) => out.push_str(&format!(" * Design critique: {} finding(s), none open\n", findings.len())),
        (false, n) => {
            out.push_str(&format!(" * Design critique: {} finding(s), {n} open:\n", findings.len()));
            for f in open.iter().take(MAX_LISTED) {
                out.push_str(&format!(" *   - [{}] {}: {}\n", t(f, "severity"), t(f, "area"), t(f, "scenario")));
            }
            if n > MAX_LISTED {
                out.push_str(&format!(" *   … and {} more\n", n - MAX_LISTED));
            }
        }
    }
    out.push_str(" */\n");
    out
}
