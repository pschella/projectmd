//! `why` (D60): follow the drivers recorded by `schedule` from an item back to its root cause in
//! one file. Pure; a hop that ends on a link names the child file, and the caller drills down.

use crate::date::Civil;
use crate::graph::split_fragment;
use crate::model::*;
use crate::record::PromiseRecord;
use std::collections::{HashMap, HashSet};

/// One item in a chain.
#[derive(Clone, Debug)]
pub struct Hop {
    pub path: String,
    pub line: u32,
    pub title: String,
    pub owner: String,
    pub start: Option<Civil>,
    pub end: Option<Civil>,
    pub declared: Option<Civil>,
    /// Declared end minus the earliest possible end (negative: late).
    pub slack: Option<i64>,
    /// What fixed this item's end.
    pub driver: Driver,
    /// What fixed its start, when the end is start plus estimate.
    pub start_driver: Option<Driver>,
    /// For a hop driven by a link's promise: the child file and the milestone id (none: its Commitment).
    pub child: Option<(String, Option<String>)>,
    /// For a promise hop: the track record of the child file (set by the caller, who can read it).
    pub record: Option<PromiseRecord>,
}

/// Every item of a project by line, with its effective owner (own `owner:`, ancestor's, file Owner).
pub(crate) fn node_index(p: &Project) -> HashMap<u32, (&Node, String)> {
    fn go<'a>(ns: &'a [Node], own: &str, out: &mut HashMap<u32, (&'a Node, String)>) {
        for n in ns {
            let o = n.keys.get("owner").and_then(|v| v.first()).map(|v| v.text.trim().to_string()).unwrap_or_else(|| own.to_string());
            out.insert(n.line, (n, o.clone()));
            go(&n.children, &o, out);
        }
    }
    let mut out = HashMap::new();
    go(&p.nodes, p.owner.as_ref().map(|v| v.text.trim()).unwrap_or(""), &mut out);
    out
}

/// Lines of the items that have an `E-CONFLICT` (the project-level one, on line 1, is the project end).
pub fn conflict_lines(p: &Project) -> Vec<u32> {
    p.diagnostics.iter().filter(|d| d.code == "E-CONFLICT" && d.line > 1).map(|d| d.line).collect()
}

/// The chain for the item on `line`, or for the project's end (`None`), in a scheduled project.
pub fn why(p: &Project, line: Option<u32>) -> Vec<Hop> {
    let idx = node_index(p);
    let mut out = Vec::new();
    let mut next = line;
    if line.is_none() {
        let Some(f) = p.forecast else { return out };
        let top = p.nodes.iter().rev().find(|n| f.end.is_some() && n.forecast.and_then(|x| x.end) == f.end);
        let driver = match (&p.done, top) {
            (Some(_), _) => Driver::new("done"),
            (None, None) => Driver::new("none"),
            (None, Some(t)) => Driver::at("child", t.line),
        };
        next = driver.line;
        out.push(Hop { path: p.path.clone(), line: 1, title: p.title.clone(), owner: p.owner.as_ref().map(|v| v.text.trim().to_string()).unwrap_or_default(), start: f.start, end: f.end, declared: f.declared_end, slack: f.slack_days, driver, start_driver: None, child: None, record: None });
    }
    let mut seen = HashSet::new();
    while let Some(l) = next.filter(|l| seen.insert(*l)) {
        let Some((n, owner)) = idx.get(&l) else { break };
        let (Some(d), f) = (n.driver.clone(), n.forecast.unwrap_or_default()) else { break };
        next = match d.kind {
            "child" | "after" => d.line,
            "estimate" => n.start_driver.as_ref().filter(|s| s.kind == "after").and_then(|s| s.line),
            // a declared end that sits above what actually determines it: follow the latest predecessor or child
            "declared" | "by" => d.line,
            _ => None,
        };
        let child = match (&n.link, &n.link_resolved) {
            (Some(l), Some(r)) if d.kind == "promise" => Some((r.clone(), split_fragment(&l.target).1.map(str::to_string))),
            _ => None,
        };
        let title = n.link.as_ref().map(|l| l.text.clone()).unwrap_or_else(|| n.title.clone());
        out.push(Hop { path: p.path.clone(), line: l, title, owner: owner.clone(), start: f.start, end: f.end, declared: f.declared_end, slack: f.slack_days, driver: d, start_driver: n.start_driver.clone(), child, record: None });
    }
    out
}
