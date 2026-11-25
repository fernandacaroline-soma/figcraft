//! A proposal's screen described by what it is made of (Eureka G3, Frente N), so a proposals
//! generator such as "gerador-propostas" can draw with the library's components without knowing
//! any layer id: a component is named as the designer sees it (`"Button"`, variant `Tipo=Primário`,
//! text `Texto`), and every size or colour is a variable name. The whole description is read and
//! checked against the file before anything is drawn, so a misspelt component, variant, property
//! or token refuses the proposal with the options that exist, instead of drawing half a screen.
//!
//! `explore.catalog` lists what a description can use: the component sets and components of the
//! file (their variant options and properties) and the variables.

use std::collections::BTreeMap;

use serde_json::{Map, Value, json};

use super::CommandSpec;
use crate::components::{slots_of, variant_options};
use crate::params::id_of;
use crate::{Document, NodeId, Result, Session, bad, spec};
use figcraft_doc::NodeKind;
use figcraft_doc::PropDef;

/// Deepest a description may nest, and most layers one screen may have.
const MAX_DEPTH: usize = 12;
const MAX_ITEMS: usize = 400;
/// Most components and variables `explore.catalog` lists.
const MAX_CATALOG: usize = 400;

pub fn specs() -> Vec<CommandSpec> {
    vec![spec!(
        "explore.catalog",
        "Exploration catalog",
        "{query?}  what a proposal's `screen` can be made of: the file's component sets and components (variant options, properties by the names a description uses) and its variables → {components: [{name, variants: {prop: [values]}, props: {name: {text?: default, boolean?: default}}, slots}], variables: [name]}",
        false,
        catalog
    )]
}

/// The name a description uses for a property or layer: without the emoji prefix and the `#id`
/// suffix Figma adds, ignoring case (`"✍️ Texto#7:26"` → `"texto"`).
pub fn short(name: &str) -> String {
    let base = name.split('#').next().unwrap_or(name);
    base.trim_start_matches(|c: char| !c.is_alphanumeric()).trim().to_lowercase()
}

fn variant_pairs(name: &str) -> BTreeMap<String, String> {
    name.split(',').filter_map(|kv| kv.split_once('=')).map(|(k, v)| (short(k), v.trim().to_string())).collect()
}

/// A layer of a described screen, read and checked.
#[derive(Debug)]
pub enum Item {
    Frame(Frame),
    Text { text: String, name: String, size: Option<String>, color: Option<String>, wrap: bool },
    Image { name: String, width: f64, height: f64, fill: Option<String>, fill_width: bool },
    Instance(Instance),
}

#[derive(Debug, Default)]
pub struct Frame {
    name: String,
    layout: Option<&'static str>,
    width: f64,
    height: f64,
    gap: Option<Value>,
    padding_h: Option<Value>,
    padding_v: Option<Value>,
    fill: Option<String>,
    fixed_width: bool,
    fixed_height: bool,
    center: bool,
    clip: bool,
    dock_bottom: bool,
    children: Vec<Item>,
}

#[derive(Debug)]
pub struct Instance {
    /// The variant (or component) to instance.
    component: NodeId,
    /// What the description called it, for the proposal's record.
    label: String,
    name: String,
    props: Map<String, Value>,
    fill_width: bool,
    dock_bottom: bool,
    /// Nested instances by layer name, with their own property values.
    layers: Vec<(String, Map<String, Value>)>,
    slot: Option<Vec<Item>>,
}

/// What a described screen uses, kept on the proposal for the designer and the reviewer.
#[derive(Debug, Default)]
pub struct Uses {
    pub components: Vec<String>,
    pub variables: Vec<String>,
}

struct Reader<'a> {
    doc: &'a Document,
    cmd: &'a str,
    count: usize,
    uses: Uses,
}

fn s_of<'v>(v: &'v Value, key: &str) -> Option<&'v str> {
    v.get(key).and_then(Value::as_str).map(str::trim).filter(|t| !t.is_empty())
}

fn num(v: &Value, key: &str) -> Option<f64> {
    v.get(key).and_then(Value::as_f64).filter(|n| n.is_finite())
}

/// Read and check a described screen: its root must be a frame with a size.
pub fn read(doc: &Document, cmd: &str, v: &Value) -> Result<(Item, Uses)> {
    let mut r = Reader { doc, cmd, count: 0, uses: Uses::default() };
    let root = r.item(v, 0, true)?;
    match &root {
        Item::Frame(f) if f.width > 0.0 && f.height > 0.0 => {}
        _ => return Err(bad(cmd, "a `screen` starts with a `frame` that has `width` and `height`")),
    }
    Ok((root, r.uses))
}

impl Reader<'_> {
    fn token(&mut self, v: &Value, key: &str) -> Result<Option<String>> {
        match v.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::String(name)) => {
                // Bound by id: `variable.bind` looks names up by first match only.
                let id = super::variables::resolve_strict(self.cmd, self.doc, name)?;
                if !self.uses.variables.contains(name) {
                    self.uses.variables.push(name.clone());
                }
                Ok(Some(id.to_string()))
            }
            Some(_) => Err(bad(self.cmd, format!("`{key}` must be a variable name"))),
        }
    }

    /// A number (kept as a fixed value, which the DS lint reports) or a variable name.
    fn amount(&mut self, v: &Value, key: &str) -> Result<Option<Value>> {
        match v.get(key) {
            None | Some(Value::Null) => Ok(None),
            Some(Value::Number(n)) => Ok(Some(Value::Number(n.clone()))),
            Some(Value::String(_)) => Ok(self.token(v, key)?.map(Value::String)),
            Some(_) => Err(bad(self.cmd, format!("`{key}` must be a number or a variable name"))),
        }
    }

    fn children(&mut self, v: &Value, key: &str, depth: usize) -> Result<Vec<Item>> {
        match v.get(key) {
            None | Some(Value::Null) => Ok(Vec::new()),
            Some(Value::Array(list)) => list.iter().map(|c| self.item(c, depth + 1, false)).collect(),
            Some(_) => Err(bad(self.cmd, format!("`{key}` must be a list of layers"))),
        }
    }

    fn item(&mut self, v: &Value, depth: usize, root: bool) -> Result<Item> {
        self.count += 1;
        if self.count > MAX_ITEMS || depth > MAX_DEPTH {
            return Err(bad(self.cmd, format!("a screen takes at most {MAX_ITEMS} layers, {MAX_DEPTH} deep")));
        }
        let dock_bottom = match s_of(v, "dock") {
            None => false,
            Some("bottom") if depth == 1 => true,
            Some(_) => return Err(bad(self.cmd, "`dock` is \"bottom\", on a layer directly in the screen")),
        };
        let fill_width = !root && v.get("width").is_none_or(|w| w.as_str() == Some("fill"));
        if let Some(name) = s_of(v, "frame") {
            let layout = match s_of(v, "layout") {
                None | Some("vertical") => Some("vertical"),
                Some("horizontal") => Some("horizontal"),
                Some("none") => None,
                Some(_) => return Err(bad(self.cmd, "`layout` is \"vertical\", \"horizontal\" or \"none\"")),
            };
            let (ph, pv) = match v.get("padding") {
                Some(Value::Object(_)) => {
                    let p = &v["padding"];
                    (self.amount(p, "h")?, self.amount(p, "v")?)
                }
                Some(_) => {
                    let a = self.amount(v, "padding")?;
                    (a.clone(), a)
                }
                None => (None, None),
            };
            let frame = Frame {
                name: name.to_string(),
                layout,
                width: num(v, "width").unwrap_or(10.0),
                height: num(v, "height").unwrap_or(10.0),
                gap: self.amount(v, "gap")?,
                padding_h: ph,
                padding_v: pv,
                fill: self.token(v, "fill")?,
                fixed_width: root || num(v, "width").is_some(),
                fixed_height: root || num(v, "height").is_some(),
                center: s_of(v, "align") == Some("center"),
                clip: root || v.get("clip").and_then(Value::as_bool).unwrap_or(false),
                dock_bottom,
                children: self.children(v, "children", depth)?,
            };
            return Ok(Item::Frame(frame));
        }
        if root {
            return Err(bad(self.cmd, "a `screen` starts with a `frame`"));
        }
        if let Some(text) = v.get("text").and_then(Value::as_str) {
            return Ok(Item::Text {
                text: text.to_string(),
                name: s_of(v, "name").map_or_else(|| text.chars().take(28).collect(), str::to_string),
                size: self.token(v, "size")?,
                color: self.token(v, "color")?,
                wrap: v.get("wrap").and_then(Value::as_bool).unwrap_or(false),
            });
        }
        if let Some(name) = s_of(v, "image") {
            let (Some(width), Some(height)) = (num(v, "width").or(Some(100.0)), num(v, "height")) else {
                return Err(bad(self.cmd, format!("image `{name}` needs a `height`")));
            };
            return Ok(Item::Image { name: format!("{name} (imagem de exemplo)"), width, height, fill: self.token(v, "fill")?, fill_width });
        }
        if let Some(label) = s_of(v, "component") {
            let component = self.component(label, v.get("variant"))?;
            let props = self.props(component, label, v.get("props"))?;
            let layers = self.layers(component, label, v.get("layers"))?;
            let slot = match v.get("slot") {
                None | Some(Value::Null) => None,
                Some(_) => {
                    if slots_in_main(self.doc, component) == 0 {
                        return Err(bad(self.cmd, format!("`{label}` has no slot")));
                    }
                    Some(self.children(v, "slot", depth)?)
                }
            };
            if !self.uses.components.iter().any(|c| c == label) {
                self.uses.components.push(label.to_string());
            }
            return Ok(Item::Instance(Instance {
                component,
                label: label.to_string(),
                name: s_of(v, "name").unwrap_or(label).to_string(),
                props,
                fill_width: v.get("width").and_then(Value::as_str) == Some("fill"),
                dock_bottom,
                layers,
                slot,
            }));
        }
        Err(bad(self.cmd, "each layer is a `frame`, `text`, `image` or `component`"))
    }

    /// The variant of set `label` with every given `variant` value (other values: the set's first
    /// matching variant), or the component `label` itself.
    fn component(&self, label: &str, wanted: Option<&Value>) -> Result<NodeId> {
        let doc = self.doc;
        let named = |want_set: bool| {
            doc.nodes().find(|n| {
                n.name.trim() == label
                    && if want_set {
                        matches!(n.kind, NodeKind::ComponentSet { .. })
                    } else {
                        matches!(n.kind, NodeKind::Component { .. })
                            && !n.parent.and_then(|p| doc.node(p)).is_some_and(|p| matches!(p.kind, NodeKind::ComponentSet { .. }))
                    }
            })
        };
        let wanted: BTreeMap<String, String> = match wanted {
            None | Some(Value::Null) => BTreeMap::new(),
            Some(Value::Object(o)) => o.iter().map(|(k, v)| (short(k), v.as_str().map_or_else(|| v.to_string(), str::to_string))).collect(),
            Some(_) => return Err(bad(self.cmd, format!("`variant` of `{label}` must be an object of property values"))),
        };
        if let Some(set) = named(true) {
            let options = variant_options(doc, set.id);
            for (k, val) in &wanted {
                let Some((_, vals)) = options.iter().find(|(p, _)| short(p) == *k) else {
                    let names: Vec<String> = options.iter().map(|(p, _)| short(p)).collect();
                    return Err(bad(self.cmd, format!("`{label}` has no variant property `{k}` (it has: {})", names.join(", "))));
                };
                if !vals.contains(val) {
                    return Err(bad(self.cmd, format!("`{label}` {k} is one of: {}", vals.join(", "))));
                }
            }
            return set
                .children
                .iter()
                .copied()
                .find(|c| doc.node(*c).is_some_and(|n| wanted.iter().all(|(k, v)| variant_pairs(&n.name).get(k) == Some(v))))
                .ok_or_else(|| bad(self.cmd, format!("`{label}` has no variant with all of those values together")));
        }
        if let Some(c) = named(false) {
            if !wanted.is_empty() {
                return Err(bad(self.cmd, format!("`{label}` is a component without variants")));
            }
            return Ok(c.id);
        }
        Err(bad(self.cmd, format!("no component or component set named `{label}` in the file (see `explore.catalog`)")))
    }

    /// Property values by short name, keyed by the component's own property names.
    fn props(&self, component: NodeId, label: &str, given: Option<&Value>) -> Result<Map<String, Value>> {
        let mut out = Map::new();
        let Some(given) = given else { return Ok(out) };
        let Value::Object(given) = given else { return Err(bad(self.cmd, format!("`props` of `{label}` must be an object"))) };
        let defs = match self.doc.node(component).map(|n| &n.kind) {
            Some(NodeKind::Component { props, .. }) => props.clone(),
            _ => BTreeMap::new(),
        };
        for (k, v) in given {
            let want = short(k);
            // Figma often has a text and a boolean property of the same name ("✍️ Subtítulo" and
            // "👁️ Subtítulo"): the value's type picks one. A variant property is set through
            // `variant`, not here.
            let same: Vec<(&String, &PropDef)> =
                defs.iter().filter(|(name, d)| short(name) == want && !matches!(d, PropDef::Variant { .. })).collect();
            if same.is_empty() {
                let names: Vec<String> = defs.iter().filter(|(_, d)| !matches!(d, PropDef::Variant { .. })).map(|(n, _)| short(n)).collect();
                return Err(bad(self.cmd, format!("`{label}` has no property `{k}` (it has: {})", names.join(", "))));
            }
            let fits = |d: &PropDef| match d {
                PropDef::Boolean { .. } => v.is_boolean(),
                PropDef::Text { .. } => v.is_string(),
                _ => false,
            };
            let Some((key, _)) = same.iter().find(|(_, d)| fits(d)) else {
                let takes: Vec<&str> = same
                    .iter()
                    .map(|(_, d)| match d {
                        PropDef::Boolean { .. } => "true or false",
                        PropDef::Text { .. } => "text",
                        _ => "a value set another way",
                    })
                    .collect();
                return Err(bad(self.cmd, format!("`{label}` property `{k}` takes {}", takes.join(" or "))));
            };
            out.insert((*key).to_string(), v.clone());
        }
        Ok(out)
    }

    /// `layers: {"layer name": {property: value}}`: nested instances of the component, found by
    /// name in its main.
    fn layers(&self, component: NodeId, label: &str, given: Option<&Value>) -> Result<Vec<(String, Map<String, Value>)>> {
        let Some(Value::Object(given)) = given else { return Ok(Vec::new()) };
        let mut out = Vec::new();
        for (layer, props) in given {
            let Some(inner) = find_named(self.doc, component, layer) else {
                return Err(bad(self.cmd, format!("`{label}` has no layer `{layer}`")));
            };
            let Some(NodeKind::Instance { component: main, .. }) = self.doc.node(inner).map(|n| &n.kind) else {
                return Err(bad(self.cmd, format!("layer `{layer}` of `{label}` is not a component instance")));
            };
            out.push((layer.clone(), self.props(*main, layer, Some(props))?));
        }
        Ok(out)
    }
}

/// The first layer under `root` (itself excluded) whose short name is `name`'s.
fn find_named(doc: &Document, root: NodeId, name: &str) -> Option<NodeId> {
    let want = short(name);
    let mut stack: Vec<NodeId> = doc.node(root)?.children.iter().rev().copied().collect();
    let mut seen = 0usize;
    while let Some(id) = stack.pop() {
        seen += 1;
        if seen > 100_000 {
            return None;
        }
        let n = doc.node(id)?;
        if short(&n.name) == want {
            return Some(id);
        }
        stack.extend(n.children.iter().rev().copied());
    }
    None
}

fn slots_in_main(doc: &Document, component: NodeId) -> usize {
    match doc.node(component).map(|n| &n.kind) {
        Some(NodeKind::Component { props, .. }) => props.values().filter(|d| matches!(d, PropDef::Slot { .. })).count(),
        _ => 0,
    }
}

fn created(r: &Value, cmd: &str) -> Result<NodeId> {
    r.get("id").and_then(id_of).ok_or_else(|| bad(cmd, "a layer of the screen was not created"))
}

fn bind(s: &mut Session, id: NodeId, field: &str, var: &str) -> Result<()> {
    s.execute("variable.bind", &json!({"ids": [id.0], "field": field, "variable": var}))?;
    Ok(())
}

fn amount(s: &mut Session, id: NodeId, field: &str, v: Option<&Value>) -> Result<()> {
    match v {
        Some(Value::String(var)) => bind(s, id, field, var),
        Some(n @ Value::Number(_)) => {
            s.execute("node.set", &json!({"id": id.0, "props": {field: n}}))?;
            Ok(())
        }
        _ => Ok(()),
    }
}

/// Draw a read screen; returns its root.
pub fn build(s: &mut Session, item: &Item, cmd: &str) -> Result<NodeId> {
    let mut docked = Vec::new();
    let root = draw(s, item, None, cmd, &mut docked)?;
    let screen_h = s.doc()?.node(root).map_or(0.0, |n| n.height);
    for id in docked {
        let h = s.doc()?.node(id).map_or(0.0, |n| n.height);
        s.execute("node.set", &json!({"id": id.0, "props": {"absolute": true, "x": 0, "y": screen_h - h}}))?;
    }
    Ok(root)
}

fn draw(s: &mut Session, item: &Item, parent: Option<NodeId>, cmd: &str, docked: &mut Vec<NodeId>) -> Result<NodeId> {
    let parent_v = parent.map_or(Value::Null, |p| json!(p.0));
    // "Fill the width" needs an auto layout parent (a slot may have none).
    let in_layout = match parent {
        Some(p) => s.doc()?.node(p).is_some_and(|n| n.layout.is_some()),
        None => false,
    };
    match item {
        Item::Frame(f) => {
            let mut props = json!({"fill": "none", "padding": 0, "gap": 0});
            if let Some(dir) = f.layout {
                props["layout"] = json!(dir);
                if !f.fixed_height {
                    props["sizingV"] = json!("hug");
                }
            }
            if in_layout && !f.fixed_width {
                props["sizingH"] = json!("fill");
            }
            if f.center {
                props["crossAlign"] = json!("center");
            }
            if f.clip {
                props["clip"] = json!(true);
            }
            let r = s.execute(
                "frame.create",
                &json!({"parent": parent_v, "x": 0, "y": 0, "width": f.width, "height": f.height, "name": f.name, "props": props}),
            )?;
            let id = created(&r, cmd)?;
            if let Some(fill) = &f.fill {
                bind(s, id, "fill", fill)?;
            }
            amount(s, id, "gap", f.gap.as_ref())?;
            amount(s, id, "paddingHorizontal", f.padding_h.as_ref())?;
            amount(s, id, "paddingVertical", f.padding_v.as_ref())?;
            for c in &f.children {
                draw(s, c, Some(id), cmd, docked)?;
            }
            if f.dock_bottom {
                docked.push(id);
            }
            Ok(id)
        }
        Item::Text { text, name, size, color, wrap } => {
            let r = s.execute("text.create", &json!({"parent": parent_v, "text": text, "x": 0, "y": 0, "name": name, "props": {"fontSize": 12}}))?;
            let id = created(&r, cmd)?;
            if let Some(v) = size {
                bind(s, id, "fontSize", v)?;
            }
            if let Some(v) = color {
                bind(s, id, "fill", v)?;
            }
            if *wrap && in_layout {
                s.execute("node.set", &json!({"id": id.0, "props": {"sizingH": "fill"}}))?;
            }
            Ok(id)
        }
        Item::Image { name, width, height, fill, fill_width } => {
            let r = s.execute("shape.rectangle", &json!({"parent": parent_v, "x": 0, "y": 0, "width": width, "height": height, "name": name}))?;
            let id = created(&r, cmd)?;
            if let Some(v) = fill {
                bind(s, id, "fill", v)?;
            }
            if *fill_width && in_layout {
                s.execute("node.set", &json!({"id": id.0, "props": {"sizingH": "fill"}}))?;
            }
            Ok(id)
        }
        Item::Instance(i) => {
            let r = s.execute("instance.create", &json!({"component": i.component.0, "parent": parent_v}))?;
            let id = created(&r, cmd)?;
            let mut props = json!({"name": i.name});
            if i.fill_width && in_layout {
                props["sizingH"] = json!("fill");
            }
            if !i.props.is_empty() {
                props["componentProperties"] = Value::Object(i.props.clone());
            }
            s.execute("node.set", &json!({"id": id.0, "props": props}))?;
            for (layer, values) in &i.layers {
                let inner = find_named(s.doc()?, id, layer).ok_or_else(|| bad(cmd, format!("layer `{layer}` of `{}` was not found", i.label)))?;
                s.execute("node.set", &json!({"id": inner.0, "props": {"componentProperties": values}}))?;
            }
            if let Some(content) = &i.slot {
                let slot = slots_of(s.doc()?, id).first().map(|(l, _)| *l).ok_or_else(|| bad(cmd, format!("`{}` has no slot", i.label)))?;
                // The slot's own placeholder content gives way to the proposal's.
                let old: Vec<u64> = s.doc()?.node(slot).map(|n| n.children.iter().map(|c| c.0).collect()).unwrap_or_default();
                if !old.is_empty() {
                    s.execute("edit.delete", &json!({"ids": old}))?;
                }
                for c in content {
                    draw(s, c, Some(slot), cmd, docked)?;
                }
            }
            if i.dock_bottom {
                docked.push(id);
            }
            Ok(id)
        }
    }
}

/// `explore.catalog`.
fn catalog(s: &mut Session, p: &Value) -> Result<Value> {
    let doc = s.doc()?;
    let query = p.get("query").and_then(Value::as_str).map(str::to_lowercase);
    let hit = |name: &str| query.as_ref().is_none_or(|q| name.to_lowercase().contains(q.as_str()));
    let mut components = Vec::new();
    for n in doc.nodes() {
        if components.len() >= MAX_CATALOG {
            break;
        }
        let in_set = n.parent.and_then(|p| doc.node(p)).is_some_and(|p| matches!(p.kind, NodeKind::ComponentSet { .. }));
        let (first, variants) = match &n.kind {
            NodeKind::ComponentSet { .. } => {
                let v: Map<String, Value> = variant_options(doc, n.id).into_iter().map(|(k, vals)| (short(&k), json!(vals))).collect();
                (n.children.first().copied(), v)
            }
            NodeKind::Component { .. } if !in_set => (Some(n.id), Map::new()),
            _ => continue,
        };
        // Hidden building blocks (".Nested / …", "_Base") are not offered.
        if n.name.starts_with(['.', '_']) || !hit(&n.name) {
            continue;
        }
        let mut props = Map::new();
        let mut slots = 0;
        if let Some(NodeKind::Component { props: defs, .. }) = first.and_then(|f| doc.node(f)).map(|f| &f.kind) {
            for (k, d) in defs {
                // By type, with its default: one short name may be both a text and a boolean.
                let (kind, default) = match d {
                    PropDef::Boolean { default } => ("boolean", json!(default)),
                    PropDef::Text { default } => ("text", json!(default)),
                    PropDef::Slot { .. } => {
                        slots += 1;
                        continue;
                    }
                    _ => continue,
                };
                if let Value::Object(o) = props.entry(short(k)).or_insert_with(|| json!({})) {
                    o.insert(kind.to_string(), default);
                }
            }
        }
        components.push(json!({"name": n.name.trim(), "variants": variants, "props": props, "slots": slots}));
    }
    let collections: BTreeMap<&str, &str> = doc.variables.collections.iter().map(|c| (c.id.as_str(), c.name.as_str())).collect();
    let variables: Vec<String> = doc
        .variables
        .variables
        .iter()
        .filter(|v| hit(&v.name))
        .take(MAX_CATALOG)
        .map(|v| format!("{}/{}", collections.get(v.collection.as_str()).copied().unwrap_or(""), v.name))
        .collect();
    Ok(json!({"components": components, "variables": variables}))
}
