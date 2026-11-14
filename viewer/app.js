// projectmd Gantt viewer: view-only. See README.md.
import { loadPmd, parseProject, parseLock } from './pmd.js';

const RH = 26;            // row height (px), must match --rh
const HH = 22;            // header tier height
const DAY = 86400000;
const $ = (id) => document.getElementById(id);
const els = {};
['ptitle', 'pmeta', 'crumbs', 'openform', 'rootinput', 'wbs', 'zout', 'zin', 'zfit', 'ztoday', 'banner', 'main', 'left', 'lbody', 'lspacer',
  'split', 'rhead', 'rhinner', 'rbody', 'canvas', 'gridlines', 'today', 'trows', 'deps', 'detail'].forEach((i) => (els[i] = $(i)));

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
const TODAY_ISO = `${now.getFullYear()}-${p2(now.getMonth() + 1)}-${p2(now.getDate())}`;
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
// A "key" is the URL pathname for same-origin files, else the full URL (without hash).
function keyOf(href) {
  const u = new URL(href, location.href);
  u.hash = '';
  return u.origin === location.origin ? u.pathname + u.search : u.href;
}
const urlOf = (key) => new URL(key, location.origin + '/').href;
function resolveKey(baseKey, rel) {
  try { return keyOf(new URL(rel, urlOf(baseKey)).href); } catch (e) { return rel; }
}
const baseName = (k) => decodeURIComponent(String(k).split('?')[0].split('/').filter(Boolean).pop() || k);
function urlParam(k) { return encodeURIComponent(k).replace(/%2F/g, '/').replace(/%3A/g, ':'); }

// ---------- state ----------
const S = {
  rootKey: null, crumbs: [], profile: null,
  lock: new Map(),          // key -> lock entry (with _depth)
  metaDirs: new Set(),
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

async function loadMeta(fileKey) {
  const u = new URL(urlOf(fileKey));
  const parts = u.pathname.split('/'); parts.pop();
  const jobs = [];
  for (;;) {
    const dirPath = parts.join('/') + '/';
    const dirUrl = new URL(dirPath, u);
    const dirKey = keyOf(dirUrl.href);
    if (!S.metaDirs.has(dirKey)) {
      S.metaDirs.add(dirKey);
      jobs.push(loadLockFrom(dirKey, dirPath.length));
      if (!S.profile) jobs.push(fetchText(dirKey + '.pmd/profile.md').then((t) => { if (!S.profile) S.profile = t; }).catch(() => {}));
    }
    if (parts.length <= 1) break;
    parts.pop();
  }
  await Promise.all(jobs);
}

async function loadLockFrom(dirKey, depth) {
  try {
    const lockKey = dirKey + 'pmd.lock';
    const text = await fetchText(lockKey);
    const res = parseLock(text);
    for (const e of res.entries || []) {
      const k = resolveKey(lockKey, e.location);
      const old = S.lock.get(k);
      if (!old || old._depth < depth) S.lock.set(k, Object.assign({}, e, { _depth: depth }));
    }
  } catch (e) { /* no lock here */ }
}

function buildInterfaces() {
  const out = {};
  for (const [k, e] of S.lock) {
    const i = { milestones: {} };
    const c = dateText(e.commitment); if (c) i.commitment = c;
    if (e.start) i.start = e.start;
    for (const [m, v] of Object.entries(e.milestones || {})) { const t = dateText(v); if (t) i.milestones[m] = t; }
    out[k] = i;
  }
  for (const [k, en] of S.projects) {
    if (en.status !== 'ok') continue;
    const i = { milestones: {} };
    const lk = S.lock.get(k); if (lk && lk.start) i.start = lk.start;
    if (en.proj.commitment && en.proj.commitment.text) i.commitment = en.proj.commitment.text;
    (function w(ns) { for (const n of ns) { if (n.id && (n.tags || []).includes('public') && n.date && n.date.text) i.milestones[n.id] = n.date.text; w(n.children || []); } })(en.proj.nodes || []);
    out[k] = i;
  }
  return out;
}

function mkItems(nodes, entry, parent) {
  return nodes.map((node, i) => {
    const it = { id: `${entry.key}:${node.line}:${i}`, node, entry, parent, kind: node.kind, children: [] };
    if (node.kind === 'subproject') {
      const t = node.link_resolved || (node.link && resolveKey(entry.key, node.link.target)) || null;
      it.target = t ? keyOf(urlOf(t.split('#')[0])) : null;
    }
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
      const src = await fetchText(key);
      await loadMeta(key);
      const req = { path: key, src, today: TODAY_ISO, interfaces: buildInterfaces() };
      if (S.profile) req.profile = S.profile;
      const proj = parseProject(req);
      if (proj.error) throw new Error(proj.error);
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
  const en = S.projects.get(key);
  if (en && en.status === 'ok' && en.proj.commitment) return exprSpan(en.proj.commitment);
  const l = S.lock.get(key);
  if (!l) return null;
  const c = exprSpan(l.commitment); const st = toDay(l.start);
  if (c && st != null && st < c.s) return { ...c, s: st };
  return c;
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
  if (n.kind === 'milestone') { const x = e ?? s; return x == null ? null : { s: x, e: x }; }
  if (s == null && e == null) return null;
  if (s == null) s = dur ? e - dur + 1 : e;
  if (e == null) e = dur ? s + dur - 1 : s;
  return { s, e };
}

// ---------- flatten ----------
function flatten() {
  const rows = [];
  const idRow = new Map(), fileRow = new Map();
  const rootEn = S.projects.get(S.rootKey);
  if (!rootEn || rootEn.status !== 'ok') { S.rows = rows; S.deps = []; return; }

  function addRow(it, depth, num) {
    const row = { it, depth, num, idx: rows.length, s: null, e: null, vk: 'task' };
    rows.push(row);
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
      else if (it.kind === 'milestone') { row.vk = 'ms'; span = own; }
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
          const l = it.target && S.lock.get(it.target);
          if (l && !(en && en.status === 'ok')) {
            row.ifaceMs = [];
            for (const [mid, v] of Object.entries(l.milestones || {})) { const sp = exprSpan(v); if (sp) row.ifaceMs.push({ id: mid, d: sp.e }); }
          }
        }
      } else { // task
        if (!own && sub) { row.vk = 'sum'; span = sub; } else { row.vk = 'task'; span = own; }
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
      let file = row.it.entry.key, id = null;
      const t = v.kind === 'link' ? v.target : v.text;
      if (!t) continue;
      if (t[0] === '^') id = t.slice(1);
      else { const [f, h] = t.split('#'); if (f) file = resolveKey(row.it.entry.key, f); id = h || null; }
      const pred = (id && idRow.get(file + '#' + id)) || fileRow.get(file);
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
function tierTypes() {
  const p = S.ppd;
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
function titleOf(it) {
  if (it.kind === 'project') return it.entry.proj.title;
  const n = it.node;
  if (n.kind === 'subproject' && n.link && n.link.text) return n.link.text;
  return n.title;
}
function durText(row) {
  if (row.s == null) return '';
  if (row.vk === 'ms') return '0d';
  const d = Math.round(row.e - row.s + 1);
  return d >= 14 && d % 7 === 0 ? d / 7 + 'w' : d + 'd';
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
  const tt = tierTypes();
  let h = '', g = '';
  tt.forEach(([type, wy], ti) => {
    for (const sg of segs(type, d0, d1, wy)) {
      const x = dx(sg.s), w = (sg.e - sg.s) * S.ppd;
      let label = sg.label;
      if (w < 14) label = ''; else if (w < 30 && sg.short) label = sg.short; else if (w < 34 && label.length > 2) label = '';
      h += `<div class="hs t${ti}" style="left:${x}px;width:${w}px">${esc(label)}</div>`;
      g += `<div class="gl${ti === 0 ? ' major' : ''}" style="left:${x}px"></div>`;
    }
  });
  els.rhinner.innerHTML = h; els.gridlines.innerHTML = g;
  els.today.style.left = dx(TODAY + 0.5) - 1 + 'px';

  // rows
  let L = '', T = '';
  for (let i = r0; i < r1; i++) {
    const row = rows[i];
    const odd = i & 1 ? ' odd' : '';
    if (row.loading) {
      L += `<div class="lrow loadrow${odd}" style="top:${i * RH}px"><div class="name" style="padding-left:${row.depth * 16 + 22}px"><span class="spin">&#8635;</span> Loading&hellip;</div><div></div><div></div><div></div><div></div></div>`;
      T += `<div class="trow${odd}" style="top:${i * RH}px"></div>`;
      continue;
    }
    const it = row.it, n = it.node;
    const sel = S.selId === it.id ? ' sel' : '';
    const cb = n ? n.checkbox : '';
    const cls = `k-${it.kind}${cb === 'done' ? ' done' : ''}${cb === 'cancelled' ? ' cancelled' : ''}${row.conflict ? ' conflict' : ''}${sel}${odd}`;
    // toggle
    let tog = '<span class="tog none"></span>', hasKids = false;
    if (it.kind === 'subproject') {
      const en = row.sub, ex = S.expanded.has(it.id);
      hasKids = !!it.target;
      if (hasKids) tog = `<span class="tog" data-act="tog">${ex && en && en.status === 'loading' ? '<span class="spin">&#8635;</span>' : ex ? '&#9662;' : '&#9656;'}</span>`;
    } else if (it.children.length) {
      tog = `<span class="tog" data-act="tog">${S.collapsed.has(it.id) ? '&#9656;' : '&#9662;'}</span>`;
    }
    let ico = '';
    if (it.kind === 'milestone') ico = '<span class="ico">&#9670;</span>';
    else if (row.failed) ico = `<span class="ico warn" title="${esc(row.sub.err)}">&#9888;</span>`;
    else if (it.kind === 'subproject') ico = `<span class="ico sub" title="${row.sub && row.sub.status === 'ok' ? 'Loaded sub-project' : 'Not loaded yet (commitment from pmd.lock)'}">${row.sub && row.sub.status === 'ok' ? '&#9635;' : '&#8615;'}</span>`;
    const num = S.wbs && row.num ? `<span class="num">${row.num}</span>` : '';
    const focus = it.kind === 'subproject' && it.target ? '<button class="focus" data-act="focus" title="Open as top level (Ctrl/Cmd-click or double-click)">&#8982;</button>' : '';
    const dateCell = (d) => esc(fmt(d));
    L += `<div class="lrow ${cls}" data-i="${i}" style="top:${i * RH}px"><div class="name" style="padding-left:${row.depth * 16 + 4}px">${tog}${ico}<span class="t" title="${esc(titleOf(it))}">${num}${esc(titleOf(it))}</span>${focus}</div>` +
      `<div class="d">${dateCell(row.s)}</div><div class="d fin">${dateCell(row.e)}</div><div>${durText(row)}</div><div title="${esc(ownerText(row))}">${esc(ownerText(row))}</div></div>`;
    T += `<div class="trow ${sel}${odd}" data-i="${i}" style="top:${i * RH}px">${bars(row)}</div>`;
  }
  els.lspacer.innerHTML = L; els.trows.innerHTML = T;
  renderDeps(r0, r1);
}

function bars(row) {
  if (row.s == null) return '';
  const it = row.it, n = it.node, f = (n && n.forecast) || {};
  const x1 = dx(row.s), x2 = dx(row.e + 1), w = Math.max(x2 - x1, 3);
  const cb = n ? n.checkbox : '';
  const title = esc(`${titleOf(it)}\n${fmt(row.s)} → ${fmt(row.e)}`);
  const conf = row.conflict ? ' conf' : '';
  const mism = row.mism ? ' mism' : '';
  let out = '';
  switch (row.vk) {
    case 'ms': {
      const cx = dx(row.e + 0.5);
      out = `<div class="ms${cb === 'done' ? ' done' : ''}${cb === 'cancelled' ? ' cancelled' : ''}${conf}" style="left:${cx}px" title="${title}"></div>` +
        `<span class="mlabel" style="left:${cx + 11}px">${fmt(row.e)}</span>`;
      break;
    }
    case 'task':
      out = `<div class="bar${cb === 'done' ? ' done' : ''}${cb === 'cancelled' ? ' cancelled' : ''}${conf}${mism}" style="left:${x1}px;width:${w}px" title="${title}">${cb === 'done' ? '<i class="prog" style="width:100%"></i>' : ''}</div>`;
      break;
    case 'sum':
      out = `<div class="sum${conf}${mism}" style="left:${x1}px;width:${w}px" title="${title}"></div>`;
      break;
    case 'sub':
      out = `<div class="sub${conf}${mism}${row.openStart ? ' openstart' : ''}" style="left:${x1}px;width:${w}px" title="${title}${row.commit ? '\nCommitment: ' + esc(row.commit.text) : ''}">&#8615;</div>`;
      for (const m of row.ifaceMs || []) out += `<div class="ms mini" style="left:${dx(m.d + 0.5)}px" title="${esc(m.id)}: ${fmt(m.d)}"></div>`;
      break;
    case 'unk':
      out = `<div class="unk" style="left:${x1}px;width:${w}px" title="${title}\nCould not load"></div>`;
      break;
  }
  if (row.conflict && f.declared_end) { const dd = toDay(f.declared_end); if (dd != null) out += `<div class="decl" style="left:${dx(dd + 1)}px" title="Declared end ${esc(f.declared_end)}"></div>`; }
  if (row.mism && row.by != null) out += `<div class="decl" style="left:${dx(row.by + 1)}px;border-top-color:var(--warn)" title="Parent by: ${fmt(row.by)}"></div>`;
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
    els.pmeta.innerHTML = chip('Owner', p.owner && (p.owner.name || p.owner.text)) + chip('State', p.state) + chip('Commitment', p.commitment && p.commitment.text);
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
function showDetail(row) {
  if (!row || row.loading) { els.detail.hidden = true; return; }
  const it = row.it, n = it.node, en = it.entry;
  const dl = [];
  const add = (k, v) => { if (v != null && v !== '') dl.push(`<dt>${esc(k)}</dt><dd>${v}</dd>`); };
  let head, line = null;
  if (it.kind === 'project') {
    const p = en.proj;
    head = `<h2>${esc(p.title)}</h2><span class="badge">project</span>${p.state ? `<span class="badge">${esc(p.state)}</span>` : ''}`;
    add('Owner', esc(p.owner && (p.owner.text || p.owner.name)));
    add('Stakeholders', esc((p.stakeholders || []).map((s) => s.text).join(', ')));
    add('Commitment', esc(p.commitment && p.commitment.text));
    add('ID', esc(p.id));
    if (p.forecast) { add('Forecast', esc(`${p.forecast.start || '?'} → ${p.forecast.end || '?'}`)); }
  } else {
    const f = n.forecast || {};
    const state = n.checkbox === 'done' ? 'done' : n.checkbox === 'cancelled' ? 'cancelled' : n.checkbox === 'open' ? 'open' : '';
    head = `<h2>${esc(titleOf(it))}</h2><span class="badge">${esc(n.kind)}</span>${state ? `<span class="badge">${state}</span>` : ''}` +
      (f.conflict ? '<span class="badge red">conflict</span>' : '') + (row.mism ? '<span class="badge warn">exceeds parent by:</span>' : '');
    line = n.line;
    const decl = [];
    if (n.date) decl.push(n.date.text); if (n.range) decl.push(n.range.text); if (n.duration) decl.push(n.duration.text);
    add('Declared', esc(decl.join('  ')));
    add('Forecast', esc(`${f.start || '?'} → ${f.end || '?'}`));
    if (f.declared_end) add('Declared end', esc(f.declared_end) + (f.conflict ? ' <span class="badge red">forecast later</span>' : ''));
    const k = n.keys || {};
    const vals = (a) => esc((a || []).map((v) => v.text).join(', '));
    add('Owner', vals(k.owner) || esc(ownerText(row)));
    add('Approvers', vals(k.approver));
    add('After', vals(k.after));
    add('By', vals(k.by));
    for (const key of Object.keys(k)) if (!['owner', 'approver', 'after', 'by'].includes(key)) add(key, vals(k[key]));
    add('Tags', esc((n.tags || []).map((t) => '#' + t).join(' ')));
    add('ID', n.id ? '^' + esc(n.id) : '');
    if (it.kind === 'subproject') {
      const c = row.commit;
      add('Commitment', c ? esc(c.text) + (row.sub && row.sub.status === 'ok' ? '' : ' (from pmd.lock)') : '');
      add('Link', it.target ? `<a href="${esc(urlOf(it.target))}" target="_blank" rel="noopener">${esc(it.target)}</a>` : '');
      if (row.sub && row.sub.status === 'ok') { const p = row.sub.proj; add('Owner', esc(p.owner && p.owner.text)); add('State', esc(p.state)); }
      if (row.failed) add('Load error', esc(row.sub.err));
      if (row.mism) add('Bound', `child commitment ${esc(fmt(row.commit.e))} is later than <b>by:</b> ${esc(fmt(row.by))}`);
    }
  }
  const diags = (en.proj.diagnostics || []).filter((d) => (it.kind === 'project' ? true : d.line === line));
  let html = `<button class="x" id="dclose" title="Close">&times;</button>${head}<dl>${dl.join('')}</dl>`;
  html += `<h3>Source</h3><div><a href="${esc(urlOf(en.key))}" target="_blank" rel="noopener">${esc(en.key)}${line ? ':' + line : ''}</a></div>`;
  if (diags.length) {
    html += '<h3>Diagnostics</h3>' + diags.map((d) => `<div class="diag ${esc(d.severity)}"><b>${esc(d.code)}</b>${d.line ? ' (line ' + d.line + ')' : ''}<br>${esc(d.message)}</div>`).join('');
  }
  els.detail.innerHTML = html; els.detail.hidden = false;
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
  render();
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

async function navigate(key, crumbs, push) {
  const token = ++S.navToken;
  S.rootKey = key; S.crumbs = crumbs;
  S.collapsed = new Set(); S.expanded = new Set(); S.selId = null;
  S.rawMin = null; S.rangeSet = false; S.needFit = true;
  els.detail.hidden = true;
  if (push) history.pushState({ crumbs }, '', '?root=' + urlParam(key));
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
  return q ? keyOf(new URL(q, location.origin + '/').href) : '/examples/car/aurora.md';
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
  S.selId = row.it.id; showDetail(row); render();
}
function onDbl(ev) {
  if (ev.target.closest('[data-act]')) return;
  const row = rowFrom(ev);
  if (row && !row.loading) focusRow(row);
}
[els.lspacer, els.trows].forEach((e) => { e.addEventListener('click', onClick); e.addEventListener('dblclick', onDbl); });
els.detail.addEventListener('click', (e) => { if (e.target.id === 'dclose') els.detail.hidden = true; });
document.addEventListener('keydown', (e) => { if (e.key === 'Escape') els.detail.hidden = true; });

els.rbody.addEventListener('scroll', schedule);
els.lbody.addEventListener('wheel', (e) => { e.preventDefault(); els.rbody.scrollTop += e.deltaY; els.rbody.scrollLeft += e.deltaX; }, { passive: false });
els.rbody.addEventListener('wheel', (e) => {
  if (!(e.ctrlKey || e.metaKey)) return;
  e.preventDefault();
  zoom(e.deltaY < 0 ? 1.15 : 1 / 1.15, e.clientX - els.rbody.getBoundingClientRect().left);
}, { passive: false });
window.addEventListener('resize', schedule);
els.zin.onclick = () => zoom(1.4); els.zout.onclick = () => zoom(1 / 1.4); els.zfit.onclick = fit;
els.ztoday.onclick = () => { els.rbody.scrollLeft = dx(TODAY) - els.rbody.clientWidth / 3; render(); };
els.wbs.onchange = () => { S.wbs = els.wbs.checked; render(); };
els.openform.onsubmit = (e) => {
  e.preventDefault();
  const v = els.rootinput.value.trim(); if (!v) return;
  const key = keyOf(new URL(v, location.origin + '/').href);
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
    try { localStorage.setItem('pmd.leftw', parseInt(els.left.style.width || getComputedStyle(els.left).width, 10)); } catch (e) { /* ignore */ }
  });
})();

// ---------- boot ----------
(async function boot() {
  try { await loadPmd(); }
  catch (e) { banner('Could not load pmd.wasm (' + e.message + '). Build it with parser/build-wasm.sh and serve the repo root, e.g. python3 -m http.server 8642.'); return; }
  history.replaceState({ crumbs: (history.state && history.state.crumbs) || [] }, '');
  navigate(rootFromLocation(), (history.state && history.state.crumbs) || [], false);
})();
