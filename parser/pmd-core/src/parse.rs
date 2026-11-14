//! The projectmd parser (GRAMMAR §1–§6, §9). Pure: no filesystem, no threads.

use crate::date::{parse_date, range_of, split_range_word, DateParse, RangeParse};
use crate::model::*;
use std::collections::{BTreeMap, HashMap};

const CORE_KEYS: [&str; 5] = ["owner", "after", "by", "approver", "replaced-by"];

#[inline]
fn is_ws(b: u8) -> bool {
    matches!(b, b' ' | b'\t' | b'\n' | b'\r' | 0x0c)
}

#[inline]
fn skip_ws(b: &[u8], mut i: usize) -> usize {
    while i < b.len() && is_ws(b[i]) {
        i += 1;
    }
    i
}

// ---------------------------------------------------------------------------------------------
// Word tests (GRAMMAR §4.2, §5.3)
// ---------------------------------------------------------------------------------------------

pub fn is_tag(w: &str) -> bool {
    let b = w.as_bytes();
    b.len() >= 2
        && b[0] == b'#'
        && b[1].is_ascii_alphabetic()
        && b[2..].iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-')
}

pub fn is_id(w: &str) -> bool {
    let b = w.as_bytes();
    b.len() >= 2
        && b[0] == b'^'
        && b[1].is_ascii_alphanumeric()
        && b[2..].iter().all(|c| c.is_ascii_alphanumeric() || *c == b'_' || *c == b'-')
}

/// `^[1-9][0-9]*[dwm]$`
pub fn is_duration(w: &str) -> Option<(u32, char)> {
    let b = w.as_bytes();
    if b.len() < 2 {
        return None;
    }
    let u = b[b.len() - 1];
    if !matches!(u, b'd' | b'w' | b'm') {
        return None;
    }
    let digits = &b[..b.len() - 1];
    if digits[0] < b'1' || digits[0] > b'9' || !digits.iter().all(|c| c.is_ascii_digit()) {
        return None;
    }
    let n: u32 = w[..w.len() - 1].parse().ok()?;
    Some((n, u as char))
}

/// `^@[A-Za-z0-9._-]+$`
pub fn is_handle(w: &str) -> bool {
    let b = w.as_bytes();
    b.len() >= 2 && b[0] == b'@' && b[1..].iter().all(|c| c.is_ascii_alphanumeric() || matches!(c, b'.' | b'_' | b'-'))
}

fn is_path_id(w: &str) -> bool {
    match w.find('#') {
        Some(i) => i > 0 && i + 1 < w.len(),
        None => false,
    }
}

fn ends_md(w: &str) -> bool {
    let b = w.as_bytes();
    b.len() >= 3 && b[b.len() - 3..].eq_ignore_ascii_case(b".md")
}

// ---------------------------------------------------------------------------------------------
// Atomisation (§4.1)
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum AK {
    Code,
    Inline,
    RefLink,
    Email,
    Word,
}

#[derive(Clone, Copy, Debug)]
struct Atom {
    kind: AK,
    s: usize,
    e: usize,
    /// link text
    t: (usize, usize),
    /// inline: target; reference: label
    u: (usize, usize),
    ti: Option<(usize, usize)>,
}

impl Atom {
    fn simple(kind: AK, s: usize, e: usize) -> Atom {
        Atom { kind, s, e, t: (0, 0), u: (0, 0), ti: None }
    }
    fn is_link(&self) -> bool {
        matches!(self.kind, AK::Inline | AK::RefLink)
    }
}

fn find_byte(b: &[u8], from: usize, c: u8) -> Option<usize> {
    b[from.min(b.len())..].iter().position(|x| *x == c).map(|i| i + from)
}

fn try_inline(text: &str, p: usize) -> Option<Atom> {
    let b = text.as_bytes();
    let n = b.len();
    let j = find_byte(b, p + 1, b']')?;
    if j + 1 >= n || b[j + 1] != b'(' {
        return None;
    }
    let q = skip_ws(b, j + 2);
    let mut run_end = q;
    while run_end < n && !is_ws(b[run_end]) {
        run_end += 1;
    }
    for e in q + 1..=run_end {
        if e < run_end {
            if b[e] == b')' {
                return Some(Atom { kind: AK::Inline, s: p, e: e + 1, t: (p + 1, j), u: (q, e), ti: None });
            }
            continue;
        }
        let r2 = skip_ws(b, e);
        if r2 > e && r2 < n && b[r2] == b'"' {
            if let Some(k) = find_byte(b, r2 + 1, b'"') {
                let r3 = skip_ws(b, k + 1);
                if r3 < n && b[r3] == b')' {
                    return Some(Atom {
                        kind: AK::Inline,
                        s: p,
                        e: r3 + 1,
                        t: (p + 1, j),
                        u: (q, e),
                        ti: Some((r2 + 1, k)),
                    });
                }
            }
        }
        if r2 < n && b[r2] == b')' {
            return Some(Atom { kind: AK::Inline, s: p, e: r2 + 1, t: (p + 1, j), u: (q, e), ti: None });
        }
    }
    None
}

fn norm_label(l: &str) -> String {
    l.trim().to_ascii_lowercase()
}

/// Returns (atom, resolved). An unresolved reference link is returned as a Word atom.
fn try_ref(text: &str, p: usize, refdefs: &BTreeMap<String, RefDef>) -> Option<Atom> {
    let b = text.as_bytes();
    let j = find_byte(b, p + 1, b']')?;
    if j + 1 >= b.len() || b[j + 1] != b'[' {
        return None;
    }
    let k = find_byte(b, j + 2, b']')?;
    let label = if k == j + 2 { &text[p + 1..j] } else { &text[j + 2..k] };
    if refdefs.contains_key(&norm_label(label)) {
        Some(Atom { kind: AK::RefLink, s: p, e: k + 1, t: (p + 1, j), u: (j + 2, k), ti: None })
    } else {
        Some(Atom::simple(AK::Word, p, k + 1))
    }
}

fn try_email(text: &str, p: usize) -> Option<Atom> {
    let b = text.as_bytes();
    let mut i = p + 1;
    let mut at = None;
    while i < b.len() {
        match b[i] {
            b'>' => break,
            b'<' => return None,
            c if is_ws(c) => return None,
            b'@' => {
                if at.is_none() && i > p + 1 {
                    at = Some(i);
                } else if at.is_some() {
                    // a later '@' is also a possible split; keep first valid
                }
            }
            _ => {}
        }
        i += 1;
    }
    if i >= b.len() || b[i] != b'>' {
        return None;
    }
    // need '@' with non-empty text on both sides
    let a = at?;
    if a + 1 >= i {
        // try to find a later '@' that works (only possible if >1 '@')
        let mut ok = false;
        for k in p + 2..i - 1 {
            if b[k] == b'@' {
                ok = true;
                break;
            }
        }
        if !ok {
            return None;
        }
    }
    Some(Atom::simple(AK::Email, p, i + 1))
}

fn atomise(text: &str, refdefs: &BTreeMap<String, RefDef>, out: &mut Vec<Atom>) {
    out.clear();
    let b = text.as_bytes();
    let n = b.len();
    let mut p = 0;
    loop {
        p = skip_ws(b, p);
        if p >= n {
            break;
        }
        let atom = match b[p] {
            b'`' => {
                let e = match find_byte(b, p + 1, b'`') {
                    Some(j) => j + 1,
                    None => n,
                };
                Some(Atom::simple(AK::Code, p, e))
            }
            b'[' => try_inline(text, p).or_else(|| try_ref(text, p, refdefs)),
            b'<' => try_email(text, p),
            _ => None,
        };
        let atom = match atom {
            Some(a) => a,
            None => {
                let mut e = p;
                while e < n && !is_ws(b[e]) {
                    e += 1;
                }
                Atom::simple(AK::Word, p, e)
            }
        };
        p = atom.e;
        out.push(atom);
    }
}

// ---------------------------------------------------------------------------------------------
// Line level (§1)
// ---------------------------------------------------------------------------------------------

enum Raw<'a> {
    Heading { line: u32, level: u8, text: &'a str },
    Item { line: u32, indent: u32, body: &'a str },
    Other,
}

fn fence_open(l: &[u8]) -> Option<(u8, usize)> {
    let mut i = 0;
    while i < l.len() && i < 3 && (l[i] == b' ' || l[i] == b'\t') {
        i += 1;
    }
    if i >= l.len() {
        return None;
    }
    let c = l[i];
    if c != b'`' && c != b'~' {
        return None;
    }
    let mut k = i;
    while k < l.len() && l[k] == c {
        k += 1;
    }
    if k - i >= 3 {
        Some((c, k - i))
    } else {
        None
    }
}

fn fence_close(l: &[u8], ch: u8, len: usize) -> bool {
    let mut i = 0;
    while i < l.len() && i < 3 && (l[i] == b' ' || l[i] == b'\t') {
        i += 1;
    }
    let s = i;
    while i < l.len() && l[i] == ch {
        i += 1;
    }
    i - s >= len && l[i..].iter().all(|c| is_ws(*c))
}

fn parse_heading(l: &str) -> Option<(u8, &str)> {
    let b = l.as_bytes();
    let mut k = 0;
    while k < b.len() && b[k] == b'#' {
        k += 1;
    }
    if k == 0 || k > 6 || k >= b.len() || !is_ws(b[k]) {
        return None;
    }
    let mut t = l[k..].trim_matches(|c: char| c.is_ascii_whitespace() || c == '\x0c');
    t = t.trim_end_matches('#');
    t = t.trim_end_matches(|c: char| c.is_ascii_whitespace());
    Some((k as u8, t))
}

fn parse_refdef(l: &str) -> Option<(String, RefDef)> {
    let b = l.as_bytes();
    let mut i = 0;
    while i < b.len() && i < 3 && (b[i] == b' ' || b[i] == b'\t') {
        i += 1;
    }
    if i >= b.len() || b[i] != b'[' {
        return None;
    }
    let j = find_byte(b, i + 1, b']')?;
    if j == i + 1 || j + 1 >= b.len() || b[j + 1] != b':' {
        return None;
    }
    let label = &l[i + 1..j];
    let q = skip_ws(b, j + 2);
    if q == j + 2 {
        return None;
    }
    let mut e = q;
    while e < b.len() && !is_ws(b[e]) {
        e += 1;
    }
    if e == q {
        return None;
    }
    let target = &l[q..e];
    let r = skip_ws(b, e);
    if r >= b.len() {
        return Some((norm_label(label), RefDef { target: target.to_string(), title: None }));
    }
    if r == e || b[r] != b'"' {
        return None;
    }
    let k = find_byte(b, r + 1, b'"')?;
    if skip_ws(b, k + 1) < b.len() {
        return None;
    }
    Some((norm_label(label), RefDef { target: target.to_string(), title: Some(l[r + 1..k].to_string()) }))
}

fn parse_item(l: &str) -> Option<(u32, &str)> {
    let b = l.as_bytes();
    let mut i = 0;
    let mut indent = 0u32;
    while i < b.len() && is_ws(b[i]) {
        indent += if b[i] == b'\t' { 4 } else { 1 };
        i += 1;
    }
    if i + 1 >= b.len() || !matches!(b[i], b'-' | b'*' | b'+') || !is_ws(b[i + 1]) {
        return None;
    }
    let r = skip_ws(b, i + 1);
    Some((indent, &l[r..]))
}

// ---------------------------------------------------------------------------------------------
// Parser
// ---------------------------------------------------------------------------------------------

enum Elem<'a> {
    Key(String, Value),
    Range(DateExpr),
    Tag(&'a str),
    Id(&'a str),
    Dur(Duration),
    Date(DateExpr),
}

struct Ext {
    ts: usize,
    te: usize,
    nt: usize,
    single_link: Option<Atom>,
    suffix: Suffix,
}

enum Slot {
    Head { level: u8, node: Node },
    Item { indent: u32, node: Node },
    Prose { indent: u32 },
}

enum Leading {
    None,
    Date(Option<DateExpr>),
    Range(Option<DateExpr>),
}

struct P<'a> {
    extra: Vec<String>,
    rep: Vec<String>,
    diags: Vec<Diagnostic>,
    refdefs: BTreeMap<String, RefDef>,
    atoms: Vec<Atom>,
    ids: HashMap<&'a str, u32>,
    refs: Vec<(u32, String)>,
}

/// Parse one projectmd file. Never fails; problems go to `diagnostics`.
pub fn parse(path: &str, src: &str, opts: &Options) -> Project {
    let mut p = P {
        extra: opts.extra_keys.iter().map(|k| k.to_ascii_lowercase()).collect(),
        rep: opts.repeatable.iter().map(|k| k.to_ascii_lowercase()).collect(),
        diags: Vec::new(),
        refdefs: BTreeMap::new(),
        atoms: Vec::new(),
        ids: HashMap::new(),
        refs: Vec::new(),
    };
    p.run(path, src)
}

impl<'a> P<'a> {
    fn diag(&mut self, sev: Severity, code: &'static str, line: u32, msg: impl Into<String>) {
        self.diags.push(Diagnostic::new(sev, code, line, msg));
    }

    fn is_known_name(&self, name: &str) -> bool {
        CORE_KEYS.iter().any(|k| k.eq_ignore_ascii_case(name)) || self.extra.iter().any(|k| k.eq_ignore_ascii_case(name))
    }

    fn is_repeatable(&self, lname: &str) -> bool {
        lname == "after" || lname == "approver" || self.rep.iter().any(|k| k == lname)
    }

    /// If the atom is a known key word `name:`, return the name slice.
    fn atom_key<'t>(&self, text: &'t str, a: &Atom) -> Option<&'t str> {
        if a.kind != AK::Word {
            return None;
        }
        let w = &text[a.s..a.e];
        let name = w.strip_suffix(':')?;
        let b = name.as_bytes();
        if b.is_empty() || !b[0].is_ascii_alphabetic() || !b.iter().all(|c| c.is_ascii_alphanumeric() || *c == b'-') {
            return None;
        }
        if self.is_known_name(name) {
            Some(name)
        } else {
            None
        }
    }

    fn link_parts<'t>(&'t self, text: &'t str, a: &Atom) -> (&'t str, &'t str, Option<&'t str>) {
        match a.kind {
            AK::Inline => (&text[a.t.0..a.t.1], &text[a.u.0..a.u.1], a.ti.map(|(s, e)| &text[s..e])),
            _ => {
                let label = if a.u.0 == a.u.1 { &text[a.t.0..a.t.1] } else { &text[a.u.0..a.u.1] };
                match self.refdefs.get(&norm_label(label)) {
                    Some(d) => (&text[a.t.0..a.t.1], d.target.as_str(), d.title.as_deref()),
                    None => (&text[a.t.0..a.t.1], "", None),
                }
            }
        }
    }

    // ---- values (§4.5) ------------------------------------------------------------------------

    fn value_from_atoms(&self, text: &str, atoms: &[Atom]) -> Option<Value> {
        let n = atoms.len();
        if n == 0 {
            return None;
        }
        let last = atoms[n - 1];
        let mk = |kind, t: &str| Value {
            kind,
            text: t.to_string(),
            target: None,
            title: None,
            name: None,
            email: None,
            start: None,
            end: None,
        };
        if last.kind == AK::Email {
            if n >= 2 && atoms[..n - 1].iter().all(|a| a.kind == AK::Word) {
                let first = atoms[0];
                let mut v = mk(ValueKind::Person, &text[first.s..last.e]);
                v.name = Some(text[first.s..last.s].trim().to_string());
                v.email = Some(text[last.s + 1..last.e - 1].to_string());
                return Some(v);
            }
            return None;
        }
        if n == 1 {
            let a = last;
            return match a.kind {
                AK::Inline | AK::RefLink => {
                    let (_, target, title) = self.link_parts(text, &a);
                    if a.kind == AK::RefLink && target.is_empty() {
                        return None;
                    }
                    let mut v = mk(ValueKind::Link, &text[a.s..a.e]);
                    v.target = Some(target.to_string());
                    v.title = title.map(|s| s.to_string());
                    Some(v)
                }
                AK::Word => {
                    let w = &text[a.s..a.e];
                    if let Some((x, y)) = split_range_word(w) {
                        let mut v = mk(ValueKind::Range, w);
                        if let RangeParse::Ok(s, e) = range_of(x, y) {
                            v.start = Some(s);
                            v.end = Some(e);
                        }
                        Some(v)
                    } else {
                        let mut v = mk(ValueKind::Word, w);
                        if let DateParse::Ok(s, e) = parse_date(w) {
                            v.start = Some(s);
                            v.end = Some(e);
                        }
                        Some(v)
                    }
                }
                _ => None,
            };
        }
        if n == 3 && atoms.iter().all(|a| a.kind == AK::Word) && &text[atoms[1].s..atoms[1].e] == ".." {
            let (x, y) = (&text[atoms[0].s..atoms[0].e], &text[atoms[2].s..atoms[2].e]);
            return match range_of(x, y) {
                RangeParse::NotRange => None,
                r => {
                    let mut v = mk(ValueKind::Range, &format!("{}..{}", x, y));
                    if let RangeParse::Ok(s, e) = r {
                        v.start = Some(s);
                        v.end = Some(e);
                    }
                    Some(v)
                }
            };
        }
        None
    }

    fn typecheck(&self, lkey: &str, v: &Value) -> bool {
        match lkey {
            "owner" | "approver" => {
                v.kind == ValueKind::Person || (v.kind == ValueKind::Word && is_handle(&v.text))
            }
            "after" => match v.kind {
                ValueKind::Link => true,
                ValueKind::Word => is_id(&v.text) || is_path_id(&v.text) || ends_md(&v.text),
                _ => false,
            },
            "by" => match v.kind {
                ValueKind::Link | ValueKind::Range => true,
                ValueKind::Word => {
                    parse_date(&v.text) != DateParse::NotDate || is_id(&v.text) || is_path_id(&v.text)
                }
                _ => false,
            },
            "replaced-by" => v.kind == ValueKind::Word && is_id(&v.text),
            _ => true,
        }
    }

    fn value_diag(&mut self, v: &Value, line: u32) {
        match v.kind {
            ValueKind::Range if v.start.is_none() => {
                let (a, b) = match v.text.find("..") {
                    Some(i) => (v.text[..i].trim(), v.text[i + 2..].trim()),
                    None => return,
                };
                if let RangeParse::Invalid(m) = range_of(a, b) {
                    self.diag(Severity::E, "E-DATE", line, format!("{}: {}", m, v.text));
                }
            }
            ValueKind::Word if v.start.is_none() => {
                if parse_date(&v.text) == DateParse::Invalid {
                    self.diag(Severity::E, "E-DATE", line, format!("invalid date: {}", v.text));
                }
            }
            _ => {}
        }
        if v.kind == ValueKind::Word && v.text.starts_with('^') && is_id(&v.text) {
            self.refs.push((line, v.text.clone()));
        }
    }

    fn add_key(&mut self, keys: &mut BTreeMap<String, Vec<Value>>, lname: String, v: Value, line: u32) {
        let rep = self.is_repeatable(&lname);
        match keys.get_mut(&lname) {
            Some(list) => {
                if !rep {
                    self.diag(Severity::E, "E-SYNTAX", line, format!("key '{}' repeated", lname));
                    list.clear();
                }
                list.push(v);
            }
            None => {
                keys.insert(lname, vec![v]);
            }
        }
    }

    // ---- dates ---------------------------------------------------------------------------------

    fn date_expr(&mut self, w: &str, line: u32) -> Option<DateExpr> {
        match parse_date(w) {
            DateParse::Ok(s, e) => Some(DateExpr { text: w.to_string(), start: s, end: e, range: false }),
            DateParse::Invalid => {
                self.diag(Severity::E, "E-DATE", line, format!("invalid date: {}", w));
                None
            }
            DateParse::NotDate => None,
        }
    }

    fn range_expr(&mut self, a: &str, b: &str, line: u32) -> Option<DateExpr> {
        match range_of(a, b) {
            RangeParse::Ok(s, e) => Some(DateExpr { text: format!("{}..{}", a, b), start: s, end: e, range: true }),
            RangeParse::Invalid(m) => {
                self.diag(Severity::E, "E-DATE", line, format!("{}: {} .. {}", m, a, b));
                None
            }
            RangeParse::NotRange => None,
        }
    }

    // ---- suffix extraction (§4.3) -------------------------------------------------------------

    fn extract(&mut self, text: &'a str, line: u32) -> Ext {
        let mut atoms = std::mem::take(&mut self.atoms);
        atomise(text, &self.refdefs, &mut atoms);
        let n = atoms.len();
        let mut end = n;
        let mut elems: Vec<Elem<'a>> = Vec::new();
        let word = |a: &Atom| -> &'a str { &text[a.s..a.e] };
        while end > 1 {
            let a = atoms[end - 1];
            // 3. key + value
            let mut found: Option<(usize, Value, String)> = None;
            if end >= 3 {
                let try_k = |k: usize, this: &Self| -> Option<(usize, Value, String)> {
                    let name = this.atom_key(text, &atoms[k])?;
                    let v = this.value_from_atoms(text, &atoms[k + 1..end])?;
                    let l = name.to_ascii_lowercase();
                    if this.typecheck(&l, &v) {
                        Some((k, v, l))
                    } else {
                        None
                    }
                };
                found = try_k(end - 2, self);
                if found.is_none() && end >= 5 {
                    found = try_k(end - 4, self);
                }
                if found.is_none() && a.kind == AK::Email && end >= 4 {
                    let mut k = end - 3;
                    loop {
                        if k < 1 {
                            break;
                        }
                        if let Some(f) = try_k(k, self) {
                            found = Some(f);
                            break;
                        }
                        if atoms[k].kind != AK::Word {
                            break;
                        }
                        k -= 1;
                    }
                }
            }
            if let Some((k, v, l)) = found {
                if k >= 1 {
                    self.value_diag(&v, line);
                    elems.push(Elem::Key(l, v));
                    end = k;
                    continue;
                }
            }
            // 4. range
            if end >= 4
                && atoms[end - 3..end].iter().all(|x| x.kind == AK::Word)
                && word(&atoms[end - 2]) == ".."
            {
                let (x, y) = (word(&atoms[end - 3]), word(&atoms[end - 1]));
                if range_of(x, y) != RangeParse::NotRange {
                    if let Some(r) = self.range_expr(x, y, line) {
                        elems.push(Elem::Range(r));
                    }
                    end -= 3;
                    continue;
                }
            }
            if a.kind == AK::Word {
                let w = word(&a);
                if let Some((x, y)) = split_range_word(w) {
                    if let Some(r) = self.range_expr(x, y, line) {
                        elems.push(Elem::Range(r));
                    }
                    end -= 1;
                    continue;
                }
                // 5. bare tokens
                if is_tag(w) {
                    elems.push(Elem::Tag(&w[1..]));
                    end -= 1;
                    continue;
                }
                if is_id(w) {
                    elems.push(Elem::Id(&w[1..]));
                    end -= 1;
                    continue;
                }
                if let Some((nn, u)) = is_duration(w) {
                    let days = match u {
                        'd' => nn,
                        'w' => nn.saturating_mul(7),
                        _ => nn.saturating_mul(30),
                    };
                    elems.push(Elem::Dur(Duration { n: nn, unit: u, days, text: w.to_string() }));
                    end -= 1;
                    continue;
                }
                if parse_date(w) != DateParse::NotDate {
                    if let Some(d) = self.date_expr(w, line) {
                        elems.push(Elem::Date(d));
                    }
                    end -= 1;
                    continue;
                }
            }
            break;
        }
        let (ts, te) = if end == 0 { (0, 0) } else { (atoms[0].s, atoms[end - 1].e) };
        let single_link = if end == 1 && atoms[0].is_link() { Some(atoms[0]) } else { None };
        let mut suffix = Suffix::default();
        for el in elems.into_iter().rev() {
            match el {
                Elem::Key(l, v) => self.add_key(&mut suffix.keys, l, v, line),
                Elem::Range(r) => {
                    if suffix.range.is_some() {
                        self.diag(Severity::E, "E-SYNTAX", line, "more than one range");
                    } else {
                        suffix.range = Some(r);
                    }
                }
                Elem::Tag(t) => suffix.tags.push(t.to_string()),
                Elem::Id(i) => {
                    if suffix.id.is_some() {
                        self.diag(Severity::E, "E-SYNTAX", line, "more than one id");
                    } else {
                        suffix.id = Some(i.to_string());
                        self.register_id(i, line);
                    }
                }
                Elem::Dur(d) => {
                    if suffix.duration.is_some() {
                        self.diag(Severity::E, "E-SYNTAX", line, "more than one duration");
                    } else {
                        suffix.duration = Some(d);
                    }
                }
                Elem::Date(d) => {
                    if suffix.date.is_some() {
                        self.diag(Severity::E, "E-SYNTAX", line, "more than one date");
                    } else {
                        suffix.date = Some(d);
                    }
                }
            }
        }
        let ext = Ext { ts, te, nt: end, single_link, suffix };
        self.atoms = atoms;
        ext
    }

    fn register_id(&mut self, id: &'a str, line: u32) {
        if let Some(first) = self.ids.get(id) {
            let first = *first;
            self.diag(Severity::E, "E-DUP-ID", line, format!("id '{}' already defined on line {}", id, first));
        } else {
            self.ids.insert(id, line);
        }
    }

    // ---- items --------------------------------------------------------------------------------

    fn leading(&mut self, rest: &'a str, line: u32) -> (Leading, &'a str) {
        let b = rest.as_bytes();
        let mut w1e = 0;
        while w1e < b.len() && !is_ws(b[w1e]) {
            w1e += 1;
        }
        if w1e == 0 {
            return (Leading::None, rest);
        }
        let w1 = &rest[..w1e];
        let more = |from: usize| -> Option<usize> {
            let r = skip_ws(b, from);
            if r > from && !rest[r..].trim_end().is_empty() {
                Some(r)
            } else {
                None
            }
        };
        if let Some((x, y)) = split_range_word(w1) {
            if let Some(r) = more(w1e) {
                let rg = self.range_expr(x, y, line);
                return (Leading::Range(rg), &rest[r..]);
            }
            return (Leading::None, rest);
        }
        if parse_date(w1) != DateParse::NotDate {
            // three-word range?
            let r2 = skip_ws(b, w1e);
            let mut w2e = r2;
            while w2e < b.len() && !is_ws(b[w2e]) {
                w2e += 1;
            }
            if r2 > w1e && &rest[r2..w2e] == ".." {
                let r3 = skip_ws(b, w2e);
                let mut w3e = r3;
                while w3e < b.len() && !is_ws(b[w3e]) {
                    w3e += 1;
                }
                if r3 > w2e && w3e > r3 && parse_date(&rest[r3..w3e]) != DateParse::NotDate {
                    if let Some(r) = more(w3e) {
                        let rg = self.range_expr(w1, &rest[r3..w3e], line);
                        return (Leading::Range(rg), &rest[r..]);
                    }
                }
            }
            if let Some(r) = more(w1e) {
                let d = self.date_expr(w1, line);
                return (Leading::Date(d), &rest[r..]);
            }
        }
        (Leading::None, rest)
    }

    fn pop_slot(&mut self, stack: &mut Vec<Slot>, nodes: &mut Vec<Node>) {
        match stack.pop() {
            Some(Slot::Head { node, .. }) => {
                if !node.children.is_empty() {
                    Self::attach(stack, nodes, node);
                }
            }
            Some(Slot::Item { node, .. }) => Self::attach(stack, nodes, node),
            _ => {}
        }
    }

    fn attach(stack: &mut [Slot], nodes: &mut Vec<Node>, node: Node) {
        match stack.last_mut() {
            Some(Slot::Head { node: p, .. }) | Some(Slot::Item { node: p, .. }) => p.children.push(node),
            Some(Slot::Prose { .. }) => {}
            None => nodes.push(node),
        }
    }

    fn item(&mut self, line: u32, indent: u32, body: &'a str, stack: &mut Vec<Slot>, nodes: &mut Vec<Node>, hdr: &mut BTreeMap<String, (String, u32)>) {
        while let Some(top) = stack.last() {
            let ti = match top {
                Slot::Item { indent: i, .. } | Slot::Prose { indent: i } => *i,
                Slot::Head { .. } => break,
            };
            if ti >= indent {
                self.pop_slot(stack, nodes);
            } else {
                break;
            }
        }
        if let Some(Slot::Prose { .. }) = stack.last() {
            stack.push(Slot::Prose { indent });
            return;
        }
        let bb = body.as_bytes();
        // 1. checkbox
        let mut checkbox = Checkbox::None;
        let mut rest = body;
        if bb.len() >= 4 && bb[0] == b'[' && bb[2] == b']' && is_ws(bb[3]) {
            let cb = match bb[1] {
                b' ' => Some(Checkbox::Open),
                b'x' | b'X' => Some(Checkbox::Done),
                b'-' => Some(Checkbox::Cancelled),
                _ => None,
            };
            if let Some(c) = cb {
                checkbox = c;
                rest = &body[skip_ws(bb, 3)..];
            }
        }
        // 2. leading date / range
        let (lead, rest) = self.leading(rest, line);
        let planning = checkbox != Checkbox::None || !matches!(lead, Leading::None);
        if !planning {
            if self.property_bullet(line, body, stack, hdr) {
                return;
            }
            stack.push(Slot::Prose { indent });
            return;
        }
        let ext = self.extract(rest, line);
        if ext.nt == 0 {
            self.diag(Severity::E, "E-SYNTAX", line, "empty title");
        }
        let mut title = rest[ext.ts..ext.te].to_string();
        let mut sfx = ext.suffix;
        let mut kind = NodeKind::Task;
        let mut link = None;
        let mut suffix_date = None;
        match lead {
            Leading::Date(d) => {
                kind = NodeKind::Milestone;
                if d.is_some() {
                    suffix_date = sfx.date.take();
                    sfx.date = d;
                }
            }
            Leading::Range(r) => {
                if r.is_some() && sfx.range.is_some() {
                    self.diag(Severity::E, "E-SYNTAX", line, "more than one range");
                } else if r.is_some() {
                    sfx.range = r;
                }
            }
            Leading::None => {
                if let Some(a) = ext.single_link {
                    let (t, target, ti) = self.link_parts(rest, &a);
                    if is_subproject_target(target) {
                        kind = NodeKind::Subproject;
                        title = t.to_string();
                        link = Some(LinkInfo { text: t.to_string(), target: target.to_string(), title: ti.map(|s| s.to_string()) });
                    }
                }
            }
        }
        let node = Node {
            kind,
            line,
            checkbox,
            title,
            date: sfx.date,
            suffix_date,
            range: sfx.range,
            duration: sfx.duration,
            tags: sfx.tags,
            id: sfx.id,
            keys: sfx.keys,
            link,
            link_resolved: None,
            forecast: None,
            children: Vec::new(),
        };
        stack.push(Slot::Item { indent, node });
    }

    /// GRAMMAR §2.2 step 4. Returns true if the item was consumed as a property bullet.
    fn property_bullet(&mut self, line: u32, body: &'a str, stack: &mut Vec<Slot>, hdr: &mut BTreeMap<String, (String, u32)>) -> bool {
        let b = body.as_bytes();
        if b.is_empty() || !b[0].is_ascii_alphabetic() {
            return false;
        }
        let mut i = 1;
        while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == b'-') {
            i += 1;
        }
        if i >= b.len() || b[i] != b':' || i + 1 >= b.len() || !is_ws(b[i + 1]) {
            return false;
        }
        let name = &body[..i];
        if !self.is_known_name(name) {
            return false;
        }
        let rest = body[i + 1..].trim();
        if rest.is_empty() {
            return false;
        }
        let lname = name.to_ascii_lowercase();
        let mut atoms = std::mem::take(&mut self.atoms);
        atomise(rest, &self.refdefs, &mut atoms);
        let v = self.value_from_atoms(rest, &atoms);
        self.atoms = atoms;
        let v = match v {
            Some(v) if self.typecheck(&lname, &v) => v,
            _ => return false,
        };
        self.value_diag(&v, line);
        match stack.last_mut() {
            Some(Slot::Head { node, .. }) | Some(Slot::Item { node, .. }) => {
                let mut keys = std::mem::take(&mut node.keys);
                self.add_key(&mut keys, lname, v, line);
                match stack.last_mut() {
                    Some(Slot::Head { node, .. }) | Some(Slot::Item { node, .. }) => node.keys = keys,
                    _ => {}
                }
            }
            Some(Slot::Prose { .. }) => return false,
            None => {
                hdr.entry(lname).or_insert((rest.to_string(), line));
            }
        }
        true
    }

    // ---- main ---------------------------------------------------------------------------------

    fn run(&mut self, path: &str, src: &'a str) -> Project {
        let mut hdr: BTreeMap<String, (String, u32)> = BTreeMap::new();
        let mut raws: Vec<Raw<'a>> = Vec::new();
        let mut pos = 0usize;
        let mut lineno = 0u32;
        let mut fm_checked = false;
        let mut fence: Option<(u8, usize)> = None;
        let mut in_comment = false;
        let mut last_other = false;
        while pos <= src.len() {
            let end = match src[pos..].find('\n') {
                Some(i) => pos + i,
                None => src.len(),
            };
            let mut line = &src[pos..end];
            let next = end + 1;
            if line.ends_with('\r') {
                line = &line[..line.len() - 1];
            }
            lineno += 1;
            pos = next;
            if pos > src.len() && line.is_empty() {
                break;
            }
            // front matter
            if !fm_checked {
                fm_checked = true;
                if line == "---" {
                    // find terminator
                    let mut q = pos;
                    let mut ln = lineno;
                    let mut found: Option<(usize, u32, usize)> = None;
                    while q <= src.len() {
                        let e = match src[q..].find('\n') {
                            Some(i) => q + i,
                            None => src.len(),
                        };
                        let mut l = &src[q..e];
                        if l.ends_with('\r') {
                            l = &l[..l.len() - 1];
                        }
                        ln += 1;
                        if l == "---" || l == "..." {
                            found = Some((e + 1, ln, q));
                            break;
                        }
                        q = e + 1;
                    }
                    match found {
                        Some((after, endln, tstart)) => {
                            let body = &src[pos.min(tstart)..tstart];
                            self.front_matter(body, lineno + 1, &mut hdr);
                            pos = after;
                            lineno = endln;
                            continue;
                        }
                        None => {
                            self.diag(Severity::E, "E-SYNTAX", 1, "front matter is not terminated");
                        }
                    }
                }
            }
            let lb = line.as_bytes();
            if let Some((ch, len)) = fence {
                if fence_close(lb, ch, len) {
                    fence = None;
                }
                continue;
            }
            if in_comment {
                if line.contains("-->") {
                    in_comment = false;
                }
                continue;
            }
            if let Some(f) = fence_open(lb) {
                fence = Some(f);
                if !last_other {
                    raws.push(Raw::Other);
                    last_other = true;
                }
                continue;
            }
            let tl = line.trim_start();
            if tl.starts_with("<!--") {
                if !tl[4..].contains("-->") {
                    in_comment = true;
                }
                continue;
            }
            if lb.is_empty() {
                continue;
            }
            if lb[0] == b'#' {
                if let Some((level, text)) = parse_heading(line) {
                    raws.push(Raw::Heading { line: lineno, level, text });
                    last_other = false;
                    continue;
                }
            }
            if let Some((label, d)) = parse_refdef(line) {
                self.refdefs.entry(label).or_insert(d);
                if !last_other {
                    raws.push(Raw::Other);
                    last_other = true;
                }
                continue;
            }
            if let Some((indent, body)) = parse_item(line) {
                raws.push(Raw::Item { line: lineno, indent, body });
                last_other = false;
                continue;
            }
            if lb.iter().any(|c| !is_ws(*c)) && !last_other {
                raws.push(Raw::Other);
                last_other = true;
            }
        }

        // ---- build tree ----
        let mut nodes: Vec<Node> = Vec::new();
        let mut stack: Vec<Slot> = Vec::new();
        let mut title = String::new();
        let mut title_line = 1u32;
        let mut have_h1 = false;
        let mut suffix = Suffix::default();
        let mut i = 0;
        while i < raws.len() {
            match &raws[i] {
                Raw::Other => {}
                Raw::Heading { line, level, text } => {
                    let (line, level, text) = (*line, *level, *text);
                    if level == 1 && !have_h1 {
                        have_h1 = true;
                        title_line = line;
                        let ext = self.extract(text, line);
                        title = text[ext.ts..ext.te].to_string();
                        suffix = ext.suffix;
                        // header run
                        i = self.header_run(&raws, i + 1, &mut hdr) ;
                        continue;
                    }
                    // pop items and equal/deeper headings
                    loop {
                        match stack.last() {
                            Some(Slot::Head { level: l, .. }) if *l < level => break,
                            None => break,
                            _ => self.pop_slot(&mut stack, &mut nodes),
                        }
                    }
                    let ext = self.extract(text, line);
                    let node = Node {
                        kind: NodeKind::Heading,
                        line,
                        checkbox: Checkbox::None,
                        title: text[ext.ts..ext.te].to_string(),
                        date: ext.suffix.date,
                        suffix_date: None,
                        range: ext.suffix.range,
                        duration: ext.suffix.duration,
                        tags: ext.suffix.tags,
                        id: ext.suffix.id,
                        keys: ext.suffix.keys,
                        link: None,
                        link_resolved: None,
                        forecast: None,
                        children: Vec::new(),
                    };
                    stack.push(Slot::Head { level, node });
                }
                Raw::Item { line, indent, body } => {
                    let (line, indent, body) = (*line, *indent, *body);
                    self.item(line, indent, body, &mut stack, &mut nodes, &mut hdr);
                }
            }
            i += 1;
        }
        while !stack.is_empty() {
            self.pop_slot(&mut stack, &mut nodes);
        }

        // ---- header interpretation ----
        let mut proj = Project {
            path: path.to_string(),
            title,
            id: None,
            owner: None,
            stakeholders: Vec::new(),
            commitment: None,
            state: None,
            header: BTreeMap::new(),
            suffix,
            nodes,
            refdefs: std::mem::take(&mut self.refdefs),
            diagnostics: Vec::new(),
            forecast: None,
        };
        for (k, (v, ln)) in &hdr {
            proj.header.insert(k.clone(), v.clone());
            match k.as_str() {
                "owner" => {
                    let a = self.single_value(v);
                    match a {
                        Some(val) if self.typecheck("owner", &val) => {
                            self.value_diag(&val, *ln);
                            proj.owner = Some(val)
                        }
                        _ => self.diag(Severity::E, "E-SYNTAX", *ln, format!("owner must be a person or @handle: {}", v)),
                    }
                }
                "stakeholders" => {
                    for part in v.split(',') {
                        let part = part.trim();
                        if part.is_empty() {
                            continue;
                        }
                        match self.single_value(part) {
                            Some(val) if val.kind == ValueKind::Person || (val.kind == ValueKind::Word && is_handle(&val.text)) => {
                                proj.stakeholders.push(val)
                            }
                            _ => self.diag(Severity::E, "E-SYNTAX", *ln, format!("stakeholder must be a person or @handle: {}", part)),
                        }
                    }
                }
                "commitment" => {
                    let w = v.trim();
                    let ex = if let Some((x, y)) = split_range_word(w) {
                        Some(self.range_expr(x, y, *ln))
                    } else if let Some(val) = self.single_value(w).filter(|x| x.kind == ValueKind::Range) {
                        let (x, y) = val.text.split_once("..").unwrap();
                        Some(self.range_expr(x.trim(), y.trim(), *ln))
                    } else if parse_date(w) != DateParse::NotDate {
                        Some(self.date_expr(w, *ln))
                    } else {
                        self.diag(Severity::E, "E-DATE", *ln, format!("commitment is not a date expression: {}", w));
                        None
                    };
                    proj.commitment = ex.flatten();
                }
                "state" => proj.state = Some(v.clone()),
                "id" => proj.id = Some(v.clone()),
                _ => {}
            }
        }
        if proj.owner.is_none() {
            self.diag(Severity::W, "W-NO-OWNER", title_line, "project has no Owner");
        }
        // local refs
        let refs = std::mem::take(&mut self.refs);
        for (ln, r) in refs {
            if !self.ids.contains_key(&r[1..]) {
                self.diag(Severity::E, "E-REF-LOCAL", ln, format!("no item with id '{}' in this file", &r[1..]));
            }
        }
        let mut d = std::mem::take(&mut self.diags);
        d.sort_by_key(|x| x.line);
        proj.diagnostics = d;
        proj
    }

    fn single_value(&mut self, text: &str) -> Option<Value> {
        let mut atoms = std::mem::take(&mut self.atoms);
        atomise(text, &self.refdefs, &mut atoms);
        let v = self.value_from_atoms(text, &atoms);
        self.atoms = atoms;
        v
    }

    /// Detect the property-list header starting at raws[from]; returns the index after the run.
    fn header_run(&mut self, raws: &[Raw<'a>], from: usize, hdr: &mut BTreeMap<String, (String, u32)>) -> usize {
        let first_indent = match raws.get(from) {
            Some(Raw::Item { indent, .. }) => *indent,
            _ => return from,
        };
        let mut j = from;
        let mut in_run = false;
        let mut found: Vec<(String, String, u32)> = Vec::new();
        while j < raws.len() {
            match &raws[j] {
                Raw::Item { line, indent, body } => {
                    if *indent > first_indent && in_run {
                        j += 1;
                        continue;
                    }
                    if *indent != first_indent {
                        break;
                    }
                    match header_kv(body) {
                        Some((k, v)) => {
                            found.push((k.to_ascii_lowercase(), v.to_string(), *line));
                            in_run = true;
                            j += 1;
                        }
                        None => break,
                    }
                }
                _ => break,
            }
        }
        for (k, v, ln) in found {
            hdr.insert(k, (v, ln));
        }
        j
    }

    fn front_matter(&mut self, body: &str, first_line: u32, hdr: &mut BTreeMap<String, (String, u32)>) {
        let mut cur: Option<(String, Vec<String>, u32)> = None;
        let flush = |cur: &mut Option<(String, Vec<String>, u32)>, hdr: &mut BTreeMap<String, (String, u32)>| {
            if let Some((k, items, ln)) = cur.take() {
                hdr.insert(k, (items.join(", "), ln));
            }
        };
        let mut ln = first_line;
        for l in body.split('\n') {
            let l = l.trim_end_matches('\r');
            let t = l.trim();
            if t.is_empty() || t.starts_with('#') {
                ln += 1;
                continue;
            }
            if l.starts_with(' ') || l.starts_with('\t') || t.starts_with("- ") {
                if let Some(item) = t.strip_prefix("- ") {
                    if let Some((_, items, _)) = cur.as_mut() {
                        items.push(unquote(item.trim()).to_string());
                    }
                }
            } else if let Some((k, v)) = t.split_once(':') {
                flush(&mut cur, hdr);
                let k = k.trim().to_ascii_lowercase();
                let v = v.trim();
                if v.is_empty() {
                    cur = Some((k, Vec::new(), ln));
                } else {
                    let v = unquote(v);
                    let v = if v.starts_with('[') && v.ends_with(']') { &v[1..v.len() - 1] } else { v };
                    hdr.insert(k, (v.to_string(), ln));
                }
            }
            ln += 1;
        }
        flush(&mut cur, hdr);
    }
}

fn unquote(s: &str) -> &str {
    let b = s.as_bytes();
    if b.len() >= 2 && ((b[0] == b'"' && b[b.len() - 1] == b'"') || (b[0] == b'\'' && b[b.len() - 1] == b'\'')) {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// `^([A-Za-z][A-Za-z0-9 _-]*):\s*(.*)$`
fn header_kv(body: &str) -> Option<(&str, &str)> {
    let b = body.as_bytes();
    if b.is_empty() || !b[0].is_ascii_alphabetic() {
        return None;
    }
    let mut i = 1;
    while i < b.len() && (b[i].is_ascii_alphanumeric() || matches!(b[i], b' ' | b'_' | b'-')) {
        i += 1;
    }
    if i >= b.len() || b[i] != b':' {
        return None;
    }
    Some((&body[..i], body[i + 1..].trim()))
}

fn is_subproject_target(t: &str) -> bool {
    if t.is_empty() || t.contains('#') {
        return false;
    }
    let p = match t.find('?') {
        Some(i) if t.starts_with("http://") || t.starts_with("https://") => &t[..i],
        _ => t,
    };
    ends_md(p)
}
