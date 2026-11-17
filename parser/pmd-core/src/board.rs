//! `board` (D66, informative): one file as four columns, derived from its `after:` and `done:` lines,
//! with the footer (work in progress, throughput, cycle time, flow forecast). Nothing is stored. Pure:
//! the caller schedules the project and supplies the link lookup, as for `ready`.

use crate::contract::Child;
use crate::date::Civil;
use crate::model::*;
use crate::ready::{items, open_below, owner_matches};
use serde_json::json;
use std::collections::{BTreeMap, HashSet};

/// Throughput window: the last 8 weeks, or fewer days when the file is younger (see `window_days`).
const THROUGHPUT_DAYS: i64 = 56;
/// The window is at least one week long.
const MIN_WINDOW_DAYS: i64 = 7;
/// Throughput and the flow forecast need at least this many finished items in the window.
const MIN_THROUGHPUT: usize = 3;
/// The Done column shows items finished within this many days of today.
const RECENT_DAYS: i64 = 14;
/// Cycle time needs at least this many finished items with a recorded start.
const MIN_CYCLE: usize = 3;

#[derive(Clone, Debug)]
pub struct Card {
    pub line: u32,
    pub title: String,
    /// Effective owner; empty when none.
    pub owner: String,
    pub id: Option<String>,
    /// Waiting: the titles (or `^id`s) of the unmet items, or the date of a future `after: DATE`.
    pub waits_on: Vec<String>,
    /// Doing: the recorded start.
    pub since: Option<Civil>,
    /// Done: the `done:` date.
    pub done: Option<Civil>,
}

#[derive(Clone, Copy, Debug)]
pub struct CycleTime {
    /// Median of the inclusive days from start to `done:`.
    pub median_days: f64,
    pub n: usize,
}

/// An open milestone that waits on open items, with the date those items would finish at the throughput.
#[derive(Clone, Debug)]
pub struct Flow {
    pub id: Option<String>,
    pub title: String,
    pub line: u32,
    pub items_left: usize,
    pub about: Civil,
    pub declared: Option<Civil>,
}

#[derive(Clone, Debug, Default)]
pub struct Board {
    pub path: String,
    pub waiting: Vec<Card>,
    pub ready: Vec<Card>,
    pub doing: Vec<Card>,
    pub done: Vec<Card>,
    /// Done items finished more than 14 days ago (not listed).
    pub older_done: usize,
    /// Doing items per owner (`""` for no owner).
    pub wip: BTreeMap<String, usize>,
    /// Items per week, when at least `MIN_THROUGHPUT` items were finished in the window.
    pub throughput: Option<f64>,
    /// The throughput window in days, ending today (set with `throughput`).
    pub throughput_days: i64,
    pub cycle_time: Option<CycleTime>,
    pub flow: Vec<Flow>,
}

/// The throughput window in days: the last 8 weeks, or the days since the earliest recorded start or
/// done date in the file (inclusive of both ends), never less than a week.
fn window_days(p: &Project, t: i64) -> i64 {
    let mut earliest: Option<i64> = None;
    p.walk(|n| {
        for d in n.keys.get("after").into_iter().flatten().filter_map(|v| v.end).chain(n.done_date()) {
            earliest = Some(earliest.map_or(d.to_days(), |e| e.min(d.to_days())));
        }
    });
    earliest.map_or(MIN_WINDOW_DAYS, |e| (t - e + 1).clamp(MIN_WINDOW_DAYS, THROUGHPUT_DAYS))
}

/// The board of a scheduled project. `owner` filters the cards and the work in progress; the
/// throughput, cycle time and flow forecast describe the whole file.
pub fn board<'a>(p: &Project, today: Civil, owner: Option<&str>, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Board {
    let t = today.to_days();
    let window = window_days(p, t);
    let mut b = Board { path: p.path.clone(), ..Default::default() };
    let (mut finished_in_window, mut cycles) = (0usize, Vec::new());
    let its = items(p, today, lookup);
    for it in its.iter().filter(|it| it.leaf) {
        let n = it.node;
        let mine = owner_matches(&it.owner, owner);
        let card = |waits_on: Vec<String>, since: Option<Civil>, done: Option<Civil>| Card { line: n.line, title: n.title.clone(), owner: it.owner.clone(), id: n.id.clone(), waits_on, since, done };
        match n.checkbox {
            Checkbox::Done => {
                // Only what is finished by today counts: a `done:` after today is not a fact yet.
                let Some(d) = n.done_date() else { continue };
                let dd = d.to_days();
                if dd > t {
                    continue;
                }
                if dd > t - window {
                    finished_in_window += 1;
                }
                if let Some(s) = n.keys.get("after").into_iter().flatten().filter_map(|v| v.end).max() {
                    let days = dd - s.to_days() + 1;
                    if days > 0 {
                        cycles.push(days);
                    }
                }
                if !mine {
                    continue;
                }
                if dd > t - RECENT_DAYS {
                    b.done.push(card(vec![], None, Some(d)));
                } else {
                    b.older_done += 1;
                }
            }
            Checkbox::Open => {
                if !mine {
                    continue;
                }
                let declared = if n.kind == NodeKind::Milestone { n.date.as_ref().map(|d| d.end) } else { None };
                if !it.blockers.is_empty() {
                    b.waiting.push(card(it.blockers.clone(), None, None));
                } else if let Some(d) = declared.filter(|d| !it.dated && *d > today) {
                    // A dated milestone with no `after:` before its date waits on its date.
                    b.waiting.push(card(vec![d.to_string()], None, None));
                } else if let Some(s) = it.started {
                    *b.wip.entry(it.owner.clone()).or_insert(0) += 1;
                    b.doing.push(card(vec![], Some(s), None));
                } else {
                    b.ready.push(card(vec![], None, None));
                }
            }
            _ => {}
        }
    }
    if finished_in_window >= MIN_THROUGHPUT {
        b.throughput = Some(finished_in_window as f64 / (window as f64 / 7.0));
        b.throughput_days = window;
    }
    if cycles.len() >= MIN_CYCLE {
        cycles.sort_unstable();
        let n = cycles.len();
        let median_days = if n % 2 == 1 { cycles[n / 2] as f64 } else { (cycles[n / 2 - 1] + cycles[n / 2]) as f64 / 2.0 };
        b.cycle_time = Some(CycleTime { median_days, n: cycles.len() });
    }
    if b.throughput.is_some() {
        for it in &its {
            let m = it.node;
            if m.kind != NodeKind::Milestone || m.checkbox != Checkbox::Open || m.is_link_node() {
                continue;
            }
            let left = open_left(p, m);
            if left == 0 {
                continue;
            }
            // ceil(left / (finished / window weeks)) weeks, in days: the integer form of the same figure.
            let k = finished_in_window as i64;
            let days = (window * left as i64 + k - 1) / k;
            b.flow.push(Flow { id: m.id.clone(), title: m.title.clone(), line: m.line, items_left: left, about: Civil::from_days(t + days), declared: m.date.as_ref().map(|d| d.end) });
        }
    }
    b
}

/// The open items that `m` waits on: its open leaf sub-items and the open items its `after:` names,
/// followed transitively through both (an open link line counts as one item).
fn open_left(p: &Project, m: &Node) -> usize {
    let mut seen: HashSet<u32> = HashSet::from([m.line]);
    let mut stack: Vec<&Node> = Vec::new();
    push_deps(p, m, &mut stack);
    let mut left = 0;
    while let Some(n) = stack.pop() {
        if matches!(n.checkbox, Checkbox::Done | Checkbox::Cancelled) || !seen.insert(n.line) {
            continue;
        }
        // An open link line is an open item of another file: it counts, its own items are not read.
        if n.is_link_node() {
            left += 1;
            continue;
        }
        if n.checkbox == Checkbox::Open && matches!(n.kind, NodeKind::Task | NodeKind::Milestone) && !open_below(n) {
            left += 1;
        }
        push_deps(p, n, &mut stack);
    }
    left
}

fn push_deps<'p>(p: &'p Project, n: &'p Node, stack: &mut Vec<&'p Node>) {
    stack.extend(n.children.iter().filter(|c| c.checkbox != Checkbox::Cancelled));
    for v in n.keys.get("after").into_iter().flatten().filter(|v| v.end.is_none()) {
        if let Some(t) = p.find_id(v.text.trim_start_matches('^')) {
            stack.push(t);
        }
    }
}

/// The board as `pmd board --json` prints it (one file), also the `pmd_board` export of pmd-wasm.
pub fn board_json(b: &Board) -> serde_json::Value {
    let card = |c: &Card| {
        let mut v = json!({ "line": c.line, "title": c.title, "owner": c.owner });
        if let Some(id) = &c.id {
            v["id"] = json!(id);
        }
        if !c.waits_on.is_empty() {
            v["waits_on"] = json!(c.waits_on);
        }
        if let Some(s) = c.since {
            v["since"] = json!(s.to_string());
        }
        if let Some(d) = c.done {
            v["done"] = json!(d.to_string());
        }
        v
    };
    let wip: BTreeMap<String, usize> = b.wip.iter().map(|(o, n)| (owner_label(o), *n)).collect();
    json!({
        "file": b.path,
        "columns": {
            "waiting": b.waiting.iter().map(card).collect::<Vec<_>>(),
            "ready": b.ready.iter().map(card).collect::<Vec<_>>(),
            "doing": b.doing.iter().map(card).collect::<Vec<_>>(),
            "done": b.done.iter().map(card).collect::<Vec<_>>(),
        },
        "older_done": b.older_done,
        "wip": wip,
        "throughput_per_week": b.throughput,
        "throughput_window_days": b.throughput.map(|_| b.throughput_days),
        "cycle_time": b.cycle_time.map(|c| json!({ "median_days": c.median_days, "n": c.n })),
        "flow_forecast": b.flow.iter().map(|f| json!({ "id": f.id, "title": f.title, "line": f.line, "items_left": f.items_left, "about": f.about.to_string(), "declared": f.declared.map(|d| d.to_string()) })).collect::<Vec<_>>(),
    })
}

/// The owner as the board shows it: `(no owner)` for items without one.
pub fn owner_label(o: &str) -> String {
    if o.is_empty() { "(no owner)".to_string() } else { o.to_string() }
}
