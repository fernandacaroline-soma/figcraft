//! Design critique of the chosen proposal (Eureka G3, Frente O): the step between the designer's
//! pick (`explore.choose`) and the DS Gatekeeper (`explore.review`), as the proposals generator
//! ("gerador-propostas", phase C, `references/critica.md`) runs it. Each finding says where it
//! happens, what the user suffers, how severe it is, what it rests on, what to change and its
//! status; the designer reviews the critique before the Gatekeeper is asked.
//!
//! What a screen shows can be measured, and the Eureka adds it itself, marked "Eureka (medido)":
//! the states of the DS components used that the screen does not show, text contrast with the
//! colours the brand's tokens resolve to, touch targets smaller than 44 pt and the values the DS
//! lint finds. The rest of the critique (flow, failures, edge cases, content, feasibility,
//! measurement) comes from a reviewer such as Claude; what nobody observed is a question, with
//! "inferência" as its evidence. A critique only advises: nothing is written to Notion and nothing
//! in the DS changes.

use std::collections::{BTreeMap, BTreeSet};

use figcraft_color::Color;
use figcraft_doc::{NodeKind, VariableValue};
use serde_json::{Value, json};

use super::CommandSpec;
use super::eureka_explore::{PROPOSAL_KEY, REVIEW_KEY, info_of};
use crate::params::{get, id_of};
use crate::{Document, EngineError, NodeId, Result, Session, bad, spec};

/// The `pluginData` key the chosen proposal keeps its critique under.
pub const CRITIQUE_KEY: &str = "eureka.critique";
/// The areas of the generator's critique script, in its order.
pub const AREAS: [&str; 8] = ["fluxo", "estados", "falhas", "casos de borda", "conteúdo e interação", "acessibilidade", "viabilidade", "medição"];
/// Severities, most severe first.
pub const SEVERITIES: [&str; 4] = ["crítica", "alta", "média", "baixa"];
/// What the designer can say of a finding.
pub const STATUSES: [&str; 3] = ["aberto", "resolvido", "decisão da designer"];
/// Evidence of what the Eureka measured on the screen itself.
pub const MEASURED: &str = "Eureka (medido)";
/// Smallest touch target, in points (Apple Human Interface Guidelines: 44 × 44 pt).
const MIN_TOUCH: f64 = 44.0;
/// Contrast WCAG 2.1 AA asks of text (1.4.3): 4.5:1, or 3:1 for large text.
const AA_TEXT: f32 = 4.5;
const AA_LARGE: f32 = 3.0;
/// Most findings a critique keeps, measured ones included; longest text per field.
const MAX_FINDINGS: usize = 100;
const MAX_MEASURED: usize = 40;
const MAX_TEXT: usize = 2000;
/// Most layers of one screen measured.
const MAX_LAYERS: usize = 5000;
/// What a variant property is named, or one of its values, when it is a state of the component.
const STATE_WORDS: [&str; 12] =
    ["estado", "state", "status", "desabilitad", "disabled", "erro", "error", "loading", "carregand", "vazio", "empty", "sucesso"];

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "explore.measure",
            "Measure the chosen proposal",
            "{id (a proposal's screen)}  what the Eureka measures on a proposal's screen for its design critique, without keeping anything: DS component states the screen does not show, text contrast with the brand's resolved tokens (WCAG 2.1 AA), touch targets under 44 pt and the DS lint → {findings: [{area, scenario, consequence, severity, evidence, recommendation, ids}]}",
            false,
            measure_cmd
        ),
        spec!(
            "explore.critique",
            "Critique the chosen proposal",
            "{id (the chosen proposal's screen), reviewer?, findings?: [{area (\"fluxo\" | \"estados\" | \"falhas\" | \"casos de borda\" | \"conteúdo e interação\" | \"acessibilidade\" | \"viabilidade\" | \"medição\"), scenario, consequence, severity (\"crítica\" | \"alta\" | \"média\" | \"baixa\"), evidence? (a source, or \"inferência\"), recommendation, ids?}], refined?, open?: [decisions left to the designer]}  keep the design critique on the chosen proposal, with what the Eureka measures on its screen added; it replaces an earlier critique and the Gatekeeper's verdict, and waits for the designer's review → {critique}",
            true,
            critique
        ),
        spec!(
            "explore.critiqueStatus",
            "Set a critique finding's status",
            "{id (the chosen proposal's screen), finding (its number), status (\"aberto\" | \"resolvido\" | \"decisão da designer\")}  the designer's answer to one finding → {finding}",
            true,
            set_status
        ),
        spec!(
            "explore.critiqueReviewed",
            "Mark the critique reviewed",
            "{id (the chosen proposal's screen)}  the designer has reviewed the critique: the DS Gatekeeper may now be asked (`explore.review`) → {critique}",
            true,
            reviewed
        ),
    ]
}

/// The critique kept on a proposal's screen, if any.
pub fn critique_of(doc: &Document, id: NodeId) -> Option<Value> {
    serde_json::from_str::<Value>(doc.node(id)?.plugin_data.get(CRITIQUE_KEY)?).ok().filter(Value::is_object)
}

/// The chosen proposal's screen named by `id`, or why it cannot be critiqued.
fn chosen(s: &Session, cmd: &'static str, p: &Value) -> Result<NodeId> {
    let id = get(p, "id").and_then(id_of).ok_or_else(|| bad(cmd, "`id` is required (the chosen proposal's screen)"))?;
    let Some(info) = info_of(s, id) else { return Err(bad(cmd, "that layer is not a proposal on an exploration board")) };
    if info.get("status").and_then(Value::as_str) != Some("escolhida") {
        return Err(bad(cmd, "only the chosen proposal is critiqued: pick it with `explore.choose` first"));
    }
    Ok(id)
}

fn store(s: &mut Session, id: NodeId, c: &Value) -> Result<()> {
    let doc = s.ds_mut()?.doc_mut();
    let n = doc.node_mut(id).ok_or(EngineError::NoNode(id))?;
    n.plugin_data.insert(CRITIQUE_KEY.to_string(), c.to_string());
    Ok(())
}

/// `explore.measure`.
fn measure_cmd(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.measure";
    let id = get(p, "id").and_then(id_of).ok_or_else(|| bad(CMD, "`id` is required (a proposal's screen)"))?;
    if s.doc()?.node(id).is_none_or(|n| !n.plugin_data.contains_key(PROPOSAL_KEY)) {
        return Err(bad(CMD, "that layer is not a proposal on an exploration board"));
    }
    Ok(json!({"findings": measure(s.doc()?, id)}))
}

/// `explore.critique`.
fn critique(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.critique";
    let id = chosen(s, CMD, p)?;
    let text = |v: Option<&Value>, what: &str| -> Result<Option<String>> {
        match v {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(t)) if t.chars().count() <= MAX_TEXT => Ok(Some(t.trim().to_string()).filter(|t| !t.is_empty())),
            Some(_) => Err(bad(CMD, format!("`{what}` must be text of at most {MAX_TEXT} characters"))),
        }
    };
    let one_of = |v: Option<&Value>, what: &str, list: &[&str]| -> Result<String> {
        let t = v.and_then(Value::as_str).map(|t| t.trim().to_lowercase()).unwrap_or_default();
        list.iter().find(|x| **x == t).map(|x| (*x).to_string()).ok_or_else(|| bad(CMD, format!("`{what}` must be one of: {}", list.join(", "))))
    };
    let mut findings = Vec::new();
    match get(p, "findings") {
        None | Some(Value::Null) => {}
        Some(Value::Array(a)) if a.len() <= MAX_FINDINGS => {
            for f in a {
                let need = |k: &str| -> Result<String> {
                    text(f.get(k), &format!("findings[].{k}"))?.ok_or_else(|| bad(CMD, format!("every finding needs `{k}`")))
                };
                let ids: Vec<u64> = match f.get("ids") {
                    None | Some(Value::Null) => Vec::new(),
                    Some(Value::Array(l)) => l.iter().take(MAX_FINDINGS).filter_map(Value::as_u64).collect(),
                    Some(_) => return Err(bad(CMD, "`findings[].ids` must be a list of layer ids")),
                };
                findings.push(json!({
                    "area": one_of(f.get("area"), "findings[].area", &AREAS)?,
                    "scenario": need("scenario")?,
                    "consequence": need("consequence")?,
                    "severity": one_of(f.get("severity"), "findings[].severity", &SEVERITIES)?,
                    "evidence": text(f.get("evidence"), "findings[].evidence")?.unwrap_or_else(|| "inferência".into()),
                    "recommendation": need("recommendation")?,
                    "ids": ids,
                }));
            }
        }
        Some(_) => return Err(bad(CMD, format!("`findings` must be a list of at most {MAX_FINDINGS}"))),
    }
    let open: Vec<String> = match get(p, "open") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(a)) if a.len() <= MAX_FINDINGS => {
            a.iter().map(|v| text(Some(v), "open[]")).collect::<Result<Vec<_>>>()?.into_iter().flatten().collect()
        }
        Some(_) => return Err(bad(CMD, format!("`open` must be a list of at most {MAX_FINDINGS}"))),
    };
    let reviewer = text(get(p, "reviewer"), "reviewer")?.or_else(|| s.agent.clone()).unwrap_or_else(|| "revisor".into());
    let refined = text(get(p, "refined"), "refined")?;
    // What the Eureka measures goes first: it is what the screen shows, not an opinion.
    let mut all = measure(s.doc()?, id);
    all.extend(findings);
    all.truncate(MAX_FINDINGS);
    all.sort_by_key(|f| SEVERITIES.iter().position(|x| Some(*x) == f.get("severity").and_then(Value::as_str)).unwrap_or(SEVERITIES.len()));
    for (i, f) in all.iter_mut().enumerate() {
        if let Some(o) = f.as_object_mut() {
            o.insert("n".into(), json!(i + 1));
            o.insert("status".into(), json!("aberto"));
        }
    }
    let c = json!({"reviewer": reviewer, "findings": all, "refined": refined, "open": open, "reviewed": false, "advisory": true});
    store(s, id, &c)?;
    // A verdict given on an earlier critique no longer follows the one the designer has to review.
    if let Some(n) = s.ds_mut()?.doc_mut().node_mut(id) {
        n.plugin_data.remove(REVIEW_KEY);
    }
    Ok(json!({"critique": c}))
}

/// `explore.critiqueStatus`.
fn set_status(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.critiqueStatus";
    let id = chosen(s, CMD, p)?;
    let Some(mut c) = critique_of(s.doc()?, id) else { return Err(bad(CMD, "this proposal has no critique yet: see `explore.critique`")) };
    let status = get(p, "status").and_then(Value::as_str).map(str::trim).filter(|t| STATUSES.contains(t));
    let Some(status) = status else { return Err(bad(CMD, format!("`status` must be one of: {}", STATUSES.join(", ")))) };
    let n = get(p, "finding").and_then(Value::as_u64).ok_or_else(|| bad(CMD, "`finding` is required (its number)"))?;
    let finding = c
        .get_mut("findings")
        .and_then(Value::as_array_mut)
        .and_then(|l| l.iter_mut().find(|f| f.get("n").and_then(Value::as_u64) == Some(n)))
        .ok_or_else(|| bad(CMD, format!("the critique has no finding {n}")))?;
    if let Some(o) = finding.as_object_mut() {
        o.insert("status".into(), json!(status));
    }
    let out = finding.clone();
    store(s, id, &c)?;
    Ok(json!({"finding": out}))
}

/// `explore.critiqueReviewed`.
fn reviewed(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "explore.critiqueReviewed";
    let id = chosen(s, CMD, p)?;
    let Some(mut c) = critique_of(s.doc()?, id) else { return Err(bad(CMD, "this proposal has no critique yet: see `explore.critique`")) };
    if let Some(o) = c.as_object_mut() {
        o.insert("reviewed".into(), json!(true));
    }
    store(s, id, &c)?;
    Ok(json!({"critique": c}))
}

/// Whether the chosen proposal's critique was reviewed by the designer (what `explore.review` needs).
pub fn is_reviewed(doc: &Document, id: NodeId) -> bool {
    critique_of(doc, id).is_some_and(|c| c.get("reviewed") == Some(&json!(true)))
}

fn finding(area: &str, severity: &str, scenario: String, consequence: &str, evidence: String, recommendation: String, ids: Vec<u64>) -> Value {
    json!({"area": area, "scenario": scenario, "consequence": consequence, "severity": severity,
           "evidence": evidence, "recommendation": recommendation, "ids": ids})
}

/// What the Eureka measures on `screen`: findings in the critique's shape.
pub fn measure(doc: &Document, screen: NodeId) -> Vec<Value> {
    let shown: Vec<NodeId> = doc
        .subtree(screen)
        .into_iter()
        .take(MAX_LAYERS)
        .filter(|id| {
            std::iter::once(*id)
                .chain(doc.ancestors(*id).into_iter().take_while(|a| *a != screen).take(64))
                .all(|i| doc.node(i).is_some_and(|n| n.visible))
        })
        .collect();
    let mut out = states(doc, &shown);
    out.extend(contrast(doc, screen, &shown));
    out.extend(touch(doc, screen, &shown));
    let lint = super::eureka_lint::lint_layers(doc, &doc.subtree(screen), MAX_MEASURED);
    if !lint.is_empty() {
        let ids: Vec<u64> = lint.iter().filter_map(|f| f.get("id").and_then(Value::as_u64)).collect();
        out.push(finding(
            "conteúdo e interação",
            "média",
            format!("{} valor(es) da tela fora das variables e estilos do DS (lint).", lint.len()),
            "Cor, espaço ou texto que não segue o DS fica inconsistente entre marcas e telas.",
            format!("{MEASURED}: ds.lint"),
            "Trocar cada valor solto pela variable ou estilo do DS que corresponde.".into(),
            ids,
        ));
    }
    out.truncate(MAX_MEASURED);
    out
}

/// The main component an instance shows and the set it belongs to (or itself, standing alone).
pub(crate) fn main_and_set(doc: &Document, id: NodeId) -> Option<(NodeId, NodeId)> {
    let NodeKind::Instance { component, .. } = doc.node(id)?.kind else { return None };
    let set = doc.node(component)?.parent.filter(|p| doc.node(*p).is_some_and(|n| matches!(n.kind, NodeKind::ComponentSet { .. })));
    Some((component, set.unwrap_or(component)))
}

/// A variant name's `property=value` pairs, the property as a description names it but with its
/// case kept for the designer ("✍️ Desabilitado#1:2" → "Desabilitado").
pub(crate) fn variant_pairs(name: &str) -> BTreeMap<String, String> {
    let label = |k: &str| k.split('#').next().unwrap_or(k).trim_start_matches(|c: char| !c.is_alphanumeric()).trim().to_string();
    name.split(',').filter_map(|kv| kv.split_once('=')).map(|(k, v)| (label(k), v.trim().to_string())).collect()
}

pub(crate) fn is_state(word: &str) -> bool {
    let w = word.to_lowercase();
    STATE_WORDS.iter().any(|s| w.contains(s))
}

/// A variant value that is itself a state ("Erro", "Loading"), whatever its property is called. The
/// whole value must be the state: "error-circle", an icon's name, is not a state of the screen.
const STATE_VALUES: [&str; 12] =
    ["erro", "error", "loading", "carregando", "vazio", "empty", "sucesso", "success", "desabilitado", "disabled", "falha", "offline"];

pub(crate) fn is_state_value(v: &str) -> bool {
    let w = v.trim().to_lowercase();
    STATE_VALUES.contains(&w.as_str())
}

/// Variant property → the values the screen shows.
type Shown = BTreeMap<String, BTreeSet<String>>;

/// The states of each DS component set used on the screen (variant values such as Erro, Loading,
/// Desabilitado=On) that no instance on the screen shows.
fn states(doc: &Document, shown: &[NodeId]) -> Vec<Value> {
    // set → (property → values shown, instances)
    let mut used: BTreeMap<NodeId, (Shown, Vec<u64>)> = BTreeMap::new();
    for id in shown {
        let Some((main, set)) = main_and_set(doc, *id) else { continue };
        if set == main {
            continue;
        }
        let e = used.entry(set).or_default();
        for (k, v) in variant_pairs(&doc.node(main).map(|n| n.name.clone()).unwrap_or_default()) {
            e.0.entry(k).or_default().insert(v);
        }
        e.1.push(id.0);
    }
    let mut out = Vec::new();
    for (set, (seen, ids)) in used {
        let Some(sn) = doc.node(set) else { continue };
        let mut options: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for v in &sn.children {
            for (k, val) in variant_pairs(&doc.node(*v).map(|n| n.name.clone()).unwrap_or_default()) {
                options.entry(k).or_default().insert(val);
            }
        }
        for (prop, values) in options {
            // A state toggle ("Desabilitado" = Off | On) asks for its "on" value; any other property
            // (a Snackbar's "Status" = Brand | Erro | Sucesso…) only for the values that are states.
            let toggle = is_state(&prop) && values.iter().all(|v| is_off(v) || is_on(v));
            let missing: Vec<String> = values
                .iter()
                .filter(|v| (if toggle { is_on(v) } else { is_state_value(v) }) && !seen.get(&prop).is_some_and(|s| s.contains(*v)))
                .map(|v| format!("{prop}={v}"))
                .collect();
            if missing.is_empty() {
                continue;
            }
            let has: Vec<String> = seen.get(&prop).into_iter().flatten().map(|v| format!("{prop}={v}")).collect();
            out.push(finding(
                "estados",
                "média",
                format!("{} aparece só com {}; a tela não mostra {}.", sn.name, has.join(", "), missing.join(", ")),
                "Sem o estado desenhado, o comportamento nesse momento fica para o desenvolvimento decidir.",
                format!("{MEASURED}: variantes de {} no DS", sn.name),
                format!("Mostrar quando a tela fica em {} (num fluxo com o estado), ou registrar por que não se aplica.", missing.join(", ")),
                ids.clone(),
            ));
        }
    }
    out
}

/// "On" and its kin: the value a state toggle takes when the state is shown.
fn is_on(v: &str) -> bool {
    matches!(v.trim().to_lowercase().as_str(), "on" | "true" | "sim" | "yes")
}

/// "Off" and its kin: the resting value of a state toggle, not a state to show.
fn is_off(v: &str) -> bool {
    matches!(v.trim().to_lowercase().as_str(), "off" | "false" | "não" | "nao" | "no" | "default" | "padrão" | "padrao")
}

/// The colour a paint field shows on a layer: its bound variable resolved in the layer's modes (the
/// brand), else the first visible solid paint.
fn fill_color(doc: &Document, id: NodeId) -> Option<Color> {
    let n = doc.node(id)?;
    let paint = n.fills.iter().find(|p| p.visible)?;
    let base = paint.solid_color()?;
    let bound = n.bound.get("fills/0/color").and_then(|var| match doc.variables.resolve(var, &doc.modes_for(id)) {
        Some(VariableValue::Color(c)) => Some(c.with_alpha(c.a * paint.opacity)),
        _ => None,
    });
    Some(bound.unwrap_or(base))
}

fn over(fg: Color, bg: Color) -> Color {
    let a = fg.a.clamp(0.0, 1.0);
    Color::new(fg.r * a + bg.r * (1.0 - a), fg.g * a + bg.g * (1.0 - a), fg.b * a + bg.b * (1.0 - a), 1.0)
}

/// Each text whose colour against what is behind it (the nearest container with a fill) falls under
/// WCAG 2.1 AA.
fn contrast(doc: &Document, screen: NodeId, shown: &[NodeId]) -> Vec<Value> {
    let mut out = Vec::new();
    for id in shown {
        let Some(n) = doc.node(*id) else { continue };
        let Some(t) = n.kind.text() else { continue };
        if t.characters.trim().is_empty() {
            continue;
        }
        let Some(fg) = fill_color(doc, *id) else { continue };
        // Up to the screen itself; a translucent fill is passed over (what it blends with is not
        // measured).
        let up: Vec<NodeId> = doc.ancestors(*id).into_iter().take(64).collect();
        let upto = up.iter().position(|a| *a == screen).map_or(up.len(), |i| i + 1);
        let behind = up.iter().take(upto).find_map(|a| fill_color(doc, *a).filter(|c| c.a >= 0.99));
        let Some(bg) = behind else { continue };
        let ratio = over(fg, bg).contrast(bg);
        let size = t.style.font_size;
        let large = size >= 24.0 || (size >= 18.66 && t.style.weight() >= 700);
        let need = if large { AA_LARGE } else { AA_TEXT };
        if !ratio.is_finite() || ratio >= need {
            continue;
        }
        let sample: String = t.characters.chars().take(40).collect();
        out.push(finding(
            "acessibilidade",
            if ratio < AA_LARGE { "alta" } else { "média" },
            format!("Texto \"{sample}\" com contraste {ratio:.1}:1 sobre o fundo (pede {need}:1)."),
            "Quem tem baixa visão, ou usa o app no sol, pode não conseguir ler.",
            format!("{MEASURED}: cores dos tokens na marca da tela; WCAG 2.1 AA 1.4.3"),
            "Usar uma cor de conteúdo do DS com contraste suficiente sobre esse fundo.".into(),
            vec![id.0],
        ));
    }
    out
}

/// The sizes (rounded, in pt) a component has on the screen under 44 pt, and its layers there.
type Small = (BTreeSet<(i64, i64)>, Vec<(u64, String)>);

/// Instances placed on the screen itself (not inside another instance) smaller than 44 × 44 pt,
/// one finding per component with the sizes it has on the screen.
fn touch(doc: &Document, screen: NodeId, shown: &[NodeId]) -> Vec<Value> {
    let mut small: BTreeMap<String, Small> = BTreeMap::new();
    for id in shown {
        let Some(n) = doc.node(*id) else { continue };
        if !matches!(n.kind, NodeKind::Instance { .. }) {
            continue;
        }
        let nested = doc
            .ancestors(*id)
            .into_iter()
            .take_while(|a| *a != screen)
            .take(64)
            .any(|a| doc.node(a).is_some_and(|x| matches!(x.kind, NodeKind::Instance { .. })));
        if nested || (n.width >= MIN_TOUCH && n.height >= MIN_TOUCH) || !n.width.is_finite() || !n.height.is_finite() {
            continue;
        }
        let what = main_and_set(doc, *id).and_then(|(_, set)| doc.node(set)).map(|s| s.name.clone()).unwrap_or_else(|| n.name.clone());
        let e = small.entry(what).or_default();
        e.0.insert((n.width.round() as i64, n.height.round() as i64));
        e.1.push((id.0, n.name.clone()));
    }
    small
        .into_iter()
        .map(|(what, (sizes, layers))| {
            let sizes: Vec<String> = sizes.iter().map(|(w, h)| format!("{w} × {h} pt")).collect();
            finding(
                "acessibilidade",
                "baixa",
                format!("{what} mede {} ({} na tela).", sizes.join(", "), layers.len()),
                "Se for tocável, um alvo pequeno gera toques errados.",
                format!("{MEASURED}; referência: Apple Human Interface Guidelines, 44 × 44 pt"),
                "Se for tocável, garantir área de toque de 44 × 44 pt (a área pode ser maior que o desenho).".into(),
                layers.into_iter().map(|(i, _)| i).collect(),
            )
        })
        .collect()
}
