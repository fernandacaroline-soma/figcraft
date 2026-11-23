//! The proposal card: while Claude's proposal is previewed (`proposal.preview`), a card beside the
//! canvas lists what it creates, changes and removes, the layers where it meets the designer's own
//! edits ("you: …" / "Claude: …", with Keep mine / Keep Claude's per layer), and the design-system
//! hints. Changing a choice previews again with `override`; a layer the proposal removes blocks
//! Apply until a choice is made. A stale apply names the layers that moved and offers to preview
//! again. The card only reads what the engine returned and dispatches commands.

use std::collections::BTreeSet;

use egui::{Align2, Id, Rect, Ui};
use serde_json::{Value, json};

use crate::FigApp;
use crate::i18n::{tr, trf, trn};
use crate::theme::{Tokens, regular, semibold};
use crate::widgets;

/// Layers listed per summary section; the rest is counted.
const MAX_LISTED: usize = 40;
const CARD_W: f32 = 320.0;

/// The proposal shown: the `proposal.preview` params and what it returned.
pub struct Card {
    /// The document the proposal belongs to.
    uid: u64,
    params: Value,
    result: Value,
    /// Removed layers the designer chose to keep (they still block Apply: the engine can't skip a
    /// removal).
    keep_removed: BTreeSet<u64>,
    /// A stale apply: the layers that changed since the preview.
    stale: Option<String>,
}

/// One edit-conflict row.
#[derive(Debug, PartialEq)]
pub struct Conflict {
    pub id: u64,
    pub name: String,
    pub removed: bool,
    /// "manual", "proposal" or "needs-choice".
    pub kept: String,
    pub manual: Vec<String>,
    pub proposal: Vec<String>,
}

/// Who wins on one layer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Choice {
    Mine,
    Claude,
}

enum Action {
    Choose(u64, Choice),
    Regenerate,
    Dismiss,
}

/// Remember what a `proposal.*` command run through the control channel returned (`params` as
/// sent): a preview shows its card, an apply or the end of the preview drops it.
pub fn observe(app: &mut FigApp, cmd: &str, params: &Value, result: &Value) {
    match cmd {
        "proposal.preview" => {
            let Ok(uid) = app.session.ds().map(|d| d.uid) else { return };
            app.ui.proposal = Some(Card { uid, params: params.clone(), result: result.clone(), keep_removed: BTreeSet::new(), stale: None });
        }
        "proposal.apply" | "proposal.previewEnd" => app.ui.proposal = None,
        _ => {}
    }
}

/// Whether the proposal preview is showing on the current document.
fn previewing(app: &FigApp) -> bool {
    app.session.ds().ok().and_then(|d| d.preview.as_ref()).is_some_and(|p| p.id == figcraft_engine::cmd::eureka::PREVIEW_ID)
}

/// The card of the document in front, if any.
fn current(app: &FigApp) -> Option<&Card> {
    let uid = app.session.ds().ok()?.uid;
    app.ui.proposal.as_ref().filter(|c| c.uid == uid)
}

/// Whether Apply must wait: a conflict still needs a choice.
pub fn blocked(app: &FigApp) -> bool {
    current(app).is_some_and(|c| result_blocked(&c.result))
}

fn result_blocked(result: &Value) -> bool {
    result.get("blocked").and_then(Value::as_bool).unwrap_or(false) || conflicts(result).iter().any(|c| c.kept == "needs-choice")
}

/// The `manualConflicts` of a preview result.
pub fn conflicts(result: &Value) -> Vec<Conflict> {
    let lines = |v: Option<&Value>| -> Vec<String> {
        v.and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect()).unwrap_or_default()
    };
    result
        .get("manualConflicts")
        .and_then(Value::as_array)
        .map(|a| {
            a.iter()
                .filter_map(|c| {
                    Some(Conflict {
                        id: c.get("id")?.as_u64()?,
                        name: c.get("name").and_then(Value::as_str).unwrap_or_default().to_string(),
                        removed: c.get("removed").and_then(Value::as_bool).unwrap_or(false),
                        kept: c.get("kept").and_then(Value::as_str).unwrap_or("manual").to_string(),
                        manual: lines(c.get("manual")),
                        proposal: lines(c.get("proposal")),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The params of a new preview where `id` goes to `choice`: `override` lists every layer where
/// the proposal wins, the rest keeps the hand edit (`keep: "manual"`).
pub fn params_with_choice(params: &Value, result: &Value, id: u64, choice: Choice) -> Value {
    let mut wins: BTreeSet<u64> = conflicts(result).iter().filter(|c| c.kept == "proposal").map(|c| c.id).collect();
    match choice {
        Choice::Claude => wins.insert(id),
        Choice::Mine => wins.remove(&id),
    };
    let mut p = params.clone();
    if let Some(o) = p.as_object_mut() {
        o.insert("keep".into(), json!("manual"));
        o.insert("override".into(), json!(wins.into_iter().collect::<Vec<_>>()));
    }
    p
}

/// The layers a `stale` error names (`Name #id [props]; …`), or the whole message.
pub fn stale_layers(msg: &str) -> Option<String> {
    if !msg.contains("stale") {
        return None;
    }
    let list = msg.split_once("conflicts:").map_or("", |(_, rest)| rest.trim());
    let names: Vec<&str> = list.split("; ").filter_map(|c| c.split(" #").next()).map(str::trim).filter(|n| !n.is_empty()).collect();
    Some(if names.is_empty() { "?".to_string() } else { names.join(", ") })
}

/// Apply the proposal (the banner's Apply); a stale preview keeps the card with the layers that moved.
pub fn apply(app: &mut FigApp) {
    if blocked(app) {
        return;
    }
    if app.run("proposal.apply", json!({})).is_some() {
        app.ui.proposal = None;
        app.toast(tr("Proposal applied"));
        return;
    }
    let stale = app.ui.last_error.as_deref().and_then(stale_layers);
    match (stale, app.ui.proposal.as_mut()) {
        (Some(layers), Some(card)) => card.stale = Some(layers),
        _ => app.ui.proposal = None,
    }
}

/// Discard the proposal (the banner's Discard).
pub fn discard(app: &mut FigApp) {
    app.ui.proposal = None;
    app.run("proposal.previewEnd", json!({}));
}

/// Preview again with `params`; the card takes the new result.
fn repreview(app: &mut FigApp, params: Value) {
    if let Some(result) = app.run("proposal.preview", params.clone())
        && let Some(card) = app.ui.proposal.as_mut()
    {
        card.params = params;
        card.result = result;
        card.stale = None;
    }
}

/// Who wins on layer `id`: preview again with the new `override` (a removed layer kept by hand has
/// nothing to preview and stays blocked).
fn choose(app: &mut FigApp, id: u64, choice: Choice) {
    let Some(c) = app.ui.proposal.as_mut() else { return };
    let needs_choice = conflicts(&c.result).iter().any(|k| k.id == id && k.kept == "needs-choice");
    if choice == Choice::Mine && needs_choice {
        c.keep_removed.insert(id);
        return;
    }
    c.keep_removed.remove(&id);
    let params = params_with_choice(&c.params, &c.result, id, choice);
    repreview(app, params);
}

/// Draw the card at the right of the canvas while the proposal preview (or its stale apply) is
/// shown.
pub fn card(app: &mut FigApp, ctx: &egui::Context, canvas: Rect) {
    let showing = previewing(app);
    let Some(c) = current(app) else { return };
    if !showing && c.stale.is_none() {
        return;
    }
    let t = Tokens::get(ctx);
    let x = (canvas.max.x - CARD_W - 12.0).max(canvas.min.x + 12.0);
    // Room for the banner above and the toolbar below.
    let max_h = (canvas.height() - 160.0).max(120.0);
    let action = egui::Area::new(Id::unique("proposal_card"))
        .order(egui::Order::Foreground)
        .fade_in(false)
        .pivot(Align2::LEFT_TOP)
        .fixed_pos(egui::pos2(x, canvas.min.y + 64.0))
        .show(ctx, |ui| {
            egui::Frame::NONE
                .fill(t.panel)
                .stroke(egui::Stroke::new(1.0, t.border))
                .corner_radius(egui::CornerRadius::same(8))
                .shadow(egui::epaint::Shadow { offset: [0, 2], blur: 12, spread: 0, color: t.shadow })
                .inner_margin(egui::Margin::same(12))
                .show(ui, |ui| {
                    ui.set_width(CARD_W - 24.0);
                    egui::ScrollArea::vertical()
                        .max_height(max_h)
                        .min_scrolled_height(max_h)
                        .auto_shrink([false, true])
                        .show(ui, |ui| body(ui, c, &t))
                })
                .inner
                .inner
        })
        .inner;
    match action {
        Some(Action::Choose(id, choice)) => choose(app, id, choice),
        Some(Action::Regenerate) => {
            let Some(params) = app.ui.proposal.as_ref().map(|c| c.params.clone()) else { return };
            repreview(app, params);
        }
        Some(Action::Dismiss) => app.ui.proposal = None,
        None => {}
    }
}

fn heading(ui: &mut Ui, text: &str, t: &Tokens) {
    ui.add_space(10.0);
    ui.label(egui::RichText::new(text).font(semibold(11.0)).color(t.text));
    ui.add_space(4.0);
}

fn line(ui: &mut Ui, text: &str, color: egui::Color32) {
    ui.add(egui::Label::new(egui::RichText::new(text).font(regular(11.0)).color(color)).wrap());
}

fn body(ui: &mut Ui, c: &Card, t: &Tokens) -> Option<Action> {
    let mut out = None;
    ui.spacing_mut().item_spacing.y = 2.0;
    let label = c.params.get("label").and_then(Value::as_str).unwrap_or_default();
    ui.label(egui::RichText::new(tr("Claude’s proposal")).font(semibold(13.0)).color(t.text));
    if !label.is_empty() {
        line(ui, label, t.text_secondary);
    }

    if let Some(layers) = &c.stale {
        ui.add_space(8.0);
        line(ui, &trf("The layer {names} changed since the preview. Nothing was applied.", &[("names", layers)]), t.danger);
        ui.add_space(8.0);
        ui.horizontal(|ui| {
            if widgets::button(ui, tr("Generate again"), true, true).clicked() {
                out = Some(Action::Regenerate);
            }
            if widgets::button(ui, tr("Close"), false, true).clicked() {
                out = Some(Action::Dismiss);
            }
        });
        return out;
    }

    let r = &c.result;
    if result_blocked(r) {
        ui.add_space(10.0);
        line(ui, tr("Apply waits for a choice on every conflict."), t.warning);
    }
    // Conflicts with the designer's own edits.
    let list = conflicts(r);
    if !list.is_empty() {
        heading(ui, tr("Conflicts with your edits"), t);
        line(ui, tr("Your edit stays unless you choose Claude’s."), t.text_secondary);
        for k in &list {
            ui.add_space(8.0);
            ui.label(egui::RichText::new(format!("⚑ {}  #{}", k.name, k.id)).font(semibold(11.0)).color(t.text));
            if k.removed {
                line(ui, tr("Claude’s proposal removes this layer, which you edited by hand."), t.text_secondary);
            }
            for m in &k.manual {
                line(ui, &trf("you: {change}", &[("change", m)]), t.text_secondary);
            }
            for p in &k.proposal {
                line(ui, &trf("Claude: {change}", &[("change", p)]), t.text_secondary);
            }
            let choice = match k.kept.as_str() {
                "proposal" => Some(Choice::Claude),
                "needs-choice" => c.keep_removed.contains(&k.id).then_some(Choice::Mine),
                _ => Some(Choice::Mine),
            };
            ui.add_space(4.0);
            ui.horizontal(|ui| {
                if widgets::button(ui, tr("Keep mine"), choice == Some(Choice::Mine), true).clicked() && choice != Some(Choice::Mine) {
                    out = Some(Action::Choose(k.id, Choice::Mine));
                }
                if widgets::button(ui, tr("Keep Claude’s"), choice == Some(Choice::Claude), true).clicked() && choice != Some(Choice::Claude) {
                    out = Some(Action::Choose(k.id, Choice::Claude));
                }
            });
            match (k.kept.as_str(), choice) {
                ("needs-choice", None) => line(ui, tr("Choose before applying."), t.warning),
                ("needs-choice", Some(_)) => {
                    line(ui, tr("To keep your layer, discard the proposal or ask Claude for another one."), t.warning);
                }
                _ => {}
            }
        }
    }

    // Design-system hints: raw values with no variable. A note, never an error.
    let warnings: Vec<&Value> = r.get("dsWarnings").and_then(Value::as_array).map(|a| a.iter().collect()).unwrap_or_default();
    if !warnings.is_empty() {
        heading(ui, tr("Design system notes"), t);
        for w in warnings.iter().take(MAX_LISTED) {
            let name = w.get("name").and_then(Value::as_str).unwrap_or_default();
            let prop = w.get("prop").and_then(Value::as_str).unwrap_or_default();
            let value = match w.get("value") {
                Some(Value::String(s)) => s.clone(),
                Some(v) => v.to_string(),
                None => String::new(),
            };
            line(ui, &trf("{name}: {prop} {value} has no variable", &[("name", &name), ("prop", &prop), ("value", &value)]), t.warning);
        }
    }

    // Summary: each created, changed and removed layer with what changes on it.
    for (key, title) in [("created", tr("Created")), ("changed", tr("Changed")), ("removed", tr("Removed"))] {
        let Some(items) = r.get(key).and_then(Value::as_array).filter(|a| !a.is_empty()) else { continue };
        heading(ui, title, t);
        for item in items.iter().take(MAX_LISTED) {
            let name = item.get("name").and_then(Value::as_str).unwrap_or_default();
            let id = item.get("id").and_then(Value::as_u64).unwrap_or_default();
            ui.label(egui::RichText::new(format!("{name}  #{id}")).font(semibold(11.0)).color(t.text));
            let summary: Vec<&str> =
                item.get("summary").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).collect()).unwrap_or_default();
            if summary.is_empty() && key == "changed" {
                line(ui, tr("Only layout changes"), t.text_tertiary);
            }
            for s in summary {
                line(ui, &format!("  {s}"), t.text_secondary);
            }
        }
        if items.len() > MAX_LISTED {
            line(ui, &trn("{n} more layers", items.len() - MAX_LISTED, &[]), t.text_tertiary);
        }
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Value {
        json!({
            "blocked": false,
            "manualConflicts": [
                {"id": 6, "name": "Preço", "props": ["style"], "kept": "manual", "manual": ["a"], "proposal": ["b"]},
                {"id": 7, "name": "Button", "props": ["fills"], "kept": "proposal", "manual": ["c"], "proposal": ["d"]},
                {"id": 9, "name": "Gone", "removed": true, "kept": "needs-choice"}
            ]
        })
    }

    #[test]
    fn conflicts_parse_and_needs_choice_blocks() {
        let r = sample();
        let list = conflicts(&r);
        assert_eq!(list.len(), 3);
        assert_eq!(list[0].manual, vec!["a".to_string()]);
        assert!(list[2].removed);
        assert!(result_blocked(&r));
        assert!(!result_blocked(&json!({"manualConflicts": []})));
    }

    #[test]
    fn a_choice_rewrites_override_from_what_is_kept() {
        let r = sample();
        let p = json!({"label": "x", "commands": [], "keep": "proposal"});
        let claude = params_with_choice(&p, &r, 6, Choice::Claude);
        assert_eq!(claude["override"], json!([6, 7]));
        assert_eq!(claude["keep"], json!("manual"));
        let mine = params_with_choice(&p, &r, 7, Choice::Mine);
        assert_eq!(mine["override"], json!([]));
        assert_eq!(mine["commands"], json!([]));
    }

    #[test]
    fn stale_errors_name_their_layers() {
        let msg = "stale: the layers this proposal touches changed since the dry run (baseHash 1); nothing was applied. Run the dry run again. conflicts: Preço #6 [fills]; Button #7 [removed]";
        assert_eq!(stale_layers(msg).as_deref(), Some("Preço, Button"));
        assert_eq!(stale_layers("something else"), None);
    }

    /// A headless app with a rectangle the designer renamed by hand after Claude's turn, and the
    /// control channel's `engine.execute` as Claude.
    fn edited_by_hand() -> (crate::present_tests::Ui, figcraft_doc::NodeId) {
        let mut u = crate::present_tests::Ui::new(figcraft_engine::Session::new());
        let r = u.app.session.execute("node.create", &json!({"type": "rectangle", "name": "Other", "width": 10, "height": 10})).unwrap();
        let node = figcraft_engine::params::id_of(&r["id"]).unwrap();
        let id = node.0;
        u.app.session.execute("proposal.turn", &json!({})).unwrap();
        u.app.session.execute("node.set", &json!({"id": id, "props": {"name": "Kept by hand"}})).unwrap();
        (u, node)
    }

    fn claude(u: &mut crate::present_tests::Ui, command: &str, params: Value) -> Value {
        let p = json!({"command": command, "params": params, "agent": "Claude"});
        crate::control::handle(&mut u.app, &egui::Context::default(), "engine.execute", &p).unwrap()
    }

    #[test]
    fn a_removed_hand_edit_blocks_apply_until_claude_wins_there() {
        let (mut u, node) = edited_by_hand();
        let id = node.0;
        claude(&mut u, "proposal.preview", json!({"label": "Tidy", "commands": [{"command": "edit.delete", "params": {"ids": [id]}}]}));
        assert!(previewing(&u.app), "the control channel's preview shows");
        assert!(blocked(&u.app), "a needs-choice layer holds Apply back");
        u.frame(vec![]);

        // Keeping it by hand can't be previewed: still blocked, nothing applied (no undo step, the
        // preview still shows).
        let steps = u.app.session.ds().unwrap().history.undo.len();
        choose(&mut u.app, id, Choice::Mine);
        assert!(blocked(&u.app));
        apply(&mut u.app);
        assert!(previewing(&u.app), "a blocked Apply keeps the preview");
        assert_eq!(u.app.session.ds().unwrap().history.undo.len(), steps);

        // Claude's choice previews again with `override` and frees Apply.
        choose(&mut u.app, id, Choice::Claude);
        let card = u.app.ui.proposal.as_ref().unwrap();
        assert_eq!(card.params["override"], json!([id]));
        assert!(!blocked(&u.app));
        apply(&mut u.app);
        assert!(u.app.ui.proposal.is_none(), "the card goes with the applied proposal");
        assert!(u.app.session.doc().unwrap().node(node).is_none());
    }

    #[test]
    fn keep_mine_is_the_default_and_discard_drops_the_card() {
        let (mut u, node) = edited_by_hand();
        let id = node.0;
        let rename = json!({"label": "Rename", "commands": [{"command": "node.set", "params": {"id": id, "props": {"name": "Claude's name"}}}]});
        claude(&mut u, "proposal.preview", rename);
        let list = conflicts(&u.app.ui.proposal.as_ref().unwrap().result);
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].kept, "manual");
        assert!(!blocked(&u.app));
        u.frame(vec![]);

        choose(&mut u.app, id, Choice::Claude);
        assert_eq!(conflicts(&u.app.ui.proposal.as_ref().unwrap().result)[0].kept, "proposal");
        choose(&mut u.app, id, Choice::Mine);
        assert_eq!(conflicts(&u.app.ui.proposal.as_ref().unwrap().result)[0].kept, "manual");

        discard(&mut u.app);
        assert!(u.app.ui.proposal.is_none());
        assert!(!previewing(&u.app));
        let node = u.app.session.doc().unwrap().node(node).map(|n| n.name.clone());
        assert_eq!(node.as_deref(), Some("Kept by hand"));
    }
}
