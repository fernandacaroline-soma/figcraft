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
fn proposal_preview_shows_the_result_read_only_and_apply_ends_it() {
    let mut f = setup();
    let bp = batch(&f, json!({}));
    let dry = run(&mut f.s, "proposal.preview", bp);
    let hash = dry["baseHash"].as_str().unwrap().to_string();
    assert!(f.s.ds().unwrap().preview.is_some());
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card v2");
    assert!(f.s.execute("node.set", &json!({"id": f.other.0, "props": {"name": "x"}})).is_err());
    assert_eq!(run(&mut f.s, "proposal.previewEnd", json!({}))["ended"], json!(true));
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card");
    let bp = batch(&f, json!({}));
    run(&mut f.s, "proposal.preview", bp);
    let bp = batch(&f, json!({"ifUnchanged": hash}));
    run(&mut f.s, "command.batch", bp);
    assert!(f.s.ds().unwrap().preview.is_none());
    assert_eq!(f.s.doc().unwrap().node(f.card).unwrap().name, "Card v2");
}
