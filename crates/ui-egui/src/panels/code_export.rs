//! "Export code" in the Export section (Eureka Design): the selected screen as the developers
//! receive it. One `export.code` run writes `<Screen>.tsx` and `tokens.css` into a folder the
//! designer picks (downloads in the browser), then the section lists what went out as a token and
//! what was left as a value with no token (the warnings), so the designer sees the handoff.

use egui::Ui;
use figcraft_doc::NodeId;
use serde_json::{Value, json};

use crate::FigApp;
use crate::i18n::{tr, trf, trn};
use crate::theme::Tokens;
use crate::widgets;

/// Most rows each list shows (the files have them all).
const MAX_ROWS: usize = 40;

/// The last code export: the layers it was for and `export.code`'s result.
pub struct Report {
    pub ids: Vec<NodeId>,
    pub result: Value,
}

/// The button, under the layer export, and the report of the last code export of `sel`.
pub fn section(app: &mut FigApp, ui: &mut Ui, sel: &[NodeId]) {
    if sel.is_empty() {
        return;
    }
    if crate::export_panel::wide_button(ui, tr("Export code")) {
        export(app, sel);
    }
    let Some(report) = app.ui.code_export.as_ref().filter(|r| r.ids == sel) else { return };
    show(ui, &report.result);
}

/// Run `export.code` for `ids`: into a picked folder, or as two downloads in the browser.
pub fn export(app: &mut FigApp, ids: &[NodeId]) {
    let ids = ids.to_vec();
    if app.services.downloads() {
        let Some(r) = app.run("export.code", json!({"ids": ids})) else { return };
        use base64::Engine as _;
        for f in r["files"].as_array().cloned().unwrap_or_default() {
            let name = f["name"].as_str().unwrap_or("export.txt");
            let data = base64::engine::general_purpose::STANDARD.encode(f["text"].as_str().unwrap_or_default());
            app.download(name, &json!({"data": data}));
        }
        done(app, ids, r);
        return;
    }
    app.pick_folder(tr("Export code"), move |app, dir| {
        if let Some(r) = app.run("export.code", json!({"ids": ids, "dir": dir.to_string_lossy()})) {
            done(app, ids, r);
        }
    });
}

fn done(app: &mut FigApp, ids: Vec<NodeId>, result: Value) {
    let loose = result["warnings"].as_array().map_or(0, Vec::len);
    app.toast(if loose == 0 {
        tr("Code exported: every value is a token").to_string()
    } else {
        trn("Code exported; {n} values have no token", loose, &[])
    });
    app.ui.code_export = Some(Report { ids, result });
}

/// What the last export gave the developers: files, tokens, values with no token, notes.
fn show(ui: &mut Ui, r: &Value) {
    let t = Tokens::get(ui.ctx());
    let files: Vec<String> = r["files"].as_array().into_iter().flatten().filter_map(|f| f["name"].as_str().map(str::to_string)).collect();
    widgets::note(ui, &trf("Files: {files}", &[("files", &files.join(", "))]), t.text_secondary);
    // Where they went (desktop): the folder of the first one.
    let first_path = r["files"].get(0).and_then(|f| f["path"].as_str()).map(std::path::Path::new);
    if let Some(dir) = first_path.and_then(std::path::Path::parent) {
        widgets::note(ui, &trf("In {dir}", &[("dir", &dir.to_string_lossy())]), t.text_tertiary);
    }
    let brand = &r["brand"];
    if let Some(b) = brand["brand"].as_str() {
        let mode = brand["mode"].as_str().unwrap_or_default();
        widgets::note(ui, &trf("Brand: {brand} ({mode})", &[("brand", &b), ("mode", &mode)]), t.text_secondary);
    }
    let tokens = r["tokens"].as_array().cloned().unwrap_or_default();
    widgets::note(ui, &trn("As tokens ({n})", tokens.len(), &[]), t.text);
    for tok in tokens.iter().take(MAX_ROWS) {
        let name = tok["token"].as_str().unwrap_or_default();
        if tok["defined"].as_bool() == Some(true) {
            widgets::note(ui, name, t.success);
        } else {
            widgets::note(ui, &trf("{token} (not in tokens.css)", &[("token", &name)]), t.warning);
        }
    }
    let loose = r["warnings"].as_array().cloned().unwrap_or_default();
    if loose.is_empty() {
        widgets::note(ui, tr("No values left without a token"), t.text_secondary);
    } else {
        widgets::note(ui, &trn("Without a token ({n})", loose.len(), &[]), t.text);
        for w in loose.iter().take(MAX_ROWS) {
            let s = |k: &str| w[k].as_str().unwrap_or_default().to_string();
            widgets::note(ui, &format!("{} · {}: {}", s("layer"), s("field"), s("value")), t.warning);
        }
    }
    for n in r["notes"].as_array().into_iter().flatten().filter_map(Value::as_str).take(MAX_ROWS) {
        widgets::note(ui, n, t.text_secondary);
    }
    let more = tokens.len().max(loose.len()) > MAX_ROWS;
    if more {
        widgets::note(ui, tr("The files have the full lists."), t.text_tertiary);
    }
}
