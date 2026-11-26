//! Eureka's agent contract (E15): an agent prepares and advises, the person decides. The designer's
//! decisions refuse an agent, also inside a batch; the person (no agent) runs them as before.

use serde_json::json;

use crate::components::tests::{ex, idv};
use crate::{NodeId, Session};

fn board(s: &mut Session) -> NodeId {
    ex(s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["NV"]}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"}));
    let screen = |name: &str| json!({"name": name, "type": "incremental", "screen": {"frame": "PDP", "width": 100, "height": 100, "fill": "Colors/Bg", "children": []}});
    let r = ex(s, "explore.board", json!({"title": "Rodada", "proposals": [screen("Um"), screen("Dois")]}));
    idv(&r["proposals"][0])
}

#[test]
fn an_agent_cannot_make_the_designers_decisions() {
    let mut s = Session::new();
    let a = board(&mut s);
    for (cmd, params) in [
        ("explore.choose", json!({"id": a.0})),
        ("explore.critiqueReviewed", json!({"id": a.0})),
        ("explore.critiqueStatus", json!({"id": a.0, "finding": 1, "status": "resolvido"})),
        ("explore.approve", json!({"id": a.0})),
        ("proposal.apply", json!({})),
    ] {
        let e = s.execute_as(Some("Claude"), cmd, &params).unwrap_err().to_string();
        assert!(e.contains("designer's decision"), "{cmd}: {e}");
    }
    // Nor through a batch it runs.
    let e = s
        .execute_as(Some("Claude"), "command.batch", &json!({"commands": [{"command": "explore.choose", "params": {"id": a.0}}]}))
        .unwrap_err()
        .to_string();
    assert!(e.contains("designer's decision"), "{e}");
    let info = s.doc().unwrap().node(a).unwrap().plugin_data.get("eureka.proposal").cloned().unwrap();
    assert!(!info.contains("\"escolhida\""), "nothing was chosen: {info}");
    // What an agent prepares still works, as an agent.
    assert!(s.execute_as(Some("Claude"), "explore.measure", &json!({"id": a.0})).is_ok());
    // The person picks (the card's button runs it with no agent).
    assert!(s.execute_as(None, "explore.choose", &json!({"id": a.0})).is_ok());
}

#[test]
fn a_reviewer_agent_reads_the_chosen_proposals_context_without_choosing() {
    // The DS Gatekeeper is an agent: it may not run explore.choose (the designer's decision), so it
    // reads what it reviews with explore.context, only once the designer has chosen.
    let mut s = Session::new();
    let a = board(&mut s);
    let e = s.execute_as(Some("DS Gatekeeper"), "explore.context", &json!({"id": a.0})).unwrap_err().to_string();
    assert!(e.contains("hasn't chosen"), "{e}");
    s.execute_as(None, "explore.choose", &json!({"id": a.0})).unwrap();
    let before = s.doc().unwrap().node(a).unwrap().plugin_data.clone();
    let r = s.execute_as(Some("DS Gatekeeper"), "explore.context", &json!({"id": a.0})).unwrap();
    assert_eq!(r["chosen"]["id"], json!(a.0));
    assert!(r.get("lint").is_some() && r.get("context").is_some(), "{r}");
    assert_eq!(s.doc().unwrap().node(a).unwrap().plugin_data, before, "read only");
}
