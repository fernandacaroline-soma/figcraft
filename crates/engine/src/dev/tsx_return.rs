//! The way back from `export.tsx` (Eureka Design, task 7): the calls to code components it writes
//! (`<Button variant="Secundário" />`, imported with `import { Button } from "<pkg>";`) read as
//! instances of the design system's components, through the code connect links in the document
//! (the code map, `codeConnect.import`).
//!
//! - `codeConnect.readTsx` reads a TSX file, read only: each call to a code component, the
//!   component of the document linked to it (by the code component's name, and the import path when
//!   the file imports it) and its properties, read back by inverting the link's template
//!   (`{prop}` placeholders, `{prop ? "a" : "b"}` picks, also as a bare attribute). An attribute
//!   the template doesn't pass is read by the property's name in code (`icone` for "Ícone",
//!   `texto` for "Texto#7:26"); one that names no property is a note. What the instance changed
//!   inside it (`export.tsx`'s `components[].inside`, else the header's notes) goes with each call.
//! - `codeConnect.placeCall` places one call as an instance of that component (variant and
//!   properties set, the changes inside it applied), as one undo step.
//!
//! A call with no component linked to it is a note, never a component made up: the layers are
//! not drawn either (the code component is the code team's). Only React, the language of
//! `export.tsx`.

use figcraft_doc::{Document, NodeId, NodeKind};
use serde_json::{Map, Value, json};

use crate::cmd::CommandSpec;
use crate::code_connect::{Piece, owner, pieces};
use crate::codegen::CodeLang;
use crate::component_codegen::{Component, Param, ParamKind, code_key, component_type, describe, main_of};
use crate::params::{get, id_of, num, opt_id};
use crate::{EngineError, Result, Session, bad, spec};

/// Largest TSX read (bytes).
const MAX_CODE: usize = 4 * 1024 * 1024;
/// Most calls one file reports.
const MAX_CALLS: usize = 2_000;
/// Most attributes read on one call.
const MAX_ATTRS: usize = 200;
/// Most changes inside one call applied.
const MAX_INSIDE: usize = 50;
/// Most layers walked inside one instance.
const MAX_WALK: usize = 20_000;
/// Most notes one result reports.
const MAX_NOTES: usize = 500;
/// The longest text `export.tsx` lists among the changes inside a call (it cuts longer ones).
const INSIDE_TEXT: usize = 80;
/// Template placeholders while a template is read as JSX (private-use characters).
const OPEN: char = '\u{E000}';
const CLOSE: char = '\u{E001}';

// ------------------------------------------------------------------ JSX

/// One JSX element as written: its tag, its attributes (raw value with its delimiters; `None` for
/// a bare attribute), its content (`None` when self-closing) and where it ends.
struct Element<'a> {
    tag: &'a str,
    attrs: Vec<(&'a str, Option<&'a str>)>,
    children: Option<&'a str>,
    end: usize,
}

/// The end (after the closing brace) of the `{…}` starting at `at`, skipping strings.
fn balanced(s: &str, at: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut quote: Option<char> = None;
    let mut escaped = false;
    for (i, c) in s.get(at..)?.char_indices() {
        if let Some(q) = quote {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == q {
                quote = None;
            }
            continue;
        }
        match c {
            '"' | '\'' | '`' => quote = Some(c),
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(at + i + 1);
                }
            }
            _ => {}
        }
    }
    None
}

fn skip_ws(s: &str, mut at: usize) -> usize {
    while let Some(c) = s.get(at..).and_then(|r| r.chars().next()).filter(|c| c.is_whitespace()) {
        at += c.len_utf8();
    }
    at
}

/// The element starting with `<` at `at` (`None` when it isn't one, or never closes).
fn element(s: &str, at: usize) -> Option<Element<'_>> {
    let body = s.get(at..)?.strip_prefix('<')?;
    let tag_len = body.find(|c: char| !(c.is_ascii_alphanumeric() || "_$.".contains(c))).unwrap_or(body.len());
    let tag = body.get(..tag_len).filter(|t| !t.is_empty())?;
    let mut pos = at + 1 + tag_len;
    let mut attrs = Vec::new();
    loop {
        pos = skip_ws(s, pos);
        let rest = s.get(pos..)?;
        if rest.starts_with("/>") {
            return Some(Element { tag, attrs, children: None, end: pos + 2 });
        }
        if rest.starts_with('>') {
            let close = format!("</{tag}>");
            let from = pos + 1;
            let at_close = from + s.get(from..)?.find(&close)?;
            return Some(Element { tag, attrs, children: s.get(from..at_close), end: at_close + close.len() });
        }
        if attrs.len() >= MAX_ATTRS {
            return None;
        }
        // A spread (`{...props}`): kept as an attribute that names nothing.
        if rest.starts_with('{') {
            let end = balanced(s, pos)?;
            attrs.push((s.get(pos..end)?, None));
            pos = end;
            continue;
        }
        let name_len = rest.find(|c: char| c.is_whitespace() || "=/>".contains(c)).unwrap_or(rest.len());
        if name_len == 0 {
            return None;
        }
        let name = rest.get(..name_len)?;
        pos = skip_ws(s, pos + name_len);
        if !s.get(pos..)?.starts_with('=') {
            attrs.push((name, None));
            continue;
        }
        pos = skip_ws(s, pos + 1);
        let v = s.get(pos..)?;
        let first = v.chars().next()?;
        let end = match first {
            '"' | '\'' => pos + 1 + v.get(1..)?.find(first)? + 1,
            '{' => balanced(s, pos)?,
            OPEN => pos + v.find(CLOSE)? + CLOSE.len_utf8(),
            _ => return None,
        };
        attrs.push((name, s.get(pos..end)));
        pos = end;
    }
}

/// A value as the call writes it.
#[derive(Debug, PartialEq)]
enum Read {
    Str(String),
    Bool(bool),
    Num(f64),
    /// `<Icon />`: an element, by its tag.
    Element(String),
    Null,
    /// Code the return doesn't read (kept as written).
    Code(String),
}

impl Read {
    /// The value as text, for comparing with a pick's strings.
    fn text(&self) -> String {
        match self {
            Read::Str(s) | Read::Code(s) | Read::Element(s) => s.clone(),
            Read::Bool(b) => b.to_string(),
            Read::Num(n) => n.to_string(),
            Read::Null => "null".into(),
        }
    }
}

/// An attribute's raw value (with its delimiters; `None`: bare, so true) as a value.
fn read_value(raw: Option<&str>) -> Read {
    let Some(raw) = raw.map(str::trim) else { return Read::Bool(true) };
    for q in ['"', '\''] {
        if let Some(inner) = raw.strip_prefix(q).and_then(|r| r.strip_suffix(q)) {
            return Read::Str(inner.to_string());
        }
    }
    match raw.strip_prefix('{').and_then(|r| r.strip_suffix('}')) {
        Some(expr) => read_expr(expr.trim()),
        None => Read::Code(raw.to_string()),
    }
}

/// A JavaScript expression as a value.
fn read_expr(e: &str) -> Read {
    if e.starts_with('"') {
        return serde_json::from_str::<String>(e).map_or_else(|_| Read::Code(e.into()), Read::Str);
    }
    if let Some(inner) = e.strip_prefix('\'').and_then(|r| r.strip_suffix('\'')).filter(|i| !i.contains(['\'', '\\'])) {
        return Read::Str(inner.into());
    }
    match e {
        "true" => return Read::Bool(true),
        "false" => return Read::Bool(false),
        "null" | "undefined" => return Read::Null,
        _ => {}
    }
    if let Ok(n) = e.parse::<f64>()
        && n.is_finite()
    {
        return Read::Num(n);
    }
    if let Some(el) = element(e, 0).filter(|el| el.end == e.len() && el.children.is_none() && el.attrs.is_empty()) {
        return Read::Element(el.tag.to_string());
    }
    Read::Code(e.into())
}

/// The content of an element as a value: one `{expression}`, else its text.
fn read_children(c: &str) -> Read {
    let t = c.trim();
    if t.starts_with('{') && balanced(t, 0) == Some(t.len()) {
        return read_value(Some(t));
    }
    Read::Str(t.to_string())
}

// ------------------------------------------------------------------ templates

/// Where a template puts a placeholder.
enum Slot<'t> {
    Value(&'t str),
    Pick {
        name: &'t str,
        yes: &'t str,
        no: &'t str,
    },
    /// Written as it is (the template's own, not a property).
    Literal,
    /// A placeholder among other text: not read back.
    Mixed,
}

/// A link's template read as one JSX element: its attributes' placeholders, the picks written as
/// bare attributes and its content's placeholder.
struct Shape<'t> {
    attrs: Vec<(String, Slot<'t>)>,
    presence: Vec<Slot<'t>>,
    children: Option<Slot<'t>>,
}

/// The placeholder of a raw value that is only a placeholder (`␀3␁`, `"␀3␁"`, `{␀3␁}`).
fn only_placeholder(raw: &str) -> Option<usize> {
    let r = raw.trim();
    let r = ['"', '\''].iter().find_map(|q| r.strip_prefix(*q).and_then(|x| x.strip_suffix(*q))).unwrap_or(r);
    let r = r.strip_prefix('{').and_then(|x| x.strip_suffix('}')).map(str::trim).unwrap_or(r);
    r.strip_prefix(OPEN)?.strip_suffix(CLOSE)?.parse().ok()
}

fn slot<'t>(ps: &[Piece<'t>], i: usize) -> Slot<'t> {
    match ps.get(i) {
        Some(Piece::Value { name, .. }) => Slot::Value(name),
        Some(Piece::Pick { name, yes, no, .. }) => Slot::Pick { name, yes, no },
        _ => Slot::Mixed,
    }
}

/// Template `t` as a [`Shape`]; `None` when it isn't one JSX element.
fn shape(t: &str) -> Option<Shape<'_>> {
    let ps = pieces(t);
    let mut skeleton = String::new();
    for (i, p) in ps.iter().enumerate() {
        match p {
            Piece::Text(x) => skeleton.push_str(x),
            _ => skeleton.push_str(&format!("{OPEN}{i}{CLOSE}")),
        }
    }
    let start = skeleton.find('<')?;
    let el = element(&skeleton, start)?;
    let mut out = Shape { attrs: Vec::new(), presence: Vec::new(), children: None };
    for (name, raw) in &el.attrs {
        if let Some(i) = only_placeholder(name) {
            out.presence.push(slot(&ps, i));
            continue;
        }
        let s = match raw {
            None => Slot::Literal,
            Some(r) => match only_placeholder(r) {
                Some(i) => slot(&ps, i),
                None if r.contains(OPEN) => Slot::Mixed,
                None => Slot::Literal,
            },
        };
        out.attrs.push((name.to_string(), s));
    }
    out.children = el.children.and_then(|c| only_placeholder(c).map(|i| slot(&ps, i)));
    Some(out)
}

// ------------------------------------------------------------------ calls

/// The parameter of `c` a name means: its property name, its name in code, or either written the
/// other way (`icone` for "Ícone", "Texto#7:26" for `texto`).
fn param<'c>(c: &'c Component, name: &str) -> Option<&'c Param> {
    let key = code_key(name);
    c.params.iter().find(|p| p.prop == name || p.ident == name).or_else(|| c.params.iter().find(|p| !key.is_empty() && code_key(&p.prop) == key))
}

/// A component of the document by its name in code: the one linked to that code component (in
/// React), else the one whose generated type name it is.
fn component_named(doc: &Document, tag: &str) -> Option<NodeId> {
    let linked = doc
        .nodes()
        .filter(|n| owner(doc, n.id) == Some(n.id))
        .find(|n| n.code_links.iter().any(|l| l.language == CodeLang::React.id() && l.name == tag))
        .and_then(|n| main_of(doc, n.id));
    linked.or_else(|| {
        doc.nodes().filter(|n| matches!(n.kind, NodeKind::Component { .. }) && component_type(doc, n.id).as_deref() == Some(tag)).map(|n| n.id).next()
    })
}

/// Value `r` for parameter `p`, as `instance.setProperties` takes it.
fn to_value(doc: &Document, p: &Param, r: &Read) -> std::result::Result<Value, String> {
    let wrong = || format!("{}: {} is not a {} value", p.prop, r.text(), p.kind.id());
    match (&p.kind, r) {
        (ParamKind::Boolean, Read::Bool(b)) => Ok(json!(b)),
        (ParamKind::Boolean, Read::Str(s)) if s == "true" || s == "false" => Ok(json!(s == "true")),
        (ParamKind::Text, Read::Str(s)) => Ok(json!(s)),
        (ParamKind::Text, Read::Num(_)) => Ok(json!(r.text())),
        (ParamKind::Variant(opts), Read::Str(s)) => {
            let key = code_key(s);
            opts.iter()
                .find(|o| *o == s)
                .or_else(|| opts.iter().find(|o| !key.is_empty() && code_key(o) == key))
                .map(|o| json!(o))
                .ok_or_else(|| format!("{}: \"{s}\" is not one of its variants ({})", p.prop, opts.join(", ")))
        }
        (ParamKind::InstanceSwap, Read::Element(tag)) => {
            component_named(doc, tag).map(|id| json!(id.0)).ok_or_else(|| format!("{}: no component of the document is {tag}", p.prop))
        }
        (ParamKind::Slot, _) => Err(format!("{}: a slot's content is not read back", p.prop)),
        _ => Err(wrong()),
    }
}

/// What a call reads as, against one linked component.
struct Reading {
    owner: NodeId,
    component: String,
    properties: Map<String, Value>,
    notes: Vec<String>,
}

/// Call `el` read against component `own` (a set or component) and its React link.
fn read_call(doc: &Document, own: NodeId, el: &Element) -> Option<Reading> {
    let link = doc.node(own)?.code_links.iter().find(|l| l.language == CodeLang::React.id())?;
    let c = main_of(doc, own).and_then(|m| describe(doc, m))?;
    let component = doc.node(own).map(|n| n.name.clone()).unwrap_or_default();
    let mut out = Reading { owner: own, component: component.clone(), properties: Map::new(), notes: Vec::new() };
    let Some(shape) = shape(&link.template) else {
        out.notes.push(format!("{}: the map's template for {component} is not one JSX element, so its values are not read", el.tag));
        return Some(out);
    };
    let set = |out: &mut Reading, p: &Param, v: std::result::Result<Value, String>| match v {
        Ok(v) => {
            out.properties.insert(p.prop.clone(), v);
        }
        Err(e) => out.notes.push(format!("<{}>: {e}", el.tag)),
    };
    let mut used: Vec<&str> = Vec::new();
    // Picks written as bare attributes: which of the two the call has.
    for s in &shape.presence {
        let Slot::Pick { name, yes, no } = s else { continue };
        let Some(p) = param(&c, name) else { continue };
        let has = |a: &str| !a.is_empty() && el.attrs.iter().any(|(n, _)| *n == a);
        let v = if has(yes) {
            used.push(yes);
            true
        } else if has(no) {
            used.push(no);
            false
        } else {
            no.is_empty()
        };
        if p.kind == ParamKind::Boolean {
            set(&mut out, p, Ok(json!(v)));
        }
    }
    for (name, raw) in &el.attrs {
        if used.contains(name) {
            continue;
        }
        let r = read_value(*raw);
        match shape.attrs.iter().find(|(n, _)| n == name).map(|(_, s)| s) {
            Some(Slot::Value(ph)) => match param(&c, ph) {
                Some(p) => set(&mut out, p, to_value(doc, p, &r)),
                None => out.notes.push(format!("<{}>: {name}: the map's placeholder {{{ph}}} is not a property of {component}", el.tag)),
            },
            Some(Slot::Pick { name: ph, yes, no }) => match param(&c, ph) {
                Some(p) if p.kind == ParamKind::Boolean && (r.text() == *yes || r.text() == *no) => set(&mut out, p, Ok(json!(r.text() == *yes))),
                _ => out.notes.push(format!("<{}>: {name}={}: the map picks it from {{{ph}}}, which can't be read back", el.tag, r.text())),
            },
            Some(Slot::Literal) => {}
            Some(Slot::Mixed) => out.notes.push(format!("<{}>: {name}: the map writes it among other text, not read back", el.tag)),
            // Not in the template: a property by its name in code, else nothing of this component.
            None => match param(&c, name) {
                Some(p) => set(&mut out, p, to_value(doc, p, &r)),
                None => out.notes.push(format!("<{}>: unknown prop {name}={}: {component} has no such property (left out)", el.tag, r.text())),
            },
        }
    }
    match (&shape.children, el.children.map(str::trim).filter(|c| !c.is_empty())) {
        (Some(Slot::Value(ph)), Some(ch)) => {
            if let Some(p) = param(&c, ph) {
                set(&mut out, p, to_value(doc, p, &read_children(ch)));
            }
        }
        (_, Some(_)) => out.notes.push(format!("<{}>: its content is not read back (the map's template doesn't place a property there)", el.tag)),
        _ => {}
    }
    Some(out)
}

/// Call `el` read against the components linked to its code component: those imported from
/// `import` first; the reading with the fewest notes. `Err`: a note, when none is linked to it.
fn resolve(doc: &Document, el: &Element, import: Option<&str>) -> std::result::Result<Reading, String> {
    let linked = |n: &figcraft_doc::Node, path: Option<&str>| {
        n.code_links.iter().any(|l| l.language == CodeLang::React.id() && l.name == el.tag && path.is_none_or(|p| l.import_path == p))
    };
    let owners: Vec<NodeId> = doc.nodes().filter(|n| owner(doc, n.id) == Some(n.id)).map(|n| n.id).collect();
    let from_import: Vec<NodeId> = owners.iter().copied().filter(|i| doc.node(*i).is_some_and(|n| linked(n, import))).collect();
    let mut notes = Vec::new();
    let cands = if from_import.is_empty() {
        let any: Vec<NodeId> = owners.iter().copied().filter(|i| doc.node(*i).is_some_and(|n| linked(n, None))).collect();
        if let (Some(path), false) = (import, any.is_empty()) {
            notes.push(format!("<{}>: the file imports it from {path}; the map links it from another package", el.tag));
        }
        any
    } else {
        from_import
    };
    if cands.is_empty() {
        return Err(format!("<{}>: no component of the document is linked to {} in the code map (left out, no layers drawn)", el.tag, el.tag));
    }
    let mut best: Option<Reading> = None;
    for id in &cands {
        if let Some(r) = read_call(doc, *id, el)
            && best.as_ref().is_none_or(|b| r.notes.len() < b.notes.len())
        {
            best = Some(r);
        }
    }
    let mut best = best.ok_or_else(|| format!("<{}>: the component linked to it can't be read", el.tag))?;
    if cands.len() > 1 {
        notes.push(format!("<{}>: {} components are linked to it; read as {}", el.tag, cands.len(), best.component));
    }
    notes.append(&mut best.notes);
    best.notes = notes;
    Ok(best)
}

/// `import { A, B as C } from "pkg";` lines of `code`: (name used, package).
fn imports(code: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for line in code.lines().map(str::trim).filter(|l| l.starts_with("import ")) {
        let Some((what, from)) = line.trim_start_matches("import ").rsplit_once(" from ") else { continue };
        let from = from.trim().trim_end_matches(';').trim().trim_matches(['"', '\'']).to_string();
        let what = what.trim();
        let names: Vec<&str> = match what.strip_prefix('{').and_then(|w| w.strip_suffix('}')) {
            Some(list) => list.split(',').map(|n| n.rsplit(" as ").next().unwrap_or(n).trim()).collect(),
            None => vec![what],
        };
        for n in names.into_iter().filter(|n| !n.is_empty()) {
            out.push((n.to_string(), from.clone()));
        }
    }
    out
}

/// The calls to components (an element whose tag starts with a capital) in `code`, outside block
/// comments: (byte offset, element). An element inside another call is that call's content.
fn calls(code: &str) -> Vec<(usize, Element<'_>)> {
    let mut out = Vec::new();
    let mut i = 0;
    while let Some(rest) = code.get(i..) {
        let Some(at) = rest.find(['<', '/']) else { break };
        let pos = i + at;
        let tail = code.get(pos..).unwrap_or_default();
        if tail.starts_with("/*") {
            i = tail.find("*/").map_or(code.len(), |e| pos + e + 2);
            continue;
        }
        if tail.starts_with('<')
            && tail.chars().nth(1).is_some_and(|c| c.is_ascii_uppercase())
            && let Some(el) = element(code, pos)
        {
            i = el.end;
            out.push((pos, el));
            if out.len() >= MAX_CALLS {
                break;
            }
            continue;
        }
        i = pos + 1;
    }
    out
}

/// The changes inside calls the header of an `export.tsx` file notes, by code component, in order.
fn header_inside(code: &str) -> Vec<(String, Vec<String>)> {
    const CALL: &str = "the call to ";
    const PASSES: &str = " passes only its own properties; changed inside it: ";
    let mut out = Vec::new();
    for line in code.lines().filter_map(|l| l.trim().strip_prefix("* Note: ")) {
        let Some(at) = line.find(CALL) else { continue };
        let Some((name, changes)) = line.get(at + CALL.len()..).and_then(|r| r.split_once(PASSES)) else { continue };
        out.push((name.to_string(), changes.split("; ").map(str::to_string).collect()));
    }
    out
}

/// The `code` (or `path`) parameter: the TSX text.
fn code_param(cmd: &str, p: &Value) -> Result<String> {
    match (get(p, "code"), get(p, "path")) {
        (Some(Value::String(t)), _) if t.len() <= MAX_CODE => Ok(t.clone()),
        (Some(Value::String(_)), _) => Err(bad(cmd, format!("code: at most {MAX_CODE} bytes"))),
        (Some(_), _) => Err(bad(cmd, "code: the TSX text")),
        (None, Some(Value::String(path))) if !path.trim().is_empty() => {
            let bytes = crate::cmd::file::read_capped(path, MAX_CODE as u64)?;
            String::from_utf8(bytes).map_err(|_| bad(cmd, "path: not UTF-8 text"))
        }
        (None, Some(_)) => Err(bad(cmd, "path: the TSX file")),
        (None, None) => Err(bad(cmd, "pass `code` or `path`")),
    }
}

fn read_tsx(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "codeConnect.readTsx";
    let code = code_param(cmd, p)?;
    // export.tsx's `components`: what each call's instance changed inside it, by code component.
    let mut given: Vec<(String, Vec<String>)> = Vec::new();
    let from_result = match get(p, "components") {
        None | Some(Value::Null) => false,
        Some(Value::Array(list)) => {
            for c in list.iter().take(MAX_CALLS) {
                let name =
                    c.get("name").and_then(Value::as_str).ok_or_else(|| bad(cmd, "components: [{name, inside}] as export.tsx returns them"))?;
                let inside = c.get("inside").and_then(Value::as_array).map(|a| a.iter().filter_map(Value::as_str).map(str::to_string).collect());
                given.push((name.to_string(), inside.unwrap_or_default()));
            }
            true
        }
        Some(_) => return Err(bad(cmd, "components: [{name, inside}] as export.tsx returns them")),
    };
    let doc = s.doc()?;
    let imported = imports(&code);
    let found = calls(&code);
    let mut notes: Vec<String> = Vec::new();
    let mut out: Vec<Value> = Vec::new();
    // The changes inside, matched to calls of the same code component in order.
    let header = if from_result { Vec::new() } else { header_inside(&code) };
    let count = |n: &str| found.iter().filter(|(_, e)| e.tag == n).count();
    for (k, (at, el)) in found.iter().enumerate() {
        let import = imported.iter().find(|(n, _)| n == el.tag).map(|(_, f)| f.as_str());
        let nth = found.iter().take(k).filter(|(_, e)| e.tag == el.tag).count();
        let inside: Vec<String> = if from_result {
            given.iter().filter(|(n, _)| n == el.tag).nth(nth).map(|(_, i)| i.clone()).unwrap_or_default()
        } else {
            let listed: Vec<&Vec<String>> = header.iter().filter(|(n, _)| n == el.tag).map(|(_, i)| i).collect();
            match (listed.len(), count(el.tag)) {
                (0, _) => Vec::new(),
                (l, c) if l == c => listed.get(nth).map(|v| v.to_vec()).unwrap_or_default(),
                _ => {
                    let n =
                        format!("<{}>: the header lists changes inside some of its calls; pass `components` (export.tsx's) to tell which", el.tag);
                    if !notes.contains(&n) {
                        notes.push(n);
                    }
                    Vec::new()
                }
            }
        };
        let line = code.get(..*at).map_or(1, |b| b.matches('\n').count() + 1);
        let text = code.get(*at..el.end).unwrap_or_default();
        let row = match resolve(doc, el, import) {
            Ok(r) => {
                notes.extend(r.notes.iter().cloned());
                json!({"index": k, "line": line, "name": el.tag, "call": text, "importPath": import, "component": r.component, "componentId": r.owner.0, "properties": r.properties, "inside": inside, "notes": r.notes})
            }
            Err(note) => {
                notes.push(note.clone());
                json!({"index": k, "line": line, "name": el.tag, "call": text, "importPath": import, "component": Value::Null, "componentId": Value::Null, "properties": {}, "inside": inside, "notes": [note]})
            }
        };
        out.push(row);
    }
    let read = out.iter().filter(|c| !c["componentId"].is_null()).count();
    let counts = json!({"calls": out.len(), "read": read, "notInMap": out.len() - read, "notes": notes.len()});
    notes.truncate(MAX_NOTES);
    let imports: Vec<Value> = imported.iter().map(|(n, f)| json!({"name": n, "from": f})).collect();
    Ok(json!({"imports": imports, "calls": out, "notes": notes, "counts": counts}))
}

// ------------------------------------------------------------------ placing a call

/// One change `export.tsx` lists inside a call.
enum Change {
    Text(String),
    Visible(bool),
    Props(Vec<(String, String)>),
}

/// `layer: "text"`, `layer: shown|hidden` or `layer: Prop=value, Prop2=On`.
fn parse_change(s: &str) -> Option<(&str, Change)> {
    let (layer, what) = s.split_once(": ")?;
    let change = match what {
        "shown" => Change::Visible(true),
        "hidden" => Change::Visible(false),
        _ => match what.strip_prefix('"').and_then(|w| w.strip_suffix('"')) {
            Some(t) => Change::Text(t.to_string()),
            None => Change::Props(what.split(", ").map(|kv| kv.split_once('=').map(|(k, v)| (k.to_string(), v.to_string()))).collect::<Option<_>>()?),
        },
    };
    Some((layer, change))
}

/// The layers inside instance `inst`, in tree order.
fn descendants(doc: &Document, inst: NodeId) -> Vec<NodeId> {
    let mut out = Vec::new();
    let mut stack: Vec<NodeId> = doc.node(inst).map(|n| n.children.iter().rev().copied().collect()).unwrap_or_default();
    while let Some(id) = stack.pop() {
        if out.len() >= MAX_WALK {
            break;
        }
        out.push(id);
        if let Some(n) = doc.node(id) {
            stack.extend(n.children.iter().rev().copied());
        }
    }
    out
}

/// Applies the changes `inside` lists to instance `inst`: each to the first layer of that name (or
/// code name) and kind not changed yet. Returns notes for what couldn't be applied.
fn apply_inside(s: &mut Session, inst: NodeId, inside: &[String]) -> Result<Vec<String>> {
    let mut notes = Vec::new();
    let mut used: Vec<NodeId> = Vec::new();
    for item in inside.iter().take(MAX_INSIDE) {
        let Some((layer, change)) = parse_change(item) else {
            notes.push(format!("inside: \"{item}\" not read"));
            continue;
        };
        let doc = s.doc()?;
        let key = code_key(layer);
        let fits = |id: &NodeId| {
            doc.node(*id).is_some_and(|n| {
                (n.name == layer || (!key.is_empty() && code_key(&n.name) == key))
                    && match change {
                        Change::Text(_) => matches!(n.kind, NodeKind::Text(_)),
                        Change::Props(_) => matches!(n.kind, NodeKind::Instance { .. }),
                        Change::Visible(_) => true,
                    }
            })
        };
        let all = descendants(doc, inst);
        let Some(target) = all.iter().find(|i| !used.contains(i) && fits(i)).or_else(|| all.iter().find(|i| fits(i))).copied() else {
            notes.push(format!("inside: no layer {layer} for \"{item}\""));
            continue;
        };
        used.push(target);
        let r = match &change {
            Change::Text(t) => {
                if t.chars().count() >= INSIDE_TEXT {
                    notes.push(format!("inside: {layer}'s text may be cut (export.tsx lists {INSIDE_TEXT} characters)"));
                }
                s.execute("text.set", &json!({"id": target.0, "characters": t}))
            }
            Change::Visible(v) => s.execute("node.set", &json!({"id": target.0, "props": {"visible": v}})),
            Change::Props(kv) => {
                let c = main_of(doc, target).and_then(|m| describe(doc, m));
                let mut props = Map::new();
                for (k, v) in kv {
                    let p = c.as_ref().and_then(|c| param(c, k));
                    let name = p.map_or_else(|| k.clone(), |p| p.prop.clone());
                    let value = match (p.map(|p| &p.kind), v.as_str()) {
                        (Some(ParamKind::Boolean), "On" | "true") => json!(true),
                        (Some(ParamKind::Boolean), "Off" | "false") => json!(false),
                        (Some(ParamKind::Variant(_) | ParamKind::Text), _) => json!(v),
                        _ => serde_json::from_str::<Value>(v).unwrap_or_else(|_| json!(v)),
                    };
                    props.insert(name, value);
                }
                s.execute("instance.setProperties", &json!({"ids": [target.0], "properties": props}))
            }
        };
        if let Err(e) = r {
            notes.push(format!("inside: \"{item}\" not applied: {e}"));
        }
    }
    Ok(notes)
}

fn place_call(s: &mut Session, p: &Value) -> Result<Value> {
    let cmd = "codeConnect.placeCall";
    let call = match get(p, "call") {
        Some(Value::String(t)) if t.len() <= MAX_CODE => t.clone(),
        _ => return Err(bad(cmd, "call: one call to a code component, e.g. <Button variant=\"Secundário\" />")),
    };
    let import = match get(p, "importPath") {
        None | Some(Value::Null) => None,
        Some(Value::String(t)) => Some(t.clone()),
        Some(_) => return Err(bad(cmd, "importPath: the package the file imports it from")),
    };
    let inside: Vec<String> = match get(p, "inside") {
        None | Some(Value::Null) => Vec::new(),
        Some(Value::Array(a)) if a.iter().all(Value::is_string) => a.iter().filter_map(Value::as_str).map(str::to_string).collect(),
        Some(_) => return Err(bad(cmd, "inside: [text] as export.tsx's components[].inside lists them")),
    };
    let name = match get(p, "name") {
        None | Some(Value::Null) => None,
        Some(Value::String(t)) => Some(t.clone()),
        Some(_) => return Err(bad(cmd, "name: the layer's name")),
    };
    let parent = opt_id(cmd, p, "parent")?;
    let (x, y) = (num(p, "x").filter(|v| v.is_finite()).unwrap_or(0.0), num(p, "y").filter(|v| v.is_finite()).unwrap_or(0.0));
    let start = call.find('<').ok_or_else(|| bad(cmd, "call: no element in it"))?;
    let doc = s.doc()?;
    let el = element(&call, start).ok_or_else(|| bad(cmd, "call: not one JSX element"))?;
    if !el.tag.starts_with(|c: char| c.is_ascii_uppercase()) {
        return Err(bad(cmd, "call: a code component's tag starts with a capital"));
    }
    let reading = match resolve(doc, &el, import.as_deref()) {
        Ok(r) => r,
        // Nothing invented: no instance, no layers, a note.
        Err(note) => return Ok(json!({"id": Value::Null, "component": Value::Null, "properties": {}, "notes": [note]})),
    };
    let Reading { owner: own, component, properties, mut notes } = reading;
    s.grouped("Place code component", |s| {
        let mut create = json!({"component": own.0, "x": x, "y": y});
        if let Some(parent) = parent {
            create["parent"] = json!(parent.0);
        }
        let made = s.execute("instance.create", &create)?;
        let inst = id_of(&made["id"]).ok_or_else(|| EngineError::Other("the instance wasn't made".into()))?;
        if !properties.is_empty() && s.execute("instance.setProperties", &json!({"ids": [inst.0], "properties": properties})).is_err() {
            // One at a time, so one wrong value doesn't lose the others.
            for (k, v) in &properties {
                if let Err(e) = s.execute("instance.setProperties", &json!({"ids": [inst.0], "properties": {k.as_str(): v}})) {
                    notes.push(format!("<{}>: {k} not set: {e}", el.tag));
                }
            }
        }
        notes.extend(apply_inside(s, inst, &inside)?);
        if let Some(name) = &name {
            s.execute("node.set", &json!({"id": inst.0, "props": {"name": name}}))?;
        }
        notes.truncate(MAX_NOTES);
        Ok(json!({"id": inst.0, "component": component, "properties": properties, "notes": notes}))
    })
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        spec!(
            "codeConnect.readTsx",
            "Read code component calls",
            "{code (TSX text, as export.tsx writes it) | path (its file), components? (export.tsx's components: [{name, inside}], for what each call's instance changed inside it; default: the header's notes)}  read only: each call to a code component (<Button variant=\"Secundário\" />) against the components of the document linked to it in React (codeConnect, e.g. from the code map), its properties read back by inverting the link's template ({prop}, {prop ? \"a\" : \"b\"}); an attribute the template doesn't pass is read by the property's name in code (icone for \"Ícone\"); a call no component is linked to, or a prop no property is, is a note → {imports: [{name, from}], calls: [{index, line, name, call, importPath, component, componentId (null: not in the map), properties {prop: value}, inside: [text], notes}], notes, counts: {calls, read, notInMap, notes}}",
            false,
            read_tsx
        ),
        spec!(
            "codeConnect.placeCall",
            "Place code component call",
            "{call (one call, e.g. <Button variant=\"Secundário\" />), importPath? (the package the file imports it from), inside? ([text]: export.tsx's components[].inside, reapplied to the layers inside), parent?, x?, y?, name? (the layer's name)}  the call as an instance of the component of the document linked to it (codeConnect, React), its variant and properties read from the link's template; a call no component is linked to places nothing (a note, never a made-up component); one undo step → {id (null: nothing placed), component, properties, notes}",
            // Not undoable as a command: it groups its steps into one undo step itself.
            false,
            place_call
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jsx_values_and_templates_read_back() {
        let code = r#"<Snackbar texto={"Oi \"você\""} icone={<Icon />} ativo n={2} />"#;
        let el = element(code, 0).unwrap();
        assert_eq!(el.tag, "Snackbar");
        let vals: Vec<Read> = el.attrs.iter().map(|(_, r)| read_value(*r)).collect();
        assert_eq!(vals, vec![Read::Str("Oi \"você\"".into()), Read::Element("Icon".into()), Read::Bool(true), Read::Num(2.0)]);
        let sh = shape(r#"<Button variant={Hierarquia} size="{Grande ? "lg" : "sm"}" {Ativo ? "disabled" : ""}>{Texto}</Button>"#).unwrap();
        assert!(matches!(sh.attrs[0], (ref n, Slot::Value("Hierarquia")) if n == "variant"));
        assert!(matches!(sh.attrs[1].1, Slot::Pick { name: "Grande", yes: "lg", no: "sm" }));
        assert!(matches!(sh.presence[0], Slot::Pick { name: "Ativo", .. }));
        assert!(matches!(sh.children, Some(Slot::Value("Texto"))));
        assert_eq!(
            imports("import { Button, Icon as I } from \"@acme/ds\";\nimport \"./a.css\";"),
            vec![("Button".to_string(), "@acme/ds".to_string()), ("I".to_string(), "@acme/ds".to_string())]
        );
        // Junk never panics.
        for junk in ["<", "<A", "<A b=", "<A b={", "<A b=\"x", "<A>", "<A {...}", "<\u{E000}", "{{{", "<A b={'}'} />"] {
            let _ = element(junk, 0);
            let _ = calls(junk);
            let _ = shape(junk);
        }
    }
}
