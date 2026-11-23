//! Proposals: run a batch "in the dark" and show what it would change (`command.batch` with
//! `dryRun`), apply it only while the layers it touches are as the dry run saw them
//! (`ifUnchanged`), and tell what changed since a checkpoint (`document.checkpoint`,
//! `document.changesSince`). A dry run is a [`Session::grouped`] call that always fails: the group
//! puts back the document, the id allocator, the undo and redo steps and the journal, so nothing is
//! left behind. Property diffs compare the layers' serialised JSON, so they need no extra format.

use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::sync::Arc;
use std::time::Instant;

use figcraft_doc::{Document, NodeId};
use serde_json::{Map, Value, json};

use super::CommandSpec;
use crate::params::get;
use crate::{BatchStep, EngineError, Result, Session, bad, spec};

/// Checkpoints kept per session; the oldest goes first.
const MAX_CHECKPOINTS: usize = 8;
/// Dry runs remembered (to name the conflicts when a guarded apply is stale).
const MAX_PROPOSALS: usize = 16;
/// Layers listed per diff section; the rest is counted in `truncated`.
const MAX_ENTRIES: usize = 500;
/// More changed layers than this and a diff is refused.
const MAX_CHANGED: usize = 200_000;
/// A property value longer than this (as JSON text) is replaced by its size.
const MAX_VALUE_BYTES: usize = 4000;
/// Layers whose before state a remembered dry run keeps.
const MAX_REMEMBERED_NODES: usize = 5000;
/// Id of the document preview a proposal shows.
pub const PREVIEW_ID: &str = "eureka-proposal";

/// Commands (and prefixes) with effects outside the document, which a rollback can't undo.
const FORBIDDEN_PREFIXES: &[&str] =
    &["file.", "doc.", "plugin", "widget", "online.", "live.", "collab", "invite", "export.", "version.", "branch", "edit.copy", "edit.cut"];

/// What the session remembers for proposals.
#[derive(Default)]
pub(crate) struct State {
    next_cursor: u64,
    checkpoints: VecDeque<Checkpoint>,
    proposals: VecDeque<Proposal>,
    /// The last `proposal.preview`: its batch and baseHash, for `proposal.apply`.
    shown: Option<(Value, String)>,
    /// The document as Claude's last turn left it (a guarded apply or `proposal.turn`): what
    /// changed after it is the designer's own edit.
    turn: Option<(u64, Arc<Document>)>,
}

/// How a proposal treats what the designer edited by hand since Claude's last turn.
struct Manual {
    /// The document at Claude's last turn (`None`: nothing is protected).
    baseline: Option<Arc<Document>>,
    /// Where the baseline came from: "since" (a checkpoint), "turn", or none.
    source: Option<&'static str>,
    /// `keep: "proposal"`: the proposal wins everywhere.
    keep_proposal: bool,
    /// Layers where the proposal wins (`override`).
    overrides: BTreeSet<u64>,
}

struct Checkpoint {
    cursor: u64,
    uid: u64,
    doc: Arc<Document>,
}

struct Proposal {
    hash: String,
    uid: u64,
    nodes: BTreeMap<NodeId, Option<Value>>,
    vars: BTreeMap<String, Option<Value>>,
}

/// The difference between two versions of a document.
struct Diff {
    created: Vec<Value>,
    removed: Vec<Value>,
    changed: Vec<Value>,
    variables: Value,
    total: usize,
    /// Created or changed layers using a raw value where the design system has variables.
    warnings: Vec<Value>,
    /// The touched layers as they were before (`None`: not there yet).
    nodes_before: BTreeMap<NodeId, Option<Value>>,
    vars_before: BTreeMap<String, Option<Value>>,
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "document.checkpoint",
            "Checkpoint the document",
            "{}  remember the document as it is now (cheap: layers are shared) → {cursor, checkpoints}; `document.changesSince` tells what changed after it. At most 8 checkpoints are kept, the oldest expires first",
            false,
            |s, _| {
                let (uid, doc) = {
                    let ds = s.ds()?;
                    (ds.uid, ds.document_itself().clone())
                };
                let cursor = s.eureka.push_checkpoint(uid, doc);
                Ok(json!({"cursor": cursor, "checkpoints": s.eureka.checkpoints.len()}))
            }
        ),
        spec!(
            "document.changesSince",
            "Changes since a checkpoint",
            "{cursor, advance? (also take a new checkpoint and return it as `cursor`; default false), limit? (layers listed per section, default 500)}  → {cursor, created: [{id, name, type, parent}], removed: [{id, name}], changed: [{id, name, props: {key: [before, after]}}], variables?, total, truncated}",
            false,
            |s, p| {
                let cursor = match get(p, "cursor") {
                    Some(v) => v.as_u64().ok_or_else(|| bad("document.changesSince", "`cursor` must be a number from `document.checkpoint`"))?,
                    None => return Err(bad("document.changesSince", "`cursor` is required (from `document.checkpoint`)")),
                };
                let limit = limit_param("document.changesSince", p)?;
                let (uid, now) = {
                    let ds = s.ds()?;
                    (ds.uid, ds.document_itself().clone())
                };
                let before = s
                    .eureka
                    .checkpoints
                    .iter()
                    .find(|c| c.cursor == cursor)
                    .ok_or_else(|| bad("document.changesSince", "unknown or expired `cursor`: take a new `document.checkpoint`"))?;
                if before.uid != uid {
                    return Err(bad("document.changesSince", "that `cursor` belongs to another document"));
                }
                let diff = diff_docs(&before.doc, &now)?;
                let mut out = diff.to_json(limit);
                let advance = match get(p, "advance") {
                    None => false,
                    Some(Value::Bool(b)) => *b,
                    Some(_) => return Err(bad("document.changesSince", "`advance` must be true or false")),
                };
                let next = if advance { s.eureka.push_checkpoint(uid, now) } else { cursor };
                if let Some(o) = out.as_object_mut() {
                    o.insert("cursor".into(), json!(next));
                }
                Ok(out)
            }
        ),
        spec!(
            "proposal.preview",
            "Preview a proposal",
            "{label?, onError?, since?, keep?, override?, commands: [{command, params, as?}]}  dry-run the batch (as `command.batch` with `dryRun`) and show the result on the canvas in place of the document, read-only; `proposal.previewEnd` goes back, `proposal.apply` applies it (guarded by its baseHash) → the dry-run diff",
            false,
            |s, p| {
                let (label, steps, cont) = parse_batch(p)?;
                let policy = manual_policy(s, p)?;
                let (mut out, after) = dry_run(s, &label, &steps, cont, &policy)?;
                let hash = out.get("baseHash").and_then(Value::as_str).unwrap_or_default().to_string();
                s.ds_mut()?.start_preview(PREVIEW_ID, &label, 0, after);
                s.eureka.shown = Some((p.clone(), hash));
                if let Some(o) = out.as_object_mut() {
                    o.insert("preview".into(), json!(true));
                }
                Ok(out)
            }
        ),
        spec!(
            "proposal.previewEnd",
            "End proposal preview",
            "{}  go back from a `proposal.preview` (nothing was applied) → {ended}",
            false,
            |s, _| { Ok(json!({"ended": end_proposal_preview(s)?})) }
        ),
        spec!(
            "proposal.apply",
            "Apply the proposal",
            "{}  leave the `proposal.preview` and apply its batch as one undo step, only if the layers it touches are unchanged since the dry run (else a `stale` error; the preview is over either way)",
            false,
            |s, _| {
                let Some((params, hash)) = s.eureka.shown.take() else {
                    return Err(bad("proposal.apply", "no proposal to apply: show one with `proposal.preview` first"));
                };
                let (label, steps, cont) = parse_batch(&params)?;
                let policy = manual_policy(s, &params)?;
                apply_guarded(s, &label, &steps, cont, &hash, &policy)
            }
        ),
        spec!(
            "proposal.turn",
            "Mark Claude's turn",
            "{}  remember the document as it is now as Claude's last turn: what the designer edits after it is protected from later proposals (a guarded apply does this by itself) → {turn: true}",
            false,
            |s, _| {
                let (uid, doc) = {
                    let ds = s.ds()?;
                    (ds.uid, ds.doc.clone())
                };
                s.eureka.turn = Some((uid, doc));
                Ok(json!({"turn": true}))
            }
        ),
    ]
}

impl State {
    fn push_checkpoint(&mut self, uid: u64, doc: Arc<Document>) -> u64 {
        self.next_cursor += 1;
        let cursor = self.next_cursor;
        self.checkpoints.push_back(Checkpoint { cursor, uid, doc });
        while self.checkpoints.len() > MAX_CHECKPOINTS {
            self.checkpoints.pop_front();
        }
        cursor
    }

    fn remember(&mut self, p: Proposal) {
        self.proposals.retain(|q| q.hash != p.hash);
        self.proposals.push_back(p);
        while self.proposals.len() > MAX_PROPOSALS {
            self.proposals.pop_front();
        }
    }
}

fn limit_param(cmd: &str, p: &Value) -> Result<usize> {
    match get(p, "limit") {
        None => Ok(MAX_ENTRIES),
        Some(v) => match v.as_u64() {
            Some(n) if n >= 1 => Ok((n as usize).min(MAX_ENTRIES)),
            _ => Err(bad(cmd, "`limit` must be a positive number")),
        },
    }
}

/// Leave a proposal preview; true when one was showing.
fn end_proposal_preview(s: &mut Session) -> Result<bool> {
    let ds = s.ds_mut()?;
    if ds.preview.as_ref().is_some_and(|p| p.id == PREVIEW_ID) { Ok(ds.end_preview()) } else { Ok(false) }
}

/// `{label?, onError?, commands}` of a batch.
fn parse_batch(p: &Value) -> Result<(String, Vec<BatchStep>, bool)> {
    let cont = match get(p, "onError") {
        None => false,
        Some(Value::String(m)) if m == "abort" => false,
        Some(Value::String(m)) if m == "continue" => true,
        Some(_) => return Err(bad("command.batch", "`onError` must be \"abort\" or \"continue\"")),
    };
    let label = crate::params::string(p, "label").unwrap_or_else(|| "Batch".into());
    let Some(Value::Array(a)) = get(p, "commands") else { return Err(bad("command.batch", "`commands` must be an array")) };
    let mut steps = Vec::new();
    for (i, c) in a.iter().enumerate() {
        let cmd = c.get("command").and_then(Value::as_str).ok_or_else(|| bad("command.batch", format!("step {i} has no `command`")))?;
        crate::batch::refuse_unknown_step_keys(i, c)?;
        let params = c.get("params").cloned().unwrap_or(json!({}));
        steps.push(BatchStep { name: crate::batch::step_name(i, c)?, ..BatchStep::new(cmd, params) });
    }
    Ok((label, steps, cont))
}

/// Refuse a command whose effects a rollback can't take back (also inside nested batches).
fn check_dry_steps(steps: &[BatchStep], depth: usize) -> Result<()> {
    for st in steps {
        if FORBIDDEN_PREFIXES.iter().any(|f| st.command.starts_with(f)) {
            return Err(bad("command.batch", format!("`{}` has effects outside the document and can't run in a dry run", st.command)));
        }
        if st.command == "command.batch" {
            if depth > 8 {
                return Err(bad("command.batch", "batches are nested too deep for a dry run"));
            }
            if let Some(Value::Array(a)) = st.params.get("commands") {
                let inner: Vec<BatchStep> = a
                    .iter()
                    .filter_map(|c| Some(BatchStep::new(c.get("command")?.as_str()?, c.get("params").cloned().unwrap_or(Value::Null))))
                    .collect();
                check_dry_steps(&inner, depth + 1)?;
            }
        }
    }
    Ok(())
}

fn run_steps(s: &mut Session, label: &str, steps: &[BatchStep], cont: bool) -> Result<Vec<Value>> {
    if cont { s.batch_continue(label, steps) } else { s.batch(label, steps) }
}

/// `command.batch` with `dryRun` and/or `ifUnchanged` (called by that command; other batches never
/// get here).
pub(crate) fn run_batch(s: &mut Session, p: &Value, label: &str, steps: &[BatchStep], cont: bool) -> Result<Value> {
    let dry = match get(p, "dryRun") {
        None => false,
        Some(Value::Bool(b)) => *b,
        Some(_) => return Err(bad("command.batch", "`dryRun` must be true or false")),
    };
    let guard = match get(p, "ifUnchanged") {
        None => None,
        Some(Value::String(h)) if !h.is_empty() => Some(h.clone()),
        Some(_) => return Err(bad("command.batch", "`ifUnchanged` must be the `baseHash` a dry run returned")),
    };
    if dry && guard.is_some() {
        return Err(bad("command.batch", "`dryRun` and `ifUnchanged` can't be used together: dry-run first, then apply with `ifUnchanged`"));
    }
    if (dry || guard.is_some()) && s.ds()?.preview.is_some() {
        // Ending the preview here would let a failure roll the document back to the preview.
        return Err(bad("command.batch", "a preview is showing: leave it first (`proposal.previewEnd`, or `proposal.apply` for a proposal)"));
    }
    let Some(guard) = guard.filter(|_| !dry) else {
        if dry {
            let policy = manual_policy(s, p)?;
            return dry_run(s, label, steps, cont, &policy).map(|(v, _)| v);
        }
        return Ok(Value::Array(run_steps(s, label, steps, cont)?));
    };
    let policy = manual_policy(s, p)?;
    apply_guarded(s, label, steps, cont, &guard, &policy)
}

/// Run the batch and roll it all back; the diff and the resulting document come out.
fn dry_run(s: &mut Session, label: &str, steps: &[BatchStep], cont: bool, policy: &Manual) -> Result<(Value, Arc<Document>)> {
    check_dry_steps(steps, 0)?;
    let started = Instant::now();
    let (uid, before) = {
        let ds = s.ds()?;
        (ds.uid, ds.doc.clone())
    };
    let mut seen: Option<(Vec<Value>, Vec<Value>, Arc<Document>)> = None;
    let run = s.grouped("Dry run", |s| {
        let results = run_steps(s, label, steps, cont)?;
        let manual = resolve_manual(s, policy, &before)?;
        seen = Some((results, manual, s.ds()?.doc.clone()));
        // Always fail: the group then restores the document and everything around it.
        Err::<(), _>(EngineError::Other("dry run".into()))
    });
    let Some((results, manual, after)) = seen else {
        return Err(run.err().unwrap_or_else(|| EngineError::Other("the dry run did not finish".into())));
    };
    let diff = diff_docs(&before, &after)?;
    let hash = diff.hash();
    let mut out = diff.to_json(MAX_ENTRIES);
    if let Some(o) = out.as_object_mut() {
        o.insert("dryRun".into(), json!(true));
        o.insert("dsWarnings".into(), Value::Array(diff.warnings.clone()));
        o.insert("baseHash".into(), json!(hash));
        o.insert("results".into(), Value::Array(results));
        o.insert("manualBaseline".into(), json!(policy.source));
        o.insert("blocked".into(), json!(manual.iter().any(|c| c.get("kept") == Some(&json!("needs-choice")))));
        o.insert("manualConflicts".into(), Value::Array(manual));
        o.insert("elapsedMs".into(), json!(started.elapsed().as_secs_f64() * 1000.0));
    }
    if diff.nodes_before.len() <= MAX_REMEMBERED_NODES {
        s.eureka.remember(Proposal { hash, uid, nodes: diff.nodes_before, vars: diff.vars_before });
    }
    Ok((out, after))
}

/// Apply the batch for real as one undo step, unless the layers it touches changed since the dry
/// run that returned `guard`.
fn apply_guarded(s: &mut Session, label: &str, steps: &[BatchStep], cont: bool, guard: &str, policy: &Manual) -> Result<Value> {
    let (uid, before) = {
        let ds = s.ds()?;
        (ds.uid, ds.doc.clone())
    };
    // A dry run we remember names the layers that moved: check them before running anything.
    if let Some(p) = s.eureka.proposals.iter().find(|p| p.hash == guard && p.uid == uid) {
        let conflicts = conflicts_with(&before, p);
        if !conflicts.is_empty() {
            return Err(stale(guard, json!(conflicts)));
        }
    }
    let mut stale_err: Option<Value> = None;
    let mut applied: Option<(Vec<Value>, Vec<Value>, String)> = None;
    let run = s.grouped(label, |s| {
        let results = run_steps(s, label, steps, cont)?;
        let manual = resolve_manual(s, policy, &before)?;
        if let Some(c) = manual.iter().find(|c| c.get("kept") == Some(&json!("needs-choice"))) {
            return Err(bad(
                "proposal.apply",
                format!(
                    "the proposal removes {} , which the designer edited by hand; nothing was applied. Preview again with `override: [{}]` to let the proposal win",
                    c.get("name").map_or_else(String::new, Value::to_string),
                    c.get("id").map_or_else(String::new, Value::to_string)
                ),
            ));
        }
        let after = s.ds()?.doc.clone();
        let diff = diff_docs(&before, &after)?;
        let hash = diff.hash();
        if hash != guard {
            let known = s.eureka.proposals.iter().find(|p| p.hash == guard && p.uid == uid);
            stale_err = Some(match known {
                Some(p) => json!(conflicts_with(&before, p)),
                None => json!({"touched": diff.nodes_before.keys().map(|k| k.0).collect::<Vec<_>>(), "note": "unknown or expired baseHash"}),
            });
            return Err(EngineError::Other("stale".into()));
        }
        applied = Some((results, manual, hash));
        Ok(())
    });
    if let Some(c) = stale_err {
        return Err(stale(guard, c));
    }
    run?;
    s.eureka.proposals.retain(|p| p.hash != guard);
    s.eureka.turn = Some((uid, s.ds()?.doc.clone()));
    let (results, manual, hash) = applied.unwrap_or_default();
    Ok(json!({"applied": true, "baseHash": hash, "results": results, "manualConflicts": manual}))
}

/// `{since?, keep?, override?}` of a proposal: which baseline counts as Claude's last turn (the
/// `since` checkpoint, else the session's last turn) and who wins where both changed something.
fn manual_policy(s: &Session, p: &Value) -> Result<Manual> {
    let cmd = "proposal";
    let uid = s.ds()?.uid;
    let (baseline, source) = match get(p, "since") {
        Some(v) => {
            let cursor = v.as_u64().ok_or_else(|| bad(cmd, "`since` must be a cursor from `document.checkpoint`"))?;
            let c = s
                .eureka
                .checkpoints
                .iter()
                .find(|c| c.cursor == cursor)
                .ok_or_else(|| bad(cmd, "unknown or expired `since` cursor: take a new `document.checkpoint`"))?;
            if c.uid != uid {
                return Err(bad(cmd, "that `since` cursor belongs to another document"));
            }
            (Some(c.doc.clone()), Some("since"))
        }
        None => match &s.eureka.turn {
            Some((u, d)) if *u == uid => (Some(d.clone()), Some("turn")),
            _ => (None, None),
        },
    };
    let keep_proposal = match get(p, "keep") {
        None => false,
        Some(Value::String(k)) if k == "manual" => false,
        Some(Value::String(k)) if k == "proposal" => true,
        Some(_) => return Err(bad(cmd, "`keep` must be \"manual\" or \"proposal\"")),
    };
    let overrides = match get(p, "override") {
        None => BTreeSet::new(),
        Some(Value::Array(a)) if a.len() <= MAX_ENTRIES => {
            a.iter().map(|v| v.as_u64().ok_or_else(|| bad(cmd, "`override` lists layer ids (numbers)"))).collect::<Result<_>>()?
        }
        Some(_) => return Err(bad(cmd, "`override` must be a list of at most 500 layer ids")),
    };
    Ok(Manual { baseline, source, keep_proposal, overrides })
}

/// The edit group a binding belongs with: binding `fill` goes with `fills`, `fontSize` with the
/// text `style`, `gap` and padding with `layout`, radii with `corners`.
fn bound_group(k: &str) -> String {
    let g = if k.starts_with("fill") {
        "fills"
    } else if k.starts_with("stroke") && k.ends_with("Weight") {
        "stroke"
    } else if k.starts_with("stroke") {
        "strokes"
    } else if k.starts_with("effect") {
        "effects"
    } else if k.starts_with("font") || k.starts_with("line") || k.starts_with("letter") || k.starts_with("paragraph") {
        "style"
    } else if k.ends_with("Gap") || k == "gap" || k.starts_with("padding") {
        "layout"
    } else if k.ends_with("Radius") {
        "corners"
    } else {
        k
    };
    g.to_string()
}

/// A layer's editable aspects: its top-level keys (bindings one by one, as `bound.<field>`),
/// without what only follows from layout.
fn aspects(v: Option<&Value>) -> BTreeMap<String, Value> {
    let mut out = BTreeMap::new();
    let Some(o) = v.and_then(Value::as_object) else { return out };
    for (k, val) in o {
        if k == "id" || LAYOUT_KEYS.contains(&k.as_str()) {
            continue;
        }
        if k == "bound" {
            for (bk, bv) in val.as_object().into_iter().flatten() {
                out.insert(format!("bound.{bk}"), bv.clone());
            }
        } else {
            out.insert(k.clone(), val.clone());
        }
    }
    out
}

fn group_of(aspect: &str) -> String {
    aspect.strip_prefix("bound.").map_or_else(|| aspect.to_string(), bound_group)
}

/// The edit groups that differ between two versions of a layer.
fn changed_groups(a: &BTreeMap<String, Value>, b: &BTreeMap<String, Value>) -> BTreeSet<String> {
    a.keys().chain(b.keys()).filter(|k| a.get(*k) != b.get(*k)).map(|k| group_of(k)).collect()
}

/// After a proposal's steps ran on `before`: where they change what the designer edited by hand
/// since the baseline, put the hand-made values back (unless the proposal wins there) and say so.
/// A layer the proposal removes can't be kept that way: it is reported as `needs-choice`.
fn resolve_manual(s: &mut Session, m: &Manual, before: &Document) -> Result<Vec<Value>> {
    let Some(base) = &m.baseline else { return Ok(Vec::new()) };
    let after = s.ds()?.doc.clone();
    let ids = after.changed_ids(before, MAX_CHANGED).ok_or_else(|| EngineError::Other("too many layers changed to list".into()))?;
    let names: BTreeMap<String, String> =
        variable_map(&after).into_iter().chain(variable_map(before)).filter_map(|(id, v)| Some((id, v.get("name")?.as_str()?.to_string()))).collect();
    let (mut out, mut restored) = (Vec::new(), false);
    for id in ids {
        let Some(b) = before.node(id).map(node_json) else { continue };
        let was = base.node(id).map(node_json);
        let (ab, bb) = (aspects(Some(&b)), aspects(was.as_ref()));
        // A layer made by hand after the baseline is all the designer's.
        let manual: BTreeSet<String> = if was.is_none() { ab.keys().map(|k| group_of(k)).collect() } else { changed_groups(&bb, &ab) };
        if manual.is_empty() {
            continue;
        }
        let proposal_wins = m.keep_proposal || m.overrides.contains(&id.0);
        let name = b.get("name").cloned().unwrap_or(Value::Null);
        let Some(a) = after.node(id).map(node_json) else {
            let kept = if proposal_wins { "proposal" } else { "needs-choice" };
            out.push(json!({"id": id.0, "name": name, "removed": true, "kept": kept}));
            continue;
        };
        let aa = aspects(Some(&a));
        let both: BTreeSet<String> = changed_groups(&ab, &aa).intersection(&manual).cloned().collect();
        if both.is_empty() {
            continue;
        }
        let keys: Vec<&str> = both.iter().map(String::as_str).chain(["bound"]).collect();
        let (by_hand, _) = describe(was.as_ref().unwrap_or(&Value::Null), &b, &names, Some(&keys));
        let (by_claude, _) = describe(&b, &a, &names, Some(&keys));
        out.push(json!({
            "id": id.0, "name": name, "props": both, "kept": if proposal_wins { "proposal" } else { "manual" },
            "manual": by_hand, "proposal": by_claude,
        }));
        if proposal_wins {
            continue;
        }
        // Back to the hand-made values for every aspect in the contested groups.
        let mut merged = a.as_object().cloned().unwrap_or_default();
        for key in ab.keys().chain(aa.keys()).collect::<BTreeSet<_>>() {
            if !both.contains(&group_of(key)) {
                continue;
            }
            if let Some(bk) = key.strip_prefix("bound.") {
                let bound = merged.entry("bound").or_insert_with(|| json!({}));
                if let Some(bo) = bound.as_object_mut() {
                    match b.get("bound").and_then(|x| x.get(bk)) {
                        Some(v) => bo.insert(bk.to_string(), v.clone()),
                        None => bo.remove(bk),
                    };
                }
            } else {
                match b.get(key) {
                    Some(v) => merged.insert(key.clone(), v.clone()),
                    None => merged.remove(key),
                };
            }
        }
        let node: figcraft_doc::Node = serde_json::from_value(Value::Object(merged))
            .map_err(|e| EngineError::Other(format!("could not keep the hand edit on layer {}: {e}", id.0)))?;
        let ds = s.ds_mut()?;
        let slot = ds.doc_mut().node_mut(id).ok_or_else(|| EngineError::Other(format!("layer {} is gone", id.0)))?;
        *slot = node;
        restored = true;
    }
    if restored {
        let ds = s.ds_mut()?;
        crate::normalize(ds.doc_mut(), before);
    }
    Ok(out)
}

fn stale(hash: &str, conflicts: Value) -> EngineError {
    let text = match conflicts.as_array() {
        Some(list) => list
            .iter()
            .map(|c| {
                let what = match (c.get("props").and_then(Value::as_object), c.get("removed"), c.get("created")) {
                    (Some(p), _, _) => p.keys().cloned().collect::<Vec<_>>().join(", "),
                    (_, Some(_), _) => "removed".to_string(),
                    _ => "created".to_string(),
                };
                format!(
                    "{} #{} [{what}]",
                    c.get("name").and_then(Value::as_str).unwrap_or("?"),
                    c.get("id").map_or(String::new(), |i| i.to_string().trim_matches('"').to_string())
                )
            })
            .collect::<Vec<_>>()
            .join("; "),
        None => conflicts.to_string(),
    };
    EngineError::Other(format!(
        "stale: the layers this proposal touches changed since the dry run (baseHash {hash}); nothing was applied. Run the dry run again. conflicts: {text}"
    ))
}

/// The remembered layers and variables that are no longer as the dry run saw them.
fn conflicts_with(now: &Document, p: &Proposal) -> Vec<Value> {
    let mut out = Vec::new();
    for (id, was) in &p.nodes {
        let cur = now.node(*id).map(node_json);
        if cur.as_ref() == was.as_ref() {
            continue;
        }
        out.push(conflict(id.0.to_string(), was.as_ref(), cur.as_ref()));
    }
    let cur_vars = variable_map(now);
    for (id, was) in &p.vars {
        let cur = cur_vars.get(id);
        if cur == was.as_ref() {
            continue;
        }
        out.push(conflict(format!("variable {id}"), was.as_ref(), cur));
    }
    out
}

fn conflict(id: String, was: Option<&Value>, cur: Option<&Value>) -> Value {
    let name = cur.or(was).and_then(|v| v.get("name")).cloned().unwrap_or(Value::Null);
    match (was, cur) {
        (Some(a), Some(b)) => json!({"id": id, "name": name, "props": props_diff(a, b)}),
        (Some(_), None) => json!({"id": id, "name": name, "removed": true}),
        _ => json!({"id": id, "name": name, "created": true}),
    }
}

fn node_json(n: &figcraft_doc::Node) -> Value {
    serde_json::to_value(n).unwrap_or(Value::Null)
}

fn variable_map(d: &Document) -> BTreeMap<String, Value> {
    d.variables
        .variables
        .iter()
        .filter_map(|v| {
            let j = serde_json::to_value(v).ok()?;
            Some((j.get("id")?.as_str()?.to_string(), j))
        })
        .collect()
}

fn clip(v: &Value) -> Value {
    match serde_json::to_string(v) {
        Ok(t) if t.len() > MAX_VALUE_BYTES => json!({"$clipped": t.len()}),
        _ => v.clone(),
    }
}

/// `{key: [before, after]}` for the keys whose value differs.
fn props_diff(a: &Value, b: &Value) -> Value {
    let (empty_a, empty_b) = (Map::new(), Map::new());
    let (ma, mb) = (a.as_object().unwrap_or(&empty_a), b.as_object().unwrap_or(&empty_b));
    let keys: BTreeSet<&String> = ma.keys().chain(mb.keys()).collect();
    let mut out = Map::new();
    for k in keys {
        let (x, y) = (ma.get(k), mb.get(k));
        if x != y {
            out.insert(k.clone(), json!([x.map_or(Value::Null, clip), y.map_or(Value::Null, clip)]));
        }
    }
    Value::Object(out)
}

fn diff_docs(before: &Document, after: &Document) -> Result<Diff> {
    let ids = after.changed_ids(before, MAX_CHANGED).ok_or_else(|| EngineError::Other("too many layers changed to list".into()))?;
    let mut d = Diff {
        created: Vec::new(),
        removed: Vec::new(),
        changed: Vec::new(),
        variables: Value::Null,
        total: 0,
        warnings: Vec::new(),
        nodes_before: BTreeMap::new(),
        vars_before: BTreeMap::new(),
    };
    let names: BTreeMap<String, String> =
        variable_map(after).into_iter().chain(variable_map(before)).filter_map(|(id, v)| Some((id, v.get("name")?.as_str()?.to_string()))).collect();
    for id in ids {
        let (b, a) = (before.node(id).map(node_json), after.node(id).map(node_json));
        let name = |v: &Value| v.get("name").cloned().unwrap_or(Value::Null);
        match (&b, &a) {
            (None, Some(a)) => {
                let (summary, _) = describe(&Value::Null, a, &names, Some(&["characters", "fills", "strokes", "bound"]));
                d.warnings.extend(ds_warnings(id, a, None));
                d.created.push(json!({"id": id.0, "name": name(a), "type": a.get("type").cloned().unwrap_or(Value::Null), "parent": a.get("parent").cloned().unwrap_or(Value::Null), "summary": summary}));
            }
            (Some(b), None) => d.removed.push(json!({"id": id.0, "name": name(b)})),
            (Some(b), Some(a)) => {
                let props = props_diff(b, a);
                let Some(changed_keys) = props.as_object().filter(|m| !m.is_empty()) else { continue };
                let (summary, layout) = describe(b, a, &names, None);
                d.warnings.extend(ds_warnings(id, a, Some(changed_keys)));
                let mut e = json!({"id": id.0, "name": name(a), "props": props, "summary": summary});
                if let (Some(o), false) = (e.as_object_mut(), layout.is_empty()) {
                    o.insert("layout".into(), Value::Object(layout));
                }
                d.changed.push(e);
            }
            (None, None) => continue,
        }
        d.nodes_before.insert(id, b);
    }
    d.total = d.created.len() + d.removed.len() + d.changed.len();
    if before.variables != after.variables {
        let (vb, va) = (variable_map(before), variable_map(after));
        let (mut created, mut removed, mut changed) = (Vec::new(), Vec::new(), Vec::new());
        for (id, a) in &va {
            let name = a.get("name").cloned().unwrap_or(Value::Null);
            match vb.get(id) {
                None => {
                    created.push(json!({"id": id, "name": name}));
                    d.vars_before.insert(id.clone(), None);
                }
                Some(b) if b != a => {
                    changed.push(json!({"id": id, "name": name, "props": props_diff(b, a)}));
                    d.vars_before.insert(id.clone(), Some(b.clone()));
                }
                Some(_) => {}
            }
        }
        for (id, b) in &vb {
            if !va.contains_key(id) {
                removed.push(json!({"id": id, "name": b.get("name").cloned().unwrap_or(Value::Null)}));
                d.vars_before.insert(id.clone(), Some(b.clone()));
            }
        }
        let collections = before.variables.collections != after.variables.collections;
        if !(created.is_empty() && removed.is_empty() && changed.is_empty() && !collections) {
            d.total += created.len() + removed.len() + changed.len();
            d.variables = json!({"created": created, "removed": removed, "changed": changed, "collectionsChanged": collections});
        }
    }
    Ok(d)
}

impl Diff {
    fn to_json(&self, limit: usize) -> Value {
        let cut = |v: &Vec<Value>| Value::Array(v.iter().take(limit).cloned().collect());
        let truncated = self.created.len().max(self.removed.len()).max(self.changed.len()) > limit;
        let mut o = Map::new();
        o.insert("created".into(), cut(&self.created));
        o.insert("removed".into(), cut(&self.removed));
        o.insert("changed".into(), cut(&self.changed));
        if !self.variables.is_null() {
            o.insert("variables".into(), self.variables.clone());
        }
        o.insert("total".into(), json!(self.total));
        o.insert("truncated".into(), json!(truncated));
        Value::Object(o)
    }

    /// A stable hash of the touched layers and variables as they were before.
    fn hash(&self) -> String {
        let mut h = 0xcbf2_9ce4_8422_2325_u64;
        let mut feed = |bytes: &[u8]| {
            for b in bytes {
                h ^= u64::from(*b);
                h = h.wrapping_mul(0x0100_0000_01b3);
            }
            h ^= 0xff;
            h = h.wrapping_mul(0x0100_0000_01b3);
        };
        for (id, v) in &self.nodes_before {
            feed(id.0.to_string().as_bytes());
            feed(v.as_ref().map_or(String::new(), Value::to_string).as_bytes());
        }
        feed(b"vars");
        for (id, v) in &self.vars_before {
            feed(id.as_bytes());
            feed(v.as_ref().map_or(String::new(), Value::to_string).as_bytes());
        }
        format!("{h:016x}")
    }
}

/// Keys that follow from layout (position, size, children), kept apart from what a person edited.
const LAYOUT_KEYS: &[&str] = &["transform", "width", "height", "children", "overrides"];

/// The design-system hint for `fills`, `strokes` and the like: `(prop, value)` pairs of raw values
/// on layer `a` that no variable binds. Only the aspects in `changed` are looked at (all for a
/// created layer, `None`). A hint, never an error.
fn ds_warnings(id: NodeId, a: &Value, changed: Option<&Map<String, Value>>) -> Vec<Value> {
    let touched = |key: &str| changed.is_none_or(|m| m.contains_key(key));
    let bound = |prefix: &str| a.get("bound").and_then(Value::as_object).is_some_and(|b| b.keys().any(|k| k.starts_with(prefix)));
    let name = a.get("name").cloned().unwrap_or(Value::Null);
    let mut out = Vec::new();
    let mut warn = |prop: &str, value: Value| out.push(json!({"id": id.0, "name": name, "prop": prop, "value": value}));
    for (key, prop, prefix) in [("fills", "fill", "fill"), ("strokes", "stroke", "stroke")] {
        if touched(key)
            && !bound(prefix)
            && let Some(paint) = a.get(key).and_then(Value::as_array).and_then(|l| l.iter().find(|p| paint_visible_solid(p)))
        {
            warn(prop, json!(paint_hex(paint).unwrap_or_default()));
        }
    }
    if a.get("type").and_then(Value::as_str) == Some("text") && touched("style") && !bound("fontSize") {
        let size = a.get("style").and_then(|s| s.get("fontSize").or_else(|| s.get("font_size"))).and_then(Value::as_f64);
        if let Some(size) = size {
            warn("fontSize", json!(size));
        }
    }
    if let (true, Some(l)) = (touched("layout"), a.get("layout").filter(|l| l.is_object())) {
        let gap = l.get("gap").and_then(Value::as_f64).unwrap_or(0.0);
        if gap > 0.0 && !bound("gap") {
            warn("gap", json!(gap));
        }
        let pad: Vec<f64> = l.get("padding").and_then(Value::as_array).map(|p| p.iter().filter_map(Value::as_f64).collect()).unwrap_or_default();
        if pad.iter().any(|p| *p > 0.0) && !bound("padding") {
            warn("padding", json!(pad));
        }
    }
    if touched("corners") && !bound("cornerRadius") {
        let radii: Vec<f64> = a
            .get("corners")
            .and_then(|c| c.get("radii"))
            .and_then(Value::as_array)
            .map(|p| p.iter().filter_map(Value::as_f64).collect())
            .unwrap_or_default();
        if radii.iter().any(|r| *r > 0.0) {
            warn("cornerRadius", json!(radii));
        }
    }
    out
}

fn paint_visible_solid(p: &Value) -> bool {
    p.get("type").and_then(Value::as_str) == Some("solid") && p.get("visible").and_then(Value::as_bool).unwrap_or(true)
}

/// `#RRGGBB` of a paint's colour.
fn paint_hex(p: &Value) -> Option<String> {
    let c = p.get("color")?;
    let ch = |k: &str| c.get(k).and_then(Value::as_f64).map(|v| (v * 255.0).round().clamp(0.0, 255.0) as u8);
    Some(format!("#{:02X}{:02X}{:02X}", ch("r")?, ch("g")?, ch("b")?))
}

/// A paint list as a person says it: `#7A4B2A`, `#0D0D0D (50% de opacidade)`, `nenhum`.
fn paints_text(v: &Value) -> String {
    let Some(list) = v.as_array() else { return val_text(v) };
    if list.is_empty() {
        return "nenhum".into();
    }
    list.iter()
        .map(|p| match (paint_visible_solid(p), paint_hex(p)) {
            (true, Some(hex)) => {
                let alpha = p.get("color").and_then(|c| c.get("a")).and_then(Value::as_f64).unwrap_or(1.0)
                    * p.get("opacity").and_then(Value::as_f64).unwrap_or(1.0);
                if alpha < 0.995 { format!("{hex} ({}% de opacidade)", (alpha * 100.0).round()) } else { hex }
            }
            _ => p.get("type").and_then(Value::as_str).unwrap_or("paint").to_string(),
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn num_text(x: f64) -> String {
    let r = (x * 100.0).round() / 100.0;
    if r.fract() == 0.0 { format!("{r:.0}") } else { format!("{r}") }
}

fn val_text(v: &Value) -> String {
    match v {
        Value::Null => "—".into(),
        Value::Bool(b) => if *b { "sim" } else { "não" }.into(),
        Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), num_text),
        Value::String(s) => format!("\"{s}\""),
        other => {
            let t = other.to_string();
            if t.chars().count() > 60 { format!("{}…", t.chars().take(60).collect::<String>()) } else { t }
        }
    }
}

fn label(key: &str) -> String {
    match key {
        "name" => "Nome",
        "characters" => "Texto",
        "fills" => "Preenchimento",
        "strokes" => "Contorno",
        "opacity" => "Opacidade",
        "visible" => "Visível",
        "locked" => "Bloqueado",
        "effects" => "Efeitos",
        "corners" => "Cantos",
        "radii" => "raios",
        "layout" => "Auto layout",
        "style" => "Texto",
        "fontSize" | "font_size" => "tamanho da fonte",
        "gap" => "espaçamento (gap)",
        "padding" => "padding",
        "mode" => "direção",
        "stroke" => "Contorno",
        other => other,
    }
    .to_string()
}

fn bound_label(key: &str) -> String {
    match key {
        "fill" => "preenchimento".into(),
        "stroke" => "contorno".into(),
        "fontSize" => "tamanho da fonte".into(),
        "cornerRadius" => "raio".into(),
        "gap" => "espaçamento (gap)".into(),
        "paddingHorizontal" => "padding horizontal".into(),
        "paddingVertical" => "padding vertical".into(),
        "characters" => "texto".into(),
        other => other.replace("fills/", "preenchimento ").replace("strokes/", "contorno ").replace("/color", ""),
    }
}

/// What changed on one layer, for a designer: lines of text (colours as hex, bindings by variable
/// name, text in quotes) and, apart, the values that only follow from layout. `only` limits the
/// keys looked at (a created layer).
fn describe(b: &Value, a: &Value, names: &BTreeMap<String, String>, only: Option<&[&str]>) -> (Vec<String>, Map<String, Value>) {
    let (empty_a, empty_b) = (Map::new(), Map::new());
    let (mb, ma) = (b.as_object().unwrap_or(&empty_b), a.as_object().unwrap_or(&empty_a));
    let keys: BTreeSet<&String> = ma.keys().chain(mb.keys()).collect();
    let (mut lines, mut layout) = (Vec::new(), Map::new());
    let var = |v: Option<&Value>| match v.and_then(Value::as_str) {
        Some(id) => names.get(id).cloned().unwrap_or_else(|| id.to_string()),
        None => "sem vínculo".into(),
    };
    for k in keys {
        if only.is_some_and(|o| !o.contains(&k.as_str())) {
            continue;
        }
        let (x, y) = (mb.get(k), ma.get(k));
        if x == y {
            continue;
        }
        let (xv, yv) = (x.cloned().unwrap_or(Value::Null), y.cloned().unwrap_or(Value::Null));
        if LAYOUT_KEYS.contains(&k.as_str()) {
            layout.insert(k.clone(), json!([clip(&xv), clip(&yv)]));
        } else if k == "bound" {
            let (empty_x, empty_y) = (Map::new(), Map::new());
            let (bx, by) = (xv.as_object().unwrap_or(&empty_x), yv.as_object().unwrap_or(&empty_y));
            let bkeys: BTreeSet<&String> = bx.keys().chain(by.keys()).collect();
            for bk in bkeys {
                if bx.get(bk) != by.get(bk) {
                    lines.push(format!("Vínculo de {}: {} → {}", bound_label(bk), var(bx.get(bk)), var(by.get(bk))));
                }
            }
        } else if k == "fills" || k == "strokes" {
            lines.push(format!("{}: {} → {}", label(k), paints_text(&xv), paints_text(&yv)));
        } else if xv.is_object() || yv.is_object() {
            let (empty_x, empty_y) = (Map::new(), Map::new());
            let (ox, oy) = (xv.as_object().unwrap_or(&empty_x), yv.as_object().unwrap_or(&empty_y));
            let skeys: BTreeSet<&String> = ox.keys().chain(oy.keys()).collect();
            for sk in skeys {
                if ox.get(sk) != oy.get(sk) {
                    lines.push(format!(
                        "{} · {}: {} → {}",
                        label(k),
                        label(sk),
                        val_text(ox.get(sk).unwrap_or(&Value::Null)),
                        val_text(oy.get(sk).unwrap_or(&Value::Null))
                    ));
                }
            }
        } else {
            lines.push(format!("{}: {} → {}", label(k), val_text(&xv), val_text(&yv)));
        }
    }
    (lines, layout)
}
