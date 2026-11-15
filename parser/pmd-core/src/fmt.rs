//! Canonical spelling of a projectmd file (FORMAT §13). Pure.
//!
//! Only list items are rewritten (indent, bullet, checkbox and suffix spacing and order), plus
//! trailing whitespace and blank lines. Prose, headings, code, comments, front matter and
//! reference definitions keep their text.

use crate::model::{Suffix, Value, ValueKind};
use crate::parse::{fence_close, fence_open, parse, parse_heading, parse_item, parse_refdef, split_item, ItemSplit};

/// Canonical form of a file.
pub fn format(src: &str) -> String {
    let proj = parse("", src);
    let mut w = Writer { out: Vec::new(), blank: false };
    let mut stack: Vec<u32> = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut comment = false;
    let mut front = false;

    for (i, raw) in src.lines().enumerate() {
        if i == 0 && raw == "---" {
            front = true;
            w.raw(raw);
            continue;
        }
        if front {
            if raw == "---" || raw == "..." {
                front = false;
            }
            w.raw(raw);
            continue;
        }
        if let Some((c, len)) = fence {
            if fence_close(raw.as_bytes(), c, len) {
                fence = None;
            }
            w.raw(raw);
            continue;
        }
        if comment {
            if raw.contains("-->") {
                comment = false;
            }
            w.raw(raw);
            continue;
        }
        if raw.starts_with("<!--") {
            comment = !raw.contains("-->");
            w.raw(raw);
            continue;
        }
        if let Some(f) = fence_open(raw.as_bytes()) {
            fence = Some(f);
            w.raw(raw);
            continue;
        }
        if raw.trim().is_empty() {
            w.blank();
            continue;
        }
        if parse_heading(raw).is_some() {
            stack.clear();
            w.content(raw.trim_end(), true);
            continue;
        }
        if parse_refdef(raw).is_some() {
            w.content(raw.trim_end(), false);
            continue;
        }
        if let Some((indent, body)) = parse_item(raw) {
            while stack.last().is_some_and(|&t| t >= indent) {
                stack.pop();
            }
            let depth = stack.len();
            stack.push(indent);
            let line = list_line(depth, body, &proj.refdefs);
            w.content(&line, false);
            continue;
        }
        w.content(raw.trim_end(), false);
    }
    w.finish()
}

fn list_line(
    depth: usize,
    body: &str,
    refdefs: &std::collections::BTreeMap<String, crate::model::RefDef>,
) -> String {
    let pad = " ".repeat(2 * depth);
    if let Some(s) = split_item(body, refdefs) {
        if !s.has_error {
            return format!("{pad}{}", render(&s));
        }
    }
    let b = body.trim_end();
    if b.is_empty() {
        // Keep it a list item (a bare "-" is not one).
        format!("{pad}- ")
    } else {
        format!("{pad}- {b}")
    }
}

fn render(s: &ItemSplit) -> String {
    let mut line = String::from("- ");
    if let Some(c) = s.checkbox {
        line.push('[');
        line.push(c);
        line.push_str("] ");
    }
    if let Some(l) = &s.lead {
        line.push_str(l);
        line.push(' ');
    }
    line.push_str(&s.title);
    for part in suffix_parts(&s.suffix) {
        line.push_str("  ");
        line.push_str(&part);
    }
    line
}

/// Suffix parts in canonical order: range, date, duration, tags, keys (after, by, owner,
/// done, then the rest alphabetically), id.
fn suffix_parts(sf: &Suffix) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(r) = &sf.range {
        v.push(range_text(&r.text));
    }
    if let Some(d) = &sf.date {
        v.push(d.text.clone());
    }
    if let Some(d) = &sf.duration {
        v.push(d.text.clone());
    }
    v.extend(sf.tags.iter().map(|t| format!("#{t}")));
    let first = ["after", "by", "owner", "done"];
    let rest = sf.keys.keys().map(|k| k.as_str()).filter(|k| !first.contains(k));
    for k in first.iter().copied().chain(rest) {
        if let Some(vals) = sf.keys.get(k) {
            for x in vals {
                v.push(format!("{k}: {}", value_text(x)));
            }
        }
    }
    if let Some(i) = &sf.id {
        v.push(format!("^{i}"));
    }
    v
}

fn range_text(t: &str) -> String {
    t.replacen("..", " .. ", 1)
}

fn value_text(v: &Value) -> String {
    if v.kind == ValueKind::Range {
        range_text(&v.text)
    } else {
        v.text.clone()
    }
}

struct Writer {
    out: Vec<String>,
    /// A blank line is owed before the next content line.
    blank: bool,
}

impl Writer {
    /// A line whose text is kept as it is (fences, comments, front matter).
    fn raw(&mut self, s: &str) {
        if self.blank && !self.out.is_empty() {
            self.out.push(String::new());
        }
        self.blank = false;
        self.out.push(s.to_string());
    }

    fn blank(&mut self) {
        if !self.out.is_empty() {
            self.blank = true;
        }
    }

    /// A content line. One blank line precedes every heading.
    fn content(&mut self, s: &str, heading: bool) {
        let last_blank = self.out.last().is_some_and(|l| l.is_empty());
        if !self.out.is_empty() && (self.blank || heading) && !last_blank {
            self.out.push(String::new());
        }
        self.blank = false;
        self.out.push(s.to_string());
    }

    fn finish(mut self) -> String {
        while self.out.last().is_some_and(|l| l.is_empty()) {
            self.out.pop();
        }
        if self.out.is_empty() {
            return String::new();
        }
        let mut s = self.out.join("\n");
        s.push('\n');
        s
    }
}
