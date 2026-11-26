// Eureka, Frente M: READ-ONLY snapshot of DS Apps components (run with the Figma MCP `use_figma`).
//
// Serialises the component sets in ROOTS (and every main component their instances use, restricted
// to the variants used) into the shape of the REST API `GET /v1/files/:key/nodes` response, plus
// the variables they bind (and alias targets), as a local-variables export under `meta`. FigCraft
// opens that JSON with `file.import` (the REST importer), keeping instances, properties, slots and
// variable bindings. Nothing is written to the Figma file.
//
// The Plugin API cannot export REST JSON (`exportAsync({format: 'JSON_REST_V1'})` is refused in
// this context) and a `use_figma` result is capped at 48 KB, so the script packs the JSON (LZW over
// its UTF-8 bytes, base64; about a third of the size, `pack` below) and returns it in chunks: run
// it once per CHUNK (0, 1, ...) until `chunks`, then join the `data` strings in order
// (`python3 ds_components_snapshot.py assemble OUT.json CHUNK0.json CHUNK1.json ...`). The result is a
// cache OUTSIDE the repository (`eureka-cache/`), never committed: it is the DS team's content.
//
// ROOTS: component set id -> null (every variant) or a filter on the variants kept.
const CHUNK = 0, SIZE = 46000;
const ROOTS = {
  // Button: every type, with its disabled and loading states (size Grande, no icon, not destructive or inverse).
  '145:1171': v => ['Tamanho=Grande','Ícone=Off','Destrutivo=Off','Inverso=Off'].every(t => v.name.includes(t)),
  '150:586': null,        // Choice Tag (Chips)
  '4608:14646': null,     // Selector Size (Selector Button)
  '3173:29745': null,     // Bottom Sheet
  // Frente P: a fixed action area, feedback and messages, headings. (Price Bar and Complete o look
  // pull the Bottom Bar and the Product Card with them: about 570 KB, left out.)
  '3169:29380': null,     // Action Button
  '3169:23397': null,     // Snackbar
  '11651:22206': null,    // Tipbar
  '362:10674': null,      // Section Heading
  // Frente T: the parts of DS Apps' official PDP template (page "Página de Produto"), so a proposal's
  // base screen is the template's, not drawn by hand. The gallery without "Complete o look" (that
  // variant brings product cards); Image only as a product photo.
  '14789:39750': v => v.name.includes('Complete o look=False'),  // .Nested / PDP-product-gallery
  '11247:54973': null,    // .Nested / pdp-main-info
  '11240:78010': null,    // .Nested / size-selection-PDP
  '11247:37602': v => v.name.includes('Sale=False'),  // .Nested / fixed-component
  '3276:26477': v => v.name.includes('Tipo=Produto'),  // Image
};
const want = new Map(); for (const [id, f] of Object.entries(ROOTS)) want.set(id, f);
const R = x => typeof x === 'number' ? Math.round(x * 1000) / 1000 : x;
const varIds = new Set();
const noteVars = bv => { const walk = v => { if (Array.isArray(v)) v.forEach(walk); else if (v && typeof v === 'object') { if (v.type === 'VARIABLE_ALIAS' && v.id) varIds.add(v.id); else Object.values(v).forEach(walk); } }; walk(bv); };
const deps = [];
const col = c => ({ r: R(c.r), g: R(c.g), b: R(c.b), a: R(c.a ?? 1) });
function paint(p) {
  const o = { type: p.type }; if (p.visible === false) o.visible = false; if ((p.opacity ?? 1) !== 1) o.opacity = R(p.opacity); if (p.blendMode && p.blendMode !== 'NORMAL') o.blendMode = p.blendMode;
  if (p.type === 'SOLID') o.color = col({ ...p.color, a: 1 });
  if (p.type.startsWith('GRADIENT')) { o.gradientStops = p.gradientStops.map(s => ({ position: R(s.position), color: col(s.color) })); const t = p.gradientTransform; const det = t[0][0]*t[1][1]-t[0][1]*t[1][0] || 1; const inv = (x, y) => { const X = x-t[0][2], Y = y-t[1][2]; return { x: R((t[1][1]*X-t[0][1]*Y)/det), y: R((-t[1][0]*X+t[0][0]*Y)/det) }; }; o.gradientHandlePositions = [inv(0,0.5), inv(1,0.5), inv(0,1)]; }
  if (p.type === 'IMAGE') { o.imageRef = p.imageHash; o.scaleMode = p.scaleMode; }
  if (p.boundVariables && Object.keys(p.boundVariables).length) { o.boundVariables = p.boundVariables; noteVars(p.boundVariables); }
  return o;
}
const has = (n, k) => { try { return k in n && n[k] !== figma.mixed && n[k] !== undefined && n[k] !== null; } catch (e) { return false; } };
const DEF = { strokeAlign: 'INSIDE', strokeCap: 'NONE', strokeJoin: 'MITER', cornerRadius: 0, cornerSmoothing: 0, clipsContent: false, layoutAlign: 'INHERIT', layoutGrow: 0, layoutPositioning: 'AUTO', itemReverseZIndex: false, strokesIncludedInLayout: false, isMask: false, layoutWrap: 'NO_WRAP', counterAxisSpacing: 0 };
const AL = ['primaryAxisSizingMode','counterAxisSizingMode','primaryAxisAlignItems','counterAxisAlignItems','paddingLeft','paddingRight','paddingTop','paddingBottom','itemSpacing','counterAxisSpacing','layoutWrap','itemReverseZIndex','strokesIncludedInLayout'];
async function ser(n) {
  const o = { id: n.id, name: n.name, type: n.type };
  if (n.visible === false) o.visible = false;
  if (has(n, 'opacity') && n.opacity !== 1) o.opacity = R(n.opacity);
  if (has(n, 'relativeTransform')) o.relativeTransform = n.relativeTransform.map(r => r.map(R));
  if (has(n, 'width')) o.size = { x: R(n.width), y: R(n.height) };
  if (has(n, 'fills') && n.fills.length) o.fills = n.fills.map(paint);
  if (has(n, 'strokes') && n.strokes.length) o.strokes = n.strokes.map(paint);
  const keys = ['strokeAlign','strokeCap','strokeJoin','cornerRadius','cornerSmoothing','clipsContent','layoutMode','layoutAlign','layoutGrow','layoutPositioning','layoutSizingHorizontal','layoutSizingVertical','minWidth','maxWidth','minHeight','maxHeight','booleanOperation','isMask'];
  if (o.strokes) keys.push('strokeWeight');
  const auto = has(n, 'layoutMode') && n.layoutMode !== 'NONE' && n.type !== 'TEXT';
  if (auto) keys.push(...AL);
  for (const k of keys) if (has(n, k) && DEF[k] !== n[k]) o[k] = R(n[k]);
  if (!auto) delete o.layoutMode;
  if ('topLeftRadius' in n && n.cornerRadius === figma.mixed) o.rectangleCornerRadii = [n.topLeftRadius, n.topRightRadius, n.bottomRightRadius, n.bottomLeftRadius];
  if (o.strokes && 'strokeTopWeight' in n && n.strokeWeight === figma.mixed) o.individualStrokeWeights = { top: n.strokeTopWeight, right: n.strokeRightWeight, bottom: n.strokeBottomWeight, left: n.strokeLeftWeight };
  if (has(n, 'dashPattern') && n.dashPattern.length) o.strokeDashes = n.dashPattern;
  if (has(n, 'constraints')) { const h = n.constraints.horizontal, v = n.constraints.vertical; if (h !== 'MIN' || v !== 'MIN') o.constraints = { horizontal: h === 'MIN' ? 'LEFT' : h === 'MAX' ? 'RIGHT' : h, vertical: v === 'MIN' ? 'TOP' : v === 'MAX' ? 'BOTTOM' : v }; }
  if (has(n, 'effects') && n.effects.length) o.effects = n.effects.map(e => { const x = { ...e }; if (x.color) x.color = col(x.color); if (x.boundVariables) noteVars(x.boundVariables); return x; });
  if (has(n, 'boundVariables') && Object.keys(n.boundVariables).length) { o.boundVariables = n.boundVariables; noteVars(n.boundVariables); }
  if (['VECTOR','STAR','POLYGON','BOOLEAN_OPERATION','LINE'].includes(n.type)) { try { o.fillGeometry = n.fillGeometry.map(g => ({ path: g.data, windingRule: g.windingRule })); if (o.strokes) o.strokeGeometry = n.strokeGeometry.map(g => ({ path: g.data, windingRule: g.windingRule })); } catch (e) { } }
  if (n.type === 'TEXT') {
    o.characters = n.characters;
    const seg = n.getStyledTextSegments(['fontName','fontSize','fontWeight','letterSpacing','lineHeight','textCase','textDecoration'])[0] || {};
    const st = { fontFamily: seg.fontName?.family, fontStyle: seg.fontName?.style, fontWeight: seg.fontWeight, fontSize: seg.fontSize, textAlignHorizontal: n.textAlignHorizontal, textAlignVertical: n.textAlignVertical, textAutoResize: n.textAutoResize };
    if (seg.letterSpacing && seg.letterSpacing.value) st.letterSpacing = R(seg.letterSpacing.unit === 'PIXELS' ? seg.letterSpacing.value : seg.letterSpacing.value * seg.fontSize / 100);
    if (seg.lineHeight?.unit === 'PIXELS') { st.lineHeightPx = R(seg.lineHeight.value); st.lineHeightUnit = 'PIXELS'; }
    else if (seg.lineHeight?.unit === 'PERCENT') { st.lineHeightPercentFontSize = R(seg.lineHeight.value); st.lineHeightPx = R(seg.lineHeight.value * seg.fontSize / 100); st.lineHeightUnit = 'FONT_SIZE_%'; }
    else st.lineHeightUnit = 'INTRINSIC_%';
    if (seg.textCase && seg.textCase !== 'ORIGINAL') st.textCase = seg.textCase;
    if (seg.textDecoration && seg.textDecoration !== 'NONE') st.textDecoration = seg.textDecoration;
    o.style = st;
  }
  if (n.type === 'COMPONENT_SET' || (n.type === 'COMPONENT' && n.parent?.type !== 'COMPONENT_SET')) o.componentPropertyDefinitions = n.componentPropertyDefinitions;
  if (has(n, 'componentPropertyReferences') && Object.keys(n.componentPropertyReferences).length) o.componentPropertyReferences = n.componentPropertyReferences;
  if (n.type === 'INSTANCE') { const mc = await n.getMainComponentAsync(); if (mc) { o.componentId = mc.id; deps.push(mc); } o.componentProperties = n.componentProperties; noteVars(n.componentProperties); }
  if ('children' in n) {
    let kids = n.children; const f = want.get(n.id), only = want.get('only:' + n.id);
    if (n.type === 'COMPONENT_SET' && (typeof f === 'function' || only)) kids = kids.filter(k => (typeof f === 'function' && f(k)) || (only && only.has(k.id)));
    o.children = []; for (const c of kids) o.children.push(await ser(c));
  }
  return o;
}
// LZW over UTF-8 bytes; the i-th code takes bitlen(256 + i) bits (the dictionary size then); 6 bits per base64 char.
function pack(s) {
  const b = []; for (const ch of s) { let c = ch.codePointAt(0);
    if (c < 0x80) b.push(c); else if (c < 0x800) b.push(0xc0 | c >> 6, 0x80 | c & 63);
    else if (c < 0x10000) b.push(0xe0 | c >> 12, 0x80 | c >> 6 & 63, 0x80 | c & 63);
    else b.push(0xf0 | c >> 18, 0x80 | c >> 12 & 63, 0x80 | c >> 6 & 63, 0x80 | c & 63); }
  const A = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/', LIM = 1 << 20;
  const dict = new Map(); let next = 256, w = -1, acc = 0, nb = 0, r = '';
  const emit = x => { const width = Math.min(next, LIM).toString(2).length; for (let i = width - 1; i >= 0; i--) { acc = acc << 1 | (x >> i & 1); if (++nb === 6) { r += A[acc]; acc = 0; nb = 0; } } };
  for (const c of b) { if (w < 0) { w = c; continue; } const k = w * 256 + c; const hit = dict.get(k);
    if (hit !== undefined) w = hit; else { emit(w); if (next < LIM) dict.set(k, next); next++; w = c; } }
  if (w >= 0) emit(w);
  if (nb) r += A[acc << (6 - nb)];
  return r;
}
const nodes = {}; const done = new Set();
async function addRoot(node) { if (done.has(node.id)) return; done.add(node.id); nodes[node.id] = { document: await ser(node) }; }
for (const id of Object.keys(ROOTS)) await addRoot(await figma.getNodeByIdAsync(id));
let guard = 0;
while (deps.length && guard++ < 400) { const mc = deps.shift(); const set = mc.parent?.type === 'COMPONENT_SET' ? mc.parent : null; const rid = set ? set.id : mc.id; if (ROOTS[rid] === null) continue;
  if (set) { const k = 'only:' + set.id; const s = want.get(k) || new Set(); if (s.has(mc.id) && done.has(set.id)) continue; s.add(mc.id); want.set(k, s); done.delete(set.id); await addRoot(set); } else await addRoot(mc); }
const exported = new Set(); const walkIds = d => { exported.add(d.id); (d.children || []).forEach(walkIds); }; Object.values(nodes).forEach(e => walkIds(e.document));
const components = {}, componentSets = {};
for (const id of exported) { const n = await figma.getNodeByIdAsync(id); if (n.type === 'COMPONENT') components[id] = { key: n.key, name: n.name, componentSetId: n.parent?.type === 'COMPONENT_SET' ? n.parent.id : undefined }; if (n.type === 'COMPONENT_SET') componentSets[id] = { key: n.key, name: n.name }; }
const variables = {}, collections = {}; const q = [...varIds]; let vg = 0;
while (q.length && vg++ < 2000) { const id = q.shift(); if (variables[id]) continue; const v = await figma.variables.getVariableByIdAsync(id); if (!v) continue;
  variables[id] = { id: v.id, name: v.name, variableCollectionId: v.variableCollectionId, resolvedType: v.resolvedType, valuesByMode: v.valuesByMode, scopes: v.scopes, remote: v.remote };
  for (const val of Object.values(v.valuesByMode)) if (val?.type === 'VARIABLE_ALIAS') q.push(val.id);
  if (!collections[v.variableCollectionId]) { const c = await figma.variables.getVariableCollectionByIdAsync(v.variableCollectionId); if (c) collections[c.id] = { id: c.id, name: c.name, modes: c.modes, defaultModeId: c.defaultModeId, remote: c.remote, variableIds: [] }; }
  collections[v.variableCollectionId]?.variableIds.push(v.id); }
for (const e of Object.values(nodes)) { e.components = {}; e.componentSets = {}; e.styles = {}; }
const s = JSON.stringify({ name: figma.root.name, source: 'U3pOCU4CEBgYKHGVR1F3sK', nodes, components, componentSets, meta: { variableCollections: collections, variables } });
const z = pack(s);
return { packed: true, total: z.length, chunks: Math.ceil(z.length / SIZE), chunk: CHUNK, data: z.slice(CHUNK * SIZE, (CHUNK + 1) * SIZE) };
