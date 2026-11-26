//! Exploration board (Eureka G3, Frente I): the proposals a generator such as the UX proposals
//! generator ("gerador-propostas") produces for a problem, drawn side by side on the canvas so the
//! designer compares and picks one. `explore.board` draws them in one section as one undo step,
//! each proposal's screen carrying what it is (`pluginData["eureka.proposal"]`); `explore.choose`
//! marks the one the designer picked, which then goes through the proposal card, the DS lint, the
//! design critique (`explore.critique`, which the designer reviews) and the Gatekeeper like any
//! other refinement. Nothing here decides for the designer, and nothing is deleted: the proposals
//! not picked stay on the board as the round's record.

use serde_json::{Map, Value, json};

use super::CommandSpec;
use crate::params::{get, id_of};
use crate::{BatchStep, EngineError, NodeId, Result, Session, bad, spec};

/// The `pluginData` key a proposal's screen carries its description under.
pub const PROPOSAL_KEY: &str = "eureka.proposal";
/// The `pluginData` key a chosen proposal's screen keeps its reviewer's verdict under.
pub const REVIEW_KEY: &str = "eureka.review";
/// Most lint findings `explore.choose` returns for the chosen screen.
const MAX_FINDINGS: usize = 200;
/// Fewest and most proposals a board takes (the generator makes 3 to 10).
const MIN_PROPOSALS: usize = 2;
const MAX_PROPOSALS: usize = 12;
/// Most commands one proposal's screen may take to draw.
const MAX_STEPS: usize = 400;
/// Room between screens on the board, and between the board and what is already on the page.
const GAP: f64 = 120.0;
/// Longest text kept per field.
const MAX_TEXT: usize = 2000;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "explore.board",
            "Exploration board",
            "{title, problem?, brand?, platform?, journey?, proposals: [{name, type (\"incremental\" | \"disruptiva\"), mechanism?, summary?, confidence? (\"alta\" | \"média\" | \"baixa\"), commands: [{command, params, as?}] (draw ONE screen: its first created layer is the screen) | screen (the screen described by its layers: {frame, width, height, layout?, gap?, padding?, fill?, children: [{frame…} | {text, size?, color?, wrap?} | {image, height, fill?} | {component, variant?: {prop: value}, props?: {name: value}, layers?: {layer: {name: value}}, slot?: [layers], width?: \"fill\", dock?: \"bottom\"}]}; names as `explore.catalog` lists them, sizes and colours as variable names)}]}  draw 2 to 12 proposals side by side in a new section to the right of the page's content, as one undo step; each screen is named \"A · Incremental · <name>\" and keeps the proposal's description (`pluginData[\"eureka.proposal\"]`) → {section, proposals: [{letter, id, name}]}",
            // Not undoable as a command: the board groups itself into its own single undo step.
            false,
            board
        ),
        spec!(
            "explore.choose",
            "Choose a proposal",
            "{id (a proposal's screen)}  mark the proposal the designer picked on its board (the others are kept, marked not chosen) and select it; returns what a reviewer needs next: the DS lint of the screen and everything in it, and its context (kind of each layer, component of each instance, brand mode of the screen) → {chosen: {id, name, proposal}, others, lint, context}",
            true,
            choose
        ),
        spec!(
            "explore.review",
            "Review the chosen proposal",
            "{id (the chosen proposal's screen), verdict (\"aprovado\" | \"com-ressalvas\" | \"bloqueado\"), reviewer?, summary?, items?: [{message, source?, ids?}]}  keep a reviewer's verdict (such as the DS Gatekeeper's, in preview mode) on the chosen proposal, once the designer has reviewed its design critique (`explore.critique`); it only advises: the designer decides → {review}",
            true,
            review
        ),
    ]
}

/// How a proposal's screen is drawn: described by its layers, or by raw commands.
enum Draw {
    Screen(super::eureka_screen::Item),
    Commands(Vec<BatchStep>),
}

/// An error's reason without the command name `bad` puts in front (it is added again).
fn reason(e: &EngineError) -> String {
    let t = e.to_string();
    t.split_once(": ").map_or(t.clone(), |(_, r)| r.to_string())
}

fn text(p: &Value, key: &str, what: &str) -> Result<Option<String>> {
    match p.get(key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(t)) if t.chars().count() <= MAX_TEXT => Ok(Some(t.trim().to_string()).filter(|t| !t.is_empty())),
        Some(_) => Err(bad("explore.board", format!("`{what}` must be text of at most {MAX_TEXT} characters"))),
    }
}

fn letter(i: usize) -> String {
    char::from_u32(u32::from(b'A') + u32::try_from(i).unwrap_or(0)).map_or_else(|| format!("{}", i + 1), String::from)
}

/// `explore.board`.
fn board(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.board";
    let title = text(p, "title", "title")?.ok_or_else(|| bad(CMD, "`title` is required"))?;
    let context: Map<String, Value> =
        ["problem", "brand", "platform", "journey"].iter().map(|k| text(p, k, k).map(|v| ((*k).to_string(), json!(v)))).collect::<Result<_>>()?;
    let Some(Value::Array(list)) = get(p, "proposals") else { return Err(bad(CMD, "`proposals` must be a list")) };
    if list.len() < MIN_PROPOSALS || list.len() > MAX_PROPOSALS {
        return Err(bad(CMD, format!("a board takes {MIN_PROPOSALS} to {MAX_PROPOSALS} proposals")));
    }
    // Read every proposal before drawing anything: a bad one refuses the whole board.
    let mut parsed = Vec::with_capacity(list.len());
    for (i, item) in list.iter().enumerate() {
        let name = text(item, "name", "proposals[].name")?.ok_or_else(|| bad(CMD, format!("proposal {} has no `name`", letter(i))))?;
        let kind = match item.get("type").and_then(Value::as_str) {
            Some("incremental") => "Incremental",
            Some("disruptiva") => "Disruptiva",
            _ => return Err(bad(CMD, "`proposals[].type` must be \"incremental\" or \"disruptiva\"")),
        };
        let confidence = match item.get("confidence") {
            None | Some(Value::Null) => None,
            Some(Value::String(c)) if matches!(c.as_str(), "alta" | "média" | "media" | "baixa") => Some(c.clone()),
            Some(_) => return Err(bad(CMD, "`proposals[].confidence` must be \"alta\", \"média\" or \"baixa\"")),
        };
        let mut ds = Value::Null;
        let draw = match (item.get("screen"), item.get("commands")) {
            (Some(screen), None) => {
                let (tree, uses) =
                    super::eureka_screen::read(s.doc()?, CMD, screen).map_err(|e| bad(CMD, format!("proposal {}: {}", letter(i), reason(&e))))?;
                ds = json!({"components": uses.components, "variables": uses.variables});
                Draw::Screen(tree)
            }
            (None, Some(Value::Array(cmds))) => {
                if cmds.is_empty() || cmds.len() > MAX_STEPS {
                    return Err(bad(CMD, format!("proposal {}: 1 to {MAX_STEPS} commands", letter(i))));
                }
                let mut steps = Vec::with_capacity(cmds.len());
                for (j, c) in cmds.iter().enumerate() {
                    let cmd = c
                        .get("command")
                        .and_then(Value::as_str)
                        .ok_or_else(|| bad(CMD, format!("proposal {} step {j} has no `command`", letter(i))))?;
                    crate::batch::refuse_unknown_step_keys(j, c)?;
                    steps.push(BatchStep {
                        name: crate::batch::step_name(j, c)?,
                        ..BatchStep::new(cmd, c.get("params").cloned().unwrap_or(json!({})))
                    });
                }
                Draw::Commands(steps)
            }
            _ => return Err(bad(CMD, format!("proposal {} needs either `screen` or `commands` that draw its screen", letter(i)))),
        };
        let info = json!({
            "letter": letter(i), "name": name, "type": kind,
            "mechanism": text(item, "mechanism", "proposals[].mechanism")?,
            "summary": text(item, "summary", "proposals[].summary")?,
            "confidence": confidence, "board": title, "context": context, "status": "proposta", "ds": ds,
        });
        parsed.push((format!("{} · {kind} · {name}", letter(i)), draw, info));
    }
    let page = s.page()?;
    let start_x = s.doc()?.page_content_bounds(page).map_or(0.0, |r| r.x1 + GAP * 2.0);
    let label = format!("Exploração: {title}");
    let mut drawn: Vec<(NodeId, String, Value)> = Vec::new();
    s.grouped(&label, |s| {
        let mut x = start_x;
        for (screen_name, draw, info) in &parsed {
            let root = match draw {
                Draw::Screen(tree) => super::eureka_screen::build(s, tree, CMD)?,
                Draw::Commands(steps) => s
                    .batch(screen_name, steps)?
                    .iter()
                    .find_map(|r| r.get("id").and_then(id_of))
                    .ok_or_else(|| bad(CMD, format!("proposal {} drew no layer", info.get("letter").and_then(Value::as_str).unwrap_or("?"))))?,
            };
            let width = s.doc()?.node(root).map_or(0.0, |n| n.width);
            s.execute("node.set", &json!({"id": root.0, "props": {"x": x, "y": 0, "name": screen_name}}))?;
            let doc = s.ds_mut()?.doc_mut();
            if let Some(n) = doc.node_mut(root) {
                n.plugin_data.insert(PROPOSAL_KEY.to_string(), info.to_string());
            }
            x += width + GAP;
            drawn.push((root, screen_name.clone(), info.clone()));
        }
        let ids: Vec<u64> = drawn.iter().map(|(id, _, _)| id.0).collect();
        s.execute("object.wrapInSection", &json!({"ids": ids, "name": label}))?;
        Ok(())
    })?;
    let section = s.doc()?.node(drawn.first().map_or(NodeId(0), |d| d.0)).and_then(|n| n.parent).map(|p| p.0);
    let out: Vec<Value> = drawn.iter().map(|(id, name, info)| json!({"letter": info.get("letter"), "id": id.0, "name": name})).collect();
    Ok(json!({"section": section, "proposals": out}))
}

pub(crate) fn info_of(s: &Session, id: NodeId) -> Option<Value> {
    let n = s.doc().ok()?.node(id)?;
    serde_json::from_str(n.plugin_data.get(PROPOSAL_KEY)?).ok()
}

/// `explore.choose`.
fn choose(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.choose";
    let id = get(p, "id").and_then(id_of).ok_or_else(|| bad(CMD, "`id` is required (a proposal's screen)"))?;
    let Some(chosen) = info_of(s, id) else { return Err(bad(CMD, "that layer is not a proposal on an exploration board")) };
    let board = chosen.get("board").cloned().unwrap_or(Value::Null);
    let parent = s.doc()?.node(id).and_then(|n| n.parent);
    let siblings: Vec<NodeId> = parent.and_then(|pid| s.doc().ok()?.node(pid).map(|n| n.children.clone())).unwrap_or_default();
    let mut others = Vec::new();
    for sid in siblings {
        let Some(mut info) = info_of(s, sid) else { continue };
        if info.get("board") != Some(&board) {
            continue;
        }
        let status = if sid == id { "escolhida" } else { "não escolhida" };
        if let Some(o) = info.as_object_mut() {
            o.insert("status".into(), json!(status));
        }
        let name = {
            let doc = s.ds_mut()?.doc_mut();
            let n = doc.node_mut(sid).ok_or(EngineError::NoNode(sid))?;
            n.plugin_data.insert(PROPOSAL_KEY.to_string(), info.to_string());
            if sid != id {
                // A verdict belongs to the proposal that was chosen when it was given.
                n.plugin_data.remove(REVIEW_KEY);
                n.plugin_data.remove(super::eureka_critique::CRITIQUE_KEY);
            }
            n.name.clone()
        };
        if sid != id {
            others.push(json!({"id": sid.0, "name": name}));
        }
    }
    s.execute("select.set", &json!({"ids": [id.0]}))?;
    let doc = s.doc()?;
    let name = doc.node(id).map(|n| n.name.clone()).unwrap_or_default();
    let layers = doc.subtree(id);
    let lint = super::eureka_lint::lint_layers(doc, &layers, MAX_FINDINGS);
    let context = super::eureka::review_context(doc, &layers);
    Ok(json!({"chosen": {"id": id.0, "name": name, "proposal": info_of(s, id)}, "others": others, "lint": lint, "context": context}))
}

/// `explore.review`: keep a reviewer's verdict on the proposal the designer chose.
fn review(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.review";
    let id = get(p, "id").and_then(id_of).ok_or_else(|| bad(CMD, "`id` is required (the chosen proposal's screen)"))?;
    let Some(info) = info_of(s, id) else { return Err(bad(CMD, "that layer is not a proposal on an exploration board")) };
    if info.get("status").and_then(Value::as_str) != Some("escolhida") {
        return Err(bad(CMD, "only the chosen proposal is reviewed: pick it with `explore.choose` first"));
    }
    // The generator's order: the designer reviews the design critique before the Gatekeeper is asked.
    if !super::eureka_critique::is_reviewed(s.doc()?, id) {
        return Err(bad(
            CMD,
            "the design critique comes first: keep it with `explore.critique` and let the designer review it (`explore.critiqueReviewed`)",
        ));
    }
    let r = super::eureka::parse_review(CMD, s.agent.clone(), p)?;
    let doc = s.ds_mut()?.doc_mut();
    let n = doc.node_mut(id).ok_or(EngineError::NoNode(id))?;
    n.plugin_data.insert(REVIEW_KEY.to_string(), r.to_string());
    Ok(json!({"review": r}))
}
