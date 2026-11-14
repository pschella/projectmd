//! Data model (GRAMMAR §9). Everything is `Serialize` with lowercase field names.

use crate::date::Civil;
use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub enum Severity {
    E,
    W,
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
        let s = if self.severity == Severity::E { "E" } else { "W" };
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
}

#[derive(Clone, Debug, Serialize)]
pub struct Node {
    pub kind: NodeKind,
    pub line: u32,
    pub checkbox: Checkbox,
    pub title: String,
    /// Leading or suffix date.
    pub date: Option<DateExpr>,
    /// A suffix date when the item also has a leading date (`date` then holds the leading one).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suffix_date: Option<DateExpr>,
    /// Declared range (suffix range or leading range).
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
    pub children: Vec<Node>,
}

impl Node {
    pub fn has_tag(&self, t: &str) -> bool {
        self.tags.iter().any(|x| x == t)
    }
    pub fn walk<'a>(&'a self, f: &mut impl FnMut(&'a Node)) {
        f(self);
        for c in &self.children {
            c.walk(f);
        }
    }
}

/// Suffix of the H1 (applies to the project).
#[derive(Clone, Debug, Default, Serialize)]
pub struct Suffix {
    pub date: Option<DateExpr>,
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
    pub id: Option<String>,
    pub owner: Option<Value>,
    pub stakeholders: Vec<Value>,
    pub commitment: Option<DateExpr>,
    pub state: Option<String>,
    pub header: BTreeMap<String, String>,
    pub suffix: Suffix,
    pub nodes: Vec<Node>,
    pub refdefs: BTreeMap<String, RefDef>,
    pub diagnostics: Vec<Diagnostic>,
    /// Span of the top-level nodes (set by `schedule`); `null` until then.
    pub forecast: Option<Forecast>,
}

impl Project {
    pub fn walk<'a>(&'a self, mut f: impl FnMut(&'a Node)) {
        for n in &self.nodes {
            n.walk(&mut f);
        }
    }
    pub fn count_nodes(&self) -> usize {
        let mut c = 0;
        self.walk(|_| c += 1);
        c
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

/// A person from the profile's people registry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Person {
    pub name: String,
    pub email: Option<String>,
}

/// Parser options (core keys are always known; a profile adds more).
#[derive(Clone, Debug, Default, Serialize)]
pub struct Options {
    pub extra_keys: Vec<String>,
    pub repeatable: Vec<String>,
    pub states: Vec<String>,
    pub people: BTreeMap<String, Person>,
}
