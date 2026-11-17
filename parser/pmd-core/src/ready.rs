//! `ready` (D59): the queue of one file: open work that may start now (`do`), and the contract moves it
//! owes (`decide`, `fix`); `child_moves` lists what its children owe it. Pure; the caller supplies link lookup.

use crate::contract::{contracts_of, deliverable_state, project_state, Child, State};
use std::collections::BTreeMap;
use crate::date::Civil;
use crate::graph::{resolve, split_fragment};
use crate::model::*;
use crate::record::{promise_record, promise_text, PromiseRecord};
use crate::why::node_index;

#[derive(Clone, Debug)]
pub struct Ready {
    /// File the entry is located in (a child's file for `answer` and `deliver`).
    pub path: String,
    pub line: u32,
    pub title: String,
    pub owner: String,
    /// `do`, `decide`, `fix`, `answer` or `deliver`.
    pub kind: &'static str,
    /// Task or milestone (`do` entries).
    pub node: NodeKind,
    pub detail: String,
    pub due: Option<Civil>,
    pub declared: Option<Civil>,
    /// Latest `after: DATE` that is already past: when the work actually began.
    pub started: Option<Civil>,
    /// `decide`: the track record of the child's promises.
    pub record: Option<PromiseRecord>,
    /// `answer` and `deliver` (see `merge_asks`): how many askers point at this promise; 0 otherwise.
    pub askers: usize,
    /// `answer` and `deliver`: the asker's name, which is the earliest asker after `merge_asks`.
    pub asker: String,
    /// `answer` and `deliver`: the child's promise, when it has one.
    pub promised: Option<Civil>,
}

/// Is some item below `n` open (not done, not cancelled)? Such an item is not a leaf.
pub(crate) fn open_below(n: &Node) -> bool {
    n.children.iter().any(|c| match c.checkbox {
        Checkbox::Cancelled => false,
        Checkbox::Open => true,
        _ => open_below(c),
    })
}

fn key_text(n: &Node, k: &str) -> Option<String> {
    n.keys.get(k).and_then(|v| v.first()).map(|v| v.text.trim().to_string())
}

/// The `after:` values of `n` that are not met today, as text: a future date, or the title of the
/// unmet item (its `^id` when untitled). Empty when every `after:` is met.
fn unmet<'a>(p: &Project, n: &Node, today: Civil, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Vec<String> {
    n.keys
        .get("after")
        .into_iter()
        .flatten()
        .filter_map(|v| {
            if let Some(e) = v.end {
                return (e > today).then(|| e.to_string());
            }
            let id = v.text.trim_start_matches('^');
            let Some(t) = p.find_id(id) else { return Some(v.text.trim().to_string()) };
            let met = match (t.checkbox, &t.link) {
                (Checkbox::Done | Checkbox::Cancelled, _) => true,
                (_, Some(l)) => {
                    let (path, frag) = split_fragment(&l.target);
                    match lookup(&resolve(&p.path, path)) {
                        Some(Child::Read(c)) => match (t.kind, frag) {
                            (NodeKind::Deliverable, Some(f)) => deliverable_state(today, c, f).0 == State::Delivered,
                            _ => project_state(today, c) == State::Delivered,
                        },
                        _ => false,
                    }
                }
                _ => false,
            };
            (!met).then(|| if t.title.trim().is_empty() { format!("^{}", id) } else { t.title.trim().to_string() })
        })
        .collect()
}

/// A task or milestone (not a link line, not cancelled), with what the queue and the board derive from it.
pub struct Item<'p> {
    pub node: &'p Node,
    /// Effective owner: its own `owner:`, else the nearest ancestor's, else the file's Owner.
    pub owner: String,
    /// Unmet `after:` of the item and of its ancestors (see `unmet`); empty when it may start.
    pub blockers: Vec<String>,
    /// Some `after:` on the item or on an ancestor.
    pub dated: bool,
    /// Latest own `after: DATE` on or before today: when the work began.
    pub started: Option<Civil>,
    /// No open item below it.
    pub leaf: bool,
}

#[derive(Default)]
struct Inherited {
    blockers: Vec<String>,
    dated: bool,
    owner: String,
}

/// The planning items of a scheduled project, in file order. Cancelled items and their subtrees are left out.
pub fn items<'p, 'l>(p: &'p Project, today: Civil, lookup: &dyn Fn(&str) -> Option<Child<'l>>) -> Vec<Item<'p>> {
    let root = Inherited { owner: p.owner.as_ref().map(|v| v.text.trim().to_string()).unwrap_or_default(), ..Default::default() };
    let mut out = Vec::new();
    go_items(p, &p.nodes, today, lookup, &root, &mut out);
    out
}

fn go_items<'p, 'l>(p: &'p Project, ns: &'p [Node], today: Civil, lookup: &dyn Fn(&str) -> Option<Child<'l>>, inh: &Inherited, out: &mut Vec<Item<'p>>) {
    for n in ns {
        if n.checkbox == Checkbox::Cancelled {
            continue;
        }
        let mut blockers = inh.blockers.clone();
        blockers.extend(unmet(p, n, today, lookup));
        let dated = inh.dated || n.keys.contains_key("after");
        let owner = key_text(n, "owner").unwrap_or_else(|| inh.owner.clone());
        if matches!(n.kind, NodeKind::Task | NodeKind::Milestone) && !n.is_link_node() {
            let started = n.keys.get("after").into_iter().flatten().filter_map(|v| v.end).filter(|e| *e <= today).max();
            out.push(Item { node: n, owner: owner.clone(), blockers: blockers.clone(), dated, started, leaf: !open_below(n) });
        }
        go_items(p, &n.children, today, lookup, &Inherited { blockers, dated, owner }, out);
    }
}

/// Ready items of a scheduled project, in file order. `owner` filters (case-insensitive substring).
pub fn ready<'a>(p: &Project, today: Civil, owner: Option<&str>, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Vec<Ready> {
    let mut out = Vec::new();
    for it in items(p, today, lookup) {
        let n = it.node;
        if !it.blockers.is_empty() || n.checkbox != Checkbox::Open || !it.leaf {
            continue;
        }
        let declared = if n.kind == NodeKind::Milestone { n.date.as_ref().map(|d| d.end) } else { None };
        // A dated milestone with no `after:` is an event, not startable work.
        if declared.is_some_and(|d| !it.dated && d > today) {
            continue;
        }
        let detail = declared.map(|d| format!("(declared {})", d)).unwrap_or_default();
        out.push(Ready { path: p.path.clone(), line: n.line, title: n.title.clone(), owner: it.owner.clone(), kind: "do", node: n.kind, detail, due: n.forecast.and_then(|f| f.end), declared, started: it.started, record: None, askers: 0, asker: String::new(), promised: None });
    }
    let idx = node_index(p);
    for c in contracts_of(p, today, lookup) {
        let (kind, mut detail) = match (c.state, c.promise) {
            (State::Disputed, Some(d)) => ("decide", format!("promises {}, asked by {}", d, c.by)),
            (State::Disputed, None) if c.cancelled => ("decide", format!("was cancelled, asked by {}", c.by)),
            (State::Unknown, _) => ("fix", format!("cannot read {}", c.target)),
            _ => continue,
        };
        let rec = match lookup(&c.target) {
            Some(Child::Read(ch)) if kind == "decide" => promise_record(ch),
            _ => None,
        };
        if let Some(r) = &rec {
            detail = format!("{}, {}", detail, promise_text(r));
        }
        out.push(Ready { path: p.path.clone(), line: c.line, title: c.title, owner: idx[&c.line].1.clone(), kind, node: c.kind, detail, due: Some(c.by), declared: None, started: None, record: rec, askers: 0, asker: String::new(), promised: None });
    }
    out.sort_by_key(|r| r.line);
    keep_owner(out, owner)
}

/// Does `owner` pass the `--owner` filter (case-insensitive substring)? No filter passes everything.
pub(crate) fn owner_matches(owner: &str, filter: Option<&str>) -> bool {
    filter.is_none_or(|o| owner.to_lowercase().contains(&o.to_lowercase()))
}

fn keep_owner(mut out: Vec<Ready>, owner: Option<&str>) -> Vec<Ready> {
    out.retain(|r| owner_matches(&r.owner, owner));
    out
}

/// What the children of `p` owe it, located in the child's file and owned by the child's Owner:
/// `answer` (the parent asked by a date, no promise yet) and `deliver` (needed by a date that has passed).
pub fn child_moves<'a>(p: &Project, today: Civil, owner: Option<&str>, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Vec<Ready> {
    let idx = node_index(p);
    let mut out = Vec::new();
    for c in contracts_of(p, today, lookup) {
        let (kind, detail) = match c.state {
            State::Requested => ("answer", format!("{} asks by {}", p.title, c.by)),
            State::Overdue => ("deliver", format!("{} needed it by {}", p.title, c.by)),
            _ => continue,
        };
        let Some(Child::Read(ch)) = lookup(&c.target) else { continue };
        let frag = idx[&c.line].0.link.as_ref().and_then(|l| split_fragment(&l.target).1);
        let target = frag.and_then(|f| ch.find_id(f));
        let line = target.map_or(1, |m| m.line);
        // the effective owner of the linked item, else the child file's Owner
        let own = match target {
            Some(m) => node_index(ch)[&m.line].1.clone(),
            None => ch.owner.as_ref().map(|v| v.text.trim().to_string()).unwrap_or_default(),
        };
        out.push(Ready { path: ch.path.clone(), line, title: c.title, owner: own, kind, node: NodeKind::Task, detail, due: Some(c.by), declared: None, started: None, record: None, askers: 1, asker: if p.title.is_empty() { p.path.clone() } else { p.title.clone() }, promised: c.promise });
    }
    keep_owner(out, owner)
}

/// One `answer` or `deliver` entry per child promise (same child file and line), however many askers
/// point at it. The entry carries the earliest need-by, the count and the earliest asker's name.
/// Its detail reads `needed by D (N askers; earliest: NAME)`, plus `; promised P` when there is one.
pub fn merge_asks(asks: Vec<Ready>) -> Vec<Ready> {
    let mut groups: BTreeMap<(String, u32), Vec<Ready>> = BTreeMap::new();
    for a in asks {
        groups.entry((a.path.clone(), a.line)).or_default().push(a);
    }
    groups
        .into_values()
        .map(|mut g| {
            g.sort_by_key(|r| r.due);
            let n = g.len();
            let mut first = g.swap_remove(0);
            let due = first.due.map(|d| d.to_string()).unwrap_or_default();
            let promise = first.promised.map(|p| format!("; promised {}", p)).unwrap_or_default();
            let noun = if n == 1 { "asker" } else { "askers" };
            first.detail = format!("needed by {} ({} {}; earliest: {}){}", due, n, noun, first.asker, promise);
            first.askers = n;
            first
        })
        .collect()
}
