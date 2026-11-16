// projectmd Gantt viewer: view-only. See README.md.
// Cache-busting: bump v in index.html (the only place) and every module/wasm fetch follows.
const V = new URL(import.meta.url).search;
const { loadPmd, parseProject } = await import('./pmd.js' + V);
const { renderMarkdown } = await import('./md.js' + V);

const RH = 28;            // row height (px), must match --rh
const HH = 22;            // header tier height
const DAY = 86400000;
const $ = (id) => document.getElementById(id);
const els = {};
['ptitle', 'pmeta', 'crumbs', 'openform', 'rootinput', 'wbs', 'zout', 'zin', 'zfit', 'ztoday', 'zoomseg', 'banner', 'main', 'left', 'lbody', 'lspacer',
  'split', 'rhead', 'rhinner', 'rbody', 'canvas', 'gridlines', 'today', 'trows', 'deps', 'detail', 'tip', 'mdview', 'mdtitle', 'mdraw', 'mdopen', 'mdclose', 'mdbody'].forEach((i) => (els[i] = $(i)));

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
const TODAY_ISO = /^\d{4}-\d{2}-\d{2}$/.test(TODAY_Q || '') ? TODAY_Q : `${now.getFullYear()}-${p2(now.getMonth() + 1)}-${p2(now.getDate())}`;
const TODAY = toDay(TODAY_ISO);
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
  u.hash = '';
  return u.href.startsWith(SITE.href) ? u.href.slice(SITE.href.length) : u.href;
}
const urlOf = (key) => new URL(key, SITE).href;
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
};

// ---------- loading ----------
async function fetchText(key) {
  const r = await fetch(urlOf(key), { cache: 'no-cache' });
  if (!r.ok) throw new Error(`HTTP ${r.status} for ${key}`);
  return r.text();
}

// Direct links are read live (one hop). A link that cannot be read is unknown (null).
async function peekLink(key) {
  if (S.peek.has(key) || S.projects.has(key)) return;
  S.peek.set(key, null);
  try { const src = await fetchText(key); const p = parseProject({ path: key, src, today: TODAY_ISO, interfaces: {} }); if (!p.error) { S.peek.set(key, p); S.src.set(key, src); } } catch (e) { /* unknown */ }
}
const linkTargets = (proj) => { const out = new Set(); (function w(ns) { for (const n of ns) { if (n.link_resolved && (n.kind === 'subproject' || n.kind === 'deliverable')) out.add(keyOf(urlOf(n.link_resolved.split('#')[0]))); w(n.children || []); } })(proj.nodes || []); return out; };
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
      it.target = t ? keyOf(urlOf(t.split('#')[0])) : null;
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
  if (!rootEn || rootEn.status !== 'ok') { S.rows = rows; S.deps = []; return; }

  function addRow(it, depth, num) {
    const row = { it, depth, num, idx: rows.length, s: null, e: null, vk: 'task' };
    rows.push(row);
    S.rowById.set(it.id, row);
    return row;
  }
  function visibleChildren(it) {
    if (it.kind === 'subproject') {
      if (!S.expanded.has(it.id) || !it.target) return null;
      const en = S.projects.get(it.target);
      if (!en) return null;
      if (en.status === 'loading') return 'loading';
      if (en.status === 'error') return null;
      return en.root.children;
    }
    if (S.collapsed.has(it.id)) return null;
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
      else if (vc) sub = walk(vc, depth + 1, num);
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
function delivLabel(it) {
  const n = it.node, t = n.link_resolved ? keyOf(urlOf(n.link_resolved.split('#')[0])) : null;
  const kp = t && known(t);
  const proj = kp && kp.title ? kp.title : t ? baseName(t).replace(/\.md$/i, '') : '';
  return proj ? `${proj} › ${n.link ? n.link.text : n.title}` : n.title;
}
const contractOf = (it) => (it.node && (it.kind === 'subproject' || it.kind === 'deliverable') && it.entry.proj.contracts && it.entry.proj.contracts[it.node.line]) || '';
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
const isOpen = (row) => (row.it.kind === 'subproject' ? S.expanded.has(row.it.id) : !S.collapsed.has(row.it.id));

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
    const pill = ct ? `<span class="ct ct-${ct}" title="Contract: ${ct}">${ct}</span>` : '';
    const flag = why.length ? `<span class="flag" title="${esc(why.join('\n'))}">!</span>` : '';
    const num = S.wbs && row.num ? `<span class="num">${row.num}</span>` : '';
    const focus = it.kind === 'subproject' && it.target ? '<button class="focus" data-act="focus" title="Open as top level (Ctrl/Cmd-click or double-click)">Open</button>' : '';
    const aria = `role="row" aria-level="${row.depth + 1}" aria-selected="${!!sel}"${isExpandable(row) ? ` aria-expanded="${isOpen(row)}"` : ''}`;
    L += `<div class="lrow ${cls}" ${aria} data-i="${i}" style="top:${i * RH}px"><div class="name" style="padding-left:${row.depth * 16 + 6}px">${tog}${ico}<span class="t" title="${esc(titleOf(it))}">${num}${esc(titleOf(it))}</span>${pill}${flag}${focus}</div>` +
      `<div class="d">${esc(fmt(row.s))}</div><div class="d fin">${esc(fmt(row.e))}</div><div>${durText(row)}</div><div title="${esc(ownerText(row))}">${esc(ownerText(row))}</div></div>`;
    T += `<div class="trow${sel}${odd}${hov}" data-i="${i}" style="top:${i * RH}px">${bars(row)}</div>`;
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
    case 'sum':
      out = `<div class="sum${row.conflict ? ' conf' : ''}${mism}" style="left:${x1}px;width:${w}px"${tip}></div>`;
      break;
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
    add('Duration', durText(row));
    if (n.kind === 'heading') add('Items', String(n.children.length));
    const k = n.keys || {};
    if (k.after) add('After', k.after.map((v) => {
      const tg = afterTarget(en.key, v), t = tg && itemOfTarget(tg);
      const label = esc(v.text);
      return t ? `<a href="#" data-sel="${esc(t.id)}" title="Select ${esc(titleOf(t))}">${label}</a>` : `<span title="Not loaded">${label}</span>`;
    }).join(', '));
    add('By', vals(k.by));
    add('Contract', contractOf(it) ? `<span class="ct ct-${contractOf(it)}">${contractOf(it)}</span>` : '');
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
  html += `<dl>${dl.join('')}</dl>`;
  html += `<h3>Location</h3><div><a href="#" data-md="${esc(en.key)}"${line ? ` data-line="${line}"` : ''}>${esc(en.key)}${line ? ':' + line : ''}</a></div>`;
  if (diags.length) html += '<h3>Diagnostics</h3>' + diags.map((d) => `<div class="diag ${esc(d.severity)}"><b>${esc(d.code)}</b>${d.line ? ' (line ' + d.line + ')' : ''}<br>${esc(d.message)}</div>`).join('');
  els.detail.innerHTML = html;
  if (els.detail.hidden) { els.detail.hidden = false; schedule(); }
}
function hideDetail() { if (!els.detail.hidden) { els.detail.hidden = true; schedule(); } S.detailOpen = false; }

// ---------- actions ----------
function rebuild() {
  flatten();
  if (S.rows.length) {
    if (S.rawMin == null) { S.rawMin = TODAY; S.rawMax = TODAY; }
    applyRange();
    if (S.needFit) { fit(); S.needFit = false; }
  }
  updateToolbar();
  render();
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
  S.rawMin = null; S.rangeSet = false; S.needFit = true;
  hideDetail();
  if (push) history.pushState({ crumbs }, '', '?root=' + urlParam(key) + (TODAY_Q ? '&today=' + encodeURIComponent(TODAY_ISO) : ''));
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
  return q ? keyOf(new URL(q, SITE).href) : 'examples/lunar/hq/lunar-programme.md';
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
  els.mdtitle.textContent = key; els.mdraw.href = urlOf(key);
  els.mdopen.hidden = !/\.md$/i.test(key.split('?')[0]);
  els.mdbody.innerHTML = '<p class="dim">Loading&hellip;</p>'; els.mdview.hidden = false;
  try {
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
  if (e.target.closest('input, textarea, select')) return;
  if (!els.mdview.hidden) { if (e.key === 'Escape') closeMd(); return; }
  if (e.key === 'Escape') { hideDetail(); return; }
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
els.wbs.onchange = () => { S.wbs = els.wbs.checked; render(); };
els.openform.onsubmit = (e) => {
  e.preventDefault();
  const v = els.rootinput.value.trim(); if (!v) return;
  const key = keyOf(new URL(v, SITE).href);
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
  navigate(rootFromLocation(), (history.state && history.state.crumbs) || [], false);
})();
