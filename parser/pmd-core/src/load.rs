//! `schedule --rows` and `load` (informative: nothing is stored). `rows` flattens a scheduled
//! project into one row per planning item; `load` counts, per owner and ISO week, how many leaf items
//! with a span run at once. Pure: the caller loads and schedules the files.

use crate::date::Civil;
use crate::model::*;
use crate::ready::{open_below, owner_matches};
use serde_json::{json, Value};
use std::collections::BTreeMap;

/// The owner label of rows without one.
pub const NO_OWNER: &str = "(no owner)";

/// One planning item of a scheduled file (a row of `pmd schedule --rows`).
#[derive(Clone, Debug)]
pub struct Row {
    pub file: String,
    pub line: u32,
    pub id: Option<String>,
    pub title: String,
    /// `task`, `milestone`, `subproject` or `deliverable` (the JSON names of the node kinds).
    pub kind: &'static str,
    /// `open`, `done` or `cancelled`.
    pub state: &'static str,
    /// Effective owner: the item's own `owner:`, else the nearest ancestor's, else the file's `Owner:`.
    pub owner: Option<String>,
    /// The owner did not come from the item itself (false when there is no owner).
    pub owner_inherited: bool,
    pub start: Option<Civil>,
    pub end: Option<Civil>,
    pub declared_end: Option<Civil>,
    pub slack_days: Option<i64>,
    pub estimate_days: Option<u32>,
    pub tags: Vec<String>,
    /// Line of the nearest enclosing planning item.
    pub parent_line: Option<u32>,
    /// No live (open) sub-item.
    pub leaf: bool,
}

fn kind_name(k: NodeKind) -> &'static str {
    match k {
        NodeKind::Task => "task",
        NodeKind::Milestone => "milestone",
        NodeKind::Subproject => "subproject",
        NodeKind::Deliverable => "deliverable",
        NodeKind::Heading => "heading",
    }
}

/// The rows of one scheduled project, in file order. Headings are not rows; cancelled items are.
pub fn rows(p: &Project) -> Vec<Row> {
    let file_owner = p.owner.as_ref().map(|v| v.text.trim().to_string()).filter(|o| !o.is_empty());
    let mut out = Vec::new();
    walk(p, &p.nodes, file_owner, None, &mut out);
    out
}

fn walk(p: &Project, ns: &[Node], inherited: Option<String>, parent: Option<u32>, out: &mut Vec<Row>) {
    for n in ns {
        if n.kind == NodeKind::Heading || n.checkbox == Checkbox::None {
            walk(p, &n.children, inherited.clone(), parent, out);
            continue;
        }
        let own = n.keys.get("owner").and_then(|v| v.first()).map(|v| v.text.trim().to_string()).filter(|o| !o.is_empty());
        let owner_inherited = own.is_none() && inherited.is_some();
        let owner = own.or_else(|| inherited.clone());
        let f = n.forecast.unwrap_or_default();
        out.push(Row {
            file: p.path.clone(),
            line: n.line,
            id: n.id.clone(),
            title: n.link.as_ref().map(|l| l.text.clone()).unwrap_or_else(|| n.title.clone()),
            kind: kind_name(n.kind),
            state: match n.checkbox {
                Checkbox::Done => "done",
                Checkbox::Cancelled => "cancelled",
                _ => "open",
            },
            owner: owner.clone(),
            owner_inherited,
            start: f.start,
            end: f.end,
            declared_end: f.declared_end,
            slack_days: f.slack_days,
            estimate_days: n.duration.as_ref().map(|d| d.days),
            tags: n.tags.clone(),
            parent_line: parent,
            leaf: !open_below(n),
        });
        walk(p, &n.children, owner, Some(n.line), out);
    }
}

/// One row as `pmd schedule --rows` prints it: the fields in the order of docs/schema/rows.schema.json.
pub fn row_json(r: &Row) -> Value {
    let day = |d: Option<Civil>| d.map(|d| d.to_string());
    json!({
        "file": r.file,
        "line": r.line,
        "id": r.id,
        "title": r.title,
        "kind": r.kind,
        "state": r.state,
        "owner": r.owner,
        "owner_inherited": r.owner_inherited,
        "start": day(r.start),
        "end": day(r.end),
        "declared_end": day(r.declared_end),
        "slack_days": r.slack_days,
        "estimate_days": r.estimate_days,
        "tags": r.tags,
        "parent_line": r.parent_line,
        "leaf": r.leaf,
    })
}

/// An item that runs in some week, as `pmd load` lists it.
#[derive(Clone, Debug)]
pub struct Item {
    pub file: String,
    pub line: u32,
    pub title: String,
}

/// One ISO week (Monday to Sunday) in which an owner has `count` items running at once.
#[derive(Clone, Debug)]
pub struct WeekLoad {
    pub week: String,
    pub count: usize,
    pub items: Vec<Item>,
}

#[derive(Clone, Debug)]
pub struct OwnerLoad {
    pub owner: String,
    /// Weeks with at least one item, in date order.
    pub weeks: Vec<WeekLoad>,
}

/// The Monday (days since 1970-01-01) of the ISO week containing day `z`. 1970-01-01 was a Thursday.
fn monday_of(z: i64) -> i64 {
    z - (z + 3).rem_euclid(7)
}

/// ISO week label `YYYY-Www` of the week that starts on the Monday `z` (days).
fn week_label(z: i64) -> String {
    let thu = Civil::from_days(z + 3);
    let jan1 = Civil::new(thu.y, 1, 1).to_days();
    format!("{}-W{:02}", thu.y, (thu.to_days() - jan1) / 7 + 1)
}

/// How many items each owner has running at once, week by week (informative). Only open leaf rows
/// with both `start` and `end` and `end > start` count: a point is not load. Weeks run from the week
/// containing `today` to the week containing the latest end. `owner` filters like `ready --owner`.
/// Only weeks with at least one item are returned; owners without any are left out.
pub fn load(rows: &[Row], today: Civil, owner: Option<&str>) -> Vec<OwnerLoad> {
    let spans: Vec<(&Row, i64, i64)> = rows
        .iter()
        .filter(|r| r.state == "open" && r.leaf && owner_matches(r.owner.as_deref().unwrap_or(""), owner))
        .filter_map(|r| match (r.start, r.end) {
            (Some(s), Some(e)) if e > s => Some((r, s.to_days(), e.to_days())),
            _ => None,
        })
        .collect();
    let Some(last) = spans.iter().map(|s| s.2).max() else {
        return Vec::new();
    };
    let mut by_owner: BTreeMap<String, Vec<(&Row, i64, i64)>> = BTreeMap::new();
    for s in spans {
        by_owner.entry(s.0.owner.clone().unwrap_or_else(|| NO_OWNER.to_string())).or_default().push(s);
    }
    let mut out = Vec::new();
    for (owner, items) in by_owner {
        let mut weeks = Vec::new();
        let mut m = monday_of(today.to_days());
        while m <= last {
            let sunday = m + 6;
            let hit: Vec<Item> = items
                .iter()
                .filter(|(_, s, e)| *s <= sunday && *e >= m)
                .map(|(r, _, _)| Item { file: r.file.clone(), line: r.line, title: r.title.clone() })
                .collect();
            if !hit.is_empty() {
                weeks.push(WeekLoad { week: week_label(m), count: hit.len(), items: hit });
            }
            m += 7;
        }
        if !weeks.is_empty() {
            out.push(OwnerLoad { owner, weeks });
        }
    }
    out
}

/// The loads as `pmd load --json` prints them.
pub fn load_json(today: Civil, loads: &[OwnerLoad]) -> Value {
    let owners: Vec<Value> = loads
        .iter()
        .map(|o| {
            let weeks: Vec<Value> = o
                .weeks
                .iter()
                .map(|w| {
                    let items: Vec<Value> = w.items.iter().map(|i| json!({ "file": i.file, "line": i.line, "title": i.title })).collect();
                    json!({ "week": w.week, "count": w.count, "items": items })
                })
                .collect();
            json!({ "owner": o.owner, "weeks": weeks })
        })
        .collect();
    json!({ "today": today.to_string(), "owners": owners })
}
