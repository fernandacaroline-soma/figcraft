//! Brand Resolver (Eureka Design, E13): which brand a proposal is designed for, read from what the
//! brand officially has, never inferred.
//!
//! - `brand.profile` reads a brand profile (the gerador-propostas skill's `references/marcas/<marca>.md`,
//!   shaped as its `_modelo.md`): its owner and last review, each section filled, still to fill
//!   (`[a preencher]` and the template's other `[…]` placeholders) or a pointer to another source,
//!   and the gaps. A profile with nothing filled, or no profile, means the proposals use only
//!   usability and the design system (`usabilidade-e-ds`). Read only.
//! - `brand.apply` sets the brand's mode of the brand collection (Tokens APP, else Tokens WEB) on
//!   the screens, and keeps on each the brand it was designed for: the mode, the profile's version
//!   (owner, review, a digest of the file) and its gaps, and what the brand registry officially
//!   lists for it (Team Components and Telas e Fluxos files, platforms). One undo step.
//!
//! Nothing here reads a rule into a section: a section is passed on as written, or listed as a gap.

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use super::CommandSpec;
use crate::params::{get, id_of};
use crate::{Document, NodeId, Result, Session, bad, spec};

/// Where a screen keeps the brand it was designed for.
pub const BRAND_KEY: &str = "eureka.brand";
/// Largest profile read (bytes).
const MAX_PROFILE: u64 = 512 * 1024;
/// Most sections a profile reports.
const MAX_SECTIONS: usize = 40;
/// Longest section text kept (chars).
const MAX_TEXT: usize = 4_000;
/// Most screens one call changes.
const MAX_IDS: usize = 200;
/// The template's sections that point to another source (Notion, the studies repository): never a
/// gap, and never read as rules.
const POINTERS: &[&str] = &["Memória visual", "Decisões e testes anteriores"];
/// The brand collections, in the order they are looked for.
const COLLECTIONS: &[&str] = &["Tokens APP", "Tokens WEB"];

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "brand.profile",
            "Read a brand profile",
            "{path? (the profile, e.g. references/marcas/nv.md) | text? (its contents)}  a brand profile shaped as the gerador-propostas skill's _modelo.md, read as written: no rule is inferred. Read only → {brand, owner, reviewed, note, digest, sections: [{title, status: \"filled\" | \"to-fill\" | \"pointer\", text?}], gaps: [text], basis: \"perfil\" (something the proposals can use is filled) | \"usabilidade-e-ds\" (nothing is: only usability and the design system)}",
            false,
            profile_cmd
        ),
        spec!(
            "brand.apply",
            "Design for a brand",
            "{ids (the proposals' screens), brand (its name: \"NV\", \"Farm\", \"Animale\"…), path? | text? (its profile, as brand.profile reads it; none: the brand has no profile), registry? (the brand's entry of the skill's brand-registry, as JSON: {plataformas, team_components_app, telas_e_fluxos_app, perfil, benchmark})}  sets the brand's mode of the brand collection (Tokens APP, else Tokens WEB; the mode named as the brand, or as the brand with a note in parentheses, \"Animale (CERTO)\") on the screens and keeps on each what it was designed with (pluginData eureka.brand): the mode, the profile's version and gaps, the registry's official entries. One undo step → {brand, mode: {collection, name} | null, profile: {owner, reviewed, note, digest, basis} | null, basis, official: {key: value}, gaps: [text]}",
            true,
            apply_cmd
        ),
    ]
}

/// The text of a profile from `path` or `text`, and its file name, if given.
fn read_profile(cmd: &str, p: &Value) -> Result<Option<String>> {
    match (get(p, "text"), get(p, "path")) {
        (Some(Value::String(t)), _) if t.len() as u64 <= MAX_PROFILE => Ok(Some(t.clone())),
        (Some(Value::String(_)), _) => Err(bad(cmd, format!("text: at most {MAX_PROFILE} bytes"))),
        (Some(Value::Null) | None, Some(Value::String(path))) if !path.trim().is_empty() => {
            let bytes = crate::cmd::file::read_capped(path, MAX_PROFILE)?;
            Ok(Some(String::from_utf8_lossy(&bytes).into_owned()))
        }
        (Some(Value::Null) | None, None | Some(Value::Null)) => Ok(None),
        _ => Err(bad(cmd, "path: the profile file; text: its contents")),
    }
}

/// Whether `line` is only the template's placeholders: `[a preencher]`, `[designer responsável]`,
/// or a list item whose value is one (`- Web: [DS N Marcas / estrutura específica]`).
fn placeholder(line: &str) -> bool {
    let l = line.trim().trim_start_matches(['-', '*']).trim();
    let value = l.split_once(':').map_or(l, |(_, v)| v.trim());
    value.starts_with('[') && value.ends_with(']') && !value.contains("](")
}

/// The value of `**label:** value` in the profile's header line (up to the next `·`).
fn header_field(header: &str, label: &str) -> Option<String> {
    let start = header.find(&format!("**{label}:**"))? + label.len() + 5;
    let rest = header.get(start..)?;
    let value = rest.split('·').next().unwrap_or_default().trim();
    (!value.is_empty()).then(|| value.to_string())
}

/// `brand.profile`'s reading of `text`.
pub fn parse_profile(text: &str) -> Value {
    let digest: String = Sha256::digest(text.as_bytes()).iter().take(6).map(|b| format!("{b:02x}")).collect();
    let mut brand = None;
    let mut header = String::new();
    let mut sections: Vec<(String, Vec<&str>)> = Vec::new();
    for line in text.lines() {
        if let Some(t) = line.strip_prefix("# ") {
            brand = Some(t.split_once('—').map_or(t, |(_, b)| b).trim().to_string());
        } else if let Some(t) = line.strip_prefix("## ") {
            if sections.len() < MAX_SECTIONS {
                sections.push((t.trim().to_string(), Vec::new()));
            }
        } else if let Some((_, body)) = sections.last_mut() {
            body.push(line);
        } else if line.contains("**Dona:**") || line.contains("**Última revisão:**") {
            header = line.to_string();
        }
    }
    let mut gaps = Vec::new();
    let owner = header_field(&header, "Dona").filter(|o| !placeholder(o));
    if owner.is_none() {
        gaps.push("Perfil sem dona".to_string());
    }
    let review = header_field(&header, "Última revisão").filter(|r| !placeholder(r));
    let (reviewed, note) = match review.as_deref() {
        // "12/11/2026 (parcial: só concorrentes e inspirações)": the date, and the note as written.
        Some(r) => match r.split_once('(') {
            Some((d, n)) => (Some(d.trim().to_string()), Some(n.trim_end_matches(')').trim().to_string())),
            None => (Some(r.to_string()), None),
        },
        None => {
            gaps.push("Perfil sem data de revisão".to_string());
            (None, None)
        }
    };
    let mut usable = false;
    let out: Vec<Value> = sections
        .iter()
        .map(|(title, body)| {
            let lines: Vec<&str> = body.iter().map(|l| l.trim()).filter(|l| !l.is_empty()).collect();
            let text: String = lines.join("\n").chars().take(MAX_TEXT).collect();
            let status = if POINTERS.iter().any(|p| title.starts_with(p)) {
                "pointer"
            } else if lines.is_empty() || lines.iter().all(|l| placeholder(l)) {
                gaps.push(format!("Seção a preencher: {title}"));
                "to-fill"
            } else {
                usable = true;
                "filled"
            };
            let mut s = json!({"title": title, "status": status});
            if status != "to-fill" {
                s["text"] = json!(text);
            }
            s
        })
        .collect();
    json!({
        "brand": brand,
        "owner": owner,
        "reviewed": reviewed,
        "note": note,
        "digest": digest,
        "sections": out,
        "gaps": gaps,
        "basis": if usable { "perfil" } else { "usabilidade-e-ds" },
    })
}

fn profile_cmd(_s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "brand.profile";
    let text = read_profile(CMD, p)?.ok_or_else(|| bad(CMD, "pass `path` or `text`"))?;
    Ok(parse_profile(&text))
}

/// The brand collection's mode for `brand`: the mode named so, else the one named so with a note in
/// parentheses ("Animale (CERTO)"). Two such modes are ambiguous: none is picked.
fn brand_mode(doc: &Document, brand: &str) -> std::result::Result<(String, String, String), String> {
    let Some(c) = COLLECTIONS.iter().find_map(|n| doc.variables.collection(n)) else {
        return Err("o arquivo não tem a coleção de marca (Tokens APP ou Tokens WEB)".into());
    };
    let b = brand.trim();
    let bare = |m: &str| m.split_once('(').map_or(m, |(n, _)| n).trim().to_string();
    if let Some(m) = c.modes.iter().find(|m| m.name.trim().eq_ignore_ascii_case(b)) {
        return Ok((c.id.clone(), c.name.clone(), m.name.clone()));
    }
    let close: Vec<_> = c.modes.iter().filter(|m| bare(&m.name).eq_ignore_ascii_case(b)).collect();
    match close.as_slice() {
        [m] => Ok((c.id.clone(), c.name.clone(), m.name.clone())),
        [] => Err(format!("a coleção {} não tem modo para a marca {b}", c.name)),
        many => Err(format!(
            "a coleção {} tem mais de um modo para a marca {b} ({}): escolha um",
            c.name,
            many.iter().map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ")
        )),
    }
}

/// What the brand registry officially lists for the brand, and its gaps (null entries).
fn official(registry: &Value, gaps: &mut Vec<String>) -> Map<String, Value> {
    let mut out = Map::new();
    let Some(r) = registry.as_object() else { return out };
    for key in ["plataformas", "team_components_app", "telas_e_fluxos_app"] {
        match r.get(key) {
            Some(Value::Null) | None => gaps.push(format!("Registro de marcas sem {key}")),
            Some(v) => {
                out.insert(key.to_string(), v.clone());
            }
        }
    }
    if let Some(b) = r.get("benchmark").and_then(Value::as_object) {
        for (k, v) in b {
            match v {
                Value::Null => gaps.push(format!("Registro de marcas sem benchmark.{k}")),
                v => {
                    out.insert(format!("benchmark.{k}"), v.clone());
                }
            }
        }
    }
    out
}

fn apply_cmd(s: &mut Session, p: &Value) -> Result<Value> {
    const CMD: &str = "brand.apply";
    let brand = match get(p, "brand") {
        Some(Value::String(b)) if !b.trim().is_empty() && b.len() <= 100 => b.trim().to_string(),
        _ => return Err(bad(CMD, "`brand` is the brand's name")),
    };
    let ids: Vec<NodeId> = match get(p, "ids") {
        Some(Value::Array(a)) if !a.is_empty() && a.len() <= MAX_IDS => {
            a.iter().map(id_of).collect::<Option<_>>().ok_or_else(|| bad(CMD, "ids: layer ids"))?
        }
        _ => return Err(bad(CMD, format!("`ids` lists 1 to {MAX_IDS} screens"))),
    };
    let registry = match get(p, "registry") {
        None | Some(Value::Null) => None,
        Some(v @ Value::Object(_)) => Some(v.clone()),
        Some(_) => return Err(bad(CMD, "registry: the brand's entry, as a JSON object")),
    };
    let profile = read_profile(CMD, p)?.map(|t| parse_profile(&t));
    let doc = s.doc()?;
    if let Some(missing) = ids.iter().find(|i| doc.node(**i).is_none()) {
        return Err(bad(CMD, format!("no layer {}", missing.0)));
    }
    let mut gaps: Vec<String> = Vec::new();
    let mode = match brand_mode(doc, &brand) {
        Ok(m) => Some(m),
        Err(why) => {
            gaps.push(why);
            None
        }
    };
    let (profile_kept, basis) = match &profile {
        Some(pr) => {
            if let Some(g) = pr["gaps"].as_array() {
                gaps.extend(g.iter().filter_map(Value::as_str).map(str::to_string));
            }
            let kept = json!({"owner": pr["owner"], "reviewed": pr["reviewed"], "note": pr["note"], "digest": pr["digest"], "basis": pr["basis"]});
            (kept, pr["basis"].as_str().unwrap_or("usabilidade-e-ds").to_string())
        }
        None => {
            gaps.push(format!("A marca {brand} não tem perfil: as propostas usam só usabilidade e o DS"));
            (Value::Null, "usabilidade-e-ds".to_string())
        }
    };
    let official = registry.as_ref().map(|r| official(r, &mut gaps)).unwrap_or_default();
    let record = json!({
        "brand": brand,
        "mode": mode.as_ref().map(|(_, c, m)| json!({"collection": c, "name": m})),
        "profile": profile_kept,
        "basis": basis,
        "official": official,
        "gaps": gaps,
    });
    let text = record.to_string();
    let mode_id = mode.as_ref().and_then(|(cid, _, name)| {
        doc.variables.collection(cid).and_then(|c| c.modes.iter().find(|m| &m.name == name)).map(|m| (cid.clone(), m.id.clone()))
    });
    let doc = s.doc_mut()?;
    for id in &ids {
        let n = doc.get_mut(*id)?;
        if let Some((cid, mid)) = &mode_id {
            n.var_modes.insert(cid.clone(), mid.clone());
        }
        n.plugin_data.insert(BRAND_KEY.to_string(), text.clone());
    }
    Ok(record)
}

/// The brand a screen was designed for (`brand.apply`), if any.
pub fn brand_of(doc: &Document, id: NodeId) -> Option<Value> {
    serde_json::from_str::<Value>(doc.node(id)?.plugin_data.get(BRAND_KEY)?).ok().filter(Value::is_object)
}
