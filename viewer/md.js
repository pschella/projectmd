// Tiny markdown renderer for the projectmd file view. No HTML from the source is ever passed through.
// Block elements carry data-line (1-based source line) so the viewer can jump to and highlight one.
const esc = (s) => String(s).replace(/[&<>"']/g, (c) => ({ '&': '&amp;', '<': '&lt;', '>': '&gt;', '"': '&quot;', "'": '&#39;' }[c]));

// Only http(s), relative paths and #fragments survive; anything else (javascript:, data:, vbscript:, mailto:...) becomes '#'.
function safeUrl(u) {
  u = u.replace(/[\u0000-\u0020\u007f-\u009f\u200b-\u200f\u2028\u2029\ufeff]/g, '');   // browsers ignore these inside a URL
  if (/^[a-z][a-z0-9+.-]*:/i.test(u)) return /^https?:/i.test(u) ? u : '#';
  if (/^[^/?#]*:/.test(u)) return '#';                  // "scheme-like" first segment such as "&colon;" tricks
  return u;
}
function anchor(text, url, refs) {
  const href = safeUrl(url);
  const isMd = /^[^:]*?\.md(#.*)?$/i.test(href) || /^https?:[^#?]*\.md(#.*)?$/i.test(href);
  return `<a href="${esc(href)}"${isMd ? ` data-md="${esc(href)}"` : ' target="_blank" rel="noopener noreferrer"'}>${inline(text, refs)}</a>`;
}
const INL = /`([^`]+)`|\[([^\]]*)\]\(([^)\s]+)(?:\s+"[^"]*")?\)|\[([^\]]+)\]\[([^\]]*)\]|\*\*(.+?)\*\*|(?<![\w*])\*(?![\s*])(.+?)\*(?![\w*])|<(https?:[^>\s]+)>|\[([^\]]+)\](?![(\[])/g;
function inline(t, refs) {
  let out = '', last = 0, m;
  const re = new RegExp(INL.source, 'g');   // own state: inline() recurses
  while ((m = re.exec(t))) {
    out += esc(t.slice(last, m.index)); last = re.lastIndex;
    if (m[1] != null) out += `<code>${esc(m[1])}</code>`;
    else if (m[3] != null) out += anchor(m[2], m[3], refs);
    else if (m[4] != null) { const r = refs[(m[5] || m[4]).toLowerCase()]; out += r ? anchor(m[4], r, refs) : esc(m[0]); }
    else if (m[6] != null) out += `<strong>${inline(m[6], refs)}</strong>`;
    else if (m[7] != null) out += `<em>${inline(m[7], refs)}</em>`;
    else if (m[8] != null) out += anchor(m[8], m[8], refs);
    else { const r = refs[m[9].toLowerCase()]; out += r ? anchor(m[9], r, refs) : esc(m[0]); }
  }
  return out + esc(t.slice(last));
}

const ITEM = /^(\s*)([-*+]|\d+[.)])\s+(?:\[([ xX-])\]\s+)?(.*)$/;
const CB = { ' ': 'open', x: 'done', X: 'done', '-': 'cancelled' };

export function renderMarkdown(src) {
  const lines = src.replace(/\r\n?/g, '\n').split('\n');
  const refs = {};
  const body = [];
  for (let i = 0; i < lines.length; i++) {          // reference definitions are not shown
    const m = /^\s{0,3}\[([^\]]+)\]:\s*(\S+)/.exec(lines[i]);
    if (m && !/^\s*[-*+]\s/.test(lines[i])) { refs[m[1].toLowerCase()] = m[2].replace(/^<|>$/g, ''); body.push(null); } else body.push(lines[i]);
  }
  let h = '';
  const blank = (l) => l == null || !l.trim();
  for (let i = 0; i < body.length;) {
    const l = body[i], ln = i + 1;
    if (blank(l)) { i++; continue; }
    let m;
    if ((m = /^\s*(```|~~~)/.exec(l))) {                // fenced code
      const f = m[1]; const code = []; i++;
      while (i < body.length && !(body[i] || '').trim().startsWith(f)) code.push(lines[i++]);
      i++; h += `<pre data-line="${ln}"><code>${esc(code.join('\n'))}</code></pre>`; continue;
    }
    if ((m = /^(#{1,6})\s+(.*?)\s*#*\s*$/.exec(l))) { h += `<h${m[1].length} data-line="${ln}">${inline(m[2], refs)}</h${m[1].length}>`; i++; continue; }
    if (/^\s*([-*_])(\s*\1){2,}\s*$/.test(l)) { h += `<hr data-line="${ln}">`; i++; continue; }
    if (/^\s*>/.test(l)) {
      const q = []; while (i < body.length && /^\s*>/.test(body[i] || '')) q.push(body[i++].replace(/^\s*>\s?/, ''));
      h += `<blockquote data-line="${ln}">${inline(q.join(' '), refs)}</blockquote>`; continue;
    }
    if (l.includes('|') && /^\s*\|?\s*:?-{2,}:?\s*(\|\s*:?-{2,}:?\s*)*\|?\s*$/.test(body[i + 1] || '')) {
      const cells = (r) => r.trim().replace(/^\||\|$/g, '').split('|').map((c) => c.trim());
      h += `<table data-line="${ln}"><thead><tr>${cells(l).map((c) => `<th>${inline(c, refs)}</th>`).join('')}</tr></thead><tbody>`;
      i += 2;
      for (; i < body.length && !blank(body[i]) && body[i].includes('|'); i++) h += `<tr data-line="${i + 1}">${cells(body[i]).map((c) => `<td>${inline(c, refs)}</td>`).join('')}</tr>`;
      h += '</tbody></table>'; continue;
    }
    if (ITEM.test(l)) {                                  // nested list, by indentation
      const items = [];
      while (i < body.length && !blank(body[i])) {
        const im = ITEM.exec(body[i]);
        if (im) items.push({ ind: im[1].replace(/\t/g, '    ').length, ord: /\d/.test(im[2]), cb: im[3], text: im[4], line: i + 1 });
        else if (items.length && /^\s+\S/.test(body[i])) items[items.length - 1].text += ' ' + body[i].trim();
        else break;
        i++;
      }
      h += list(items, refs); continue;
    }
    const p = [];                                        // paragraph
    while (i < body.length && !blank(body[i]) && !/^(#{1,6}\s|\s*(```|~~~)|\s*>)/.test(body[i]) && !ITEM.test(body[i])) p.push(body[i++].trim());
    h += `<p data-line="${ln}">${inline(p.join(' '), refs)}</p>`;
  }
  return h;
}

function list(items, refs) {
  let h = '', stack = [];                                // stack of {ind, tag}
  const close = () => { const s = stack.pop(); h += `</li></${s.tag}>`; };
  for (const it of items) {
    while (stack.length && it.ind < stack[stack.length - 1].ind) close();
    const top = stack[stack.length - 1];
    if (top && it.ind === top.ind) h += '</li>';
    else if (!top || it.ind > top.ind) { const tag = it.ord ? 'ol' : 'ul'; h += `<${tag}>`; stack.push({ ind: it.ind, tag }); }
    const box = it.cb ? `<span class="cb ${CB[it.cb]}"></span>` : '';
    h += `<li data-line="${it.line}"${it.cb ? ` class="task ${CB[it.cb]}"` : ''}>${box}${inline(it.text, refs)}`;
  }
  while (stack.length) close();
  return h;
}
