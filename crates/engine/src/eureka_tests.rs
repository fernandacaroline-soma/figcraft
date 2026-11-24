//! Proposals: dry run, guarded apply, checkpoints and the changes since one.

use serde_json::{Value, json};

use crate::{NodeId, Session, params::id_of};

fn run(s: &mut Session, command: &str, params: Value) -> Value {
    s.execute(command, &params).unwrap_or_else(|e| panic!("{command}: {e}"))
}

struct Fixture {
    s: Session,
    card: NodeId,
    title: NodeId,
    other: NodeId,
    var: String,
}

fn setup() -> Fixture {
    let mut s = Session::new();
    run(&mut s, "variable.createCollection", json!({"name": "Tokens", "modes": ["Animale"]}));
    let v = run(&mut s, "variable.create", json!({"collection": "Tokens", "name": "Colors/Surface", "type": "color", "value": "#112233"}));
    let var = v["id"].as_str().unwrap_or_default().to_string();
    let card = id_of(&run(&mut s, "node.create", json!({"type": "frame", "name": "Card", "width": 200, "height": 100}))["id"]).unwrap();
    let title = id_of(&run(&mut s, "text.create", json!({"text": "Title", "x": 0, "y": 0}))["id"]).unwrap();
    let other = id_of(&run(&mut s, "node.create", json!({"type": "rectangle", "name": "Other", "width": 10, "height": 10}))["id"]).unwrap();
    Fixture { s, card, title, other, var }
}

fn refine(f: &Fixture) -> Value {
    json!([
        {"command": "node.set", "params": {"id": f.card.0, "props": {"name": "Card v2", "fill": "#FF0000"}}},
        {"command": "variable.bind", "params": {"ids": [f.card.0], "field": "fill", "variable": f.var}},
        {"command": "node.create", "params": {"type": "rectangle", "name": "Badge", "width": 20, "height": 8}}
    ])
}

fn batch(f: &Fixture, extra: Value) -> Value {
    let mut p = json!({"label": "Refine", "commands": refine(f)});
    if let (Some(o), Some(e)) = (p.as_object_mut(), extra.as_object()) {
        o.extend(e.clone());
    }
    p
}

#[test]
fn dry_run_leaves_no_trace_and_reports_the_diff() {
    let mut f = setup();
    let doc = f.s.doc().unwrap().clone();
    let depth = f.s.undo_depth();
    let journal = f.s.journal.len();
    let p = batch(&f, json!({"dryRun": true}));
    let out = run(&mut f.s, "command.batch", p);
    assert_eq!(out["dryRun"], json!(true));
    assert!(out["baseHash"].as_str().is_some_and(|h| h.len() == 16));
    let changed = out["changed"].as_array().unwrap();
    let card = changed.iter().find(|c| c["id"] == json!(f.card.0)).expect("card changed");
    assert_eq!(card["props"]["name"], json!(["Card", "Card v2"]));
    assert!(card["props"].get("bound").is_some(), "variable binding is in the diff: {card}");
    let created = out["created"].as_array().unwrap();
    assert!(created.iter().any(|c| c["name"] == "Badge" && c["type"] == "rectangle"), "{created:?}");
    assert_eq!(*f.s.doc().unwrap(), doc);
    assert_eq!(f.s.undo_depth(), depth);
    // Only the dry-run call itself is logged (as any command is), none of its steps.
    assert_eq!(f.s.journal.len(), journal + 1);
    assert_eq!(f.s.journal.last().map(|(c, _)| c.as_str()), Some("command.batch"));
    // The id allocator is back where it was: the real run makes the same ids.
    let p = batch(&f, json!({}));
    run(&mut f.s, "command.batch", p);
    let badge = out["created"][0]["id"].clone();
    let after = f.s.doc().unwrap();
    assert!(after.node(id_of(&badge).unwrap()).is_some_and(|n| n.name == "Badge"));
}

#[test]
fn dry_run_refuses_effects_outside_the_document() {
    let mut f = setup();
    let r = f.s.execute("command.batch", &json!({"dryRun": true, "commands": [{"command": "file.new", "params": {}}]}));
    assert!(r.is_err());
    let r = f
        .s
        .execute("command.batch", &json!({"dryRun": true, "commands": [{"command": "node.set", "params": {"id": 424242, "props": {"name": "x"}}}]}));
    assert!(r.is_err());
    assert!(f.s.execute("command.batch", &json!({"dryRun": "yes", "commands": []})).is_err());
    assert!(f.s.execute("command.batch", &json!({"dryRun": true, "ifUnchanged": "x", "commands": []})).is_err());
}

#[test]
fn accept_with_guard_is_one_undo_step_and_undo_restores() {
    let mut f = setup();
    let doc = f.s.doc().unwrap().clone();
    let depth = f.s.undo_depth();
    let bp = batch(&f, json!({"dryRun": true}));
    let dry = run(&mut f.s, "command.batch", bp);
    let hash = dry["baseHash"].as_str().unwrap().to_string();
    // A layer the batch does not touch changes meanwhile: still accepted.
    run(&mut f.s, "node.set", json!({"id": f.other.0, "props": {"name": "Other edited"}}));
    let mid_depth = f.s.undo_depth();
    let bp = batch(&f, json!({"ifUnchanged": hash}));
    let out = run(&mut f.s, "command.batch", bp);
    assert_eq!(out["applied"], json!(true));
    assert_eq!(f.s.undo_depth().0, mid_depth.0 + 1);
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card v2");
    run(&mut f.s, "edit.undo", json!({}));
    assert_eq!(f.s.undo_depth().0, mid_depth.0);
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card");
    assert!(depth.0 < mid_depth.0);
    let _ = doc;
}

#[test]
fn touched_layer_edited_since_the_dry_run_is_stale() {
    let mut f = setup();
    let bp = batch(&f, json!({"dryRun": true}));
    let dry = run(&mut f.s, "command.batch", bp);
    let hash = dry["baseHash"].as_str().unwrap().to_string();
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"fill": "#00FF00"}}));
    let doc = f.s.doc().unwrap().clone();
    let depth = f.s.undo_depth();
    let err = f.s.execute("command.batch", &batch(&f, json!({"ifUnchanged": hash}))).unwrap_err().to_string();
    assert!(err.contains("stale"), "{err}");
    assert!(err.contains("Card") && err.contains("fills"), "names the conflict: {err}");
    assert_eq!(*f.s.doc().unwrap(), doc);
    assert_eq!(f.s.undo_depth(), depth);
    // An unknown hash is stale too, and nothing is applied.
    let err = f.s.execute("command.batch", &batch(&f, json!({"ifUnchanged": "0000000000000000"}))).unwrap_err().to_string();
    assert!(err.contains("stale"), "{err}");
    assert_eq!(*f.s.doc().unwrap(), doc);
}

#[test]
fn changes_since_a_checkpoint_see_text_colour_and_variable_edits() {
    let mut f = setup();
    let cp = run(&mut f.s, "document.checkpoint", json!({}));
    let cursor = cp["cursor"].as_u64().unwrap();
    run(&mut f.s, "node.set", json!({"id": f.title.0, "props": {"characters": "Changed"}}));
    run(&mut f.s, "node.set", json!({"id": f.other.0, "props": {"fill": "#7A4B2A"}}));
    run(&mut f.s, "variable.bind", json!({"ids": [f.card.0], "field": "fill", "variable": f.var}));
    run(&mut f.s, "variable.update", json!({"variable": f.var, "name": "Colors/Surface 2"}));
    let out = run(&mut f.s, "document.changesSince", json!({"cursor": cursor, "advance": true}));
    let changed = out["changed"].as_array().unwrap();
    let by = |id: NodeId| changed.iter().find(|c| c["id"] == json!(id.0)).cloned().unwrap_or(Value::Null);
    assert!(by(f.title)["props"].to_string().contains("Changed"), "{out}");
    assert!(by(f.other)["props"].get("fills").is_some(), "{out}");
    assert!(by(f.card)["props"].get("bound").is_some(), "{out}");
    assert!(out["variables"]["changed"].as_array().is_some_and(|a| !a.is_empty()), "{out}");
    assert_ne!(out["cursor"].as_u64(), Some(cursor));
    // The new cursor starts from now.
    let next = out["cursor"].as_u64().unwrap();
    let again = run(&mut f.s, "document.changesSince", json!({"cursor": next}));
    assert_eq!(again["total"], json!(0));
}

#[test]
fn bad_or_expired_cursors_are_errors() {
    let mut f = setup();
    for junk in [
        json!({}),
        json!({"cursor": "x"}),
        json!({"cursor": -1}),
        json!({"cursor": 999}),
        json!({"cursor": 1.5e300}),
        json!({"cursor": 1, "limit": 0}),
    ] {
        assert!(f.s.execute("document.changesSince", &junk).is_err(), "{junk}");
    }
    let first = run(&mut f.s, "document.checkpoint", json!({}))["cursor"].as_u64().unwrap();
    for _ in 0..12 {
        run(&mut f.s, "document.checkpoint", json!({}));
    }
    assert!(f.s.execute("document.changesSince", &json!({"cursor": first})).is_err(), "the oldest checkpoint expired");
}

#[test]
fn proposal_preview_is_read_only_and_apply_is_guarded() {
    let mut f = setup();
    let bp = batch(&f, json!({}));
    let shown = run(&mut f.s, "proposal.preview", bp);
    assert_eq!(shown["preview"], json!(true));
    assert!(f.s.ds().unwrap().preview.as_ref().is_some_and(|p| p.id == crate::cmd::eureka::PREVIEW_ID));
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card v2");
    assert!(f.s.execute("node.set", &json!({"id": f.other.0, "props": {"name": "x"}})).is_err());
    // A guarded batch can't run on top of the preview (a failure would roll back into it).
    let bp = batch(&f, json!({"ifUnchanged": shown["baseHash"]}));
    let err = f.s.execute("command.batch", &bp).unwrap_err().to_string();
    assert!(err.contains("preview is showing"), "{err}");
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card v2", "still previewing");
    // Discard: back to the document, nothing applied.
    assert_eq!(run(&mut f.s, "proposal.previewEnd", json!({}))["ended"], json!(true));
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card");
    // Apply: leaves the preview and is one undo step.
    let depth = f.s.undo_depth().0;
    let bp = batch(&f, json!({}));
    run(&mut f.s, "proposal.preview", bp);
    run(&mut f.s, "proposal.apply", json!({}));
    assert!(f.s.ds().unwrap().preview.is_none());
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card v2");
    assert_eq!(f.s.undo_depth().0, depth + 1);
    assert!(f.s.execute("proposal.apply", &json!({})).is_err(), "nothing left to apply");
}

#[test]
fn applying_a_stale_proposal_leaves_the_preview_and_the_document_as_it_was() {
    let mut f = setup();
    let bp = batch(&f, json!({}));
    run(&mut f.s, "proposal.preview", bp);
    run(&mut f.s, "proposal.previewEnd", json!({}));
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"fill": "#00FF00"}}));
    let doc = f.s.doc().unwrap().clone();
    let bp = batch(&f, json!({}));
    run(&mut f.s, "proposal.preview", bp);
    run(&mut f.s, "proposal.previewEnd", json!({}));
    // The designer edits a touched layer after the preview was taken.
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"fill": "#0000FF"}}));
    let doc2 = f.s.doc().unwrap().clone();
    assert_ne!(doc, doc2);
    let err = f.s.execute("proposal.apply", &json!({})).unwrap_err().to_string();
    assert!(err.contains("stale") && err.contains("Card"), "{err}");
    assert!(f.s.ds().unwrap().preview.is_none());
    assert_eq!(*f.s.doc().unwrap(), doc2, "not the preview's document");
}

#[test]
fn diff_entries_have_a_designer_summary_and_keep_layout_values_apart() {
    let mut f = setup();
    let bp = batch(&f, json!({"dryRun": true}));
    let out = run(&mut f.s, "command.batch", bp);
    let card = out["changed"].as_array().unwrap().iter().find(|c| c["id"] == json!(f.card.0)).cloned().unwrap();
    let lines: Vec<String> = card["summary"].as_array().unwrap().iter().map(|l| l.as_str().unwrap_or_default().to_string()).collect();
    let all = lines.join("\n");
    assert!(all.contains("Nome: \"Card\" → \"Card v2\""), "{all}");
    assert!(all.contains("#FF0000") || all.contains("#112233"), "colours as hex: {all}");
    assert!(all.contains("Vínculo de preenchimento: sem vínculo → Colors/Surface"), "variable name, not its id: {all}");
    assert!(!all.contains("VariableID"), "{all}");
    assert!(!all.contains("children"), "{all}");
    assert!(card["props"]["name"].is_array(), "raw props stay");
    // A child added to the card moves layout-derived values: apart from the summary.
    let add = json!([{"command": "node.create", "params": {"type": "rectangle", "parent": f.card.0, "name": "Kid", "width": 5, "height": 5}}]);
    let out = run(&mut f.s, "command.batch", json!({"dryRun": true, "commands": add}));
    let card = out["changed"].as_array().unwrap().iter().find(|c| c["id"] == json!(f.card.0)).cloned().unwrap();
    assert!(card["layout"]["children"].is_array(), "layout values are apart: {card}");
    assert!(card["summary"].as_array().unwrap().is_empty(), "{card}");
    // Text is quoted.
    run(&mut f.s, "document.checkpoint", json!({}));
    let cursor = run(&mut f.s, "document.checkpoint", json!({}))["cursor"].clone();
    run(&mut f.s, "node.set", json!({"id": f.title.0, "props": {"characters": "Novo"}}));
    let d = run(&mut f.s, "document.changesSince", json!({"cursor": cursor}));
    let t = d["changed"].as_array().unwrap().iter().find(|c| c["id"] == json!(f.title.0)).cloned().unwrap();
    assert!(t["summary"].to_string().contains("Texto: \\\"Title\\\" → \\\"Novo\\\""), "{t}");
}

#[test]
fn ds_warnings_flag_raw_values_without_blocking() {
    let mut f = setup();
    let cmds = json!([
        {"command": "node.create", "as": "tag", "params": {"type": "frame", "name": "Tag", "width": 50, "height": 20,
            "props": {"fill": "#FF8800", "layout": "horizontal", "padding": 8, "gap": 4, "cornerRadius": 6}}},
        {"command": "text.create", "params": {"parent": {"$ref": "tag"}, "text": "Hi", "x": 0, "y": 0, "name": "Hi", "props": {"fontSize": 11}}},
        {"command": "node.create", "as": "ok", "params": {"type": "rectangle", "name": "Bound", "width": 5, "height": 5, "props": {"fill": {"$var": f.var}}}}
    ]);
    let out = run(&mut f.s, "command.batch", json!({"dryRun": true, "commands": cmds}));
    let w = out["dsWarnings"].as_array().unwrap();
    let has = |name: &str, prop: &str| w.iter().any(|x| x["name"] == name && x["prop"] == prop);
    assert!(has("Tag", "fill") && has("Tag", "cornerRadius") && has("Tag", "gap") && has("Tag", "padding"), "{w:?}");
    assert!(has("Hi", "fontSize"), "{w:?}");
    assert!(!w.iter().any(|x| x["name"] == "Bound" && x["prop"] == "fill"), "bound fills are not flagged: {w:?}");
    // A warning is not an error: the same batch applies.
    let applied = f.s.execute("command.batch", &json!({"commands": cmds}));
    assert!(applied.is_ok());
}

fn node_value(s: &Session, id: NodeId) -> Value {
    serde_json::to_value(s.doc().unwrap().node(id).unwrap()).unwrap()
}

fn dry_then_apply(f: &mut Fixture, extra: Value) -> (Value, Value) {
    let mut dp = batch(f, extra.clone());
    dp["dryRun"] = json!(true);
    let dry = run(&mut f.s, "command.batch", dp);
    let mut ap = batch(f, extra);
    ap["ifUnchanged"] = dry["baseHash"].clone();
    let out = run(&mut f.s, "command.batch", ap);
    (dry, out)
}

#[test]
fn the_designer_hand_edit_wins_over_a_proposal_by_default() {
    let mut f = setup();
    run(&mut f.s, "proposal.turn", json!({}));
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"name": "Mine", "fill": "#00FF00"}}));
    let (dry, out) = dry_then_apply(&mut f, json!({}));
    assert_eq!(dry["manualBaseline"], json!("turn"));
    assert_eq!(dry["blocked"], json!(false));
    let c = &dry["manualConflicts"][0];
    assert_eq!(c["id"], json!(f.card.0));
    assert_eq!(c["kept"], json!("manual"));
    assert!(c["props"].as_array().unwrap().contains(&json!("fills")) && c["props"].as_array().unwrap().contains(&json!("name")), "{c}");
    assert!(c["manual"].to_string().contains("#00FF00") && c["proposal"].to_string().contains("Card v2"), "both sides are told: {c}");
    // The dry run shows no change on the card: the proposal does not reach it.
    assert!(!dry["changed"].as_array().unwrap().iter().any(|e| e["id"] == json!(f.card.0)), "{dry}");
    assert_eq!(out["applied"], json!(true));
    let card = node_value(&f.s, f.card);
    assert_eq!(card["name"], json!("Mine"));
    assert!(
        card["bound"].get("fill").is_none() && card["bound"].as_object().is_none_or(|b| b.keys().all(|k| !k.starts_with("fill"))),
        "binding not forced: {card}"
    );
    // What the designer did not touch still changes: the badge is made.
    assert!(out["results"].to_string().contains("Badge") || f.s.doc().unwrap().nodes().any(|n| n.name == "Badge"));
}

#[test]
fn override_lets_the_proposal_win_on_that_layer() {
    let mut f = setup();
    run(&mut f.s, "proposal.turn", json!({}));
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"name": "Mine", "fill": "#00FF00"}}));
    let card_id = f.card.0;
    let (dry, _) = dry_then_apply(&mut f, json!({"override": [card_id]}));
    assert_eq!(dry["manualConflicts"][0]["kept"], json!("proposal"));
    let card = node_value(&f.s, f.card);
    assert_eq!(card["name"], json!("Card v2"));
    assert!(card["bound"].to_string().contains(&f.var), "proposal binding applied: {card}");
}

#[test]
fn a_hand_edit_on_other_aspects_is_no_conflict() {
    let mut f = setup();
    run(&mut f.s, "proposal.turn", json!({}));
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"opacity": 0.5}}));
    let (dry, _) = dry_then_apply(&mut f, json!({}));
    assert_eq!(dry["manualConflicts"], json!([]));
    let card = node_value(&f.s, f.card);
    assert_eq!(card["name"], json!("Card v2"));
    assert_eq!(card["opacity"], json!(0.5));
}

#[test]
fn removing_a_hand_edited_layer_needs_a_choice() {
    let mut f = setup();
    run(&mut f.s, "proposal.turn", json!({}));
    run(&mut f.s, "node.set", json!({"id": f.other.0, "props": {"name": "Kept by hand"}}));
    let del = json!({"label": "Tidy", "commands": [{"command": "edit.delete", "params": {"ids": [f.other.0]}}]});
    let mut dp = del.clone();
    dp["dryRun"] = json!(true);
    let dry = run(&mut f.s, "command.batch", dp);
    assert_eq!(dry["blocked"], json!(true));
    assert_eq!(dry["manualConflicts"][0]["kept"], json!("needs-choice"));
    let doc = f.s.doc().unwrap().clone();
    let mut ap = del.clone();
    ap["ifUnchanged"] = dry["baseHash"].clone();
    let err = f.s.execute("command.batch", &ap).unwrap_err().to_string();
    assert!(err.contains("override"), "{err}");
    assert_eq!(*f.s.doc().unwrap(), doc);
    let mut ap = del;
    ap["override"] = json!([f.other.0]);
    ap["ifUnchanged"] = dry["baseHash"].clone();
    run(&mut f.s, "command.batch", ap);
    assert!(f.s.doc().unwrap().node(f.other).is_none());
}

#[test]
fn a_guarded_apply_starts_a_new_turn_and_no_baseline_protects_nothing() {
    let mut f = setup();
    // No turn yet: nothing is protected.
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"name": "Early"}}));
    let (dry, _) = dry_then_apply(&mut f, json!({}));
    assert_eq!(dry["manualBaseline"], Value::Null);
    assert_eq!(node_value(&f.s, f.card)["name"], json!("Card v2"));
    // The apply was Claude's turn: an edit after it is the designer's.
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"name": "Later"}}));
    let p = json!({"dryRun": true, "commands": [{"command": "node.set", "params": {"id": f.card.0, "props": {"name": "Claude"}}}]});
    let dry = run(&mut f.s, "command.batch", p);
    assert_eq!(dry["manualBaseline"], json!("turn"));
    assert_eq!(dry["manualConflicts"][0]["kept"], json!("manual"));
    assert!(f.s.execute("command.batch", &json!({"dryRun": true, "keep": "both", "commands": []})).is_err());
    assert!(f.s.execute("command.batch", &json!({"dryRun": true, "override": ["x"], "commands": []})).is_err());
}

#[test]
fn a_text_size_set_by_hand_survives_a_token_binding() {
    let mut f = setup();
    let v = run(&mut f.s, "variable.create", json!({"collection": "Tokens", "name": "Font/Size/fs-xs", "type": "number", "value": 16}));
    let fs = v["id"].as_str().unwrap().to_string();
    run(&mut f.s, "node.set", json!({"id": f.title.0, "props": {"fontSize": 14}}));
    let cp = run(&mut f.s, "document.checkpoint", json!({}));
    run(&mut f.s, "node.set", json!({"id": f.title.0, "props": {"fontSize": 18, "characters": "R$ 349,90"}}));
    let p = json!({"dryRun": true, "since": cp["cursor"], "commands": [
        {"command": "variable.bind", "params": {"ids": [f.title.0], "field": "fontSize", "variable": fs}}
    ]});
    let dry = run(&mut f.s, "command.batch", p.clone());
    assert_eq!(dry["manualBaseline"], json!("since"));
    assert_eq!(dry["manualConflicts"][0]["kept"], json!("manual"), "{dry}");
    let mut ap = p;
    ap.as_object_mut().unwrap().remove("dryRun");
    ap["ifUnchanged"] = dry["baseHash"].clone();
    run(&mut f.s, "command.batch", ap);
    let t = node_value(&f.s, f.title);
    assert!(t.to_string().contains("18"), "size kept: {t}");
    assert!(!t["bound"].to_string().contains(&fs), "binding left out: {t}");
}

fn doc_events(s: &Session, after: u64) -> Vec<crate::events::Event> {
    s.events.poll(after, &["document".to_string()], 100, std::time::Duration::ZERO).events
}

#[test]
fn a_hand_edit_emits_document_changed_with_the_layers() {
    let mut f = setup();
    let at = f.s.events.latest();
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"name": "Edited"}}));
    let ev = doc_events(&f.s, at);
    assert_eq!(ev.len(), 1, "{ev:?}");
    assert_eq!(ev[0].kind, "document.changed");
    assert_eq!(ev[0].data["source"], json!("human"));
    assert_eq!(ev[0].data["command"], json!("node.set"));
    assert!(ev[0].data["changed"].as_array().unwrap().contains(&json!(f.card.0)), "{:?}", ev[0].data);
    // Reading changes nothing and says nothing.
    let at = f.s.events.latest();
    run(&mut f.s, "document.checkpoint", json!({}));
    assert!(doc_events(&f.s, at).is_empty());
}

#[test]
fn an_agent_edit_is_marked_as_the_agent_and_a_batch_is_one_change() {
    let mut f = setup();
    let at = f.s.events.latest();
    let p = batch(&f, json!({}));
    f.s.execute_as(Some("Claude"), "command.batch", &p).unwrap();
    let ev = doc_events(&f.s, at);
    assert_eq!(ev.len(), 1, "one event for the whole batch: {ev:?}");
    assert_eq!(ev[0].data["source"], json!("agent"));
    assert_eq!(ev[0].data["agent"], json!("Claude (agent)"));
    assert!(ev[0].data["changed"].as_array().unwrap().contains(&json!(f.card.0)), "{:?}", ev[0].data);
}

#[test]
fn a_dry_run_and_a_refused_apply_emit_nothing() {
    let mut f = setup();
    let at = f.s.events.latest();
    let dp = batch(&f, json!({"dryRun": true}));
    let dry = run(&mut f.s, "command.batch", dp);
    let pp = batch(&f, json!({}));
    run(&mut f.s, "proposal.preview", pp);
    run(&mut f.s, "proposal.previewEnd", json!({}));
    assert!(doc_events(&f.s, at).is_empty(), "{:?}", doc_events(&f.s, at));
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"fill": "#00FF00"}}));
    let at = f.s.events.latest();
    let mut ap = batch(&f, json!({}));
    ap["ifUnchanged"] = dry["baseHash"].clone();
    assert!(f.s.execute("command.batch", &ap).is_err());
    assert!(doc_events(&f.s, at).is_empty());
}

#[test]
fn a_hand_edit_tells_the_edited_layer_apart_from_the_ones_that_moved() {
    let mut s = Session::new();
    let card = id_of(
        &run(
            &mut s,
            "frame.create",
            json!({"x": 0, "y": 0, "width": 200, "height": 100, "name": "Card", "props": {"layout": "vertical", "sizingV": "hug"}}),
        )["id"],
    )
    .unwrap();
    let a = id_of(&run(&mut s, "text.create", json!({"parent": card.0, "text": "A", "x": 0, "y": 0}))["id"]).unwrap();
    let b = id_of(&run(&mut s, "text.create", json!({"parent": card.0, "text": "B", "x": 0, "y": 0}))["id"]).unwrap();
    let at = s.events.latest();
    run(&mut s, "node.set", json!({"id": a.0, "props": {"fontSize": 40}}));
    let ev = doc_events(&s, at);
    assert_eq!(ev.len(), 1);
    assert_eq!(ev[0].data["edited"], json!([a.0]), "{:?}", ev[0].data);
    let changed = ev[0].data["changed"].as_array().unwrap();
    assert!(changed.contains(&json!(b.0)) || changed.contains(&json!(card.0)), "layout followed: {:?}", ev[0].data);
}

fn origin(s: &mut Session, id: NodeId) -> Value {
    run(s, "origin.get", json!({"id": id.0}))["aspects"].clone()
}

#[test]
fn origin_records_who_changed_each_aspect() {
    let mut f = setup();
    run(&mut f.s, "origin.track", json!({"on": true}));
    f.s.execute_as(Some("Claude"), "node.set", &json!({"id": f.card.0, "props": {"fill": "#FF0000"}})).unwrap();
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"name": "Mine"}}));
    let o = origin(&mut f.s, f.card);
    assert_eq!(o["fills"]["by"], json!("ai"), "{o}");
    assert_eq!(o["fills"]["agent"], json!("Claude (agent)"));
    assert_eq!(o["name"]["by"], json!("human"), "{o}");
    // A later hand edit of the same aspect takes it over.
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"fill": "#00FF00"}}));
    assert_eq!(origin(&mut f.s, f.card)["fills"]["by"], json!("human"));
    let ai = run(&mut f.s, "origin.list", json!({}));
    assert!(ai.as_array().unwrap().iter().all(|e| e["id"] != json!(f.card.0)), "{ai}");
}

#[test]
fn origin_goes_back_with_undo_and_a_dry_run_leaves_none() {
    let mut f = setup();
    run(&mut f.s, "origin.track", json!({"on": true}));
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"name": "Card"}}));
    let card = id_of(&run(&mut f.s, "node.create", json!({"type": "frame", "name": "Made by hand", "width": 10, "height": 10}))["id"]).unwrap();
    let made_new = origin(&mut f.s, card);
    assert_eq!(made_new["name"]["by"], json!("human"), "{made_new}");
    assert!(made_new.get("type").is_none(), "the kind of layer is no aspect: {made_new}");
    let made = origin(&mut f.s, f.card);
    let dp = batch(&f, json!({"dryRun": true}));
    f.s.execute_as(Some("Claude"), "command.batch", &dp).unwrap();
    assert_eq!(origin(&mut f.s, f.card), made, "a dry run stamps nothing");
    f.s.execute_as(Some("Claude"), "node.set", &json!({"id": f.card.0, "props": {"name": "AI"}})).unwrap();
    assert_eq!(origin(&mut f.s, f.card)["name"]["by"], json!("ai"));
    run(&mut f.s, "edit.undo", json!({}));
    assert_eq!(origin(&mut f.s, f.card), made, "undo takes the stamp back with the edit");
    run(&mut f.s, "edit.redo", json!({}));
    assert_eq!(origin(&mut f.s, f.card)["name"]["by"], json!("ai"));
}

#[test]
fn a_proposal_applied_by_a_click_is_credited_to_its_author() {
    let mut f = setup();
    run(&mut f.s, "origin.track", json!({"on": true}));
    let pp = batch(&f, json!({}));
    f.s.execute_as(Some("Claude"), "proposal.preview", &pp).unwrap();
    // The designer clicks Apply: no agent on this call.
    run(&mut f.s, "proposal.apply", json!({}));
    let o = origin(&mut f.s, f.card);
    assert_eq!(o["name"]["by"], json!("ai"), "{o}");
    assert_eq!(o["name"]["agent"], json!("Claude (agent)"));
    let ai = run(&mut f.s, "origin.list", json!({"by": "ai"}));
    assert!(ai.as_array().unwrap().iter().any(|e| e["id"] == json!(f.card.0)));
    assert!(f.s.execute("origin.list", &json!({"by": "robot"})).is_err());
}

#[test]
fn origin_is_not_tracked_unless_asked() {
    let mut f = setup();
    f.s.execute_as(Some("Claude"), "node.set", &json!({"id": f.card.0, "props": {"name": "AI"}})).unwrap();
    assert_eq!(origin(&mut f.s, f.card), json!({}));
    assert!(f.s.doc().unwrap().node(f.card).unwrap().plugin_data.is_empty());
    assert!(f.s.execute("origin.track", &json!({"on": "yes"})).is_err());
}

#[test]
fn undo_only_the_ai_keeps_the_designer_edits() {
    let mut f = setup();
    run(&mut f.s, "origin.track", json!({"on": true}));
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"fill": "#00FF00"}}));
    let claude = |s: &mut Session, p: Value| {
        s.execute_as(Some("Claude"), "node.set", &p).unwrap();
    };
    claude(&mut f.s, json!({"id": f.card.0, "props": {"fill": "#FF0000", "opacity": 0.5}}));
    claude(&mut f.s, json!({"id": f.card.0, "props": {"fill": "#0000FF"}}));
    // The designer then renames it: theirs, untouched by the undo.
    run(&mut f.s, "node.set", json!({"id": f.card.0, "props": {"name": "Mine"}}));
    let depth = f.s.undo_depth();
    let r = run(&mut f.s, "origin.undoAI", json!({"ids": [f.card.0]}));
    assert_eq!(r["reverted"][0]["id"], json!(f.card.0), "{r}");
    assert_eq!(f.s.undo_depth().0, depth.0 + 1, "one undo step");
    let card = node_value(&f.s, f.card);
    assert_eq!(card["name"], json!("Mine"));
    assert!(
        card["fills"].to_string().contains("0.0") && !card["fills"].to_string().contains("\"b\":1.0"),
        "fill back to the designer's green, not Claude's red or blue: {}",
        card["fills"]
    );
    assert!(card.get("opacity").is_none_or(|o| o == &json!(1.0)), "opacity back: {card}");
    let o = origin(&mut f.s, f.card);
    assert_eq!(o["fills"]["by"], json!("human"), "{o}");
    // A layer the agent made is listed, never deleted.
    let made = f.s.execute_as(Some("Claude"), "node.create", &json!({"type": "rectangle", "name": "AI made", "width": 5, "height": 5})).unwrap();
    let mid = id_of(&made["id"]).unwrap();
    let r = run(&mut f.s, "origin.undoAI", json!({"ids": [mid.0]}));
    assert!(r["reverted"].as_array().unwrap().is_empty() && r["kept"][0]["id"] == json!(mid.0), "{r}");
    assert!(f.s.doc().unwrap().node(mid).is_some());
    assert!(f.s.execute("origin.undoAI", &json!({"ids": "x"})).is_err());
}

// ds.lint (Frente F): design-system lint, read-only, every finding cites its source.

struct LintFixture {
    s: Session,
    screen: NodeId,
    price: NodeId,
    button: NodeId,
    color: String,
    size: String,
    space: String,
}

fn lint_setup() -> LintFixture {
    let mut s = Session::new();
    run(&mut s, "variable.createCollection", json!({"name": "Tokens WEB", "modes": ["Farm", "Animale"]}));
    let id = |v: Value| v["id"].as_str().unwrap_or_default().to_string();
    let color = id(run(
        &mut s,
        "variable.create",
        json!({"collection": "Tokens WEB", "name": "Colors/Surface/Brand/Background", "type": "color", "value": "#112233"}),
    ));
    let size = id(run(&mut s, "variable.create", json!({"collection": "Tokens WEB", "name": "Font/Size/fs-xs", "type": "number", "value": 16})));
    run(&mut s, "variable.update", json!({"variable": size, "scopes": ["FONT_SIZE"]}));
    let space = id(run(&mut s, "variable.create", json!({"collection": "Tokens WEB", "name": "Spacing/sp-md", "type": "number", "value": 16})));
    run(&mut s, "variable.update", json!({"variable": space, "scopes": ["GAP"]}));
    let screen = id_of(&run(&mut s, "node.create", json!({"type": "frame", "name": "Tela", "width": 375, "height": 600}))["id"]).unwrap();
    let price = id_of(&run(&mut s, "text.create", json!({"text": "Preço", "x": 10, "y": 10, "parent": screen.0}))["id"]).unwrap();
    run(&mut s, "node.set", json!({"id": price.0, "props": {"fontSize": 18}}));
    let button =
        id_of(&run(&mut s, "node.create", json!({"type": "frame", "name": "Botão", "width": 120, "height": 40, "parent": screen.0}))["id"]).unwrap();
    run(&mut s, "node.set", json!({"id": button.0, "props": {"fill": "#7A4B2A"}}));
    LintFixture { s, screen, price, button, color, size, space }
}

fn findings(s: &mut Session, p: Value) -> Vec<Value> {
    run(s, "ds.lint", p)["findings"].as_array().cloned().unwrap_or_default()
}

fn has(list: &[Value], id: NodeId, rule: &str, prop: &str) -> bool {
    list.iter().any(|f| f["id"] == json!(id.0) && f["rule"] == json!(rule) && f["prop"] == json!(prop))
}

#[test]
fn lint_flags_raw_values_where_a_token_exists_and_cites_the_source() {
    let mut f = lint_setup();
    let list = findings(&mut f.s, json!({}));
    assert!(has(&list, f.price, "valor-solto", "fontSize"), "{list:?}");
    assert!(has(&list, f.button, "valor-solto", "fill"), "{list:?}");
    let price = list.iter().find(|x| x["id"] == json!(f.price.0) && x["prop"] == json!("fontSize")).unwrap();
    assert_eq!(price["value"], json!(18.0));
    assert!(price["message"].as_str().unwrap().contains("18"), "{price}");
    assert!(price["source"].as_str().unwrap().contains("notion.com"), "{price}");
    let button = list.iter().find(|x| x["id"] == json!(f.button.0) && x["prop"] == json!("fill")).unwrap();
    assert_eq!(button["value"], json!("#7A4B2A"));
    // Bound to a token: no longer raw.
    run(&mut f.s, "variable.bind", json!({"ids": [f.price.0], "field": "fontSize", "variable": f.size}));
    let list = findings(&mut f.s, json!({}));
    assert!(!has(&list, f.price, "valor-solto", "fontSize"), "{list:?}");
}

#[test]
fn lint_skips_raw_values_with_no_token_for_them() {
    let mut s = Session::new();
    let frame = id_of(&run(&mut s, "node.create", json!({"type": "frame", "name": "Tela", "width": 100, "height": 100}))["id"]).unwrap();
    run(&mut s, "node.set", json!({"id": frame.0, "props": {"fill": "#7A4B2A"}}));
    let out = run(&mut s, "ds.lint", json!({}));
    assert_eq!(out["counted"], json!(0), "{out}");
    // A number variable scoped to font size is no token for a radius.
    run(&mut s, "variable.createCollection", json!({"name": "Tokens WEB", "modes": ["Farm"]}));
    let v = run(&mut s, "variable.create", json!({"collection": "Tokens WEB", "name": "Font/Size/fs-xs", "type": "number", "value": 16}));
    run(&mut s, "variable.update", json!({"variable": v["id"], "scopes": ["FONT_SIZE"]}));
    run(&mut s, "node.set", json!({"id": frame.0, "props": {"cornerRadius": 8}}));
    assert!(!has(&findings(&mut s, json!({})), frame, "valor-solto", "cornerRadius"));
}

#[test]
fn lint_flags_variables_outside_their_scope_and_d4_padding() {
    let mut f = lint_setup();
    run(&mut f.s, "node.set", json!({"id": f.button.0, "props": {"layoutMode": "horizontal"}}));
    run(&mut f.s, "variable.bind", json!({"ids": [f.button.0], "field": "cornerRadius", "variable": f.size}));
    run(&mut f.s, "variable.bind", json!({"ids": [f.button.0], "field": "paddingHorizontal", "variable": f.space}));
    run(&mut f.s, "variable.bind", json!({"ids": [f.button.0], "field": "gap", "variable": f.space}));
    let list = findings(&mut f.s, json!({}));
    assert!(has(&list, f.button, "escopo", "cornerRadius"), "{list:?}");
    // GAP covers gap and padding in Figma: neither is out of scope (D4 withdrawn on 24/11).
    assert!(!has(&list, f.button, "escopo", "paddingHorizontal"), "{list:?}");
    assert!(!has(&list, f.button, "escopo", "gap"), "gap is the GAP scope: {list:?}");
}

#[test]
fn lint_flags_a_screen_left_on_the_default_brand() {
    let mut f = lint_setup();
    // No brand token in the screen yet: nothing to choose.
    assert!(!has(&findings(&mut f.s, json!({})), f.screen, "marca-nao-escolhida", "variableMode"));
    run(&mut f.s, "variable.bind", json!({"ids": [f.button.0], "field": "fill", "variable": f.color}));
    let list = findings(&mut f.s, json!({}));
    let miss = list.iter().find(|x| x["rule"] == json!("marca-nao-escolhida")).expect("screen without brand");
    assert_eq!(miss["id"], json!(f.screen.0));
    assert!(miss["message"].as_str().unwrap().contains("Farm"), "names the default it inherits: {miss}");
    run(&mut f.s, "variable.setMode", json!({"ids": [f.screen.0], "collection": "Tokens WEB", "mode": "Animale"}));
    assert!(!findings(&mut f.s, json!({})).iter().any(|x| x["rule"] == json!("marca-nao-escolhida")));
    // Another collection name: the rule follows `brandCollection`, and is off when it is absent.
    let out = run(&mut f.s, "ds.lint", json!({"brandCollection": "Tokens APP"}));
    assert!(!out["findings"].as_array().unwrap().iter().any(|x| x["rule"].as_str().unwrap_or("").starts_with("marca")));
}

#[test]
fn lint_flags_a_brand_mode_on_the_main_component() {
    let mut f = lint_setup();
    let out = run(&mut f.s, "component.create", json!({"ids": [f.button.0]}));
    let comp = id_of(&out["id"]).unwrap_or(f.button);
    run(&mut f.s, "variable.setMode", json!({"ids": [comp.0], "collection": "Tokens WEB", "mode": "Animale"}));
    let list = findings(&mut f.s, json!({}));
    let on = list.iter().find(|x| x["rule"] == json!("marca-no-componente")).expect("brand on component");
    assert_eq!(on["id"], json!(comp.0));
    assert_eq!(on["value"], json!("Animale"));
    assert!(on["source"].as_str().unwrap().contains("39e690e9a7e281978c0dfc4333a87e8c"), "{on}");
    // The same mode on the screen is fine.
    run(&mut f.s, "variable.setMode", json!({"ids": [comp.0], "collection": "Tokens WEB", "mode": null}));
    run(&mut f.s, "variable.setMode", json!({"ids": [f.screen.0], "collection": "Tokens WEB", "mode": "Animale"}));
    assert!(!findings(&mut f.s, json!({})).iter().any(|x| x["rule"] == json!("marca-no-componente")));
}

#[test]
fn lint_is_read_only_bounded_and_refuses_bad_params() {
    let mut f = lint_setup();
    let doc = f.s.doc().unwrap().clone();
    let depth = f.s.undo_depth();
    let all = run(&mut f.s, "ds.lint", json!({}));
    assert_eq!(*f.s.doc().unwrap(), doc);
    assert_eq!(f.s.undo_depth(), depth);
    let counted = all["counted"].as_u64().unwrap();
    assert!(counted >= 2, "{all}");
    let one = run(&mut f.s, "ds.lint", json!({"limit": 1}));
    assert_eq!(one["findings"].as_array().unwrap().len(), 1);
    assert_eq!(one["counted"], json!(counted));
    assert_eq!(one["truncated"], json!(true));
    // `ids`: only that subtree.
    let only = findings(&mut f.s, json!({"ids": [f.price.0]}));
    assert!(only.iter().all(|x| x["id"] == json!(f.price.0)), "{only:?}");
    for bad in [
        json!({"limit": 0}),
        json!({"limit": "x"}),
        json!({"ids": "x"}),
        json!({"ids": []}),
        json!({"ids": ["nope"]}),
        json!({"ids": [999_999]}),
        json!({"brandCollection": 5}),
    ] {
        assert!(f.s.execute("ds.lint", &bad).is_err(), "{bad}");
    }
}

#[test]
fn lint_flags_a_variable_from_a_non_official_collection() {
    let mut s = Session::new();
    let id = |v: Value| v["id"].as_str().unwrap_or_default().to_string();
    run(&mut s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["Farm", "Animale (CERTO)"]}));
    run(&mut s, "variable.createCollection", json!({"name": "Rascunho", "modes": ["Mode 1"]}));
    let ok = id(run(
        &mut s,
        "variable.create",
        json!({"collection": "Tokens APP", "name": "Colors/Surface/Background", "type": "color", "value": "#FFFFFF"}),
    ));
    let draft = id(run(&mut s, "variable.create", json!({"collection": "Rascunho", "name": "Cor nova", "type": "color", "value": "#123456"})));
    let a = id_of(&run(&mut s, "node.create", json!({"type": "rectangle", "name": "Oficial", "width": 10, "height": 10}))["id"]).unwrap();
    let b = id_of(&run(&mut s, "node.create", json!({"type": "rectangle", "name": "Solta", "width": 10, "height": 10}))["id"]).unwrap();
    run(&mut s, "variable.bind", json!({"ids": [a.0], "field": "fill", "variable": ok}));
    run(&mut s, "variable.bind", json!({"ids": [b.0], "field": "fill", "variable": draft}));
    let list = findings(&mut s, json!({}));
    let hit: Vec<&Value> = list.iter().filter(|f| f["rule"] == json!("colecao-nao-oficial")).collect();
    assert_eq!(hit.len(), 1, "{list:?}");
    assert_eq!(hit[0]["id"], json!(b.0));
    assert!(hit[0]["message"].as_str().unwrap().contains("Rascunho") && hit[0]["source"].as_str().unwrap().contains("Fluxo Oficial"), "{:?}", hit[0]);
    // The list can be given: with both collections official, nothing is flagged.
    let list = findings(&mut s, json!({"officialCollections": ["Tokens APP", "Rascunho"]}));
    assert!(!list.iter().any(|f| f["rule"] == json!("colecao-nao-oficial")), "{list:?}");
    // None of the official collections in the document: nothing to compare with.
    let list = findings(&mut s, json!({"officialCollections": ["Outra"]}));
    assert!(!list.iter().any(|f| f["rule"] == json!("colecao-nao-oficial")), "{list:?}");
    assert!(s.execute("ds.lint", &json!({"officialCollections": []})).is_err());
    assert!(s.execute("ds.lint", &json!({"officialCollections": [3]})).is_err());
}

#[test]
fn a_preview_lints_what_the_proposal_makes_and_takes_a_review() {
    let mut s = Session::new();
    run(&mut s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["Farm", "Animale (CERTO)"]}));
    run(&mut s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Surface/Background", "type": "color", "value": "#FFFFFF"}));
    let keep = id_of(
        &run(&mut s, "node.create", json!({"type": "rectangle", "name": "Untouched", "width": 10, "height": 10, "props": {"fill": "#FF0000"}}))["id"],
    )
    .unwrap();
    assert!(s.execute("proposal.review", &json!({"verdict": "aprovado"})).is_err(), "no proposal yet");
    let p = json!({"commands": [{"command": "node.create", "params": {"type": "rectangle", "name": "Raw", "width": 10, "height": 10, "props": {"fill": "#00FF00"}}}]});
    let out = s.execute_as(Some("Claude"), "proposal.preview", &p).unwrap();
    let lint = out["lint"].as_array().unwrap();
    assert!(lint.iter().any(|f| f["name"] == json!("Raw") && f["rule"] == json!("valor-solto")), "{lint:?}");
    assert!(!lint.iter().any(|f| f["id"] == json!(keep.0)), "only what the proposal makes or edits: {lint:?}");
    let r = run(
        &mut s,
        "proposal.review",
        json!({"verdict": "com-ressalvas", "reviewer": "DS Gatekeeper", "summary": "Cor solta no retângulo novo.",
        "items": [{"message": "Use um token de Colors/Surface.", "source": "Fluxo Oficial §4"}]}),
    );
    assert_eq!(r["review"]["verdict"], json!("com-ressalvas"));
    assert_eq!(r["review"]["advisory"], json!(true));
    assert!(s.execute("proposal.review", &json!({"verdict": "ok"})).is_err());
    assert!(s.execute("proposal.review", &json!({"verdict": "aprovado", "items": [{"source": "x"}]})).is_err());
    // A review is advisory: even "bloqueado" leaves Apply to the designer.
    run(&mut s, "proposal.review", json!({"verdict": "bloqueado"}));
    assert_eq!(run(&mut s, "proposal.apply", json!({}))["applied"], json!(true));
}
