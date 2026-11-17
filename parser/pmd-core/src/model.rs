//! Data model (GRAMMAR §8). Everything is `Serialize` with lowercase field names.

use crate::date::Civil;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Severity {
    E,
    W,
    I,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: &'static str,
    pub line: u32,
    pub message: String,
    /// File the diagnostic belongs to (filled by graph-level checks; empty for parser output).
    #[serde(skip_serializing_if = "String::is_empty")]
    pub path: String,
}

impl Diagnostic {
    pub fn new(severity: Severity, code: &'static str, line: u32, message: impl Into<String>) -> Self {
        Diagnostic { severity, code, line, message: message.into(), path: String::new() }
    }
    pub fn is_error(&self) -> bool {
        self.severity == Severity::E
    }
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self.severity {
            Severity::E => "E",
            Severity::W => "W",
            Severity::I => "I",
        };
        write!(f, "{}:{}: {} {} {}", self.path, self.line, s, self.code, self.message)
    }
}

/// A date expression or range, with its expansion to civil days.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct DateExpr {
    /// Source text (ranges are normalised to `A..B`).
    pub text: String,
    pub start: Civil,
    pub end: Civil,
    pub range: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Duration {
    pub n: u32,
    /// `d`, `w` or `m`.
    pub unit: char,
    /// Length in days for `d`/`w`. For `m` this is only a rough size hint (30 per month);
    /// use [`Duration::end_from`] for real calendar arithmetic.
    pub days: u32,
    pub text: String,
}

impl Duration {
    /// Inclusive last day of a span of this duration starting on `start`:
    /// `d`/`w` span N days (start + N - 1); `m` is start + N calendar months - 1 day
    /// (2027-01-15 + 1m ends 2027-02-14).
    pub fn end_from(&self, start: Civil) -> Civil {
        match self.unit {
            'm' => Civil::from_days(start.add_months(self.n as i64).to_days() - 1),
            _ => Civil::from_days(start.to_days() + self.days as i64 - 1),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Checkbox {
    None,
    Open,
    Done,
    Cancelled,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeKind {
    Task,
    Milestone,
    Subproject,
    Deliverable,
    Heading,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ValueKind {
    Word,
    Person,
    Link,
    Range,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Value {
    pub kind: ValueKind,
    pub text: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    /// Expansion when the value is a (valid) date expression or range.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<Civil>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<Civil>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct LinkInfo {
    /// Link text.
    pub text: String,
    pub target: String,
    pub title: Option<String>,
}

fn ser_date<S: serde::Serializer>(d: &Option<Civil>, s: S) -> Result<S::Ok, S::Error> {
    match d {
        Some(c) => s.collect_str(c),
        None => s.serialize_none(),
    }
}

/// Forecast of a node or project; dates serialize as `"YYYY-MM-DD"` strings.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Forecast {
    #[serde(serialize_with = "ser_date")]
    pub start: Option<Civil>,
    #[serde(serialize_with = "ser_date")]
    pub end: Option<Civil>,
    #[serde(serialize_with = "ser_date")]
    pub declared_end: Option<Civil>,
    pub conflict: bool,
    /// Declared end minus forecast end, in days (negative in conflict).
    pub slack_days: Option<i64>,
}

/// What fixed a date (D60). End kinds: `declared`, `estimate`, `child`, `promise`, `by`, `today`, `done`,
/// `after` (a point that follows `line`), `undated`. Start kinds: `after` (item at `line`), `after-date`, `today`, `declared`, `none`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Driver {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
}

impl Driver {
    pub fn new(kind: &'static str) -> Driver {
        Driver { kind, line: None }
    }
    pub fn at(kind: &'static str, line: u32) -> Driver {
        Driver { kind, line: Some(line) }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Node {
    pub kind: NodeKind,
    pub line: u32,
    pub checkbox: Checkbox,
    pub title: String,
    /// Leading date (a milestone's declared end).
    pub date: Option<DateExpr>,
    /// Declared range (`A .. B` after the title).
    pub range: Option<DateExpr>,
    pub duration: Option<Duration>,
    pub tags: Vec<String>,
    pub id: Option<String>,
    pub keys: BTreeMap<String, Vec<Value>>,
    pub link: Option<LinkInfo>,
    /// Normalized path or URL of `link.target` (set by `schedule`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub link_resolved: Option<String>,
    /// Set by `schedule`; `null` until then.
    pub forecast: Option<Forecast>,
    /// Set by `schedule`: what fixed the end of this item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver: Option<Driver>,
    /// Set by `schedule`: what fixed the start, when the end is start plus estimate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_driver: Option<Driver>,
    /// Text under the item, indented deeper than it (D58): paragraphs and non-planning bullets.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub children: Vec<Node>,
}

impl Node {
    pub fn has_tag(&self, t: &str) -> bool {
        self.tags.iter().any(|x| x.eq_ignore_ascii_case(t))
    }
    /// Exact date of the `done:` key, if any.
    pub fn done_date(&self) -> Option<Civil> {
        self.keys.get("done").and_then(|v| v.first()).and_then(|v| v.end)
    }
    /// A link node: sub-project or deliverable.
    pub fn is_link_node(&self) -> bool {
        self.link.is_some()
    }
    /// The link of a deliverable or dated deliverable (target has a `#fragment`).
    pub fn deliverable_link(&self) -> Option<&LinkInfo> {
        self.link.as_ref().filter(|l| l.target.contains('#') && self.kind != NodeKind::Subproject)
    }
    pub fn walk<'a>(&'a self, f: &mut impl FnMut(&'a Node)) {
        f(self);
        for c in &self.children {
            c.walk(f);
        }
    }
}

/// The suffix of an item line.
#[derive(Clone, Debug, Default, Serialize)]
pub struct Suffix {
    pub range: Option<DateExpr>,
    pub duration: Option<Duration>,
    pub tags: Vec<String>,
    pub id: Option<String>,
    pub keys: BTreeMap<String, Vec<Value>>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RefDef {
    pub target: String,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
pub struct Project {
    pub path: String,
    pub title: String,
    pub owner: Option<Value>,
    pub commitment: Option<DateExpr>,
    /// `Done:` header: the project was delivered on this date.
    pub done: Option<DateExpr>,
    pub header: BTreeMap<String, String>,
    /// Lines of the header run (for `fmt`).
    #[serde(skip)]
    pub header_lines: Vec<u32>,
    pub nodes: Vec<Node>,
    pub refdefs: BTreeMap<String, RefDef>,
    pub diagnostics: Vec<Diagnostic>,
    /// Span of the top-level nodes (set by `schedule`); `null` until then.
    pub forecast: Option<Forecast>,
}

impl Project {
    /// Like `walk`, but does not descend into a cancelled `[-]` item (it cancels its descendants).
    pub fn walk_live<'a>(&'a self, mut f: impl FnMut(&'a Node)) {
        fn go<'a>(n: &'a Node, f: &mut impl FnMut(&'a Node)) {
            f(n);
            if n.checkbox != Checkbox::Cancelled {
                for c in &n.children {
                    go(c, f);
                }
            }
        }
        for n in &self.nodes {
            go(n, &mut f);
        }
    }
    pub fn walk<'a>(&'a self, mut f: impl FnMut(&'a Node)) {
        for n in &self.nodes {
            n.walk(&mut f);
        }
    }
    pub fn find_id(&self, id: &str) -> Option<&Node> {
        let mut r = None;
        self.walk(|n| {
            if r.is_none() && n.id.as_deref() == Some(id) {
                r = Some(n);
            }
        });
        r
    }
}

/// Suggested fix for a diagnostic code (FORMAT §11). `None` when the table has none.
pub fn fix_hint(code: &str) -> Option<&'static str> {
    Some(match code {
        "E-SYNTAX" => "Keep one of the conflicting parts, or give the line a title",
        "E-REF-LOCAL" => "Correct the id, or add `^id` to the target item",
        "E-REF-PUBLIC" => "The target must be a dated milestone not marked #private: ask its owner to add one, or depend on the whole project",
        "W-OVERDUE" => "Ask the child's owner for the delivery (if it was delivered, the owner ticks the milestone or sets `Done:`), or move the `by:` if the need changed",
        "W-DISPUTED" => "Parent's move: accept by moving `by:` to the promised date, re-plan, or talk to the child's owner",
        "W-UNKNOWN" => "Check the link and its access",
        "I-REQUESTED" => "Wait, or ask the child's owner",
        "W-KEYLIKE" => "Write a valid value, or reword; tick the box before `done:`; use `by: DATE` for a deadline",
        "E-CYCLE" => "Remove one sub-project link",
        "E-CONFLICT" => "Move the declared date, shorten the estimate, or ask the container to move its `by:`",
        "W-NO-OWNER" => "Add `- Owner: Name <email>` under the title",
        "E-DATE" => "Write a valid date, rough date or range",
        "E-DUP-ID" => "Rename one of them",
        "E-REF-MISSING" => "Fix the path, or restore the file",
        _ => return None,
    })
}

/// Fix hint for one diagnostic: `E-CYCLE` has two meanings (include cycle, `after:` cycle).
pub fn fix_for(d: &Diagnostic) -> Option<&'static str> {
    if d.code == "E-CYCLE" && d.message.contains("`after:`") {
        return Some("Remove one `after:`");
    }
    fix_hint(d.code)
}

/// Do two `Owner:` texts name the same owner? The same `<email>` when both have one, else the
/// same text ignoring case and extra spaces.
pub fn same_owner(a: &str, b: &str) -> bool {
    let email = |t: &str| t.rfind('<').and_then(|i| t[i + 1..].split_once('>')).map(|(e, _)| e.trim().to_ascii_lowercase()).filter(|e| e.contains('@'));
    let norm = |t: &str| t.split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase();
    match (email(a), email(b)) {
        (Some(x), Some(y)) => x == y,
        _ => !a.trim().is_empty() && norm(a) == norm(b),
    }
}
