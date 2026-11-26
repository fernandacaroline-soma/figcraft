//! Brand Resolver (E13): a brand's profile read as written, the brand's mode set on the proposals,
//! and the gaps said, never filled in.

use serde_json::{Value, json};

use crate::components::tests::{ex, idv};
use crate::{NodeId, Session};

/// A profile shaped as the skill's `_modelo.md`, partly filled (as a brand's first revision is).
fn partial() -> String {
    "# Perfil — Marca Teste

**Dona:** Ana · **Última revisão:** 10/11/2026 (parcial: só concorrentes)

## Posicionamento e público
[a preencher]

## Concorrentes e referências de mercado (lente de benchmarking)
Concorrentes servem para comparar padrões da categoria.

## Memória visual
Ainda sem entradas. Ver a página do Notion.

## Plataformas e arquitetura de DS
- Web: [DS N Marcas / estrutura específica]
- App: [DS Apps / Team Components]

## O que não fazer
[a preencher]
"
    .to_string()
}

/// The template itself: nothing filled.
fn template() -> String {
    "# Perfil — [marca]\n\n**Dona:** [designer responsável] · **Última revisão:** [data]\n\n## Posicionamento e público\n[quem é a cliente]\n\n## Decisões e testes anteriores\nNão duplicar aqui.\n".to_string()
}

fn screens(s: &mut Session) -> (NodeId, NodeId) {
    ex(s, "variable.createCollection", json!({"name": "Tokens APP", "modes": ["Marca Teste", "Outra (CERTO)", "Dupla (A)", "Dupla (B)"]}));
    ex(s, "variable.create", json!({"collection": "Tokens APP", "name": "Colors/Bg", "type": "color", "value": "#ffffff"}));
    let screen = |name: &str| json!({"name": name, "type": "incremental", "screen": {"frame": "PDP", "width": 100, "height": 100, "fill": "Colors/Bg", "children": []}});
    let r = ex(s, "explore.board", json!({"title": "Rodada", "proposals": [screen("Um"), screen("Dois")]}));
    (idv(&r["proposals"][0]), idv(&r["proposals"][1]))
}

fn mode_of(s: &Session, id: NodeId) -> Option<String> {
    let doc = s.doc().unwrap();
    let c = doc.variables.collection("Tokens APP").unwrap();
    let mid = doc.node(id).unwrap().var_modes.get(&c.id)?.clone();
    c.modes.iter().find(|m| m.id == mid).map(|m| m.name.clone())
}

#[test]
fn a_profile_is_read_as_written_with_its_owner_review_and_gaps() {
    let mut s = Session::new();
    let r = ex(&mut s, "brand.profile", json!({"text": partial()}));
    assert_eq!((r["brand"].clone(), r["owner"].clone(), r["reviewed"].clone()), (json!("Marca Teste"), json!("Ana"), json!("10/11/2026")));
    assert_eq!(r["note"], json!("parcial: só concorrentes"), "the review's note as written");
    let status = |t: &str| r["sections"].as_array().unwrap().iter().find(|x| x["title"].as_str().unwrap().starts_with(t)).unwrap()["status"].clone();
    assert_eq!(status("Posicionamento"), json!("to-fill"));
    assert_eq!(status("Concorrentes"), json!("filled"));
    assert_eq!(status("Memória visual"), json!("pointer"), "a pointer to Notion is never a gap");
    assert_eq!(status("Plataformas"), json!("to-fill"), "list items with only placeholders");
    let gaps = r["gaps"].to_string();
    assert!(gaps.contains("Seção a preencher: Posicionamento e público") && gaps.contains("O que não fazer"), "{gaps}");
    assert_eq!(r["basis"], json!("perfil"), "something usable is filled");
    // A section still to fill carries no text: nothing is read into it.
    let pos = r["sections"].as_array().unwrap().iter().find(|x| x["title"] == json!("Posicionamento e público")).unwrap().clone();
    assert!(pos.get("text").is_none(), "{pos}");
    // The digest tells revisions apart.
    let other = ex(&mut s, "brand.profile", json!({"text": partial().replace("Ana", "Bia")}));
    assert_ne!(r["digest"], other["digest"]);
}

#[test]
fn an_unfilled_profile_means_usability_and_the_design_system_only() {
    let mut s = Session::new();
    let r = ex(&mut s, "brand.profile", json!({"text": template()}));
    assert_eq!(r["basis"], json!("usabilidade-e-ds"));
    assert_eq!((r["owner"].clone(), r["reviewed"].clone()), (Value::Null, Value::Null));
    let gaps = r["gaps"].to_string();
    assert!(gaps.contains("Perfil sem dona") && gaps.contains("Perfil sem data de revisão"), "{gaps}");
}

#[test]
fn applying_a_brand_sets_its_mode_and_keeps_what_it_was_designed_with() {
    let mut s = Session::new();
    let (a, b) = screens(&mut s);
    let registry = json!({"plataformas": ["web", "app"], "team_components_app": "tc-key", "telas_e_fluxos_app": "tf-key", "perfil": "marcas/teste.md",
        "benchmark": {"concorrentes": "https://example.com/c", "referencia_de_marca": null}});
    let r = ex(&mut s, "brand.apply", json!({"ids": [a.0, b.0], "brand": "Marca Teste", "text": partial(), "registry": registry}));
    assert_eq!(r["mode"], json!({"collection": "Tokens APP", "name": "Marca Teste"}));
    assert_eq!((mode_of(&s, a), mode_of(&s, b)), (Some("Marca Teste".into()), Some("Marca Teste".into())));
    assert_eq!(r["profile"]["owner"], json!("Ana"));
    assert_eq!(r["official"]["team_components_app"], json!("tc-key"));
    assert!(r["gaps"].to_string().contains("Registro de marcas sem benchmark.referencia_de_marca"), "{r}");
    let kept = crate::cmd::eureka_brand::brand_of(s.doc().unwrap(), a).unwrap();
    assert_eq!(kept["profile"]["digest"], r["profile"]["digest"], "the version is kept on the screen");
    // One undo step takes it all back.
    ex(&mut s, "edit.undo", json!({}));
    assert_eq!(mode_of(&s, a), None);
    assert!(crate::cmd::eureka_brand::brand_of(s.doc().unwrap(), a).is_none());
}

#[test]
fn switching_to_a_brand_with_no_profile_says_so_and_still_sets_its_mode() {
    let mut s = Session::new();
    let (a, _) = screens(&mut s);
    ex(&mut s, "brand.apply", json!({"ids": [a.0], "brand": "Marca Teste", "text": partial()}));
    // "Outra" is the mode "Outra (CERTO)": a note in parentheses.
    let r = ex(&mut s, "brand.apply", json!({"ids": [a.0], "brand": "Outra"}));
    assert_eq!(mode_of(&s, a), Some("Outra (CERTO)".into()));
    assert_eq!((r["basis"].clone(), r["profile"].clone()), (json!("usabilidade-e-ds"), Value::Null));
    assert!(r["gaps"].to_string().contains("não tem perfil"), "{r}");
    // Two modes for one brand: none is picked, and the gap names them.
    let r = ex(&mut s, "brand.apply", json!({"ids": [a.0], "brand": "Dupla"}));
    assert_eq!(r["mode"], Value::Null);
    assert!(r["gaps"].to_string().contains("Dupla (A), Dupla (B)"), "{r}");
    assert_eq!(mode_of(&s, a), Some("Outra (CERTO)".into()), "the mode stays as it was");
    // A brand with no mode at all.
    let r = ex(&mut s, "brand.apply", json!({"ids": [a.0], "brand": "Inexistente"}));
    assert!(r["gaps"].to_string().contains("não tem modo para a marca Inexistente"), "{r}");
}

#[test]
fn bad_brand_parameters_are_refused() {
    let mut s = Session::new();
    let (a, _) = screens(&mut s);
    for bad in [
        json!({}),
        json!({"ids": [a.0]}),
        json!({"brand": "X"}),
        json!({"ids": [], "brand": "X"}),
        json!({"ids": [999999], "brand": "X"}),
        json!({"ids": [a.0], "brand": "X", "registry": 3}),
        json!({"ids": [a.0], "brand": "X", "path": 3}),
    ] {
        assert!(s.execute("brand.apply", &bad).is_err(), "{bad}");
    }
    assert!(s.execute("brand.profile", &json!({})).is_err());
    assert!(s.execute("brand.profile", &json!({"path": "/no/such/profile.md"})).is_err());
}
