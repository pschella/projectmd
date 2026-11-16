//! `ready`: open work that may start now (after scheduling). Pure; the caller supplies link lookup.

use crate::contract::{deliverable_state, project_state, Child, State};
use crate::date::Civil;
use crate::graph::{resolve, split_fragment};
use crate::model::*;

#[derive(Clone, Debug)]
pub struct Ready {
    pub line: u32,
    pub title: String,
    pub owner: String,
    pub kind: NodeKind,
    pub due: Option<Civil>,
    pub declared: Option<Civil>,
    /// Latest `after: DATE` that is already past: when the work actually began.
    pub started: Option<Civil>,
}

fn open_below(n: &Node) -> bool {
    n.children.iter().any(|c| match c.checkbox {
        Checkbox::Cancelled => false,
        Checkbox::Open => true,
        _ => open_below(c),
    })
}

fn key_text(n: &Node, k: &str) -> Option<String> {
    n.keys.get(k).and_then(|v| v.first()).map(|v| v.text.trim().to_string())
}

fn met<'a>(p: &Project, n: &Node, today: Civil, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> bool {
    n.keys.get("after").into_iter().flatten().all(|v| {
        if let Some(e) = v.end {
            return e <= today;
        }
        let Some(t) = p.find_id(v.text.trim_start_matches('^')) else { return false };
        match (t.checkbox, &t.link) {
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
        }
    })
}

/// Ready items of a scheduled project, in file order. `owner` filters (case-insensitive substring).
pub fn ready<'a>(p: &Project, today: Civil, owner: Option<&str>, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Vec<Ready> {
    fn go<'a>(p: &Project, ns: &[Node], ok: bool, dated: bool, own: &str, a: (Civil, &dyn Fn(&str) -> Option<Child<'a>>), out: &mut Vec<Ready>) {
        for n in ns {
            if n.checkbox == Checkbox::Cancelled {
                continue;
            }
            let ok = ok && met(p, n, a.0, a.1);
            let dated = dated || n.keys.contains_key("after");
            let started = n.keys.get("after").into_iter().flatten().filter_map(|v| v.end).filter(|e| *e <= a.0).max();
            let own = key_text(n, "owner").unwrap_or_else(|| own.to_string());
            if ok && n.checkbox == Checkbox::Open && matches!(n.kind, NodeKind::Task | NodeKind::Milestone) && !n.is_link_node() && !open_below(n) {
                let declared = if n.kind == NodeKind::Milestone { n.date.as_ref().map(|d| d.end) } else { None };
                // A dated milestone with no `after:` is an event, not startable work.
                if declared.is_some_and(|d| !dated && d > a.0) {
                    go(p, &n.children, ok, dated, &own, a, out);
                    continue;
                }
                out.push(Ready { line: n.line, title: n.title.clone(), owner: own.clone(), kind: n.kind, due: n.forecast.and_then(|f| f.end), declared, started });
            }
            go(p, &n.children, ok, dated, &own, a, out);
        }
    }
    let mut out = Vec::new();
    go(p, &p.nodes, true, false, p.owner.as_ref().map(|v| v.text.trim()).unwrap_or(""), (today, lookup), &mut out);
    if let Some(o) = owner {
        let o = o.to_lowercase();
        out.retain(|r| r.owner.to_lowercase().contains(&o));
    }
    out
}
