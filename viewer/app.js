// projectmd Gantt viewer: view-only. See README.md.
// Cache-busting: bump v in index.html (the only place) and every module/wasm fetch follows.
const V = new URL(import.meta.url).search;
const { loadPmd, parseProject, boardOf } = await import('./pmd.js' + V);
const { renderMarkdown } = await import('./md.js' + V);

const RH = 28;            // row height (px), must match --rh
const HH = 22;            // header tier height
const DAY = 86400000;
const $ = (id) => document.getElementById(id);
const els = {};
['ptitle', 'pmeta', 'crumbs', 'openform', 'rootinput', 'wbs', 'zout', 'zin', 'zfit', 'ztoday', 'zoomseg', 'banner', 'main', 'left', 'lbody', 'lspacer',
  'split', 'rhead', 'rhinner', 'rbody', 'canvas', 'gridlines', 'today', 'trows', 'deps', 'detail', 'tip', 'mdview', 'mdtitle', 'mdraw', 'mdopen', 'mdclose', 'mdbody', 'tsl', 'tplay', 'tdate', 'treset',
  'lhead', 'attn', 'fstate', 'qsearch', 'ownerf', 'hidedone', 'msonly', 'copylink', 'printbtn', 'phTitle', 'phMeta', 'levelseg', 'viewseg', 'board'].forEach((i) => (els[i] = $(i)));

const esc = (s) => String(s ?? '').replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

// ---------- dates ----------
const dayOf = (y, m, d) => Date.UTC(y, m - 1, d) / DAY;
function toDay(v) {
  if (v == null) return null;
  if (typeof v === 'string') {
    const m = /^(\d{4})-(\d\d)-(\d\d)/.exec(v);
    return m ? dayOf(+m[1], +m[2], +m[3]) : null;
  }
  if (typeof v === 'object' && v.y) return dayOf(v.y, v.m, v.d);
  return null;
}
const ymd = (day) => { const d = new Date(day * DAY); return [d.getUTCFullYear(), d.getUTCMonth() + 1, d.getUTCDate()]; };
const p2 = (n) => String(n).padStart(2, '0');
const fmt = (day) => { if (day == null) return ''; const [y, m, d] = ymd(Math.floor(day)); return `${y}-${p2(m)}-${p2(d)}`; };
const now = new Date();
// `?today=YYYY-MM-DD` views the plan as of another date (same as the CLI's --today / PMD_TODAY).
const TODAY_Q = new URLSearchParams(location.search).get('today');
const REAL_ISO = `${now.getFullYear()}-${p2(now.getMonth() + 1)}-${p2(now.getDate())}`;
// The plan date moves with the time slider (setToday); BASE_ISO is what the page opened with.
let TODAY_ISO = /^\d{4}-\d{2}-\d{2}$/.test(TODAY_Q || '') ? TODAY_Q : REAL_ISO;
const BASE_ISO = TODAY_ISO;
let TODAY = toDay(TODAY_ISO);
// "#L<line>" as loaded: the selection to restore on boot (the hash is rewritten later).
const BOOT_SEL = /^#L(\d+)$/.exec(location.hash);
const MONTHS = ['Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun', 'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec'];
const dateText = (x) => (x == null ? null : typeof x === 'string' ? x : x.text || null);
function exprSpan(x) { // DateExpr -> {s,e,range,text}
  if (x == null) return null;
  if (typeof x === 'string') { const d = toDay(x); return d == null ? null : { s: d, e: d, range: false, text: x }; }
  const s = toDay(x.start), e = toDay(x.end);
  if (s == null && e == null) return null;
  return { s: s ?? e, e: e ?? s, range: !!x.range, text: x.text };
}

// ---------- URL / key handling ----------
// A "key" is the path relative to SITE (the folder above viewer/) for files under it, else the full URL (without hash).
const SITE = new URL('..', location.href);
function keyOf(href) {
  const u = new URL(href, location.href);
  if (u.protocol !== 'http:' && u.protocol !== 'https:') throw new Error('Only http(s) URLs are allowed');
  u.hash = '';
  return u.href.startsWith(SITE.href) ? u.href.slice(SITE.href.length) : u.href;
}
const urlOf = (key) => {
  const u = new URL(key, SITE);
  if (u.protocol !== 'http:' && u.protocol !== 'https:') throw new Error('Only http(s) URLs are allowed');
  return u.href;
};
const tryKey = (t) => { try { return keyOf(urlOf(t)); } catch (e) { return null; } };
function resolveKey(baseKey, rel) {
  try { return keyOf(new URL(rel, urlOf(baseKey)).href); } catch (e) { return rel; }
}
const baseName = (k) => decodeURIComponent(String(k).split('?')[0].split('/').filter(Boolean).pop() || k);
function urlParam(k) { return encodeURIComponent(k).replace(/%2F/g, '/').replace(/%3A/g, ':'); }

// ---------- state ----------
const S = {
  rootKey: null, crumbs: [],
  src: new Map(),                // key -> source text (sent so the parser can compute contract states)
  peek: new Map(),               // key -> parsed direct link read live (null: unreadable)
  linkItem: new Map(),      // target key -> first sub-project item linking it
  idItem: new Map(),        // 'file#id' -> item
  rowById: new Map(), hover: -1, detailOpen: false,
  projects: new Map(),      // key -> {key,status,proj,root,err}
  collapsed: new Set(),     // item ids (headings/projects/tasks) the user collapsed
  expanded: new Set(),      // subproject item ids that are expanded
  selId: null,
  ppd: 2,                   // pixels per day
  rows: [], min: 0, max: 1, rawMin: null, rawMax: null,
  wbs: true, needFit: true, navToken: 0,
  deps: [],
  tRoot: null, tMin: null, tMax: null,  // time slider range (root key it was computed for, day numbers)
  chainIds: null,           // item ids of the driver chain of the selection (null: no highlight)
  q: '', owner: '', hideDone: false, ms: false, chip: null,  // filters (URL: q, owner, hide_done, ms, attn)
  cols: null,               // optional columns shown: {slack, next} (localStorage pmd.cols)
  allRows: [],              // every flattened row before filtering (counts, owner list)
  ownersKey: null,
  view: 'gantt', boardCache: new Map(),  // view: gantt | board (URL: view=board); boardCache: key -> pmd_board result
};
// Filters and columns come from the URL (shared links) and from this browser's settings.
{
  const QP = new URLSearchParams(location.search);
  S.q = (QP.get('q') || '').trim(); S.owner = QP.get('owner') || ''; S.hideDone = QP.get('hide_done') === '1'; S.ms = QP.get('ms') === '1';
  const at = QP.get('attn');
  S.chip = ['conflict', 'disputed', 'overdue', 'decide', 'due14', 'late', 'tight'].includes(at) ? at : null;
  S.view = QP.get('view') === 'board' ? 'board' : 'gantt';
  S.cols = loadCols();
}

// ---------- loading ----------
const MAX_BYTES = 5 * 1024 * 1024, FETCH_MS = 20000;
async function fetchText(key) {
  const ctl = new AbortController();
  const timer = setTimeout(() => ctl.abort(), FETCH_MS);
  try {
    const r = await fetch(urlOf(key), { cache: 'no-cache', credentials: 'same-origin', referrerPolicy: 'no-referrer', signal: ctl.signal });
    if (!r.ok) throw new Error(`HTTP ${r.status} for ${key}`);
    const tooBig = () => new Error(`File too large (limit ${MAX_BYTES / 1048576} MB): ${key}`);
    if (+r.headers.get('content-length') > MAX_BYTES) throw tooBig();
    const chunks = []; let n = 0;
    const rd = r.body.getReader();
    for (;;) {
      const { done, value } = await rd.read();
      if (done) break;
      n += value.length;
      if (n > MAX_BYTES) { ctl.abort(); throw tooBig(); }
      chunks.push(value);
    }
    return new TextDecoder().decode(await new Blob(chunks).arrayBuffer());
  } catch (e) {
    if (e && e.name === 'AbortError') throw new Error(`Timed out after ${FETCH_MS / 1000}s: ${key}`);
    throw e;
  } finally { clearTimeout(timer); }
}

// Direct links are read live (one hop). A link that cannot be read is unknown (null).
async function peekLink(key) {
  if (S.peek.has(key) || S.projects.has(key)) return;
  S.peek.set(key, null);
  try { const src = await fetchText(key); const p = parseProject({ path: key, src, today: TODAY_ISO, interfaces: {} }); if (!p.error) { S.peek.set(key, p); S.src.set(key, src); } } catch (e) { /* unknown */ }
}
const linkTargets = (proj) => { const out = new Set(); (function w(ns) { for (const n of ns) { if (n.link_resolved && (n.kind === 'subproject' || n.kind === 'deliverable')) { const k = tryKey(n.link_resolved.split('#')[0]); if (k) out.add(k); }; w(n.children || []); } })(proj.nodes || []); return out; };
const known = (key) => { const en = S.projects.get(key); return en && en.status === 'ok' ? en.proj : S.peek.get(key) || null; };

// Interface membership (D47): milestones unless #private.
const inIface = (n) => !(n.tags || []).includes('private') && n.kind === 'milestone';

function buildInterfaces() {
  const out = {};
  for (const k of new Set([...S.projects.keys(), ...S.peek.keys()])) {
    const p = known(k);
    if (!p) continue;
    const i = { milestones: {} };
    if (p.commitment && p.commitment.text) i.commitment = p.commitment.text;
    (function w(ns) { for (const n of ns) { if (n.id && inIface(n) && n.checkbox !== 'cancelled' && n.date && n.date.text) i.milestones[n.id] = n.date.text; w(n.children || []); } })(p.nodes || []);
    if (S.src.has(k)) i.src = S.src.get(k);
    out[k] = i;
  }
  return out;
}

function mkItems(nodes, entry, parent) {
  return nodes.map((node, i) => {
    const it = { id: `${entry.key}:${node.line}:${i}`, node, entry, parent, kind: node.kind, children: [] };
    if (node.kind === 'deliverable') it.target = null;
    if (node.kind === 'subproject') {
      const t = node.link_resolved || (node.link && resolveKey(entry.key, node.link.target)) || null;
      it.target = t ? tryKey(t.split('#')[0]) : null;
      if (it.target && !S.linkItem.has(it.target)) S.linkItem.set(it.target, it);
    }
    if (node.id) S.idItem.set(entry.key + '#' + node.id, it);
    it.children = mkItems(node.children || [], entry, it);
    return it;
  });
}

function loadProject(key) {
  let en = S.projects.get(key);
  if (en) return en.promise || Promise.resolve(en);
  en = { key, status: 'loading', proj: null, root: null, err: null };
  S.projects.set(key, en);
  en.promise = (async () => {
    try {
      const src = await fetchText(key); S.src.set(key, src);
      const req = { path: key, src, today: TODAY_ISO, interfaces: buildInterfaces() };
      let proj = parseProject(req);
      if (proj.error) throw new Error(proj.error);
      await Promise.all([...linkTargets(proj)].map(peekLink));
      req.interfaces = buildInterfaces();
      proj = parseProject(req);
      en.proj = proj;
      en.root = { id: `${key}:root`, kind: 'project', node: null, entry: en, parent: null, children: [] };
      en.root.children = mkItems(proj.nodes || [], en, en.root);
      en.status = 'ok';
    } catch (e) {
      en.status = 'error'; en.err = String((e && e.message) || e);
    }
    delete en.promise;
    S.boardCache = new Map();
    return en;
  })();
  return en.promise;
}

// ---------- spans ----------
const union = (a, b) => (!a ? b : !b ? a : { s: Math.min(a.s, b.s), e: Math.max(a.e, b.e) });
const nodeOf = (it) => it.node;

function commitOf(key) {
  const p = known(key);
  return p && p.commitment ? exprSpan(p.commitment) : null;
}

function ownSpan(it) {
  const n = it.node; if (!n) return null;
  const f = n.forecast || {};
  let s = toDay(f.start), e = toDay(f.end);
  const dur = n.duration ? n.duration.days : null;
  if (s == null && e == null) {
    const r = n.range && exprSpan(n.range), d = n.date && exprSpan(n.date);
    if (r) { s = r.s; e = r.e; } else if (d) { e = d.e; s = n.kind === 'milestone' ? e : d.s; }
  }
  if (n.kind === 'milestone' || n.kind === 'deliverable') { const x = e ?? s; return x == null ? null : { s: x, e: x }; }
  if (s == null && e == null) return null;
  if (s == null) s = dur ? e - dur + 1 : e;
  if (e == null) e = dur ? s + dur - 1 : s;
  return { s, e };
}

// ---------- after: targets ----------
function afterTarget(entryKey, v) {
  const t = v.kind === 'link' ? v.target : v.text;
  if (!t) return null;
  if (t[0] === '^') return { file: entryKey, id: t.slice(1) };
  const [f, h] = t.split('#');
  return { file: f ? resolveKey(entryKey, f) : entryKey, id: h || null };
}
function itemOfTarget(tg) { return (tg.id && S.idItem.get(tg.file + '#' + tg.id)) || S.linkItem.get(tg.file) || null; }

// ---------- flatten ----------
function flatten() {
  const rows = [];
  const idRow = new Map(), fileRow = new Map();
  S.rowById = new Map();
  const rootEn = S.projects.get(S.rootKey);
  if (!rootEn || rootEn.status !== 'ok') { S.rows = rows; S.allRows = rows; S.deps = []; return; }

  function addRow(it, depth, num) {
    const row = { it, depth, num, idx: rows.length, s: null, e: null, vk: 'task' };
    rows.push(row);
    S.rowById.set(it.id, row);
    return row;
  }
  const active = new Set([rootEn.key]);                    // files on the current expansion path: A -> B -> A is not followed
  function visibleChildren(it) {
    if (it.kind === 'subproject') {
      if (!S.expanded.has(it.id) || !it.target || active.has(it.target) || active.size > 40) return null;
      const en = S.projects.get(it.target);
      if (!en) return null;
      if (en.status === 'loading') return 'loading';
      if (en.status === 'error') return null;
      return en.root.children;
    }
    if (S.collapsed.has(it.id) && !filtering()) return null;   // filters show matches under their parents
    return it.children.length ? it.children : null;
  }
  function walk(items, depth, prefix) {
    let agg = null;
    items.forEach((it, i) => {
      const num = (prefix ? prefix + '.' : '') + (i + 1);
      const row = addRow(it, depth, num);
      const n = it.node;
      if (n && n.id) idRow.set(it.entry.key + '#' + n.id, row);
      if (it.kind === 'subproject' && it.target) { if (!fileRow.has(it.target)) fileRow.set(it.target, row); }
      const own = ownSpan(it);
      let sub = null;
      const vc = visibleChildren(it);
      if (vc === 'loading') { rows.push({ loading: true, depth: depth + 1, idx: rows.length, it }); }
      else if (vc) { const t = it.kind === 'subproject' ? it.target : null; if (t) active.add(t); sub = walk(vc, depth + 1, num); if (t) active.delete(t); }
      let span = null;
      if (it.kind === 'heading') { row.vk = 'sum'; span = sub || own; }
      else if (it.kind === 'milestone' || it.kind === 'deliverable') { row.vk = 'ms'; span = own; }
      else if (it.kind === 'subproject') {
        const en = it.target && S.projects.get(it.target);
        row.sub = en || null;
        if (en && en.status === 'error') row.failed = true;
        if (vc && vc !== 'loading') { row.vk = 'sum'; span = union(sub, own); }
        else {
          row.vk = row.failed ? 'unk' : 'sub';
          const c = it.target ? commitOf(it.target) : null;
          row.commit = c;
          const by = n && n.keys && n.keys.by && n.keys.by[0];
          row.by = by ? toDay(by.end || by.text) : null;
          if (c && row.by != null && c.e > row.by) row.mism = true;
          const f = (n && n.forecast) || {};
          const e = toDay(f.end) ?? (c && c.e) ?? row.by ?? (own && own.e);
          let s = toDay(f.start) ?? (c && c.range ? c.s : null) ?? (own && own.s);
          if (e != null) { if (s == null || s > e) { s = e - 45; row.openStart = true; } span = { s, e }; }
        }
      } else { // task
        if (!own && sub) { row.vk = 'sum'; span = sub; }
        else {
          row.vk = 'task'; span = own;
          const dd = (n.range && exprSpan(n.range)) || (n.date && exprSpan(n.date));
          if (!dd && own) row.fc = true;                       // placed only by dependencies/duration
          else if (dd && own) { const ds = n.range ? dd.s : (n.duration ? dd.e - n.duration.days + 1 : dd.e); if (ds !== own.s || dd.e !== own.e) row.decl = { s: ds, e: dd.e }; }
        }
      }
      if (n && n.forecast && n.forecast.conflict) row.conflict = true;
      if (span) { row.s = span.s; row.e = span.e; agg = union(agg, span); }
      if (sub) agg = union(agg, sub);
    });
    return agg;
  }
  const rootItem = rootEn.root;
  const rootRow = addRow(rootItem, 0, '');
  rootRow.vk = 'sum';
  const pf = rootEn.proj.forecast || {};
  const pown = toDay(pf.start) != null || toDay(pf.end) != null ? { s: toDay(pf.start) ?? toDay(pf.end), e: toDay(pf.end) ?? toDay(pf.start) } : null;
  const c = exprSpan(rootEn.proj.commitment);
  let span = null;
  if (!S.collapsed.has(rootItem.id)) span = walk(rootItem.children, 1, '');
  span = union(span, pown); if (c) span = union(span, { s: c.s, e: c.e });
  if (span) { rootRow.s = span.s; rootRow.e = span.e; }
  if (pf.conflict) rootRow.conflict = true;
  S.rows = rows;

  // dependencies
  const deps = [];
  rows.forEach((row) => {
    const n = row.it && row.it.node;
    if (row.loading || !n || !n.keys || !n.keys.after) return;
    for (const v of n.keys.after) {
      const tg = afterTarget(row.it.entry.key, v);
      if (!tg) continue;
      const pred = (tg.id && idRow.get(tg.file + '#' + tg.id)) || fileRow.get(tg.file);
      if (pred && pred !== row) deps.push({ from: pred, to: row });
    }
  });
  S.deps = deps;

  // time range (monotonic union within a root)
  let mn = TODAY, mx = TODAY;
  for (const r of rows) { if (r.s != null) { mn = Math.min(mn, r.s); mx = Math.max(mx, r.e); } }
  if (S.rawMin == null) { S.rawMin = mn; S.rawMax = mx; } else { S.rawMin = Math.min(S.rawMin, mn); S.rawMax = Math.max(S.rawMax, mx); }

  // progress of container rows (done / leaf items below)
  for (const r of rows) if (!r.loading && r.vk === 'sum') { const p = progOf(r.it); if (p) r.prog = p; }
  // filters: matching rows with their ancestors (or flat under the project in Milestones mode)
  S.allRows = rows;
  if (filtering()) {
    S.rows = filterRows(rows);
    S.rows.forEach((r, i) => { r.idx = i; });
    const shown = new Set(S.rows);
    S.deps = S.deps.filter((d) => shown.has(d.from) && shown.has(d.to));
    S.rowById = new Map(S.rows.filter((r) => !r.loading).map((r) => [r.it.id, r]));
  }
}

// ---------- view geometry ----------
function applyRange() {
  const [y0, m0] = ymd(S.rawMin - 14);
  const [y1, m1] = ymd(S.rawMax + 45);
  const nmin = dayOf(y0, m0, 1), nmax = dayOf(y1, m1 + 1, 1);
  if (S.min !== nmin && S.rangeSet) els.rbody.scrollLeft += (S.min - nmin) * S.ppd;
  S.min = nmin; S.max = nmax; S.rangeSet = true;
}
const dx = (day) => (day - S.min) * S.ppd;
const canvasW = () => Math.max(Math.round((S.max - S.min) * S.ppd), els.rbody.clientWidth);

function segs(type, d0, d1, withYear) {
  const out = [];
  if (type === 'year') {
    for (let y = ymd(d0)[0]; ; y++) { const s = dayOf(y, 1, 1); if (s > d1) break; out.push({ s, e: dayOf(y + 1, 1, 1), label: String(y) }); }
  } else if (type === 'quarter' || type === 'month') {
    const step = type === 'quarter' ? 3 : 1;
    let [y, m] = ymd(d0); m = type === 'quarter' ? Math.floor((m - 1) / 3) * 3 + 1 : m;
    for (;;) {
      const s = dayOf(y, m, 1); if (s > d1) break;
      let y2 = y, m2 = m + step; if (m2 > 12) { m2 -= 12; y2++; }
      const label = type === 'quarter' ? (withYear ? `${y} ` : '') + 'Q' + ((m - 1) / 3 + 1) : MONTHS[m - 1] + (withYear ? ' ' + y : '');
      out.push({ s, e: dayOf(y2, m2, 1), label, short: type === 'month' ? MONTHS[m - 1][0] : null });
      y = y2; m = m2;
    }
  } else if (type === 'week') {
    for (let s = d0 - ((((d0 + 3) % 7) + 7) % 7); s <= d1; s += 7) out.push({ s, e: s + 7, label: String(ymd(s)[2]) });
  } else {
    for (let s = Math.floor(d0); s <= d1; s++) out.push({ s, e: s + 1, label: String(ymd(s)[2]) });
  }
  return out;
}
const ZOOMS = { week: 8, month: 2.5, quarter: 0.9, year: 0.3 };
const zoomLevel = () => (S.ppd >= 4 ? 'week' : S.ppd >= 1.2 ? 'month' : S.ppd >= 0.5 ? 'quarter' : 'year');
function tierTypes() {
  const p = S.ppd;
  if (p < 0.5) return [['year']];
  if (p < 1.2) return [['year'], ['quarter']];
  if (p < 4) return [['quarter', true], ['month']];
  if (p < 14) return [['month', true], ['week']];
  return [['month', true], ['day']];
}

// ---------- rendering ----------
let raf = 0;
function schedule() { if (!raf) raf = requestAnimationFrame(() => { raf = 0; render(); }); }

function ownerText(row) {
  const it = row.it;
  if (it.kind === 'project') { const o = it.entry.proj.owner; return o ? o.name || o.text : ''; }
  const o = it.node.keys && it.node.keys.owner && it.node.keys.owner[0];
  if (o) return (o.name || o.text || '').replace(/^@/, '');
  if (row.sub && row.sub.status === 'ok' && row.sub.proj.owner) { const p = row.sub.proj.owner; return p.name || p.text; }
  return '';
}
// A deliverable is labelled "project › milestone": the target's title when loaded, else its file name.
function delivParts(it) {
  const n = it.node, t = n.link_resolved ? tryKey(n.link_resolved.split('#')[0]) : null;
  const kp = t && known(t);
  const proj = kp && kp.title ? kp.title : t ? baseName(t).replace(/\.md$/i, '') : '';
  return { proj, title: n.link ? n.link.text : n.title };
}
function delivLabel(it) {
  const p = delivParts(it);
  return p.proj ? `${p.proj} › ${p.title}` : it.node.title;
}
// Name cell: a deliverable shows its milestone first and the project name after it, muted.
function labelHtml(it) {
  if (it.kind === 'deliverable') { const p = delivParts(it); if (p.proj) return `${esc(p.title)}<span class="pj"> · ${esc(p.proj)}</span>`; }
  return esc(titleOf(it));
}
// A contract value is a state string, or {state, actor} (actor: "parent" | "child").
const contractObj = (it) => { const c = it.entry.proj.contracts, v = c && it.node ? c[it.node.line] : null; if (!v) return null; return typeof v === 'string' ? { state: v, actor: null } : { state: String(v.state || ''), actor: v.actor || null }; };
const contractOf = (it) => ((it.kind === 'subproject' || it.kind === 'deliverable') && (contractObj(it) || {}).state) || '';
function titleOf(it) {
  if (it.kind === 'project') return it.entry.proj.title;
  const n = it.node;
  if (n.kind === 'subproject' && n.link && n.link.text) return n.link.text;
  if (n.kind === 'deliverable') return delivLabel(it);
  return n.title;
}
function durText(row) {
  if (row.s == null) return '';
  if (row.vk === 'ms') return '0d';
  const d = Math.round(row.e - row.s + 1);
  return d >= 14 && d % 7 === 0 ? d / 7 + 'w' : d + 'd';
}
const cbOf = (it) => (it.node ? it.node.checkbox : '');
const stCls = (it) => { const c = cbOf(it); return c === 'done' ? 'st-done' : c === 'cancelled' ? 'st-cancelled' : 'st-open'; };
const stText = (it) => { const c = cbOf(it); return c === 'done' ? 'done' : c === 'cancelled' ? 'cancelled' : c === 'open' ? 'open' : ''; };
const isExpandable = (row) => !row.loading && (row.it.kind === 'subproject' ? !!row.it.target : row.it.children.length > 0);
const isOpen = (row) => (row.it.kind === 'subproject' ? S.expanded.has(row.it.id) : (!S.collapsed.has(row.it.id) || filtering()));

// Why a row is flagged (conflict / bound exceeded).
function reasonsOf(row) {
  const out = [];
  const it = row.it;
  const f = it.kind === 'project' ? it.entry.proj.forecast : it.node && it.node.forecast;
  if (row.conflict) {
    const e = f && toDay(f.end), de = f && toDay(f.declared_end);
    if (e != null && de != null) { const d = Math.round(e - de); out.push(`Forecast end ${fmt(e)} is ${d} day${d === 1 ? '' : 's'} after the declared end ${fmt(de)}`); }
    else out.push('Forecast end is later than the declared end');
  }
  if (row.mism && row.commit && row.by != null) out.push(`Commitment ${fmt(row.commit.e)} is later than by: ${fmt(row.by)}`);
  return out;
}
function declText(n) {
  const d = [];
  if (n.date) d.push(n.date.text); if (n.suffix_date) d.push(n.suffix_date.text); if (n.range) d.push(n.range.text); if (n.duration) d.push(n.duration.text);
  return d.join('  ');
}
// Done later than declared: {d: declared day, e: done day, n: days late} (declared = own date, else by:).
function lateOf(row) {
  const n = row.it.node;
  if (!n || n.checkbox !== 'done' || !n.keys) return null;
  const dn = n.keys.done && n.keys.done[0], dc = (n.date && exprSpan(n.date)) || (n.keys.by && n.keys.by[0] && { e: toDay(n.keys.by[0].end || n.keys.by[0].text) });
  const e = dn && toDay(dn.end || dn.text);
  if (e == null || !dc || dc.e == null || dc.e >= e) return null;
  return { d: dc.e, e, n: e - dc.e };
}
function tipText(row) {
  const it = row.it, n = it.node;
  const lines = [titleOf(it)];
  const st = stText(it);
  lines.push(`${row.vk === 'ms' ? 'milestone' : n ? n.kind : 'project'}${st ? ' · ' + st : ''}`);
  if (row.s != null) lines.push(row.vk === 'ms' ? fmt(row.e) : `${fmt(row.s)} → ${fmt(row.e)} (${durText(row)})${row.fc ? ' forecast' : ''}`);
  if (n && row.decl) lines.push(`Declared ${declText(n)}`);
  const lt = lateOf(row); if (lt) lines.push(`! done ${lt.n}d late (declared ${fmt(lt.d)})`);
  if (row.vk === 'sub') lines.push(row.commit ? `Commitment ${row.commit.text} (live)` : 'No interface known');
  if (row.vk === 'unk') lines.push('Could not load');
  for (const r of reasonsOf(row)) lines.push('! ' + r);
  return lines.join('\n');
}

function render() {
  applyCols();
  const rows = S.rows, rb = els.rbody;
  const W = canvasW(), H = rows.length * RH;
  els.canvas.style.width = W + 'px'; els.canvas.style.height = H + 'px';
  els.rhinner.style.width = W + 'px';
  els.lspacer.style.height = H + 24 + 'px';
  els.deps.setAttribute('width', W); els.deps.setAttribute('height', H);
  const st = rb.scrollTop, sl = rb.scrollLeft;
  els.lbody.scrollTop = st; els.rhead.scrollLeft = sl;
  const r0 = Math.max(0, Math.floor(st / RH) - 8), r1 = Math.min(rows.length, Math.ceil((st + rb.clientHeight) / RH) + 8);

  // header + gridlines (visible x window only)
  const d0 = S.min + (sl - 300) / S.ppd, d1 = S.min + (sl + rb.clientWidth + 300) / S.ppd;
  let h = '', g = '';
  tierTypes().forEach(([type, wy], ti) => {
    for (const sg of segs(type, d0, d1, wy)) {
      const x = dx(sg.s), w = (sg.e - sg.s) * S.ppd;
      let label = sg.label;
      if (w < 14) label = ''; else if (w < 30 && sg.short) label = sg.short; else if (w < 34 && label.length > 2) label = '';
      h += `<div class="hs t${ti}" style="left:${x}px;width:${w}px">${esc(label)}</div>`;
      g += `<div class="gl${ti === 0 ? ' major' : ''}" style="left:${x}px"></div>`;
    }
  });
  h += `<div class="todaytag" style="left:${dx(TODAY + 0.5)}px" title="Today ${TODAY_ISO}"></div>`;
  els.rhinner.innerHTML = h; els.gridlines.innerHTML = g;
  els.today.style.left = dx(TODAY + 0.5) - 1 + 'px';
  const zl = zoomLevel();
  for (const b of els.zoomseg.children) b.classList.toggle('on', b.dataset.z === zl);

  // rows
  let L = '', T = '';
  for (let i = r0; i < r1; i++) {
    const row = rows[i];
    const odd = i & 1 ? ' odd' : '';
    const hov = i === S.hover ? ' hov' : '';
    if (row.loading) {
      L += `<div class="lrow loadrow${odd}" role="row" style="top:${i * RH}px"><div class="name" style="padding-left:${row.depth * 16 + 10}px"><span class="spin">&#8635;</span> Loading&hellip;</div><div></div><div></div><div></div><div></div></div>`;
      T += `<div class="trow${odd}" style="top:${i * RH}px"></div>`;
      continue;
    }
    const it = row.it;
    const sel = S.selId === it.id ? ' sel' : '';
    const stc = it.kind === 'heading' || it.kind === 'project' ? '' : ' ' + stCls(it);
    const cls = `k-${it.kind}${stc}${row.conflict ? ' conflict' : ''}${sel}${odd}${hov}`;
    // expand / collapse
    let tog = '<span class="tog none"></span>';
    if (isExpandable(row)) {
      const open = isOpen(row), busy = open && it.kind === 'subproject' && row.sub && row.sub.status === 'loading';
      tog = `<span class="tog${open ? ' open' : ''}${busy ? ' busy' : ''}" data-act="tog" title="${open ? 'Collapse' : 'Expand'}">${busy ? '<span class="spin">&#8635;</span>' : ''}</span>`;
    }
    let ico = '';
    if (it.kind === 'heading') ico = '<span class="st hd"></span>';
    else if (it.kind === 'milestone' || it.kind === 'deliverable') ico = `<span class="st diamond ${stCls(it)}"></span>`;
    else if (row.failed) ico = `<span class="st sub warnic" title="${esc(row.sub.err)}"></span>`;
    else if (it.kind === 'subproject') ico = `<span class="st sub ${row.sub && row.sub.status === 'ok' ? 'loaded ' : ''}${stCls(it)}" title="${row.sub && row.sub.status === 'ok' ? 'Loaded sub-project' : 'Not expanded: promise read live'}"></span>`;
    else if (it.kind === 'task') ico = `<span class="st ${stCls(it)}"></span>`;
    const why = reasonsOf(row);
    const ct = contractOf(it);
    const pill = ct ? `<span class="ct ct-${esc(ct)}" title="Contract: ${esc(ct)}">${esc(ct)}</span>` : '';
    const flag = why.length ? `<span class="flag" title="${esc(why.join('\n'))}">!</span>` : '';
    const num = S.wbs && row.num ? `<span class="num">${row.num}</span>` : '';
    const focus = it.kind === 'subproject' && it.target ? '<button class="focus" data-act="focus" title="Open as top level (Ctrl/Cmd-click or double-click)">Open</button>' : '';
    const aria = `role="row" aria-level="${row.depth + 1}" aria-selected="${!!sel}"${isExpandable(row) ? ` aria-expanded="${isOpen(row)}"` : ''}`;
    const prog = row.prog ? `<span class="prog" title="Done / leaf items below">${row.prog.d}/${row.prog.t}</span>` : '';
    const slk = S.cols.slack ? `<div class="sl">${slackHtml(row)}</div>` : '';
    const nxt = S.cols.next ? `<div title="${esc(nextText(row))}">${esc(nextText(row))}</div>` : '';
    L += `<div class="lrow ${cls}" ${aria} data-i="${i}" style="top:${i * RH}px"><div class="name" style="padding-left:${row.depth * 16 + 6}px">${tog}${ico}<span class="t" title="${esc(titleOf(it))}">${num}${labelHtml(it)}</span>${pill}${prog}${flag}${focus}</div>` +
      `<div class="d">${esc(fmt(row.s))}</div><div class="d fin">${esc(fmt(row.e))}</div><div>${durText(row)}</div><div title="${esc(ownerText(row))}">${esc(ownerText(row))}</div>${slk}${nxt}</div>`;
    const ch = S.chainIds ? (S.chainIds.has(it.id) ? ' inchain' : ' dimmed') : '';
    T += `<div class="trow${sel}${odd}${hov}${ch}" data-i="${i}" style="top:${i * RH}px">${bars(row)}</div>`;
  }
  els.lspacer.innerHTML = L; els.trows.innerHTML = T;
  renderDeps(r0, r1);
}

function bars(row) {
  if (row.s == null) return '';
  const it = row.it, n = it.node, f = (n && n.forecast) || {};
  const x1 = dx(row.s), x2 = dx(row.e + 1), w = Math.max(x2 - x1, 3);
  const tip = ` data-tip="${esc(tipText(row))}"`;
  const sc = stCls(it) + (row.conflict ? ' conf' : '');
  const mism = row.mism ? ' mism' : '';
  let out = '';
  switch (row.vk) {
    case 'ms': {
      const cx = dx(row.e + 0.5);
      out = `<div class="ms ${sc}" style="left:${cx}px"${tip}></div><span class="mlabel" style="left:${cx + 12}px">${fmt(row.e)}</span>`;
      break;
    }
    case 'task':
      out = `<div class="bar ${sc}${row.fc ? ' fc' : ''}${mism}" style="left:${x1}px;width:${w}px"${tip}></div>`;
      if (row.decl) out += `<div class="declln" style="left:${dx(row.decl.s)}px;width:${Math.max(dx(row.decl.e + 1) - dx(row.decl.s), 3)}px" data-tip="Declared ${esc(declText(n))}"></div>`;
      break;
    case 'sum': {
      const pf = row.prog ? `<i class="pf" style="width:${Math.round((100 * row.prog.d) / row.prog.t)}%"></i>` : '';
      out = `<div class="sum${row.conflict ? ' conf' : ''}${mism}" style="left:${x1}px;width:${w}px"${tip}>${pf}</div>`;
      break;
    }
    case 'sub':
      out = `<div class="sub ${sc}${mism}${row.openStart ? ' openstart' : ''}" style="left:${x1}px;width:${w}px"${tip}>&#8615;</div>`;
      break;
    case 'unk':
      out = `<div class="unk" style="left:${x1}px;width:${w}px"${tip}></div>`;
      break;
  }
  const lt = lateOf(row);
  if (lt) {
    const a = dx(lt.d + 0.5), b = dx(lt.e + 0.5), tp = ` data-tip="Declared ${esc(fmt(lt.d))}\n! done ${lt.n}d late"`;
    out += `<div class="lateln" style="left:${a}px;width:${b - a}px"${tp}></div><div class="late" style="left:${a}px"${tp}></div>`;
  }
  if (row.conflict && f.declared_end) { const dd = toDay(f.declared_end); if (dd != null) out += `<div class="decl" style="left:${dx(dd + 1)}px" data-tip="Declared end ${esc(f.declared_end)}"></div>`; }
  if (row.mism && row.by != null) out += `<div class="decl w" style="left:${dx(row.by + 1)}px" data-tip="Parent by: ${fmt(row.by)}"></div>`;
  return out;
}

function renderDeps(r0, r1) {
  let svg = '';
  const edge = (r, end) => {
    const ms = r.vk === 'ms';
    return end ? (ms ? dx(r.e + 0.5) + 7 : dx(r.e + 1)) : (ms ? dx(r.s + 0.5) - 7 : dx(r.s));
  };
  for (const d of S.deps) {
    const a = d.from, b = d.to;
    if (a.s == null || b.s == null) continue;
    if ((a.idx < r0 && b.idx < r0) || (a.idx >= r1 && b.idx >= r1)) continue;
    const x1 = edge(a, true), x2 = edge(b, false);
    const y1 = a.idx * RH + RH / 2, y2 = b.idx * RH + RH / 2;
    let p;
    if (x2 >= x1 + 12) { const xm = x1 + 6; p = `M${x1} ${y1}H${xm}V${y2}H${x2}`; }
    else { const ym = y2 > y1 ? y1 + RH / 2 : y1 - RH / 2; p = `M${x1} ${y1}H${x1 + 6}V${ym}H${x2 - 8}V${y2}H${x2}`; }
    const c = b.conflict ? ' class="c"' : '';
    svg += `<path${c} d="${p}"/><polygon${c} points="${x2},${y2} ${x2 - 5},${y2 - 3.5} ${x2 - 5},${y2 + 3.5}"/>`;
  }
  els.deps.innerHTML = svg;
}

// ---------- toolbar ----------
function updateToolbar() {
  const en = S.projects.get(S.rootKey);
  els.rootinput.value = S.rootKey || '';
  if (!en || en.status !== 'ok') { els.ptitle.textContent = en && en.status === 'error' ? 'Error' : 'Loading…'; els.pmeta.innerHTML = ''; }
  else {
    const p = en.proj;
    document.title = `${p.title} – projectmd`;
    els.ptitle.textContent = p.title;
    const chip = (k, v) => (v ? `<span class="chip">${k} <b>${esc(v)}</b></span>` : '');
    els.pmeta.innerHTML = chip('Owner', p.owner && (p.owner.name || p.owner.text)) + chip('Commitment', p.commitment && p.commitment.text);
  }
  let c = '';
  S.crumbs.forEach((k, i) => {
    const e = S.projects.get(k);
    c += `<a data-c="${i}" title="${esc(k)}">${esc(e && e.proj ? e.proj.title.split(/ [—-] /)[0] : baseName(k))}</a><span>&rsaquo;</span>`;
  });
  els.crumbs.innerHTML = c;
}

function banner(msg) { els.banner.hidden = !msg; els.banner.textContent = msg || ''; }

// ---------- detail panel ----------
const pubMilestones = (proj) => {
  const out = [];
  (function w(ns) { for (const n of ns) { if (n.id && inIface(n) && n.date) out.push({ id: n.id, text: n.date.text, done: n.checkbox === 'done' }); w(n.children || []); } })(proj.nodes || []);
  return out;
};
// Interface of a sub-project: read live from its file (loaded, or its one-hop read).
function ifaceOf(it) {
  const p = it.target && known(it.target);
  const o = { ms: [], src: p ? 'read live' : 'unknown (not readable here)' };
  if (p) {
    o.commitment = p.commitment && p.commitment.text;
    o.start = (p.forecast && p.forecast.start) || null;
    o.ms = pubMilestones(p);
    o.done = p.done && p.done.text;
  }
  return o;
}

// Whose move, by contract state (the parent or the child; "actor" is shown when given).
const WHOSE = {
  delivered: 'Nobody.',
  unknown: 'Parent: fix the link or access.',
  overdue: 'Child: deliver, or move the promise.',
  disputed: 'Parent: accept, re-plan or talk.',
  requested: 'Child: answer the request.',
  agreed: 'Child: deliver.',
};
function whoseText(it) {
  const o = contractObj(it); if (!o) return '';
  const t = WHOSE[o.state] || o.state;
  return o.actor ? `${t} (actor: ${o.actor})` : t;
}

// Why chain. A node's end is set by `driver`; its start by `start_driver`. An estimate/start-type
// driver with a start_driver line hands over to the start driver; otherwise follow the driver line.
const START_RE = /estim|duration|range|start|after|today/i;
function driverStep(it) {
  const n = it.node;
  if (!n) return null;
  const d = n.driver || null, sd = n.start_driver || null;
  if (d && sd && sd.line && START_RE.test(String(d.kind || ''))) return { drv: sd, from: 'start' };
  if (d && d.line) return { drv: d, from: 'end' };
  if (sd && sd.line) return { drv: sd, from: 'start' };
  if (d) return { drv: d, from: 'end' };
  if (sd) return { drv: sd, from: 'start' };
  return null;
}
// The item whose node sits on source line `line` in the same file (any depth, collapsed or not).
function itemAtLine(root, line) {
  const want = Number(line), st = [root];
  while (st.length) { const x = st.pop(); if (x.node && Number(x.node.line) === want) return x; st.push(...x.children); }
  return null;
}
// Plain words for one hop: why the node's date (or start) is what it is.
function driverWords(h) {
  const k = String(h.drv.kind || '').toLowerCase(), n = h.it.node || {}, nx = h.next;
  if (h.from === 'start' && /today|not.?started|earliest|none/.test(k)) return 'not started: earliest start is today';
  if (h.from === 'end' && /undated|today/.test(k)) return 'open and undated: due today';
  if (/done/.test(k)) { const dn = n.keys && n.keys.done && n.keys.done[0]; return 'done on ' + (dn ? dn.text || dn.end || '' : ''); }
  if (/promise|interface|commit/.test(k)) return 'the promise of ' + (nx ? titleOf(nx) : 'the linked project');
  if (/child|sub.?item|latest/.test(k)) return 'latest sub-item';
  if (/after|depend|successor|start/.test(k)) { const a = n.keys && n.keys.after && n.keys.after[0]; return 'starts after ' + (nx ? titleOf(nx) : a ? a.text : 'an earlier item'); }
  if (/estim|duration|range/.test(k)) return 'its estimate';
  if (/declar|date|deadline/.test(k)) return 'declared date';
  return k ? k.replace(/_/g, ' ') : 'driver';
}
// Ordered hops from the item (at most 50; stops on a repeat or a driver without a further line).
function driverChain(it) {
  const out = [], seen = new Set();
  let cur = it;
  while (cur && out.length < 50 && !seen.has(cur.id)) {
    seen.add(cur.id);
    const step = driverStep(cur);
    const next = step && step.drv.line ? itemAtLine(cur.entry.root, step.drv.line) : null;
    out.push({ it: cur, drv: step ? step.drv : null, from: step ? step.from : null, next });
    cur = next;
  }
  return out;
}
// "Why this date" section; also sets the chart highlight (S.chainIds) for the selection.
function chainHtml(it) {
  const chain = driverChain(it);
  const any = chain.some((h) => h.drv);
  S.chainIds = any ? new Set(chain.map((h) => h.it.id)) : null;
  if (!any) return '<h3>Why this date</h3><div class="dim">no driver information</div>';
  const items = chain.map((h) => {
    const e = toDay(((h.it.node && h.it.node.forecast) || {}).end);
    const words = h.drv ? driverWords(h) : 'no driver';
    return `<li><a href="#" data-sel="${esc(h.it.id)}">${esc(titleOf(h.it))}</a>${e != null ? ` <span class="dim">${esc(fmt(e))}</span>` : ''}<br><span class="dim">${esc(words)}</span></li>`;
  }).join('');
  return `<h3>Why this date</h3><ol class="why">${items}</ol>`;
}
function rawLinkHtml(key) {
  try { const u = new URL(urlOf(key)); if (u.origin !== location.origin) return ''; return `<a href="${esc(u.href)}" target="_blank" rel="noopener noreferrer">raw file</a>`; } catch (e) { return ''; }
}
// The selection lives in the hash as #L<line> when it is a node of the root file.
function setSelHash(it) {
  const n = it && it.node;
  const want = it && it.entry.key === S.rootKey && n && n.line ? '#L' + n.line : '';
  if ((location.hash || '') === want) return;
  try { history.replaceState(history.state, '', location.pathname + location.search + want); } catch (e) { /* ignore */ }
}
function restoreSel() {
  if (!BOOT_SEL) return;
  const root = S.projects.get(S.rootKey);
  if (!root || root.status !== 'ok') return;
  const it = itemAtLine(root.root, +BOOT_SEL[1]);
  if (it) selectItem(it);
}

function showDetail(row) {
  if (!row || row.loading) { hideDetail(); return; }
  const it = row.it, n = it.node, en = it.entry;
  const dl = [];
  const add = (k, v) => { if (v != null && v !== '') dl.push(`<dt>${esc(k)}</dt><dd>${v}</dd>`); };
  const person = (v) => esc(v.text);
  const vals = (a) => (a || []).map((v) => esc(v.text)).join(', ');
  const why = reasonsOf(row);
  let head, line = null;
  if (it.kind === 'project') {
    const p = en.proj, pf = p.forecast || {};
    head = `<h2>${esc(p.title)}</h2><span class="badge">project</span>${pf.conflict ? '<span class="badge red">conflict</span>' : ''}`;
    add('Commitment', esc(p.commitment && p.commitment.text));
    add('Forecast start', esc(pf.start || ''));
    add('Forecast end', esc(pf.end || ''));
    add('Duration', durText(row));
    add('Owner', esc(p.owner && p.owner.text));
    const sfx = p.suffix || {};
    add('Declared', esc([sfx.date, sfx.range, sfx.duration].filter(Boolean).map((x) => x.text).join('  ')));
    add('Tags', esc((sfx.tags || []).map((t) => '#' + t).join(' ')));
    for (const [k, v] of Object.entries(p.header || {})) if (!['owner', 'stakeholders', 'commitment'].includes(k.toLowerCase())) add(k, esc(v));
    for (const [k, v] of Object.entries(sfx.keys || {})) add(k, vals(v));
  } else {
    const f = n.forecast || {};
    const st = stText(it);
    head = `<h2>${esc(titleOf(it))}</h2><span class="badge">${esc(n.kind)}</span>${st ? `<span class="badge b-${st === 'done' ? 'done' : st === 'open' ? 'open' : ''}">${st}</span>` : ''}` +
      (f.conflict ? '<span class="badge red">conflict</span>' : '') + (row.mism ? '<span class="badge warn">exceeds by:</span>' : '');
    line = n.line;
    add('Declared', esc(declText(n)));
    { const lt = lateOf(row); if (lt) add('Late', `<span class="r">done ${lt.n}d late (declared ${esc(fmt(lt.d))})</span>`); }
    add('Forecast start', esc(f.start || (row.s != null && !row.fc ? fmt(row.s) + ' (shown)' : '')));
    add('Forecast end', esc(f.end || (row.e != null && !row.fc ? fmt(row.e) + ' (shown)' : '')));
    if (f.declared_end) add('Declared end', esc(f.declared_end));
    { const sl = f.slack ?? n.slack; if (sl != null && sl !== '') add('Slack', esc(typeof sl === 'number' ? sl + ' day' + (sl === 1 ? '' : 's') : sl)); }
    add('Duration', durText(row));
    if (n.kind === 'heading') add('Items', String(n.children.length));
    const k = n.keys || {};
    if (k.after) add('After', k.after.map((v) => {
      const tg = afterTarget(en.key, v), t = tg && itemOfTarget(tg);
      const label = esc(v.text);
      return t ? `<a href="#" data-sel="${esc(t.id)}" title="Select ${esc(titleOf(t))}">${label}</a>` : `<span title="Not loaded">${label}</span>`;
    }).join(', '));
    add('By', vals(k.by));
    add('Contract', contractOf(it) ? `<span class="ct ct-${esc(contractOf(it))}">${esc(contractOf(it))}</span>` : '');
    if (contractOf(it)) add('Whose move', esc(whoseText(it)));
    add('Tags', esc((n.tags || []).map((t) => '#' + t).join(' ')));
    add('Owner', vals(k.owner) || esc(ownerText(row)));
    const inKeys = ['after', 'by', 'owner'];
    for (const key of Object.keys(k)) if (!inKeys.includes(key)) add(key, vals(k[key]));
    add('ID', n.id ? '^' + esc(n.id) : '');
    if (it.kind === 'subproject') {
      const ifc = ifaceOf(it);
      add('Link', it.target ? `<a href="#" data-md="${esc(it.target)}">${esc(it.target)}</a>` : '');
      if (row.failed) add('Load error', esc(row.sub.err));
      dl.push('<dt></dt><dd><b>Interface</b></dd>');
      add('Source', esc(ifc.src));
      add('Commitment', esc(ifc.commitment));
      add('Start', esc(ifc.start));
      add('Done', esc(ifc.done));
      add('Public milestones', ifc.ms.length ? '<ul>' + ifc.ms.map((m) => `<li>${esc(m.id)}: ${esc(m.text)}</li>`).join('') + '</ul>' : '');
    }
  }
  const diags = (en.proj.diagnostics || []).filter((d) => (it.kind === 'project' ? true : d.line === line));
  let html = `<button class="x" id="dclose" title="Close (Esc)" aria-label="Close">&times;</button>${head}`;
  if (why.length) html += why.map((r) => `<div class="reason">${esc(r)}</div>`).join('');
  if (it.kind !== 'project' && n.description) html += `<h3>Description</h3><div class="desc">${esc(n.description)}</div>`;
  html += `<dl>${dl.join('')}</dl>`;
  if (it.kind !== 'project') html += chainHtml(it);
  html += `<h3>Location</h3><div><a href="#" data-md="${esc(en.key)}"${line ? ` data-line="${line}"` : ''}>${esc(en.key)}${line ? ':' + line : ''}</a></div>`;
  html += `<div class="dim">${rawLinkHtml(en.key)}</div>`;
  if (diags.length) html += '<h3>Diagnostics</h3>' + diags.map((d) => `<div class="diag ${esc(d.severity)}"><b>${esc(d.code)}</b>${d.line ? ' (line ' + esc(d.line) + ')' : ''}<br>${esc(d.message)}</div>`).join('');
  els.detail.innerHTML = html;
  els.detail.hidden = false; els.main.classList.add('dopen'); setSelHash(it); schedule();
}
function hideDetail() { els.detail.hidden = true; els.main.classList.remove('dopen'); S.detailOpen = false; S.chainIds = null; setSelHash(null); schedule(); }

// ---------- time slider (plan date) ----------
// Moving the date re-parses every loaded file with the new `today`, coalesced per animation frame.
const TSTEPS = 60, TICK_MS = 100;
let reparseRaf = 0, playTimer = 0;
function syncTodayUrl() {
  try {
    const u = new URL(location.href);
    if (TODAY_ISO === REAL_ISO) u.searchParams.delete('today'); else u.searchParams.set('today', TODAY_ISO);
    history.replaceState(history.state, '', u.href);
  } catch (e) { /* some embedding contexts refuse replaceState */ }
}
function updateTimeUi() {
  if (!els.tsl) return;
  const en = S.projects.get(S.rootKey);
  const ok = !!(en && en.status === 'ok');
  if (S.tRoot !== S.rootKey && (ok || S.tMin == null)) {
    const pf = ok ? en.proj.forecast || {} : {};
    let a = toDay(pf.start), b = toDay(pf.end);
    if (a == null && b == null) { a = TODAY - 365; b = TODAY + 365; }
    a = a ?? b; b = b ?? a;
    if (b <= a) b = a + 1;
    S.tMin = a; S.tMax = b; S.tRoot = ok ? S.rootKey : null;
    els.tsl.min = a; els.tsl.max = b;
  }
  els.tsl.value = Math.min(S.tMax, Math.max(S.tMin, TODAY));
  els.tdate.textContent = TODAY_ISO;
}
// Re-parse every loaded file (root first) with the current TODAY_ISO; rebuild the rows.
function reparseAll() {
  const root = S.projects.get(S.rootKey);
  if (!root || root.status !== 'ok') return;
  const ifc = buildInterfaces();
  for (const [k, p] of S.peek) if (p && S.src.has(k)) { const r = parseProject({ path: k, src: S.src.get(k), today: TODAY_ISO, interfaces: {} }); if (!r.error) S.peek.set(k, r); }
  S.linkItem = new Map(); S.idItem = new Map();
  for (const [k, en] of S.projects) {
    if (en.status !== 'ok' || !S.src.has(k)) continue;
    const r = parseProject({ path: k, src: S.src.get(k), today: TODAY_ISO, interfaces: ifc });
    if (r.error) continue;
    en.proj = r;
    en.root.children = mkItems(r.nodes || [], en, en.root);
  }
  S.boardCache = new Map();
  rebuild();
}
// Sets the plan date (day number) and schedules a re-parse; `today` is then what the wasm parser receives.
function setToday(day) {
  const iso = fmt(day);
  if (iso !== TODAY_ISO) {
    TODAY_ISO = iso; TODAY = toDay(iso);
    syncTodayUrl();
    if (!reparseRaf) reparseRaf = requestAnimationFrame(() => { reparseRaf = 0; reparseAll(); });
  }
  updateTimeUi();
}
function stopPlay() {
  if (playTimer) clearInterval(playTimer);
  playTimer = 0; els.tplay.textContent = 'Play'; els.tplay.setAttribute('aria-pressed', 'false');
}
// About TSTEPS steps from the current date to the end of the range, at 1000/TICK_MS per second.
function togglePlay() {
  if (playTimer) { stopPlay(); return; }
  if (S.tMin == null || S.tMax == null) return;
  const end = S.tMax;
  let cur = TODAY >= end ? S.tMin : TODAY;
  if (cur >= end) return;
  const step = Math.max(1, Math.ceil((end - cur) / TSTEPS));
  els.tplay.textContent = 'Pause'; els.tplay.setAttribute('aria-pressed', 'true');
  playTimer = setInterval(() => {
    cur = Math.min(end, cur + step);
    setToday(cur);
    if (cur >= end) stopPlay();
  }, TICK_MS);
}

// ---------- attention bar: item predicates over the loaded tree ----------
const childItemsOf = (it) => {
  if (it.kind === 'subproject') { const en = it.target && S.projects.get(it.target); return en && en.status === 'ok' ? en.root.children : []; }
  return it.children || [];
};
const isOpenIt = (it) => !!it.node && it.kind !== 'heading' && !['done', 'cancelled'].includes(cbOf(it));
const fcEnd = (it) => toDay(((it.node && it.node.forecast) || {}).end);
const declEnd = (it) => { const n = it.node || {}; const x = (n.date && exprSpan(n.date)) || (n.range && exprSpan(n.range)); return x ? x.e : null; };
const inNext14 = (d) => d != null && d >= TODAY && d <= TODAY + 14;
// Decisions owed: an open leaf task with an owner: and no date, range or estimate, whose forecast ends today
// (an undated open task is placed at today, so it is waiting on its owner now).
function isDecision(it) {
  const n = it.node;
  if (!n || n.kind !== 'task' || !isOpenIt(it) || childItemsOf(it).length) return false;
  if (n.date || n.range || n.duration) return false;
  if (!(n.keys && n.keys.owner && n.keys.owner.length)) return false;
  return fcEnd(it) === TODAY;
}
const isDue14 = (it) => isOpenIt(it) && (inNext14(declEnd(it)) || inNext14(fcEnd(it)));
// Tight: an open item with a declared end (or a by:) and 0 to 2 days of slack. Zero is a risk a manager wants to see.
const isTight = (it) => {
  if (!isOpenIt(it) || (declEnd(it) == null && askOf(it) == null)) return false;
  const s = slackNum(it);
  return s != null && s >= 0 && s <= 2;
};
const ATTN = [
  ['conflict', 'Conflicts', (it) => !!(it.node && it.node.forecast && it.node.forecast.conflict)],
  ['disputed', 'Disputed', (it) => contractOf(it) === 'disputed'],
  ['overdue', 'Overdue', (it) => contractOf(it) === 'overdue'],
  ['decide', 'Decisions owed', (it) => isDecision(it)],
  ['due14', 'Due in 14 days', (it) => isDue14(it)],
  ['late', 'Late deliveries', (it) => !!lateOf({ it })],
  ['tight', 'Tight', (it) => isTight(it)],
];
const filtering = () => !!(S.q || S.owner || S.hideDone || S.ms || S.chip);
// Visits every item of every loaded project (counts do not depend on what is expanded).
function loadedItems(cb) {
  for (const en of S.projects.values()) {
    if (en.status !== 'ok' || !en.root) continue;
    const st = [...en.root.children];
    while (st.length) { const it = st.pop(); cb(it); st.push(...it.children); }
  }
}
function updateAttn() {
  const cnt = {};
  for (const a of ATTN) cnt[a[0]] = 0;
  loadedItems((it) => { for (const a of ATTN) if (a[2](it)) cnt[a[0]]++; });
  for (const a of ATTN) {
    const b = document.querySelector(`[data-attn="${a[0]}"]`);
    if (!b) continue;
    b.querySelector('b').textContent = String(cnt[a[0]]);
    b.classList.toggle('zero', cnt[a[0]] === 0);
    b.classList.toggle('on', S.chip === a[0]);
    b.setAttribute('aria-pressed', String(S.chip === a[0]));
  }
  if (els.phMeta) els.phMeta.textContent = `As of ${TODAY_ISO}. ` + ATTN.map((a) => `${a[1]}: ${cnt[a[0]]}`).join('. ') + '.';
  if (els.phTitle && els.ptitle) els.phTitle.textContent = els.ptitle.textContent;
  return cnt;
}
function updateFstate() {
  if (!els.fstate) return;
  const count = (rows) => rows.filter((r) => !r.loading && r.depth > 0).length;
  els.fstate.textContent = filtering() ? `Showing ${count(S.rows)} of ${count(S.allRows)} rows` : '';
}

// ---------- filters ----------
function matchesQ(row) {
  const q = S.q.toLowerCase(), n = row.it.node || {};
  return titleOf(row.it).toLowerCase().includes(q) || ownerText(row).toLowerCase().includes(q) || (!!n.id && ('^' + n.id).toLowerCase().includes(q));
}
// A row passes when it meets every active filter (the project header always passes).
function rowPass(row) {
  if (row.loading) return false;
  const it = row.it;
  if (it.kind === 'project') return true;
  if (S.chip) { const a = ATTN.find((x) => x[0] === S.chip); if (!a || !a[2](it)) return false; }
  if (S.owner && ownerText(row) !== S.owner) return false;
  if (S.hideDone && cbOf(it) === 'done') return false;
  if (S.ms && !(row.vk === 'ms' || it.kind === 'subproject')) return false;
  if (S.q && !matchesQ(row)) return false;
  return true;
}
// Keeps the passing rows and, unless Milestones mode, their ancestors (rows are in display order, so the
// ancestors are the open rows on a depth stack). Milestones mode lists the passing rows flat under the project.
function filterRows(rows) {
  const keep = new Array(rows.length).fill(false), stack = [];
  rows.forEach((r, i) => {
    while (stack.length && rows[stack[stack.length - 1]].depth >= r.depth) stack.pop();
    if (i === 0) keep[i] = true;
    else if (rowPass(r)) { keep[i] = true; if (!S.ms) for (const j of stack) keep[j] = true; }
    stack.push(i);
  });
  const out = [];
  rows.forEach((r, i) => { if (!keep[i]) return; if (S.ms && i > 0) r.depth = 1; out.push(r); });
  return out;
}
function syncFilterUrl() {
  try {
    const u = new URL(location.href);
    const put = (k, v) => (v ? u.searchParams.set(k, v) : u.searchParams.delete(k));
    put('q', S.q); put('owner', S.owner); put('hide_done', S.hideDone ? '1' : ''); put('ms', S.ms ? '1' : ''); put('attn', S.chip || '');
    put('view', S.view === 'board' ? 'board' : '');
    history.replaceState(history.state, '', u.href);
  } catch (e) { /* some embedding contexts refuse replaceState */ }
}
// Owner select: distinct owners of the loaded rows (kept only when the list changes).
function fillOwners() {
  if (!els.ownerf) return;
  const set = new Set();
  for (const r of S.allRows) { if (r.loading) continue; const o = ownerText(r); if (o) set.add(o); }
  if (S.owner) set.add(S.owner);
  const list = [...set].sort();
  const key = list.join('\n');
  if (key !== S.ownersKey) {
    S.ownersKey = key;
    els.ownerf.textContent = '';
    const add = (v, label) => { const o = document.createElement('option'); o.value = v; o.textContent = label; els.ownerf.appendChild(o); };
    add('', 'All');
    for (const o of list) add(o, o);
  }
  els.ownerf.value = S.owner;
}

// ---------- progress, slack / late, next move ----------
// Done / total leaf items below a container (cancelled items are not counted); null when there are none.
function progOf(it) {
  let d = 0, t = 0, guard = 0;
  const st = [...childItemsOf(it)];
  while (st.length && guard++ < 20000) {
    const x = st.pop(), kids = childItemsOf(x);
    if (kids.length) { st.push(...kids); continue; }
    if (x.kind === 'heading' || cbOf(x) === 'cancelled') continue;
    t++; if (cbOf(x) === 'done') d++;
  }
  return t ? { d, t } : null;
}
// Link rows (sub-project, deliverable) with a by: have an ask (by:) and a promise (the live commitment, or the milestone date).
const isLinkIt = (it) => it.kind === 'subproject' || it.kind === 'deliverable';
function askOf(it) { const b = isLinkIt(it) && it.node.keys && it.node.keys.by && it.node.keys.by[0]; return b ? toDay(b.end || b.text) : null; }
function promiseOf(it) {
  if (it.kind === 'subproject') { const c = it.target ? commitOf(it.target) : null; return c ? c.e : null; }
  return toDay(((it.node && it.node.forecast) || {}).end);
}
// Slack in days (negative: late). The JSON slack when given; else on link rows the ask minus the promise. null: unknown.
function slackNum(it) {
  const n = (it && it.node) || {}, f = n.forecast || {};
  const sl = f.slack ?? n.slack;
  if (typeof sl === 'number') return sl;
  if (sl != null && sl !== '') return null;
  const a = askOf(it), p = a != null ? promiseOf(it) : null;
  return p != null ? a - p : null;
}
function slackHtml(row) {
  const lt = row.loading ? null : lateOf(row);
  if (lt) return `<span class="r">done ${lt.n}d late</span>`;
  const n = row.it.node || {}, f = n.forecast || {}, sl = f.slack ?? n.slack;
  if (sl != null && sl !== '' && typeof sl !== 'number') return esc(sl);
  const d = slackNum(row.it);
  if (d == null) return '';
  const tip = isLinkIt(row.it) ? ` title="${esc(`by: ${fmt(askOf(row.it))}, promise ${fmt(promiseOf(row.it))}`)}"` : '';
  return `<span${d < 0 ? ' class="r"' : ''}${tip}>${d > 0 ? '+' : ''}${d}d</span>`;
}
const MOVE = { unknown: 'parent: fix', overdue: 'child: deliver', disputed: 'parent: decide', requested: 'child: answer', agreed: 'child: deliver' };
function nextText(row) {
  const mv = MOVE[contractOf(row.it)];
  if (typeof mv === 'string') return mv;
  return isDecision(row.it) ? ownerText(row) : '';
}

// ---------- optional columns (remembered in localStorage) ----------
function loadCols() {
  const def = { slack: true, next: true };
  try { return Object.assign(def, JSON.parse(localStorage.getItem('pmd.cols') || '{}')); } catch (e) { return def; }
}
function saveCols() { try { localStorage.setItem('pmd.cols', JSON.stringify(S.cols)); } catch (e) { /* ignore */ } }
function applyCols() {
  if (!els.left) return;
  const tpl = ['minmax(240px, 1fr)', '98px', '98px', '50px', '96px'];
  if (S.cols.slack) tpl.push('88px');
  if (S.cols.next) tpl.push('130px');
  els.left.style.setProperty('--tpl', tpl.join(' '));
  for (const h of els.lhead.querySelectorAll('[data-col]')) h.hidden = !S.cols[h.dataset.col];
  for (const c of document.querySelectorAll('[data-colopt]')) c.checked = !!S.cols[c.dataset.colopt];
}

// ---------- tree levels and time presets ----------
// Level L: items above depth L are expanded, deeper ones collapsed. Sub-projects are fetched only when a level
// needs them; "All" expands what is already loaded and fetches nothing.
function setLevel(lvl) {
  const key = S.rootKey, root = S.projects.get(key);
  if (!root || root.status !== 'ok') return;
  const all = lvl === 'all', L = all ? Infinity : lvl;
  const pending = [];
  const visit = (items, d) => {
    for (const it of items) {
      if (it.kind === 'subproject') {
        if (!it.target) continue;
        const en = S.projects.get(it.target);
        const want = all ? (!!en && en.status === 'ok') : d < L;
        if (!want) { S.expanded.delete(it.id); continue; }
        S.expanded.add(it.id);
        if (!en || en.status === 'loading') pending.push(it.target);
        else if (en.status === 'ok') visit(en.root.children, d + 1);
      } else if (it.children.length) {
        if (all || d < L) S.collapsed.delete(it.id); else S.collapsed.add(it.id);
        visit(it.children, d + 1);
      }
    }
  };
  visit(root.root.children, 1);
  rebuild();
  if (pending.length) Promise.all([...new Set(pending)].map((k) => loadProject(k))).then(() => { if (S.rootKey === key) setLevel(lvl); });
}
// 4 weeks: today -1w .. +4w. Quarter: today -2w .. +3 months.
function setPreset(kind) {
  if (TODAY == null) return;
  const [y, m, d] = ymd(TODAY);
  const four = kind === '4w';
  const a = four ? TODAY - 7 : TODAY - 14;
  const b = four ? TODAY + 28 : dayOf(y, m + 3, d);
  S.rawMin = Math.min(S.rawMin ?? a, a); S.rawMax = Math.max(S.rawMax ?? b, b);
  applyRange();
  S.ppd = clampZ(Math.max(300, els.rbody.clientWidth - 20) / (b - a));
  render();
  els.rbody.scrollLeft = dx(a); render();
}

// ---------- share and print ----------
function shareUrl() {
  const u = new URL(location.href);
  if (S.rootKey) u.searchParams.set('root', S.rootKey);
  if (TODAY_ISO !== REAL_ISO) u.searchParams.set('today', TODAY_ISO); else u.searchParams.delete('today');
  return u.href;
}

// ---------- filter, attention, column, level, preset and share controls ----------
els.qsearch.value = S.q; els.hidedone.checked = S.hideDone; els.msonly.checked = S.ms;
els.qsearch.addEventListener('input', () => { S.q = els.qsearch.value.trim(); syncFilterUrl(); rebuild(); });
els.qsearch.addEventListener('keydown', (e) => {
  if (e.key !== 'Escape') return;
  e.preventDefault(); els.qsearch.value = ''; S.q = ''; syncFilterUrl(); rebuild();
});
els.ownerf.addEventListener('change', () => { S.owner = els.ownerf.value; syncFilterUrl(); rebuild(); });
els.hidedone.addEventListener('change', () => { S.hideDone = els.hidedone.checked; syncFilterUrl(); rebuild(); });
els.msonly.addEventListener('change', () => { S.ms = els.msonly.checked; syncFilterUrl(); rebuild(); });
els.attn.addEventListener('click', (e) => {
  const b = e.target.closest('[data-attn]'); if (!b) return;
  const k = b.dataset.attn; S.chip = S.chip === k ? null : k; syncFilterUrl(); rebuild();
});
for (const c of document.querySelectorAll('[data-colopt]')) {
  c.addEventListener('change', () => { S.cols[c.dataset.colopt] = c.checked; saveCols(); rebuild(); });
}
els.levelseg.addEventListener('click', (e) => {
  const b = e.target.closest('[data-lvl]'); if (b) setLevel(b.dataset.lvl === 'all' ? 'all' : +b.dataset.lvl);
});
for (const b of document.querySelectorAll('[data-preset]')) b.addEventListener('click', () => setPreset(b.dataset.preset));
els.copylink.addEventListener('click', async () => {
  const btn = els.copylink, old = btn.textContent;
  try { await navigator.clipboard.writeText(shareUrl()); btn.textContent = 'Copied'; } catch (e) { btn.textContent = 'Copy failed'; }
  setTimeout(() => { btn.textContent = old; }, 1500);
});
els.printbtn.addEventListener('click', () => { updateAttn(); window.print(); });
els.viewseg.addEventListener('click', (e) => { const b = e.target.closest('[data-view]'); if (b) setView(b.dataset.view); });
els.board.addEventListener('click', (e) => { const c = e.target.closest('.card'); if (c) openCard(c.dataset.key, +c.dataset.line); });
els.board.addEventListener('keydown', (e) => {
  if (e.key !== 'Enter' && e.key !== ' ') return;
  const c = e.target.closest ? e.target.closest('.card') : null;
  if (!c) return;
  e.preventDefault(); openCard(c.dataset.key, +c.dataset.line);
});

// ---------- board view: pmd_board for each loaded file (read-only) ----------
const COLS = [['waiting', 'Waiting'], ['ready', 'Ready'], ['doing', 'Doing'], ['done', 'Done']];
const BOARD_NOTE = 'A board is a view. To move a card, edit the file: start = add `after: <today>`, finish = tick the box and add `done:`.';
const mk = (tag, cls, text) => { const x = document.createElement(tag); if (cls) x.className = cls; if (text != null) x.textContent = text; return x; };
const who = (o) => String(o || '').replace(/^@/, '');
// Switches the view (URL: view=board). The board follows the table's filters, owner and search.
function setView(v) {
  S.view = v === 'board' ? 'board' : 'gantt';
  const board = S.view === 'board';
  document.body.classList.toggle('view-board', board);
  els.main.hidden = board; els.board.hidden = !board;
  for (const b of els.viewseg.children) { const on = b.dataset.view === S.view; b.classList.toggle('on', on); b.setAttribute('aria-pressed', String(on)); }
  syncFilterUrl();
  if (board) renderBoard(); else schedule();
}
// The root file, then every loaded (expanded) sub-project file in tree order; each file once.
function boardFiles() {
  const out = [], seen = new Set();
  const root = S.projects.get(S.rootKey);
  if (!root || root.status !== 'ok') return out;
  const visit = (en) => {
    if (seen.has(en.key)) return;
    seen.add(en.key); out.push(en);
    const walk = (list) => {
      for (const it of list) {
        if (it.kind === 'subproject' && it.target && S.expanded.has(it.id)) { const c = S.projects.get(it.target); if (c && c.status === 'ok') visit(c); }
        walk(it.children);
      }
    };
    walk(en.root.children);
  };
  visit(root);
  return out;
}
// pmd_board of one file, cached until the projects or the plan date change (S.boardCache is reset then).
function boardFor(en) {
  let r = S.boardCache.get(en.key);
  if (!r) {
    try { r = boardOf({ path: en.key, src: S.src.get(en.key), today: TODAY_ISO, interfaces: buildInterfaces() }); }
    catch (e) { r = { error: String((e && e.message) || e) }; }
    S.boardCache.set(en.key, r);
  }
  return r;
}
const cardMatches = (c) => {
  if (S.owner && who(c.owner) !== S.owner) return false;
  if (!S.q) return true;
  return `${c.title || ''} ${who(c.owner)} ${c.id ? '^' + c.id : ''}`.toLowerCase().includes(S.q.toLowerCase());
};
function cardEl(en, c, k) {
  const card = mk('div', 'card');
  card.setAttribute('role', 'button'); card.tabIndex = 0;
  card.dataset.key = en.key; card.dataset.line = String(c.line);
  card.appendChild(mk('div', 'card-t', c.title || ''));
  if (c.owner) card.appendChild(mk('div', 'card-o', who(c.owner)));
  let d = '';
  if (k === 'waiting') d = c.waits_on && c.waits_on.length ? 'waits on ' + c.waits_on.join(', ') : '';
  else if (k === 'doing') d = c.since ? 'since ' + c.since : '';
  else if (k === 'done') d = c.done ? 'done ' + c.done : '';
  if (d) card.appendChild(mk('div', 'card-d', d));
  return card;
}
function boardFoot(b) {
  const f = mk('div', 'bfoot');
  const wip = b.wip || {};
  const names = Object.keys(wip).filter((o) => !S.owner || who(o) === S.owner);
  f.appendChild(mk('div', '', 'Work in progress: ' + (names.length ? names.map((o) => `${who(o)} ${wip[o]}`).join(', ') : 'none')));
  const tp = b.throughput_per_week;
  f.appendChild(mk('div', '', typeof tp === 'number' ? `Throughput: ${tp.toFixed(1)} per week${b.throughput_window_days ? ` (last ${b.throughput_window_days} days)` : ''}` : 'Throughput: no finished items yet'));
  const ct = b.cycle_time;
  f.appendChild(mk('div', '', ct ? `Cycle time: median ${Number(ct.median_days).toFixed(1)} days (n ${ct.n})` : 'Cycle time: none yet'));
  for (const x of b.flow_forecast || []) {
    const late = !!x.declared && x.about > x.declared;   // ISO dates compare as text
    f.appendChild(mk('div', late ? 'r' : '', `Flow forecast, ${x.title}: ${x.items_left} left, likely ${x.about}${x.declared ? ', declared ' + x.declared : ''}`));
  }
  return f;
}
function boardSection(en) {
  const sec = mk('section', 'bsec');
  sec.appendChild(mk('h2', 'bsec-h', (en.proj && en.proj.title) || baseName(en.key)));
  const b = boardFor(en);
  if (b.error) { sec.appendChild(mk('p', 'r', 'Board unavailable: ' + b.error)); return sec; }
  const cols = mk('div', 'bcols');
  for (const [k, label] of COLS) {
    const shown = ((b.columns && b.columns[k]) || []).filter(cardMatches);
    const col = mk('div', 'bcol bcol-' + k);
    const head = mk('h3', 'bcol-h');
    head.appendChild(mk('span', '', label));
    head.appendChild(mk('span', 'bcount', String(shown.length)));
    col.appendChild(head);
    for (const c of shown) col.appendChild(cardEl(en, c, k));
    if (k === 'done' && b.older_done) col.appendChild(mk('div', 'bolder dim', `${b.older_done} older done`));
    cols.appendChild(col);
  }
  sec.appendChild(cols);
  sec.appendChild(boardFoot(b));
  return sec;
}
function renderBoard() {
  const box = els.board;
  box.textContent = '';
  const root = S.projects.get(S.rootKey);
  if (!root || root.status !== 'ok') { box.appendChild(mk('p', 'dim', root && root.status === 'error' ? 'Could not load the project.' : 'Loading…')); return; }
  for (const en of boardFiles()) box.appendChild(boardSection(en));
  box.appendChild(mk('p', 'board-note', BOARD_NOTE));
}
// A card opens the detail panel of its item (file + line): switches to the table and selects it.
function openCard(key, line) {
  const en = S.projects.get(key);
  if (!en || en.status !== 'ok') return;
  const it = itemAtLine(en.root, line);
  setView('gantt');
  if (it) selectItem(it);
}

// ---------- actions ----------
function rebuild() {
  flatten();
  if (S.rows.length) {
    if (S.rawMin == null) { S.rawMin = TODAY; S.rawMax = TODAY; }
    applyRange();
    if (S.needFit) { fit(); S.needFit = false; }
  }
  updateToolbar();
  updateTimeUi();
  updateAttn();
  fillOwners();
  updateFstate();
  if (S.view === 'board') renderBoard(); else render();
  if (S.detailOpen) { const r = S.rowById.get(S.selId); if (r) showDetail(r); }
}
function fit() {
  const w = Math.max(300, els.rbody.clientWidth - 20);
  S.ppd = clampZ(w / Math.max(1, S.max - S.min));
  els.rbody.scrollLeft = 0;
  render();
}
const clampZ = (p) => Math.min(60, Math.max(0.12, p));
function zoom(factor, anchorX) {
  const rb = els.rbody;
  const ax = anchorX ?? rb.clientWidth / 2;
  const day = S.min + (rb.scrollLeft + ax) / S.ppd;
  S.ppd = clampZ(S.ppd * factor);
  render(); // sizes canvas first so scrollLeft is valid
  rb.scrollLeft = (day - S.min) * S.ppd - ax;
  render();
}

function toggle(row) {
  const it = row.it;
  if (it.kind === 'subproject') {
    if (!it.target) return;
    if (S.expanded.has(it.id)) S.expanded.delete(it.id);
    else {
      S.expanded.add(it.id);
      const had = S.projects.get(it.target);
      const p = loadProject(it.target);
      if (!had || had.status === 'loading') p.then(() => rebuild());
    }
  } else if (S.collapsed.has(it.id)) S.collapsed.delete(it.id);
  else S.collapsed.add(it.id);
  rebuild();
}

function focusRow(row) {
  const it = row.it;
  if (it.kind !== 'subproject' || !it.target) return;
  navigate(it.target, [...S.crumbs, S.rootKey], true);
}

function scrollToRow(idx) {
  const rb = els.rbody, top = idx * RH;
  if (top < rb.scrollTop) rb.scrollTop = top;
  else if (top + RH > rb.scrollTop + rb.clientHeight) rb.scrollTop = top + RH - rb.clientHeight;
}
function selectRow(row, scroll) {
  S.selId = row.it.id; S.detailOpen = true;
  if (scroll) scrollToRow(row.idx);
  showDetail(row); render();
}
// Select an item that may be hidden (collapsed ancestors, unexpanded sub-project): reveal it first.
function selectItem(it) {
  for (let cur = it; cur;) {
    if (cur.parent) { S.collapsed.delete(cur.parent.id); cur = cur.parent; continue; }
    if (cur.entry.key === S.rootKey) break;
    const via = S.linkItem.get(cur.entry.key);
    if (!via) break;
    S.expanded.add(via.id); cur = via;
  }
  S.selId = it.id; S.detailOpen = true;
  rebuild();
  const row = S.rowById.get(it.id);
  if (row) { scrollToRow(row.idx); render(); }
}

async function navigate(key, crumbs, push) {
  const token = ++S.navToken;
  S.rootKey = key; S.crumbs = crumbs;
  S.collapsed = new Set(); S.expanded = new Set(); S.selId = null;
  S.projects = new Map(); S.peek = new Map(); S.src = new Map(); S.linkItem = new Map(); S.idItem = new Map(); S.rowById = new Map();
  S.rawMin = null; S.rangeSet = false; S.needFit = true; S.boardCache = new Map();
  hideDetail();
  if (push) { history.pushState({ crumbs }, '', '?root=' + urlParam(key) + (TODAY_Q || TODAY_ISO !== REAL_ISO ? '&today=' + encodeURIComponent(TODAY_ISO) : '')); syncFilterUrl(); }
  banner('');
  S.rows = []; updateToolbar(); render();
  const en = await loadProject(key);
  if (token !== S.navToken) return;
  if (en.status === 'error') { banner(`Could not load ${key}: ${en.err}`); updateToolbar(); return; }
  rebuild();
  els.rbody.scrollTop = 0;
}

function rootFromLocation() {
  const q = new URLSearchParams(location.search).get('root');
  const dflt = 'examples/lunar/hq/lunar-programme.md';
  if (!q) return dflt;
  try { return keyOf(new URL(q, SITE).href); } catch (e) { banner('Ignoring invalid root: only http(s) URLs and relative paths are allowed.'); return dflt; }
}

// ---------- events ----------
function rowFrom(ev) {
  const r = ev.target.closest('[data-i]');
  return r ? S.rows[+r.dataset.i] : null;
}
function onClick(ev) {
  const row = rowFrom(ev);
  if (!row || row.loading) return;
  const act = ev.target.closest('[data-act]');
  if (act && act.dataset.act === 'tog') { toggle(row); return; }
  if (act && act.dataset.act === 'focus') { focusRow(row); return; }
  if ((ev.ctrlKey || ev.metaKey) && row.it.kind === 'subproject') { ev.preventDefault(); focusRow(row); return; }
  selectRow(row, false);
}
function onDbl(ev) {
  if (ev.target.closest('[data-act]')) return;
  const row = rowFrom(ev);
  if (row && !row.loading) focusRow(row);
}
[els.lspacer, els.trows].forEach((e) => { e.addEventListener('click', onClick); e.addEventListener('dblclick', onDbl); });
els.detail.addEventListener('click', (e) => {
  if (e.target.id === 'dclose') hideDetail();
  const md = e.target.closest('[data-md]');
  if (md) { e.preventDefault(); openMd(md.dataset.md, +md.dataset.line || 0); return; }
  const a = e.target.closest('[data-sel]');
  if (a) { e.preventDefault(); const it = findItem(a.dataset.sel); if (it) selectItem(it); }
});
function findItem(id) {
  for (const en of S.projects.values()) {
    if (en.status !== 'ok') continue;
    const st = [en.root];
    while (st.length) { const x = st.pop(); if (x.id === id) return x; st.push(...x.children); }
  }
  return null;
}

// hover: highlight the row in both table and chart; tooltip on bars
function setHover(i) {
  if (i === S.hover) return;
  S.hover = i;
  for (const e of document.querySelectorAll('.hov')) e.classList.remove('hov');
  if (i >= 0) for (const e of document.querySelectorAll(`[data-i="${i}"]`)) e.classList.add('hov');
}
[els.lspacer, els.trows].forEach((e) => {
  e.addEventListener('mouseover', (ev) => { const r = ev.target.closest('[data-i]'); setHover(r ? +r.dataset.i : -1); });
  e.addEventListener('mouseleave', () => { setHover(-1); els.tip.hidden = true; });
});
els.trows.addEventListener('mousemove', (ev) => {
  const t = ev.target.closest('[data-tip]');
  if (!t) { els.tip.hidden = true; return; }
  const lines = t.dataset.tip.split('\n');
  els.tip.innerHTML = lines.map((l, i) => (l.startsWith('! ') ? `<div class="r">${esc(l.slice(2))}</div>` : i === 0 ? `<b>${esc(l)}</b>` : `<div>${esc(l)}</div>`)).join('');
  els.tip.hidden = false;
  const w = els.tip.offsetWidth, h = els.tip.offsetHeight;
  els.tip.style.left = Math.min(ev.clientX + 14, innerWidth - w - 8) + 'px';
  els.tip.style.top = (ev.clientY + 18 + h > innerHeight ? ev.clientY - h - 10 : ev.clientY + 18) + 'px';
});

// markdown view of a project file
let mdKey = null;
async function openMd(key, line) {
  mdKey = key;
  els.mdtitle.textContent = key; els.mdraw.removeAttribute('href');
  els.mdopen.hidden = !/\.md$/i.test(key.split('?')[0]);
  els.mdbody.innerHTML = '<p class="dim">Loading&hellip;</p>'; els.mdview.hidden = false;
  try {
    els.mdraw.href = urlOf(key);
    const text = await fetchText(key);
    if (mdKey !== key) return;
    els.mdbody.innerHTML = renderMarkdown(text);
    if (line) {
      let best = null;
      for (const e of els.mdbody.querySelectorAll('[data-line]')) if (+e.dataset.line <= line) best = e; else break;
      if (best) { best.classList.add('hl'); best.scrollIntoView({ block: 'center' }); }
    } else els.mdbody.scrollTop = 0;
  } catch (e) { els.mdbody.innerHTML = `<p class="dim">Could not load: ${esc(e.message)}</p>`; }
}
function closeMd() { els.mdview.hidden = true; mdKey = null; }
els.mdclose.onclick = closeMd;
els.mdopen.onclick = () => { const k = mdKey; closeMd(); if (k && k !== S.rootKey) navigate(k, [...S.crumbs, S.rootKey], true); };
els.mdbody.addEventListener('click', (e) => {
  const a = e.target.closest('a[data-md]'); if (!a) return;
  e.preventDefault();
  const [p, hash] = a.dataset.md.split('#');
  if (p) openMd(resolveKey(mdKey, p), 0); else if (hash) { const h = document.getElementById(hash); if (h) h.scrollIntoView(); }
});

// keyboard: arrows move, Enter/Space toggle, Left/Right collapse/expand, Esc closes details
document.addEventListener('keydown', (e) => {
  if (e.key === '/' && !e.altKey && !e.ctrlKey && !e.metaKey && els.mdview.hidden && !e.target.closest('input, textarea, select')) {
    e.preventDefault(); els.qsearch.focus(); els.qsearch.select(); return;
  }
  if (e.target.closest('input, textarea, select')) return;
  if (!els.mdview.hidden) { if (e.key === 'Escape') closeMd(); return; }
  if (e.key === 'Escape') { hideDetail(); return; }
  if (!e.altKey && !e.ctrlKey && !e.metaKey && (e.key === 'g' || e.key === 'b')) { e.preventDefault(); setView(e.key === 'b' ? 'board' : 'gantt'); return; }
  if (S.view === 'board') return;   // the table's arrow keys do not apply to the board
  if (e.altKey || e.ctrlKey || e.metaKey || e.target.closest('button, a')) return;
  const rows = S.rows;
  const cur = rows.findIndex((r) => r.it && r.it.id === S.selId);
  const row = cur >= 0 ? rows[cur] : null;
  const step = (from, d) => { for (let i = from + d; i >= 0 && i < rows.length; i += d) if (!rows[i].loading) return i; return -1; };
  const go = (i) => { if (i >= 0) selectRow(rows[i], true); };
  switch (e.key) {
    case 'ArrowDown': go(cur < 0 ? step(-1, 1) : step(cur, 1)); break;
    case 'ArrowUp': go(cur < 0 ? step(rows.length, -1) : step(cur, -1)); break;
    case 'Home': go(step(-1, 1)); break;
    case 'End': go(step(rows.length, -1)); break;
    case 'ArrowRight':
      if (!row) return;
      if (isExpandable(row) && !isOpen(row)) toggle(row);
      else if (isExpandable(row) && rows[cur + 1] && rows[cur + 1].depth > row.depth) go(step(cur, 1));
      break;
    case 'ArrowLeft': {
      if (!row) return;
      if (isExpandable(row) && isOpen(row)) toggle(row);
      else { let i = cur - 1; while (i >= 0 && (rows[i].loading || rows[i].depth >= row.depth)) i--; go(i); }
      break;
    }
    case 'i': if (row) selectRow(row, true); break;
    case 'Enter': case ' ':
      if (!row) return;
      if (isExpandable(row)) toggle(row); else selectRow(row, true);
      break;
    default: return;
  }
  e.preventDefault();
});

els.rbody.addEventListener('scroll', schedule);
els.lbody.addEventListener('wheel', (e) => { e.preventDefault(); els.rbody.scrollTop += e.deltaY; els.rbody.scrollLeft += e.deltaX; }, { passive: false });
els.rbody.addEventListener('wheel', (e) => {
  if (!(e.ctrlKey || e.metaKey)) return;
  e.preventDefault();
  zoom(e.deltaY < 0 ? 1.15 : 1 / 1.15, e.clientX - els.rbody.getBoundingClientRect().left);
}, { passive: false });
window.addEventListener('resize', schedule);
els.zin.onclick = () => zoom(1.4); els.zout.onclick = () => zoom(1 / 1.4); els.zfit.onclick = fit;
els.zoomseg.addEventListener('click', (e) => { const b = e.target.closest('[data-z]'); if (b) zoom(ZOOMS[b.dataset.z] / S.ppd); });
els.ztoday.onclick = () => { els.rbody.scrollLeft = dx(TODAY) - els.rbody.clientWidth / 3; render(); };
els.tplay.onclick = togglePlay;
els.tsl.addEventListener('input', () => { stopPlay(); setToday(+els.tsl.value); });
els.treset.onclick = () => { stopPlay(); setToday(toDay(BASE_ISO)); };
els.wbs.onchange = () => { S.wbs = els.wbs.checked; render(); };
els.openform.onsubmit = (e) => {
  e.preventDefault();
  const v = els.rootinput.value.trim(); if (!v) return;
  let key;
  try { key = keyOf(new URL(v, SITE).href); } catch (err) { banner('Invalid location: only http(s) URLs and relative paths are allowed.'); return; }
  if (key !== S.rootKey) navigate(key, [...S.crumbs, S.rootKey], true);
};
els.crumbs.addEventListener('click', (e) => {
  const a = e.target.closest('[data-c]'); if (!a) return;
  const i = +a.dataset.c;
  navigate(S.crumbs[i], S.crumbs.slice(0, i), true);
});
window.addEventListener('popstate', (e) => navigate(rootFromLocation(), (e.state && e.state.crumbs) || [], false));

// splitter
(function () {
  const saved = +localStorage.getItem('pmd.leftw'); if (saved) els.main.style.setProperty('--leftw', saved + 'px');
  let drag = false;
  els.split.addEventListener('mousedown', (e) => { drag = true; els.split.classList.add('drag'); e.preventDefault(); });
  window.addEventListener('mousemove', (e) => {
    if (!drag) return;
    const r = els.main.getBoundingClientRect();
    const w = Math.min(r.width - 200, Math.max(160, e.clientX - r.left));
    els.main.style.setProperty('--leftw', w + 'px'); schedule();
  });
  window.addEventListener('mouseup', () => {
    if (!drag) return; drag = false; els.split.classList.remove('drag');
    try { localStorage.setItem('pmd.leftw', parseInt(getComputedStyle(els.left).width, 10)); } catch (e) { /* ignore */ }
  });
})();

// ---------- boot ----------
(async function boot() {
  try { await loadPmd(new URL('./pmd.wasm' + V, import.meta.url)); }
  catch (e) { banner('Could not load pmd.wasm (' + e.message + '). Build it with parser/build-wasm.sh and serve the repo root, e.g. python3 -m http.server 8642.'); return; }
  history.replaceState({ crumbs: (history.state && history.state.crumbs) || [] }, '');
  setView(S.view);
  navigate(rootFromLocation(), (history.state && history.state.crumbs) || [], false).then(restoreSel);
})();
