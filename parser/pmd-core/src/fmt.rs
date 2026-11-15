//! Canonical spelling of a projectmd file (FORMAT §14) and automatic ids. Pure.
//!
//! Only list items are rewritten (indent, bullet, checkbox and suffix spacing and order), plus
//! trailing whitespace and blank lines. Prose, headings, code, comments, front matter and
//! reference definitions keep their text.

use crate::date::{parse_date, split_range_word, DateParse};
use crate::model::{Options, Suffix, Value, ValueKind};
use crate::parse::{fence_close, fence_open, parse, parse_heading, parse_item, parse_refdef, split_item, ItemSplit};
use std::collections::BTreeSet;

/// Canonical form with default options.
pub fn format(src: &str, ids: bool) -> String {
    format_with(src, ids, &Options::default())
}

/// Canonical form, reading the file with the profile `opts` (its extra and repeatable keys).
/// With `ids`, every planning item without `^id` gets one made from its title.
pub fn format_with(src: &str, ids: bool, opts: &Options) -> String {
    let proj = parse("", src, opts);
    let mut taken: BTreeSet<String> = BTreeSet::new();
    if ids {
        proj.walk(|n| {
            if let Some(i) = &n.id {
                taken.insert(i.clone());
            }
        });
    }
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
            let line = list_line(depth, body, opts, &proj.refdefs, &mut taken, ids);
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
    opts: &Options,
    refdefs: &std::collections::BTreeMap<String, crate::model::RefDef>,
    taken: &mut BTreeSet<String>,
    ids: bool,
) -> String {
    let pad = " ".repeat(2 * depth);
    if let Some(mut s) = split_item(body, refdefs, opts) {
        if !s.has_error {
            if ids && s.suffix.id.is_none() {
                s.suffix.id = Some(new_id(&s.title, taken));
            }
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
/// approver, then the rest alphabetically), id.
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
    let first = ["after", "by", "owner", "approver"];
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

/// Readable id from a title: link targets and dates dropped, at most four lowercase words.
fn title_words(title: &str) -> Vec<String> {
    let cs: Vec<char> = title.chars().collect();
    let mut plain = String::new();
    let mut i = 0;
    while i < cs.len() {
        if cs[i] == ']' && i + 1 < cs.len() && (cs[i + 1] == '(' || cs[i + 1] == '[') {
            let close = if cs[i + 1] == '(' { ')' } else { ']' };
            let mut j = i + 2;
            while j < cs.len() && cs[j] != close {
                j += 1;
            }
            plain.push(' ');
            i = j + 1;
            continue;
        }
        if cs[i] != '[' && cs[i] != ']' {
            plain.push(cs[i]);
        }
        i += 1;
    }
    let mut words = Vec::new();
    for w in plain.split_whitespace() {
        let w = w.trim_matches(|c: char| !c.is_ascii_alphanumeric());
        if w.is_empty() || w.contains("..") || parse_date(w) != DateParse::NotDate || split_range_word(w).is_some() {
            continue;
        }
        for piece in w.split(|c: char| !c.is_ascii_alphanumeric()).filter(|p| !p.is_empty()) {
            words.push(piece.to_ascii_lowercase());
        }
    }
    words.truncate(4);
    words
}

/// New id for a title, unique against `taken` (which it joins). Suffixes -2, -3, ... on clashes.
fn new_id(title: &str, taken: &mut BTreeSet<String>) -> String {
    let words = title_words(title);
    let base = if words.is_empty() { "item".to_string() } else { words.join("-") };
    let mut id = base.clone();
    let mut n = 2;
    while taken.contains(&id) {
        id = format!("{base}-{n}");
        n += 1;
    }
    taken.insert(id.clone());
    id
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
