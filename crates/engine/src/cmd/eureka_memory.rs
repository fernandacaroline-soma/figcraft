//! Visual memory (Eureka G3, Frente U): the last step of the proposals generator ("gerador-propostas",
//! phase E, `references/memoria-visual.md`). When the designer approves the refined layout
//! (`explore.approve`), `explore.memory` drafts the entry the generator keeps per brand: what the
//! approved layout uses and how (DS components and tokens, layout, images, texts, states), what the
//! designer adjusted (from the recorded origin of each aspect: her edits, not the agent's) and what
//! was set aside (the proposals not chosen). It only drafts: writing the entry to Notion is the
//! generator's, after the designer's explicit ok.

use std::collections::BTreeSet;

use figcraft_doc::NodeKind;
use serde_json::{Value, json};

use super::CommandSpec;
use super::eureka_critique::{is_reviewed, is_state, is_state_value, main_and_set, variant_pairs};
use super::eureka_explore::{PROPOSAL_KEY, REVIEW_KEY, info_of};
use crate::params::{get, id_of};
use crate::{Document, EngineError, NodeId, Result, Session, bad, spec};

/// The `pluginData` key the chosen proposal keeps the designer's approval of its layout under.
pub const APPROVAL_KEY: &str = "eureka.approval";
/// Longest text kept per field; most items listed per field; most layers looked at.
const MAX_TEXT: usize = 200;
const MAX_LISTED: usize = 12;
const MAX_LAYERS: usize = 5000;

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "explore.approve",
            "Approve the chosen layout",
            "{id (the chosen proposal's screen), by? (who approves), date? (AAAA-MM-DD)}  the designer approves the refined layout of the chosen proposal, once its design critique was reviewed and the DS Gatekeeper gave its verdict; `explore.memory` then drafts its visual-memory entry → {approval}",
            true,
            approve
        ),
        spec!(
            "explore.memory",
            "Draft the visual-memory entry",
            "{id (the approved proposal's screen), date? (AAAA-MM-DD), registry? (link of its page in the proposals' record), figma? (link of the approved layout)}  read-only: the entry the proposals generator keeps per brand for an approved layout (DS components and tokens, layout, images, texts, states, what the designer adjusted, what was set aside), as Markdown and as fields; writing it is the generator's, after the designer's explicit ok → {entry, fields}",
            false,
            memory
        ),
    ]
}

fn text_param(cmd: &'static str, p: &Value, key: &str) -> Result<Option<String>> {
    match get(p, key) {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(t)) if t.chars().count() <= MAX_TEXT => Ok(Some(t.trim().to_string()).filter(|t| !t.is_empty())),
        Some(_) => Err(bad(cmd, format!("`{key}` must be text of at most {MAX_TEXT} characters"))),
    }
}

fn date_param(cmd: &'static str, p: &Value) -> Result<Option<String>> {
    let d = text_param(cmd, p, "date")?;
    if let Some(d) = &d
        && chrono::NaiveDate::parse_from_str(d, "%Y-%m-%d").is_err()
    {
        return Err(bad(cmd, "`date` must be a date as AAAA-MM-DD"));
    }
    Ok(d)
}

/// `explore.approve`.
fn approve(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.approve";
    let id = get(p, "id").and_then(id_of).ok_or_else(|| bad(CMD, "`id` is required (the chosen proposal's screen)"))?;
    let Some(info) = info_of(s, id) else { return Err(bad(CMD, "that layer is not a proposal on an exploration board")) };
    if info.get("status").and_then(Value::as_str) != Some("escolhida") {
        return Err(bad(CMD, "only the chosen proposal's layout is approved: pick it with `explore.choose` first"));
    }
    let doc = s.doc()?;
    if !is_reviewed(doc, id) {
        return Err(bad(CMD, "the design critique comes first, reviewed by the designer (`explore.critiqueReviewed`)"));
    }
    if !doc.node(id).is_some_and(|n| n.plugin_data.contains_key(REVIEW_KEY)) {
        return Err(bad(CMD, "the DS Gatekeeper's verdict comes before the approval (`explore.review`)"));
    }
    let approval = json!({"approved": true, "by": text_param(CMD, p, "by")?, "date": date_param(CMD, p)?});
    let n = s.ds_mut()?.doc_mut().node_mut(id).ok_or(EngineError::NoNode(id))?;
    n.plugin_data.insert(APPROVAL_KEY.to_string(), approval.to_string());
    Ok(json!({"approval": approval}))
}

/// The approval kept on a proposal's screen, if any.
pub fn approval_of(doc: &Document, id: NodeId) -> Option<Value> {
    serde_json::from_str::<Value>(doc.node(id)?.plugin_data.get(APPROVAL_KEY)?).ok().filter(Value::is_object)
}

fn push_unique(list: &mut Vec<String>, item: String) {
    if list.len() < MAX_LISTED && !list.contains(&item) {
        list.push(item);
    }
}

/// `explore.memory`.
fn memory(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.memory";
    let id = get(p, "id").and_then(id_of).ok_or_else(|| bad(CMD, "`id` is required (the approved proposal's screen)"))?;
    let Some(info) = info_of(s, id) else { return Err(bad(CMD, "that layer is not a proposal on an exploration board")) };
    let doc = s.doc()?;
    let Some(approval) = approval_of(doc, id) else {
        return Err(bad(CMD, "only an approved layout goes into the visual memory: approve it with `explore.approve` first"));
    };
    let date = date_param(CMD, p)?.or_else(|| approval.get("date").and_then(Value::as_str).map(str::to_string));
    let registry = text_param(CMD, p, "registry")?;
    let figma = text_param(CMD, p, "figma")?;
    let text = |k: &str| info.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    let ctx = info.get("context").cloned().unwrap_or(Value::Null);
    let ctx_text = |k: &str| ctx.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    let names = |k: &str| -> Vec<String> {
        info.pointer(&format!("/ds/{k}"))
            .and_then(Value::as_array)
            .map(|l| l.iter().filter_map(Value::as_str).map(str::to_string).collect())
            .unwrap_or_default()
    };

    let layers: Vec<NodeId> = doc.subtree(id).into_iter().take(MAX_LAYERS).collect();
    let (mut ctas, mut states, mut adjusted, mut layout_tokens) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
    let mut images = 0_usize;
    for lid in &layers {
        let Some(n) = doc.node(*lid) else { continue };
        if !n.visible {
            continue;
        }
        // The instances this layer is inside (nearest first, up to the screen).
        let up: Vec<NodeId> = doc.ancestors(*lid).into_iter().take_while(|a| *a != id).take(64).collect();
        let in_instance = up.iter().any(|a| doc.node(*a).is_some_and(|x| matches!(x.kind, NodeKind::Instance { .. })));
        // Calls to action: the texts of buttons (not icon glyph names, not size or chip labels).
        if let Some(t) = n.kind.text() {
            let in_button = up
                .iter()
                .take(4)
                .filter_map(|a| main_and_set(doc, *a))
                .any(|(_, set)| doc.node(set).is_some_and(|x| matches!(x.name.as_str(), "Button" | "Button Link")));
            let label = t.characters.trim();
            let glyph = !label.is_empty() && label.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
            if in_button && !label.is_empty() && !glyph {
                push_unique(&mut ctas, format!("\"{}\"", label.chars().take(60).collect::<String>()));
            }
        }
        if n.name.ends_with("(imagem de exemplo)") {
            images += 1;
        }
        if let Some((main, set)) = main_and_set(doc, *lid) {
            let set_name = doc.node(set).map(|x| x.name.clone()).unwrap_or_default();
            if set_name == "Image" {
                images += 1;
            }
            // States shown: a state toggle on, or a value that is itself a state (as the critique
            // measures them); a kind such as a tag's "Status=Brand" is not one.
            for (k, v) in variant_pairs(&doc.node(main).map(|x| x.name.clone()).unwrap_or_default()) {
                let on = matches!(v.trim().to_lowercase().as_str(), "on" | "true" | "sim" | "yes");
                if (is_state(&k) && on) || is_state_value(&v) {
                    push_unique(&mut states, format!("{set_name} {k}={v}"));
                }
            }
        }
        // Spacing tokens of the screen's own frames (not inside DS components).
        if matches!(n.kind, NodeKind::Frame) && !in_instance {
            for (field, vid) in &n.bound {
                if matches!(field.as_str(), "gap" | "paddingHorizontal" | "paddingVertical" | "paddingTop" | "paddingLeft")
                    && let Some(v) = doc.variables.get_by_id(vid)
                {
                    push_unique(&mut layout_tokens, v.name.clone());
                }
            }
        }
        // What the designer changed by hand (the recorded origin says "human"), not the agent; a
        // change inside an instance is told by the outermost instance it belongs to.
        let origin = super::eureka::origin_of(n);
        let mine: BTreeSet<String> =
            origin.as_object().into_iter().flatten().filter(|(_, st)| st.get("by") == Some(&json!("human"))).map(|(g, _)| g.clone()).collect();
        if !mine.is_empty() {
            let owner = up.iter().rev().find(|a| doc.node(**a).is_some_and(|x| matches!(x.kind, NodeKind::Instance { .. }))).copied();
            let who = owner.and_then(|o| doc.node(o)).map_or_else(|| n.name.clone(), |o| o.name.clone());
            let item = format!("{who} ({})", mine.into_iter().collect::<Vec<_>>().join(", "));
            match adjusted.iter_mut().find(|e: &&mut String| e.starts_with(&format!("{who} ("))) {
                Some(e) => {
                    let extra = item.trim_start_matches(&format!("{who} (")).trim_end_matches(')').to_string();
                    let merged = format!("{}, {extra})", e.trim_end_matches(')'));
                    *e = merged;
                }
                None => push_unique(&mut adjusted, item),
            }
        }
    }
    let blocks: Vec<String> =
        doc.node(id).map(|n| n.children.iter().filter_map(|c| doc.node(*c)).map(|c| c.name.clone()).take(MAX_LISTED).collect()).unwrap_or_default();
    let set_aside: Vec<String> = doc
        .node(id)
        .and_then(|n| n.parent)
        .and_then(|p| doc.node(p))
        .map(|b| {
            b.children
                .iter()
                .filter(|c| **c != id)
                .filter_map(|c| {
                    let i: Value = serde_json::from_str(doc.node(*c)?.plugin_data.get(PROPOSAL_KEY)?).ok()?;
                    (i.get("board") == info.get("board")).then(|| doc.node(*c).map(|n| n.name.clone())).flatten()
                })
                .take(MAX_LISTED)
                .collect()
        })
        .unwrap_or_default();

    let or = |v: Option<String>, placeholder: &str| v.unwrap_or_else(|| placeholder.to_string());
    let list = |l: &[String], empty: &str| if l.is_empty() { empty.to_string() } else { l.join("; ") };
    let fields = json!({
        "title": format!("{} — {} · {} — {}", or(date.clone(), "<AAAA-MM-DD>"), ctx_text("journey"), ctx_text("platform"), text("name")),
        "registro": or(registry, "<link da página no Registro de propostas>"),
        "layout": format!("{} · Aprovado por: {} em {}", or(figma, "<link do layout aprovado com node-id>"),
            or(approval.get("by").and_then(Value::as_str).map(str::to_string), "<designer>"), or(date, "<data>")),
        "componentes_tokens": format!("Componentes: {}. Tokens: {}.", list(&names("components"), "nenhum"), list(&names("variables"), "nenhum")),
        "padroes_layout": format!("Blocos, de cima para baixo: {}. Espaçamento: {}.", list(&blocks, "—"), list(&layout_tokens, "sem token de espaçamento próprio")),
        "imagem": format!("{images} imagem(ns) na tela (conteúdo de exemplo; tipo e enquadramento a descrever pela designer)."),
        "texto": format!("Chamadas para ação: {}.", list(&ctas, "nenhuma")),
        "interacoes": format!("Estados mostrados: {}.", list(&states, "só o estado padrão")),
        "ajustes_designer": list(&adjusted, "nenhum ajuste registrado: aprovado como foi proposto"),
        "evitar": if set_aside.is_empty() { "—".to_string() } else { format!("Não escolhidas nesta rodada: {} (motivo: <a registrar pela designer>).", set_aside.join("; ")) },
        "status": "observação",
        "validacao": "<sem teste ainda; aprovação de design não é validação com clientes>",
    });
    let f = |k: &str| fields.get(k).and_then(Value::as_str).unwrap_or_default().to_string();
    let entry = format!(
        "## {}\n- Registro: {}\n- Layout aprovado (Figma): {}\n- Componentes e tokens usados (DS): {}\n- Padrões de layout: {}\n- Imagem: {}\n- Texto: {}\n- Interações: {}\n- O que a designer ajustou: {}\n- O que evitar: {}\n- Status do aprendizado: {}\n- Validação: {}\n",
        f("title"),
        f("registro"),
        f("layout"),
        f("componentes_tokens"),
        f("padroes_layout"),
        f("imagem"),
        f("texto"),
        f("interacoes"),
        f("ajustes_designer"),
        f("evitar"),
        f("status"),
        f("validacao")
    );
    Ok(json!({"entry": entry, "fields": fields, "written": false}))
}
