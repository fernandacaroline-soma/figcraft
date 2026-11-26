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
/// Longest text the summary keeps per field.
const MAX_FIELD: usize = 400;

pub fn specs() -> Vec<CommandSpec> {
    vec![spec!(
        "explore.exportCode",
        "Export the chosen proposal as code",
        "{id (the chosen proposal's screen), dir? (folder to write the files into; else they come back as text), collection? (as export.tsx)}  once the designer has chosen the proposal (explore.choose) and the reviewer's verdict is in (explore.review), export.code of its screen with a summary for the developers at the top of the TSX: the proposal (letter, name, type, board, summary), the verdict (reviewer, verdict, summary, items) and the design critique (its findings still open, by severity). Read only. No dialog → export.code's result and {proposal, review, critique: {findings, open}}",
        false,
        export_chosen
    )]
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
