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
const PREVIEW_ID: &str = "eureka-proposal";

/// Commands (and prefixes) with effects outside the document, which a rollback can't undo.
const FORBIDDEN_PREFIXES: &[&str] =
    &["file.", "doc.", "plugin", "widget", "online.", "live.", "collab", "invite", "export.", "version.", "branch", "edit.copy", "edit.cut"];

/// What the session remembers for proposals.
#[derive(Default)]
pub(crate) struct State {
    next_cursor: u64,
    checkpoints: VecDeque<Checkpoint>,
    proposals: VecDeque<Proposal>,
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
            "{label?, onError?, commands: [{command, params, as?}]}  dry-run the batch (as `command.batch` with `dryRun`) and show the result on the canvas in place of the document, read-only; `proposal.previewEnd` goes back. Applying with `command.batch {ifUnchanged}` ends it → the dry-run diff",
            false,
            |s, p| {
                let (label, steps, cont) = parse_batch(p)?;
                end_proposal_preview(s)?;
                let (out, after) = dry_run(s, &label, &steps, cont)?;
                s.ds_mut()?.start_preview(PREVIEW_ID, &label, 0, after);
                Ok(out)
            }
        ),
        spec!("proposal.previewEnd", "End proposal preview", "{}  go back from a `proposal.preview` → {ended}", false, |s, _| {
            Ok(json!({"ended": end_proposal_preview(s)?}))
        }),
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
    if dry {
        end_proposal_preview(s)?;
        return dry_run(s, label, steps, cont).map(|(v, _)| v);
    }
    let Some(guard) = guard else { return Ok(Value::Array(run_steps(s, label, steps, cont)?)) };
    end_proposal_preview(s)?;
    apply_guarded(s, label, steps, cont, &guard)
}

/// Run the batch and roll it all back; the diff and the resulting document come out.
fn dry_run(s: &mut Session, label: &str, steps: &[BatchStep], cont: bool) -> Result<(Value, Arc<Document>)> {
    check_dry_steps(steps, 0)?;
    let started = Instant::now();
    let (uid, before) = {
        let ds = s.ds()?;
        (ds.uid, ds.doc.clone())
    };
    let mut seen: Option<(Vec<Value>, Arc<Document>)> = None;
    let run = s.grouped("Dry run", |s| {
        let results = run_steps(s, label, steps, cont)?;
        seen = Some((results, s.ds()?.doc.clone()));
        // Always fail: the group then restores the document and everything around it.
        Err::<(), _>(EngineError::Other("dry run".into()))
    });
    let Some((results, after)) = seen else {
        return Err(run.err().unwrap_or_else(|| EngineError::Other("the dry run did not finish".into())));
    };
    let diff = diff_docs(&before, &after)?;
    let hash = diff.hash();
    let mut out = diff.to_json(MAX_ENTRIES);
    if let Some(o) = out.as_object_mut() {
        o.insert("dryRun".into(), json!(true));
        o.insert("baseHash".into(), json!(hash));
        o.insert("results".into(), Value::Array(results));
        o.insert("elapsedMs".into(), json!(started.elapsed().as_secs_f64() * 1000.0));
    }
    if diff.nodes_before.len() <= MAX_REMEMBERED_NODES {
        s.eureka.remember(Proposal { hash, uid, nodes: diff.nodes_before, vars: diff.vars_before });
    }
    Ok((out, after))
}

/// Apply the batch for real as one undo step, unless the layers it touches changed since the dry
/// run that returned `guard`.
fn apply_guarded(s: &mut Session, label: &str, steps: &[BatchStep], cont: bool, guard: &str) -> Result<Value> {
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
    let mut applied: Option<(Vec<Value>, String)> = None;
    let run = s.grouped(label, |s| {
        let results = run_steps(s, label, steps, cont)?;
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
        applied = Some((results, hash));
        Ok(())
    });
    if let Some(c) = stale_err {
        return Err(stale(guard, c));
    }
    run?;
    s.eureka.proposals.retain(|p| p.hash != guard);
    let (results, hash) = applied.unwrap_or_default();
    Ok(json!({"applied": true, "baseHash": hash, "results": results}))
}

fn stale(hash: &str, conflicts: Value) -> EngineError {
    EngineError::Other(format!(
        "stale: the layers this proposal touches changed since the dry run (baseHash {hash}); nothing was applied. Run the dry run again. conflicts: {conflicts}"
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
        nodes_before: BTreeMap::new(),
        vars_before: BTreeMap::new(),
    };
    for id in ids {
        let (b, a) = (before.node(id).map(node_json), after.node(id).map(node_json));
        let name = |v: &Value| v.get("name").cloned().unwrap_or(Value::Null);
        match (&b, &a) {
            (None, Some(a)) => d.created.push(json!({"id": id.0, "name": name(a), "type": a.get("type").cloned().unwrap_or(Value::Null), "parent": a.get("parent").cloned().unwrap_or(Value::Null)})),
            (Some(b), None) => d.removed.push(json!({"id": id.0, "name": name(b)})),
            (Some(b), Some(a)) => {
                let props = props_diff(b, a);
                if props.as_object().is_none_or(Map::is_empty) {
                    continue;
                }
                d.changed.push(json!({"id": id.0, "name": name(a), "props": props}));
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
