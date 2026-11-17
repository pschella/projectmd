//! Canonical spelling of a projectmd file (FORMAT §13). Pure.
//!
//! Only list items are rewritten (indent, bullet, checkbox and suffix spacing and order, lowercase keys), plus
//! trailing whitespace and blank lines. Prose, headings, code, comments and reference
//! definitions keep their text.

use crate::model::{Suffix, Value, ValueKind};
use crate::parse::{fence_close, fence_open, parse, parse_heading, parse_item, parse_refdef, split_item, ItemSplit};

/// Canonical form of a file.
pub fn format(src: &str) -> String {
    let proj = parse("", src);
    let mut w = Writer { out: Vec::new(), blank: false };
    let mut stack: Vec<u32> = Vec::new();
    let mut fence: Option<(u8, usize)> = None;
    let mut comment = false;
    let mut have_h1 = false;

    for (i, raw) in src.lines().enumerate() {
        // `lines()` leaves a lone "\r" on a final line without "\n"; it is a line ending too.
        let raw = raw.strip_suffix('\r').unwrap_or(raw);
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
        if let Some((level, _)) = parse_heading(raw) {
            // The first H1 is the title: it does not close list items that precede it.
            if level == 1 && !have_h1 {
                have_h1 = true;
            } else {
                stack.clear();
            }
            let t = raw.trim_end();
            if parse_heading(t).is_some() {
                w.content(t, true);
            } else {
                // An empty heading: the space after the hashes is what makes it one.
                w.content(&format!("{t} "), true);
            }
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
            let mut line = list_line(depth, body, &proj.refdefs);
            if proj.header_lines.contains(&(i as u32 + 1)) {
                line = header_line(&line);
            }
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
        if !s.has_error && !s.skipped {
            return format!("{pad}{}", render(&s));
        }
    }
    let b = body.trim_end();
    if b.is_empty() {
        // Keep it a list item (a bare "-" is not one).
        format!("{pad}- ")
    } else if b.len() == 3 && b.starts_with('[') && b.ends_with(']') && b.len() < body.len() {
        // An empty-titled checkbox item: the space after "]" is what makes it a task.
        format!("{pad}- {b} ")
    } else {
        format!("{pad}- {b}")
    }
}

/// Header keys are capitalised: `- owner: x` becomes `- Owner: x` (other keys keep their spelling).
fn header_line(line: &str) -> String {
    let Some((pre, rest)) = line.split_once("- ") else { return line.to_string() };
    match rest.split_once(':') {
        Some((k, v)) if ["owner", "commitment", "done"].contains(&k.to_ascii_lowercase().as_str()) => {
            let k = k.to_ascii_lowercase();
            format!("{pre}- {}{}:{v}", k[..1].to_uppercase(), &k[1..])
        }
        _ => line.to_string(),
    }
}

fn render(s: &ItemSplit) -> String {
    let mut line = String::from("- ");
    if let Some(c) = s.checkbox {
        line.push('[');
        line.push(c.to_ascii_lowercase());
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

/// Suffix parts in canonical order: range, estimate, tags, keys (after, by, owner, done, then
/// the rest alphabetically), id.
fn suffix_parts(sf: &Suffix) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(r) = &sf.range {
        v.push(range_text(&r.text));
    }
    if let Some(d) = &sf.duration {
        v.push(d.text.clone());
    }
    v.extend(sf.tags.iter().map(|t| if t.eq_ignore_ascii_case("private") { "#private".to_string() } else { format!("#{t}") }));
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
    /// A line whose text is kept as it is (fences, comments).
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
