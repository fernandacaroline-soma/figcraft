//! Visual regression for Eureka's exploration board (E14): a proposal screen drawn from its
//! description (frames, auto layout, spacing and colour tokens, a bottom-docked bar), exported as
//! PNG in two brands, against reference images kept in `src/snapshots/`. No text: glyphs come from
//! the system's fonts and differ between machines.
//!
//! A deliberate change of the drawing updates the references: run the tests with
//! `FIGCRAFT_UPDATE_SNAPSHOTS=1` and review the new PNGs in the diff.

use serde_json::json;

use crate::components::tests::{ex, idv};
use crate::{NodeId, Session};

/// The board's first proposal screen, with two brands in the token collection.
fn screen(s: &mut Session) -> NodeId {
    ex(s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["NV", "Farm"]}));
    for (name, nv, farm) in [
        ("Colors/Surface/Background", "#ffffff", "#fbf6ee"),
        ("Colors/Surface/Container", "#e9ecea", "#f1dcc4"),
        ("Colors/Brand/Primary", "#1f7a5a", "#c8442b"),
        ("Colors/Content/Muted", "#9aa6a1", "#b59a7c"),
    ] {
        ex(s, "variable.create", json!({"collection": "Tokens APP", "name": name, "type": "color", "values": {"NV": nv, "Farm": farm}}));
    }
    for (name, v) in [("Spacing/sp-xs", 8), ("Spacing/sp-md", 16)] {
        ex(s, "variable.create", json!({"collection": "Tokens APP", "name": name, "type": "number", "value": v}));
    }
    let pdp = json!({"frame": "PDP", "width": 180, "height": 320, "fill": "Colors/Surface/Background", "gap": "Spacing/sp-xs", "children": [
        {"image": "Galeria", "height": 150, "fill": "Colors/Surface/Container"},
        {"frame": "Tamanhos", "layout": "horizontal", "gap": "Spacing/sp-xs", "padding": {"h": "Spacing/sp-md", "v": "Spacing/sp-xs"}, "children": [
            {"image": "P", "width": 28, "height": 28, "fill": "Colors/Content/Muted"},
            {"image": "M", "width": 28, "height": 28, "fill": "Colors/Brand/Primary"},
            {"image": "G", "width": 28, "height": 28, "fill": "Colors/Content/Muted"}
        ]},
        {"frame": "Área fixa", "dock": "bottom", "height": 56, "padding": "Spacing/sp-md", "fill": "Colors/Surface/Container", "children": [
            {"image": "Comprar", "height": 24, "fill": "Colors/Brand/Primary"}
        ]}
    ]});
    let proposal = |name: &str| json!({"name": name, "type": "incremental", "summary": "Tamanho antes do preço.", "screen": pdp.clone()});
    let r = ex(s, "explore.board", json!({"title": "Snapshot", "proposals": [proposal("Um"), proposal("Dois")]}));
    idv(&r["proposals"][0])
}

/// `id` exported as PNG at scale 1.
fn png(s: &mut Session, id: NodeId) -> image::RgbaImage {
    let doc = s.doc().unwrap();
    let bytes = crate::cmd::file::export_bytes(doc, &[id], figcraft_doc::ExportFormat::Png, 1.0).unwrap();
    image::load_from_memory(&bytes).unwrap().to_rgba8()
}

/// `got` matches the reference `name` (written instead with `FIGCRAFT_UPDATE_SNAPSHOTS=1`, or when
/// missing, which fails so a new reference is always reviewed).
fn matches_reference(got: &image::RgbaImage, name: &str) {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src/snapshots").join(format!("{name}.png"));
    let update = std::env::var("FIGCRAFT_UPDATE_SNAPSHOTS").is_ok_and(|v| v == "1");
    if update || !path.exists() {
        got.save(&path).unwrap();
        assert!(update, "{name}: no reference yet; wrote {} (review it, then run again)", path.display());
        return;
    }
    let want = image::open(&path).unwrap().to_rgba8();
    assert_eq!(got.dimensions(), want.dimensions(), "{name}: size changed");
    // Antialiasing may differ by a step between platforms; a changed colour, a moved or missing
    // block does not.
    let diffs: Vec<u8> = got.as_raw().iter().zip(want.as_raw()).map(|(a, b)| a.abs_diff(*b)).collect();
    let max = diffs.iter().copied().max().unwrap_or(0);
    let off = diffs.iter().filter(|d| **d > 8).count();
    assert!(off * 1000 <= diffs.len(), "{name}: {off} channel values differ by more than 8 (max {max}); see {}", path.display());
}

#[test]
fn a_proposal_screen_draws_as_its_reference_in_each_brand() {
    let mut s = Session::new();
    let id = screen(&mut s);
    ex(&mut s, "variable.setMode", json!({"ids": [id.0], "collection": "Tokens APP", "mode": "NV"}));
    let nv = png(&mut s, id);
    matches_reference(&nv, "eureka-pdp-nv");
    ex(&mut s, "variable.setMode", json!({"ids": [id.0], "collection": "Tokens APP", "mode": "Farm"}));
    let farm = png(&mut s, id);
    matches_reference(&farm, "eureka-pdp-farm");
    assert_ne!(nv.as_raw(), farm.as_raw(), "the brand changes the drawing");
}

#[test]
fn a_changed_token_fails_the_snapshot() {
    // The check itself: a token value that moved is caught (this is what a regression looks like).
    // Not while references are being written: it would write the wrong one.
    if std::env::var("FIGCRAFT_UPDATE_SNAPSHOTS").is_ok_and(|v| v == "1") {
        return;
    }
    let mut s = Session::new();
    let id = screen(&mut s);
    ex(&mut s, "variable.setMode", json!({"ids": [id.0], "collection": "Tokens APP", "mode": "NV"}));
    ex(&mut s, "variable.set", json!({"variable": "Colors/Brand/Primary", "mode": "NV", "value": "#2f6fb5"}));
    let got = png(&mut s, id);
    let caught = std::panic::catch_unwind(|| matches_reference(&got, "eureka-pdp-nv"));
    assert!(caught.is_err(), "a different primary colour must not pass as the reference");
}
