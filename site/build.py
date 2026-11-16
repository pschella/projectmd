#!/usr/bin/env python3
"""Render docs/*.md to site pages. Usage: build.py OUTDIR (needs the `markdown` package)."""
import re, shutil, sys
from pathlib import Path
import markdown

ROOT = Path(__file__).resolve().parent.parent
GH = "https://github.com/pschella/projectmd/blob/main/"
PAGES = {"DESIGN": ("design", "Design", True), "FORMAT": ("format", "Format", True),
         "CHEATSHEET": ("cheatsheet", "Cheat sheet", False), "GRAMMAR": ("grammar", "Grammar", False)}
LOGO = '<svg viewBox="0 0 32 32" aria-hidden="true"><rect width="32" height="32" rx="8" fill="#4338ca"/><path d="M7 11.5l2.5 2.5L14 8.5" fill="none" stroke="#fff" stroke-width="2.4" stroke-linecap="round" stroke-linejoin="round"/><rect x="17" y="8" width="9" height="4.5" rx="2.2" fill="#5eead4"/><rect x="8" y="18.5" width="16" height="4.5" rx="2.2" fill="#fff"/></svg>'
NAV = [("index.html", "projectmd"), ("design.html", "Design"), ("format.html", "Format"),
       ("cheatsheet.html", "Cheat sheet"), ("grammar.html", "Grammar"),
       ("viewer/?root=examples/lunar/hq/lunar-programme.md&today=1967-11-15", "Live demo"),
       ("https://github.com/pschella/projectmd", "GitHub")]

def slugify(value, sep):
    s = re.sub(r"[^\w\- ]", "", value.strip().lower())
    return s.replace(" ", sep)

def rewrite(m, doc_dir):
    url = m.group(1)
    if re.match(r"^(https?:|mailto:|#)", url):
        return m.group(0)
    path, _, frag = url.partition("#")
    name = Path(path).name
    if Path(path).parent == Path(".") and name[:-3] in PAGES and name.endswith(".md"):
        return 'href="%s.html%s"' % (PAGES[name[:-3]][0], "#" + frag if frag else "")
    target = (doc_dir / path).resolve().relative_to(ROOT).as_posix()
    if path.endswith("/") and not target.endswith("/"):
        target += "/"
    return 'href="%s%s%s"' % (GH, target, "#" + frag if frag else "")

def main(out):
    out = Path(out); out.mkdir(parents=True, exist_ok=True)
    tpl = (Path(__file__).parent / "page.template.html").read_text()
    for stem, (slug, title, toc) in PAGES.items():
        nav = "".join('<a href="%s"%s%s>%s</a>' % (h, ' class="home"' if i == 0 else "",
                      ' aria-current="page"' if h == slug + ".html" else "", LOGO + t if i == 0 else t)
                      for i, (h, t) in enumerate(NAV))
        md = markdown.Markdown(extensions=["tables", "fenced_code", "toc"],
                               extension_configs={"toc": {"slugify": slugify, "toc_depth": "2-3"}})
        src = ROOT / "docs" / (stem + ".md")
        body = md.convert(src.read_text())
        body = re.sub(r'href="([^"]*)"', lambda m: rewrite(m, src.parent), body)
        body = re.sub(r'src="(img/[^"]*)"', r'src="\1"', body)
        h1 = re.search(r"^# (.+)$", src.read_text(), re.M)
        page = (tpl.replace("{{title}}", (h1.group(1) if h1 else title) + " · projectmd")
                   .replace("{{nav}}", nav).replace("{{body}}", body)
                   .replace("{{toc}}", '<aside class="pagetoc" aria-label="Contents">%s</aside>' % md.toc if toc else "")
                   .replace("{{cls}}" , "withtoc" if toc else ""))
        (out / (slug + ".html")).write_text(page)
    for f in ("highlight.js", "style.css"):
        if (out / f).resolve() != (Path(__file__).parent / f).resolve():
            shutil.copy(Path(__file__).parent / f, out / f)
    img = ROOT / "docs" / "img"
    if img.is_dir():
        shutil.copytree(img, out / "img", dirs_exist_ok=True)

main(sys.argv[1])
