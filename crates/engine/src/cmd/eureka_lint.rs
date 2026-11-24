//! Design-system lint (`ds.lint`): point out where a screen strays from the design system's rules.
//! It only warns: it never blocks and never changes the document. Every rule cites the written
//! source it comes from (`docs/eureka/G2-F-regras.md`); a rule without one does not belong here.
//!
//! Rules: `valor-solto` (a raw value where the document has a variable for that property; reuses
//! the proposals' [`ds_warnings`]), `escopo` (a variable bound outside its declared scope, and
//! decision D4: padding bound to a `GAP` variable), `marca-nao-escolhida` (a screen using the brand
//! collection without choosing its mode) and `marca-no-componente` (a brand mode set on a main
//! component, which every instance inherits).

use std::collections::BTreeMap;

use figcraft_doc::{Document, NodeId, NodeKind, Variable, VariableType};
use serde_json::{Value, json};

use super::CommandSpec;
use super::eureka::{ds_warnings, num_text};
use super::variables::field_scope;
use crate::params::{get, id_of};
use crate::{EngineError, Result, bad, spec};

const CMD: &str = "ds.lint";
/// Findings listed by default; the rest is only counted.
const DEFAULT_LIMIT: usize = 200;
/// Most findings a call lists.
const MAX_LIMIT: usize = 2000;
/// Layers looked at per call; past it the lint stops and says `truncated`.
const MAX_SCANNED: usize = 100_000;
/// Deepest nesting walked.
const MAX_DEPTH: usize = 64;
/// Most `ids` a call takes.
const MAX_IDS: usize = 10_000;
/// The brand collection when `brandCollection` is not given (E05 D3; DS N Marcas — Web).
const DEFAULT_BRAND: &str = "Tokens WEB";

const SRC_TOKENS: &str =
    "https://app.notion.com/p/3a4690e9a7e281e5baefc847c98ee64a (Fluxo Oficial §4: \"use tokens e variáveis das coleções oficiais\")";
const SRC_SCOPE: &str = "docs/eureka/E05-arquitetura-adrs.md §7 D4 (\"aviso, não erro, até a liderança de DS decidir\")";
const SRC_BRAND_SCREEN: &str = "docs/eureka/E05-arquitetura-adrs.md §7 D3 (\"a marca deve ser escolhida na tela, nunca deixada ao padrão\"); https://app.notion.com/p/39e690e9a7e281978c0dfc4333a87e8c (\"conferir modos de marca\")";
const SRC_BRAND_COMPONENT: &str = "https://app.notion.com/p/39e690e9a7e281978c0dfc4333a87e8c (DS N Marcas — Web §3: \"nunca deixando um modo específico preso ao componente original\")";

pub fn specs() -> Vec<CommandSpec> {
    vec![spec!(
        CMD,
        "Lint the design system",
        "{ids? (layers to look at with everything inside them; default the current page), limit? (findings listed, default 200, at most 2000), brandCollection? (the brand collection, default \"Tokens WEB\")}  read-only: where the layers stray from the design system's rules (raw value where a variable exists, variable out of its scope, screen without a brand mode, brand mode on a main component). Only warns → {findings: [{id, name, rule, message, source, prop?, value?}], counted, scanned, truncated}",
        false,
        run
    )]
}

fn run(s: &mut crate::Session, p: &Value) -> Result<Value> {
    let limit = match get(p, "limit") {
        None => DEFAULT_LIMIT,
        Some(v) => match v.as_u64() {
            Some(n) if n >= 1 => usize::try_from(n).unwrap_or(MAX_LIMIT).min(MAX_LIMIT),
            _ => return Err(bad(CMD, "`limit` must be a whole number from 1")),
        },
    };
    let brand_name = match get(p, "brandCollection") {
        None => DEFAULT_BRAND.to_string(),
        Some(Value::String(t)) if !t.trim().is_empty() && t.chars().count() <= 200 => t.trim().to_string(),
        Some(_) => return Err(bad(CMD, "`brandCollection` must be a collection name or id")),
    };
    let page = s.page()?;
    let doc = s.doc()?;
    let roots = match get(p, "ids") {
        None => vec![page],
        Some(Value::Array(a)) => {
            if a.is_empty() || a.len() > MAX_IDS {
                return Err(bad(CMD, format!("`ids` must list 1 to {MAX_IDS} layers")));
            }
            let mut ids = Vec::with_capacity(a.len());
            for v in a {
                let id = id_of(v).ok_or_else(|| bad(CMD, "`ids`: every entry must be a node id"))?;
                if !doc.contains(id) {
                    return Err(EngineError::NoNode(id));
                }
                ids.push(id);
            }
            ids
        }
        Some(_) => return Err(bad(CMD, "`ids` must be an array of node ids")),
    };
    let brand = doc.variables.collection(&brand_name).map(|c| c.id.clone());
    let mut lint = Lint { doc, brand, findings: Vec::new(), counted: 0, limit, scanned: 0, truncated: false, tokens: BTreeMap::new() };
    for root in roots {
        lint.walk(root, 0);
    }
    Ok(json!({"findings": lint.findings, "counted": lint.counted, "scanned": lint.scanned, "truncated": lint.truncated}))
}

struct Lint<'a> {
    doc: &'a Document,
    /// The brand collection's id, when the document has it.
    brand: Option<String>,
    findings: Vec<Value>,
    counted: usize,
    limit: usize,
    scanned: usize,
    truncated: bool,
    /// Whether the document has a variable of a type for a scope (memo).
    tokens: BTreeMap<(&'static str, Option<&'static str>), bool>,
}

impl Lint<'_> {
    fn walk(&mut self, id: NodeId, depth: usize) {
        if self.scanned >= MAX_SCANNED || depth > MAX_DEPTH {
            self.truncated = true;
            return;
        }
        let Some(node) = self.doc.node(id) else { return };
        self.scanned += 1;
        if !matches!(node.kind, NodeKind::Page { .. }) {
            self.check(node);
        }
        for c in &node.children {
            self.walk(*c, depth + 1);
        }
    }

    fn push(&mut self, node: &figcraft_doc::Node, rule: &str, message: String, source: &str, prop: Option<&str>, value: Option<Value>) {
        self.counted += 1;
        if self.findings.len() >= self.limit {
            self.truncated = true;
            return;
        }
        let mut f = json!({"id": node.id.0, "name": node.name, "rule": rule, "message": message, "source": source});
        if let Some(o) = f.as_object_mut() {
            if let Some(prop) = prop {
                o.insert("prop".into(), json!(prop));
            }
            if let Some(value) = value {
                o.insert("value".into(), value);
            }
        }
        self.findings.push(f);
    }

    fn check(&mut self, node: &figcraft_doc::Node) {
        self.raw_values(node);
        self.scopes(node);
        self.brand_mode(node);
    }

    /// `valor-solto`: [`ds_warnings`] on the whole layer, kept where the document has a variable
    /// that could take that property.
    fn raw_values(&mut self, node: &figcraft_doc::Node) {
        let Ok(a) = serde_json::to_value(node) else { return };
        for w in ds_warnings(node.id, &a, None) {
            let Some(prop) = w.get("prop").and_then(Value::as_str) else { continue };
            let value = w.get("value").cloned().unwrap_or(Value::Null);
            let (field, kind, what, family) = match prop {
                "fill" => ("fills/0/color", VariableType::Color, "preenchimento", "de cor"),
                "stroke" => ("strokes/0/color", VariableType::Color, "contorno", "de cor"),
                "fontSize" => ("fontSize", VariableType::Number, "tamanho da fonte", "de tamanho de fonte"),
                "gap" => ("gap", VariableType::Number, "espaçamento (gap)", "de espaçamento"),
                "padding" => ("paddingHorizontal", VariableType::Number, "padding", "de espaçamento"),
                "cornerRadius" => ("cornerRadius", VariableType::Number, "raio", "de raio"),
                _ => continue,
            };
            let scope = field_scope(self.doc, node.id, field);
            if !self.has_token(kind, scope) {
                continue;
            }
            let message = format!("{}: {what} {} está fixo; use um token {family} do DS.", node.name, value_text(&value));
            self.push(node, "valor-solto", message, SRC_TOKENS, Some(prop), Some(value));
        }
    }

    fn has_token(&mut self, kind: VariableType, scope: Option<&'static str>) -> bool {
        let doc = self.doc;
        *self
            .tokens
            .entry((kind.id(), scope))
            .or_insert_with(|| doc.variables.variables.iter().any(|v| v.kind == kind && scope.is_none_or(|sc| v.in_scope(sc))))
    }

    /// `escopo`: bindings to a variable whose declared scopes leave the field out; and, by decision
    /// D4, padding bound to a variable scoped to `GAP`.
    fn scopes(&mut self, node: &figcraft_doc::Node) {
        for (field, vid) in &node.bound {
            let Some(var) = self.doc.variables.get_by_id(vid) else { continue };
            let Some(scope) = field_scope(self.doc, node.id, field) else { continue };
            let declared = scopes_text(var);
            if !var.in_scope(scope) {
                let message = format!(
                    "{}: {} usa {}, cujo escopo declarado é {declared}; está fora do escopo (só aviso).",
                    node.name,
                    field_label(field),
                    var.name
                );
                self.push(node, "escopo", message, SRC_SCOPE, Some(field), Some(json!(var.name)));
            } else if field.starts_with("padding") && var.scopes.as_ref().is_some_and(|l| l.iter().any(|s| s == "GAP")) {
                let message = format!(
                    "{}: {} usa {} (escopo {declared}). Pela decisão D4, variable de gap em padding é aviso até a liderança de DS decidir.",
                    node.name,
                    field_label(field),
                    var.name
                );
                self.push(node, "escopo", message, SRC_SCOPE, Some(field), Some(json!(var.name)));
            }
        }
    }

    /// `marca-nao-escolhida` (a screen) and `marca-no-componente` (a main component or a layer in
    /// one).
    fn brand_mode(&mut self, node: &figcraft_doc::Node) {
        let Some(brand) = self.brand.clone() else { return };
        let coll = self.doc.variables.collection_by_id(&brand).map_or_else(|| brand.clone(), |c| c.name.clone());
        if let Some(mode) = node.var_modes.get(&brand)
            && self.in_main_component(node.id)
        {
            let mode_name =
                self.doc.variables.collection_by_id(&brand).and_then(|c| c.mode_by_id(mode)).map_or_else(|| mode.clone(), |m| m.name.clone());
            let message = format!(
                "{}: o componente original está com a marca {mode_name} ({coll}) fixada. Ele deve ficar neutro; teste a marca numa instância ou tela.",
                node.name
            );
            self.push(node, "marca-no-componente", message, SRC_BRAND_COMPONENT, Some("variableMode"), Some(json!(mode_name)));
        }
        if self.is_screen(node) && !self.doc.modes_for(node.id).contains_key(&brand) && self.uses_collection(node.id, &brand) {
            let default = self.doc.variables.collection_by_id(&brand).and_then(|c| c.modes.first()).map(|m| m.name.clone()).unwrap_or_default();
            let message = format!(
                "{}: a tela usa tokens de {coll} sem escolher a marca; está herdando o modo padrão ({default}). Escolha a marca na tela.",
                node.name
            );
            self.push(node, "marca-nao-escolhida", message, SRC_BRAND_SCREEN, Some("variableMode"), None);
        }
    }

    /// A frame on the page, or in a section on the page: a screen.
    fn is_screen(&self, node: &figcraft_doc::Node) -> bool {
        if !matches!(node.kind, NodeKind::Frame) {
            return false;
        }
        let parent = node.parent.and_then(|p| self.doc.node(p));
        match parent.map(|p| &p.kind) {
            Some(NodeKind::Page { .. }) => true,
            Some(NodeKind::Section { .. }) => {
                parent.and_then(|p| p.parent).and_then(|g| self.doc.node(g)).is_some_and(|g| matches!(g.kind, NodeKind::Page { .. }))
            }
            _ => false,
        }
    }

    /// Whether the layer is a main component or component set, or inside one.
    fn in_main_component(&self, id: NodeId) -> bool {
        std::iter::once(id)
            .chain(self.doc.ancestors(id).into_iter().take(MAX_DEPTH))
            .filter_map(|i| self.doc.node(i))
            .any(|n| matches!(n.kind, NodeKind::Component { .. } | NodeKind::ComponentSet { .. }))
    }

    /// Whether a layer in this subtree is bound to a variable of collection `coll` (bounded walk).
    fn uses_collection(&self, id: NodeId, coll: &str) -> bool {
        let mut stack = vec![(id, 0_usize)];
        let mut seen = 0_usize;
        while let Some((i, depth)) = stack.pop() {
            seen += 1;
            if seen > MAX_SCANNED {
                return false;
            }
            let Some(n) = self.doc.node(i) else { continue };
            if n.bound.values().any(|v| self.doc.variables.get_by_id(v).is_some_and(|v| v.collection == coll)) {
                return true;
            }
            if depth < MAX_DEPTH {
                stack.extend(n.children.iter().map(|c| (*c, depth + 1)));
            }
        }
        false
    }
}

fn scopes_text(v: &Variable) -> String {
    match &v.scopes {
        None => "todos".into(),
        Some(l) if l.is_empty() => "nenhum".into(),
        Some(l) => l.join(", "),
    }
}

fn field_label(field: &str) -> String {
    match field {
        "fontSize" => "o tamanho da fonte".into(),
        "cornerRadius" | "topLeftRadius" | "topRightRadius" | "bottomRightRadius" | "bottomLeftRadius" => "o raio".into(),
        "gap" | "columnGap" | "rowGap" | "crossGap" => "o espaçamento (gap)".into(),
        "paddingHorizontal" => "o padding horizontal".into(),
        "paddingVertical" => "o padding vertical".into(),
        "paddingTop" => "o padding de cima".into(),
        "paddingBottom" => "o padding de baixo".into(),
        "paddingLeft" => "o padding da esquerda".into(),
        "paddingRight" => "o padding da direita".into(),
        f if f.starts_with("padding") => "o padding".into(),
        f if f.starts_with("fills/") => "o preenchimento".into(),
        f if f.starts_with("strokes/") => "o contorno".into(),
        "width" | "height" | "minWidth" | "maxWidth" | "minHeight" | "maxHeight" => "o tamanho".into(),
        other => format!("`{other}`"),
    }
}

/// `#7A4B2A`, `18`, `16, 16, 16, 16`.
fn value_text(v: &Value) -> String {
    match v {
        Value::String(t) => t.clone(),
        Value::Number(n) => n.as_f64().map_or_else(|| n.to_string(), num_text),
        Value::Array(a) => {
            let nums: Vec<f64> = a.iter().filter_map(Value::as_f64).collect();
            match nums.first() {
                Some(first) if nums.iter().all(|x| (x - first).abs() < f64::EPSILON) => num_text(*first),
                _ => nums.into_iter().map(num_text).collect::<Vec<_>>().join(", "),
            }
        }
        other => other.to_string(),
    }
}
