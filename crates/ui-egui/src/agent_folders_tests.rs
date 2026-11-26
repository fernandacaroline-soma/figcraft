//! E15: with the control channel on, an agent's requests read and write only in the temporary
//! folder and the folders the designer allowed (Preferences); the designer's own work isn't confined.

use figcraft_engine::Session;
use serde_json::json;

use crate::layers_drag_tests::Ui;

fn app_with_a_frame() -> (Ui, u64) {
    let mut s = Session::new();
    let f = s.execute("frame.create", &json!({"x": 0, "y": 0, "width": 20, "height": 20})).unwrap()["id"].as_u64().unwrap();
    let mut u = Ui::new(s);
    u.app.agents_confined = true;
    u.frame(vec![]);
    (u, f)
}

/// A folder of its own for test `name`, outside the temporary one (under the crate; tests run in
/// parallel, so each has its own).
fn outside(name: &str) -> std::path::PathBuf {
    let d = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("target-agent-folders-{name}"));
    let _ = std::fs::create_dir_all(&d);
    std::fs::canonicalize(&d).unwrap()
}

fn export(u: &mut Ui, id: u64, path: &std::path::Path) -> Result<serde_json::Value, String> {
    let ctx = u.ctx.clone();
    let p = json!({"command": "file.export", "params": {"ids": [id], "format": "png", "path": path.to_string_lossy()}});
    crate::control::handle(&mut u.app, &ctx, "engine.execute", &p)
}

#[test]
fn an_agent_writes_only_where_the_designer_allowed() {
    let (mut u, f) = app_with_a_frame();
    let dir = outside("agent");
    let target = dir.join("x.png");
    let _ = std::fs::remove_file(&target);
    // Outside the allowed folders: refused, nothing written.
    let e = export(&mut u, f, &target).unwrap_err();
    assert!(!target.exists(), "{e}");
    // The temporary folder is allowed (scripts' work folders).
    let tmp = std::env::temp_dir().join(format!("agent-folders-{}.png", std::process::id()));
    assert!(export(&mut u, f, &tmp).is_ok());
    let _ = std::fs::remove_file(&tmp);
    // The designer allows the folder in Preferences: now it works.
    u.app.session.prefs.agent_folders.push(dir.to_string_lossy().into_owned());
    assert!(export(&mut u, f, &target).is_ok());
    assert!(target.exists());
    // Removed again: refused again.
    u.app.session.prefs.agent_folders.clear();
    let _ = std::fs::remove_file(&target);
    assert!(export(&mut u, f, &target).is_err());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn the_designers_own_work_is_not_confined() {
    let (mut u, f) = app_with_a_frame();
    let dir = outside("designer");
    let target = dir.join("mine.png");
    // The window's own command (a menu, a button): not a control request, not confined.
    let r = u.app.run("file.export", json!({"ids": [f], "format": "png", "path": target.to_string_lossy()}));
    assert!(r.is_some() && target.exists(), "{r:?}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn no_command_lets_an_agent_change_the_allowed_folders() {
    // The list is changed in Preferences only: no engine command an agent could run names it.
    let ids: Vec<&str> = figcraft_engine::cmd::all_commands().iter().map(|c| c.id).collect();
    assert!(!ids.iter().any(|id| id.to_lowercase().contains("agentfolder")), "{ids:?}");
}
