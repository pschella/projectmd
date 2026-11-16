/* projectmd site: syntax highlighting, heading anchors, TOC tracking. No dependencies. */
(function () {
  var RULES = [
    ["h", "^#{1,6} .*"], ["k", "^- (?:Owner|Commitment):"], ["c", "\\[[ x-]\\](?=\\s)"],
    ["l", "\\[[^\\]\\n]+\\]\\([^)\\n]+\\)"], ["i", "\\^[\\w-]+"], ["p", "#private\\b"],
    ["k", "\\b(?:owner|after|by|done):"], ["d", "\\b\\d{4}-(?:Q[1-4]|\\d\\d(?:-\\d\\d)?)(?!\\w)"],
    ["u", "(?<![\\w-])\\d+[wdm]\\b"]
  ];
  var RE = new RegExp(RULES.map(function (r) { return "(" + r[1] + ")"; }).join("|"), "gm");
  var LANG = /(?:^|\s)language-(?:markdown|md|projectmd)\b/, TOKENS = /\[[ x-]\]|^# |\b(?:owner|after|by|done):|\^[\w-]+/m;

  function highlight(code) {
    var text = code.textContent, out = document.createDocumentFragment(), last = 0, m;
    RE.lastIndex = 0;
    while ((m = RE.exec(text))) {
      if (!m[0]) { RE.lastIndex++; continue; }
      var idx = 1; while (!m[idx]) idx++;
      if (m.index > last) out.appendChild(document.createTextNode(text.slice(last, m.index)));
      var s = document.createElement("span");
      s.className = "t-" + RULES[idx - 1][0]; s.textContent = m[0]; out.appendChild(s);
      last = m.index + m[0].length;
    }
    out.appendChild(document.createTextNode(text.slice(last)));
    code.textContent = ""; code.appendChild(out); code.classList.add("hl");
  }

  document.querySelectorAll("pre code").forEach(function (c) {
    var plain = !/language-/.test(c.className);
    if (LANG.test(c.className) || (plain && TOKENS.test(c.textContent))) highlight(c);
  });

  document.querySelectorAll(".doc article h2[id], .doc article h3[id]").forEach(function (h) {
    var a = document.createElement("a");
    a.className = "anchor"; a.href = "#" + h.id; a.setAttribute("aria-label", "Link to this section"); a.textContent = "#";
    h.appendChild(a);
  });

  var links = {};
  document.querySelectorAll(".pagetoc a[href^='#']").forEach(function (a) { links[a.getAttribute("href").slice(1)] = a; });
  var heads = Array.prototype.filter.call(document.querySelectorAll(".doc article h2[id], .doc article h3[id]"), function (h) { return links[h.id]; });
  if (heads.length && "IntersectionObserver" in window) {
    var cur = null, io = new IntersectionObserver(function (es) {
      es.forEach(function (e) {
        if (!e.isIntersecting) return;
        if (cur) cur.removeAttribute("aria-current");
        cur = links[e.target.id]; cur.setAttribute("aria-current", "true");
      });
    }, { rootMargin: "-10% 0px -80% 0px" });
    heads.forEach(function (h) { io.observe(h); });
  }
})();
