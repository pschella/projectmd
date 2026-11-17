//! Forecasting (FORMAT §7): pure, one file plus the declared dates of the files it links.

use crate::date::{parse_date, Civil, DateParse};
use crate::graph::{resolve, split_fragment};
use crate::model::*;
use std::collections::{BTreeMap, HashMap};

/// The declared dates a file offers to others: its Commitment and milestones.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Interface {
    /// (start, end) of the Commitment.
    pub commitment: Option<(Civil, Civil)>,
    /// The Commitment's start, only when it states one (a range); a bare date is a point (§7.5).
    pub commitment_start: Option<Civil>,
    /// The `Done:` date: the project is finished.
    pub done: Option<Civil>,
    /// Public milestone id -> declared (end) date.
    pub milestones: BTreeMap<String, Civil>,
    /// Public milestones that are done -> the day they were done (their date if no `done:`).
    pub done_milestones: BTreeMap<String, Civil>,
    /// Header `Owner:` text (trimmed); references between same-owner files may target any item.
    pub owner: Option<String>,
    /// Any item id -> forecast end, for same-owner references.
    pub item_forecasts: BTreeMap<String, Civil>,
    /// Any done item id -> the day it was done (forecast end if no `done:`).
    pub item_done: BTreeMap<String, Civil>,
}

/// Interfaces by normalized path or URL.
pub type Interfaces = BTreeMap<String, Interface>;

/// `2027-Q3`, `2027-01-04`, `2026-09-01..2027-11-30` or `a .. b`, as (start, end).
pub fn span_of_text(s: &str) -> Option<(Civil, Civil)> {
    let one = |w: &str| match parse_date(w.trim()) {
        DateParse::Ok(a, b) => Some((a, b)),
        _ => None,
    };
    match s.find("..") {
        Some(i) => {
            let (a, b) = (one(&s[..i])?, one(&s[i + 2..])?);
            Some((a.0, b.1))
        }
        None => one(s),
    }
}

impl Interface {
    pub fn from_texts<'a>(
        commitment: Option<&str>,
        milestones: impl IntoIterator<Item = (&'a str, &'a str)>,
    ) -> Interface {
        let mut i = Interface {
            commitment: commitment.and_then(span_of_text),
            commitment_start: commitment.filter(|s| s.contains("..")).and_then(span_of_text).map(|c| c.0),
            ..Default::default()
        };
        for (id, d) in milestones {
            if let Some((_, e)) = span_of_text(d) {
                i.milestones.insert(id.to_string(), e);
            }
        }
        i
    }
    pub fn from_project(p: &Project) -> Interface {
        let mut i = Interface { commitment: p.commitment.as_ref().map(|c| (c.start, c.end)), commitment_start: p.commitment.as_ref().filter(|c| c.range).map(|c| c.start), done: p.done.as_ref().map(|d| d.end), owner: owner_text(p), ..Default::default() };
        for it in crate::iface::interface_items(p) {
            if it.checkbox != '-' {
                if let Some(id) = it.id {
                    if it.checkbox == 'x' {
                        let d = it.done.as_deref().and_then(|t| match parse_date(t) { DateParse::Ok(d, _) => Some(d), _ => None }).unwrap_or(it.end);
                        i.done_milestones.entry(id.clone()).or_insert(d);
                    }
                    i.milestones.entry(id).or_insert(it.end);
                }
            }
        }
        i
    }
}

fn owner_text(p: &Project) -> Option<String> {
    p.owner.as_ref().map(|v| v.text.trim().to_string())
}

#[derive(Clone, Copy)]
enum Dep {
    Local(usize),
    Fixed(Civil),
}

struct Info<'a> {
    node: &'a Node,
    parent: Option<usize>,
    children: Vec<usize>,
    deps: Vec<Dep>,
    own_date: Option<Civil>,
    own_by: Option<Civil>,
    decl_start: Option<Civil>,
    decl_end: Option<Civil>,
    iface: Option<Interface>,
    /// Deliverable: the milestone's date in the target's interface.
    deliv: Option<Civil>,
    /// The linked milestone is done: when.
    deliv_done: Option<Civil>,
    /// The linked milestone's own forecast end.
    deliv_fc: Option<Civil>,
}

#[derive(Clone, Copy, Default)]
struct Res {
    start: Option<Civil>,
    end: Option<Civil>,
}

struct Ctx<'a> {
    infos: Vec<Info<'a>>,
    res: Vec<Option<Res>>,
    state: Vec<u8>,
    /// What fixed each item's end and (for estimates) its start (D60).
    drv: Vec<Option<(Driver, Driver)>>,
    /// Cancelled, or under a cancelled item.
    canc: Vec<bool>,
    today: Civil,
    /// The project has `Done:`: nothing is clamped to today.
    finished: bool,
    diags: Vec<Diagnostic>,
}

fn cx_cancelled(i: &Info) -> bool {
    i.node.checkbox == Checkbox::Cancelled
}

fn unfinished(n: &Node) -> bool {
    matches!(n.checkbox, Checkbox::None | Checkbox::Open)
}

fn mx(a: Option<Civil>, b: Option<Civil>) -> Option<Civil> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.max(y)),
        (x, None) => x,
        (None, y) => y,
    }
}
fn mn(a: Option<Civil>, b: Option<Civil>) -> Option<Civil> {
    match (a, b) {
        (Some(x), Some(y)) => Some(x.min(y)),
        (x, None) => x,
        (None, y) => y,
    }
}

impl<'a> Ctx<'a> {
    /// One node, in dependency order (FORMAT §7.3, steps 1 to 5).
    fn eval(&mut self, i: usize) {
        if self.state[i] != 0 {
            return;
        }
        self.state[i] = 1;
        let node = self.infos[i].node;
        // children first (they never depend on us: such `after:` cycles were cut beforehand)
        let kids = self.infos[i].children.clone();
        for k in kids.iter().copied() {
            self.eval_dep(i, k);
        }
        let deps = self.infos[i].deps.clone();
        // A point (milestone, deliverable, leaf task with no estimate) may fall on the day its
        // predecessor ends; work with an estimate starts the day after.
        // a range item is a window, never a point: it starts the day after its inputs end
        let point = node.duration.is_none()
            && node.range.is_none()
            && (matches!(node.kind, NodeKind::Milestone | NodeKind::Deliverable) || (node.kind == NodeKind::Task && self.infos[i].children.is_empty()));
        let today = self.today;
        let mut after: Option<Civil> = None;
        let mut pred_end: Option<Civil> = None;
        let mut after_drv: Option<Driver> = None;
        let mut pred_line: Option<u32> = None; // the predecessor that ends last
        let mut started = false; // an `after: DATE` on or before today records an actual start
        let step = |c: Civil| if point { c } else { Civil::from_days(c.to_days() + 1) };
        for d in deps {
            match d {
                Dep::Fixed(c) => {
                    started |= c <= today;
                    if after.is_none_or(|a| c > a) {
                        after_drv = Some(Driver::new("after-date"));
                    }
                    after = mx(after, Some(c));
                    pred_end = mx(pred_end, Some(c));
                }
                Dep::Local(j) => {
                    if self.eval_dep(i, j) {
                        // finished with unknown date: no constraint on successors
                        let pn = self.infos[j].node;
                        if pn.checkbox == Checkbox::Done && pn.done_date().is_none() {
                            continue;
                        }
                        if let Some(e) = self.res[j].and_then(|r| r.end) {
                            if after.is_none_or(|a| step(e) > a) {
                                after_drv = Some(Driver::at("after", self.infos[j].node.line));
                            }
                            after = mx(after, Some(step(e)));
                            if pred_end.is_none_or(|p| e > p) {
                                pred_line = Some(self.infos[j].node.line);
                            }
                            pred_end = mx(pred_end, Some(e));
                        }
                    }
                }
            }
        }
        self.state[i] = 2;
        if self.canc[i] {
            self.res[i] = None;
            return;
        }
        let info = &self.infos[i];
        let heading = node.kind == NodeKind::Heading;
        let child_done = node.kind == NodeKind::Subproject && info.iface.as_ref().is_some_and(|f| f.done.is_some());
        let unfinished_ = unfinished(node) && !self.finished && !child_done && info.deliv_done.is_none();
        // 1. start
        // A leading date is a declared end, not a start, once an `after:` dates the node (D68)
        let decl_start = if after.is_some() && node.range.is_none() { None } else { info.decl_start };
        let start0 = mx(decl_start, after);
        let mut start = start0;
        let mut sd = match (decl_start, after) {
            (Some(d), a) if a.is_none_or(|a| d >= a) => Driver::new("declared"),
            _ => after_drv.unwrap_or(Driver::new("none")),
        };
        // 2. not started yet
        if unfinished_ && node.duration.is_some() && !started {
            if start.is_none_or(|s| s < today) {
                sd = Driver::new("today");
            }
            start = mx(start, Some(today));
        }
        let mut kid_start = None;
        let mut kid_end = None;
        let mut kid_line = 0;
        for &k in &info.children {
            if let Some(r) = self.res[k] {
                kid_start = mn(kid_start, r.start);
                if r.end.is_some_and(|e| kid_end.is_none_or(|b| e >= b)) {
                    kid_line = self.infos[k].node.line;
                }
                kid_end = mx(kid_end, r.end);
            }
        }
        // 3. end
        let own_end = info.decl_end.or(info.own_by);
        let mut ed: Driver;
        let mut end: Option<Civil>;
        let mut promised = false; // a link line with a promise: `after:` moves its start, never its end
        if let (false, Some(d)) = (heading, info.deliv) {
            promised = true;
            end = Some(info.deliv_done.unwrap_or(d));
            ed = Driver::new("promise");
            if let (None, true, Some(fe)) = (info.deliv_done, node.kind == NodeKind::Milestone, info.deliv_fc) {
                // dated deliverable: the declared date is the promise, the linked milestone the forecast
                end = Some(fe);
            }
            start = if let Some(dd) = info.deliv_done {
                Some(start0.map_or(dd, |s| s.min(dd)))
            } else if node.kind == NodeKind::Milestone {
                Some(end.map_or(d, |e| e.min(d)))
            } else {
                start0.or(Some(d))
            };
        } else if let Some(iface) = info.iface.as_ref().filter(|f| (f.commitment.is_some() || f.done.is_some()) && node.kind == NodeKind::Subproject) {
            // sub-project: the child's Done, else its Commitment end. The start is the Commitment's start when
            // it states one (moved later by `after:`); else the `after:` start; else the end (§7.5).
            promised = true;
            let (_, ce) = iface.commitment.or(iface.done.map(|d| (d, d))).unwrap();
            let e = iface.done.unwrap_or(ce);
            end = Some(e);
            ed = Driver::new("promise");
            start = Some(match (iface.commitment_start, start0) {
                (Some(cs), s) => s.map_or(cs, |s| s.max(cs)),
                (None, s) => s.unwrap_or(e),
            });
        } else if kid_end.is_some() {
            // live sub-items date the node; a declared end (a range's too) only gives slack and conflict (D68)
            end = kid_end;
            ed = Driver::at("child", kid_line);
            start = mn(start, kid_start);
        } else if let Some(dur) = &node.duration {
            ed = Driver::new("estimate");
            end = start.map(|s| dur.end_from(s));
            if end.is_none() && node.kind == NodeKind::Milestone {
                // a done milestone with an estimate: its date stands
                end = info.decl_end;
                start = end;
            }
        } else if node.range.is_some() {
            // a range without live sub-items is a window: start = max(A, after:), end = B (D68)
            end = info.decl_end;
            ed = Driver::new("declared");
        } else if own_end.is_some() && after.is_none() {
            // a declared end is a promise, not a prediction: it dates the node only when nothing else does (D68)
            end = own_end;
            ed = Driver { kind: if info.decl_end.is_none() { "by" } else { "declared" }, line: pred_line };
        } else {
            // a point has a start; with neither, an open item is due today (5. nothing to date it)
            end = start;
            ed = Driver::new("start");
            if start.is_none() && unfinished_ && !heading {
                start = Some(today);
                end = start;
                ed = Driver::new("undated");
            }
        }
        if node.checkbox == Checkbox::Done {
            // actuals drive successors: a done item ends when it was done
            if let Some(d) = node.done_date() {
                ed = Driver::new("done");
                end = Some(d);
                if start.is_some_and(|s| s > d) {
                    start = Some(d);
                }
            }
        }
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                if promised {
                    start = Some(e);
                } else {
                    end = Some(s);
                    ed = Driver::new("start");
                }
            }
        }
        if end.is_none() {
            end = start;
            ed = Driver::new("start");
        }
        // 4. unfinished work never ends before today
        if unfinished_ && !heading && end.is_some_and(|e| e < today) {
            end = Some(today);
            ed = Driver::new("today");
        }
        if ed.kind == "start" {
            // a point that ends where it starts: the start's cause is its cause
            ed = sd.clone();
        }
        self.drv[i] = Some((ed, sd));
        self.res[i] = Some(Res { start, end });
    }

    /// Evaluate `j` as a dependency of `i`; false when that would close a cycle (edge dropped).
    fn eval_dep(&mut self, _i: usize, j: usize) -> bool {
        if self.state[j] == 1 {
            return false;
        }
        self.eval(j);
        true
    }
}

fn collect<'a>(n: &'a Node, parent: Option<usize>, infos: &mut Vec<Info<'a>>) -> usize {
    let idx = infos.len();
    let span = |d: &Option<DateExpr>| d.as_ref().map(|d| (d.start, d.end));
    let (ds, de) = match (span(&n.range), span(&n.date)) {
        (Some(r), _) => (Some(r.0), Some(r.1)),
        (None, Some(d)) => (Some(d.0), Some(d.1)),
        _ => (None, None),
    };
    // Headings are grouping only and carry no dates (D47).
    let heading = n.kind == NodeKind::Heading;
    // A dated milestone with an estimate is work with a deadline: the date is its end, not its start.
    let (ds, de) = if n.kind == NodeKind::Milestone && n.duration.is_some() { (None, de) } else { (ds, de) };
    infos.push(Info {
        node: n,
        parent,
        children: Vec::new(),
        deps: Vec::new(),
        own_date: if heading { None } else { n.date.as_ref().or(n.range.as_ref()).map(|d| d.end) },
        own_by: None,
        decl_start: ds,
        decl_end: de,
        iface: None,
        deliv: None,
        deliv_done: None,
        deliv_fc: None,
    });
    for c in &n.children {
        let ci = collect(c, Some(idx), infos);
        infos[idx].children.push(ci);
    }
    idx
}

fn apply(n: &mut Node, idx: &mut usize, res: &[Option<Res>], slack: &[Option<i64>], flags: &[(Option<Civil>, bool)], links: &[Option<String>], drv: &[Option<(Driver, Driver)>]) {
    let i = *idx;
    *idx += 1;
    let r = res[i].unwrap_or_default();
    n.forecast = Some(Forecast { start: r.start, end: r.end, declared_end: flags[i].0, conflict: flags[i].1, slack_days: slack[i] });
    n.link_resolved = links[i].clone();
    n.driver = drv[i].as_ref().map(|d| d.0.clone());
    n.start_driver = drv[i].as_ref().filter(|d| d.0.kind == "estimate").map(|d| d.1.clone());
    for c in &mut n.children {
        apply(c, idx, res, slack, flags, links, drv);
    }
}

/// Compute forecasts (FORMAT §7.3) and conflicts (§7.7) for one file.
///
/// `interfaces` holds the declared dates of other files (§7.8). Sets `forecast` on every node and on
/// the project, `link_resolved` on linked nodes, and appends `E-CONFLICT` and `E-CYCLE`
/// diagnostics.
pub fn schedule(project: &mut Project, today: Civil, interfaces: &Interfaces) {
    let path = project.path.clone();
    let proj_end = project.commitment.as_ref().map(|c| c.end);
    let mine = owner_text(project);
    let (diags, res, slack, flags, links, pf, drv) = {
        let mut infos: Vec<Info> = Vec::new();
        let tops: Vec<usize> = project.nodes.iter().map(|n| collect(n, None, &mut infos)).collect();
        let mut ids: HashMap<&str, usize> = HashMap::new();
        for (i, inf) in infos.iter().enumerate() {
            if let Some(id) = &inf.node.id {
                ids.entry(id.as_str()).or_insert(i);
            }
        }
        let mut links: Vec<Option<String>> = vec![None; infos.len()];
        for i in 0..infos.len() {
            let n = infos[i].node;
            if let Some(l) = &n.link {
                let r = resolve(&path, &l.target);
                infos[i].iface = interfaces.get(&r).cloned();
                if n.deliverable_link().is_some() {
                    let frag = split_fragment(&l.target).1.unwrap_or("");
                    infos[i].deliv = infos[i].iface.as_ref().and_then(|f| f.milestones.get(frag).copied());
                    infos[i].deliv_done = infos[i].iface.as_ref().and_then(|f| f.done_milestones.get(frag).copied());
                    // another owner's forecast never crosses the boundary (D69): only declared dates do
                    infos[i].deliv_fc = None;
                    // same owner: any item with that id; its promise is its forecast end (§7.5 rule 2)
                    let same = infos[i].iface.as_ref().filter(|f| matches!((&f.owner, &mine), (Some(a), Some(b)) if same_owner(a, b))).and_then(|f| f.item_forecasts.get(frag).map(|&fe| (fe, f.item_done.get(frag).copied())));
                    if let Some((fe, dd)) = same {
                        infos[i].deliv = Some(fe);
                        infos[i].deliv_done = infos[i].deliv_done.or(dd);
                        infos[i].deliv_fc = Some(fe);
                    }
                }
                links[i] = Some(r);
            }
            let local = |t: &str| -> Option<usize> { ids.get(t.trim_start_matches('^')).copied() };
            let mut deps = Vec::new();
            if let Some(vs) = n.keys.get("after") {
                for v in vs {
                    // a coarse date is the first day of its span as a start
                    if let Some(e) = v.start {
                        deps.push(Dep::Fixed(e));
                        continue;
                    }
                    if let Some(j) = local(&v.text) {
                        deps.push(Dep::Local(j));
                    }
                }
            }
            infos[i].deps = deps;
            let mut by = None;
            if let Some(vs) = n.keys.get("by") {
                for v in vs {
                    let d = match v.end {
                        Some(e) => Some(e),
                        None => local(&v.text).and_then(|j| infos[j].own_date.or(infos[j].decl_end)),
                    };
                    by = mn(by, d);
                }
            }
            infos[i].own_by = by;
        }
        let n = infos.len();
        let mut canc = vec![false; n];
        for (i, c) in canc.iter_mut().enumerate() {
            let mut a = Some(i);
            while let Some(j) = a {
                if cx_cancelled(&infos[j]) {
                    *c = true;
                    break;
                }
                a = infos[j].parent;
            }
        }
        let mut cdiags = cut_cycles(&mut infos, &canc);
        let mut cx = Ctx { infos, res: vec![None; n], state: vec![0; n], drv: vec![None; n], canc, today, finished: project.done.is_some(), diags: Vec::new() };
        cx.diags.append(&mut cdiags);
        for i in 0..n {
            cx.eval(i);
        }
        // conflicts, children before parents
        // Declared end: the range end, leading date or `by:`. A conflict replaces it with the bound broken.
        let mut flags: Vec<(Option<Civil>, bool)> = cx.infos.iter().map(|inf| (if inf.node.kind == NodeKind::Heading { None } else { mn(inf.decl_end, inf.own_by) }, false)).collect();
        // End used against inherited bounds (container `by:`, Commitment). An unmoved coarse
        // range estimate (`2027-Q3`) is an estimate, not a forecast: it is not checked there.
        let mut eff: Vec<Option<Civil>> = vec![None; n];
        for i in (0..n).rev() {
            let inf = &cx.infos[i];
            let end = cx.res[i].and_then(|r| r.end);
            eff[i] = if cx.canc[i] {
                None
            } else {
                let kids = inf.children.iter().fold(None, |a, &k| mx(a, eff[k]));
                if inf.iface.is_none() && kids.is_some() { kids } else { end }
            };
        }
        for i in (0..n).rev() {
            let inf = &cx.infos[i];
            let node = inf.node;
            if node.kind == NodeKind::Heading {
                continue;
            }
            let Some(end) = cx.res[i].and_then(|r| r.end) else { continue };
            if cx.canc[i] || cx.finished || node.checkbox == Checkbox::Done {
                continue;
            }
            // (bound, is the sub-project's own `by:`)
            let mut bounds: Vec<(Civil, &'static str)> = Vec::new();
            if let Some(d) = inf.own_date {
                bounds.push((d, "own"));
            }
            // A link line is a contract, not a bound: `by:` against the promise or today is a
            // negotiation (W-DISPUTED, W-OVERDUE from the contracts), never a conflict.
            let link_line = matches!(inf.node.kind, NodeKind::Subproject | NodeKind::Deliverable);
            if let (Some(b), false) = (inf.own_by, link_line) {
                bounds.push((b, "by"));
            }
            let mut a = inf.parent;
            while let Some(p) = a {
                if let Some(b) = cx.infos[p].own_by {
                    bounds.push((b, "container"));
                }
                a = cx.infos[p].parent;
            }
            if let Some(c) = proj_end {
                bounds.push((c, "commitment"));
            }
            let inh = eff[i];
            let worst = bounds
                .iter()
                .filter(|(b, why)| if matches!(*why, "container" | "commitment") { inh.is_some_and(|e| end > *b && e > *b) } else { end > *b })
                .min_by_key(|(b, _)| *b)
                .copied();
            if let Some((b, why)) = worst {
                flags[i] = (Some(b), true);
                // The Commitment is reported once for the file (below); other bounds where the
                // conflict originates, not on every container it passes through.
                if why == "commitment" {
                    continue;
                }
                let kid_conflict = inf.children.iter().any(|&k| flags[k].1);
                if kid_conflict && node.duration.is_none() {
                    continue;
                }
                let bound_name = match why {
                    "own" => format!("its date {}", b),
                    "by" => format!("`by: {}`", b),
                    "container" => format!("its container's `by: {}`", b),
                    _ => format!("the Commitment {}", b),
                };
                let msg = if b < today && matches!(why, "own" | "by") {
                    format!("`{}` is overdue: not marked done by {}", node.title, b)
                } else {
                    let started_today = cx.res[i].and_then(|r| r.start) == Some(today) && node.duration.is_some();
                    let why_today = if started_today { " (not started yet: earliest start is today)" } else { "" };
                    format!("`{}` is forecast to end {}, after {}{}", node.title, end, bound_name, why_today)
                };
                let d = Diagnostic::new(Severity::E, "E-CONFLICT", node.line, msg);
                cx.diags.push(d);
            }
        }
        // Slack (§7.6): declared end minus forecast end, for every node that has both
        let slack: Vec<Option<i64>> = (0..n).map(|i| match (flags[i].0, cx.res[i].and_then(|r| r.end)) { (Some(d), Some(e)) => Some(d.to_days() - e.to_days()), _ => None }).collect();
        let mut ps = None;
        let mut pe = None;
        for &t in &tops {
            if let Some(r) = cx.res[t] {
                ps = mn(ps, r.start);
                pe = mx(pe, r.end);
            }
        }
        let finished = project.done.as_ref().map(|d| d.end);
        if finished.is_some() {
            pe = finished;
        }
        // Finished work is history: a late finish shows as "done Nd late", never as a conflict
        // with the Commitment. Only unfinished work can still break the promise.
        let peff = tops.iter().filter(|&&t| cx.infos[t].node.checkbox != Checkbox::Done).fold(None, |a, &t| mx(a, eff[t]));
        let pconf = finished.is_none() && matches!((peff, proj_end), (Some(e), Some(c)) if e > c);
        if pconf {
            let mut d = Diagnostic::new(
                Severity::E,
                "E-CONFLICT",
                1,
                format!("project is forecast to end {}, after its Commitment {}", pe.unwrap(), proj_end.unwrap()),
            );
            d.path = path.clone();
            cx.diags.push(d);
        }
        let pf = Forecast { start: ps, end: pe, declared_end: proj_end, conflict: pconf, slack_days: match (proj_end, pe) { (Some(d), Some(e)) => Some(d.to_days() - e.to_days()), _ => None } };
        (cx.diags, cx.res, slack, flags, links, pf, cx.drv)
    };
    let mut idx = 0;
    for n in &mut project.nodes {
        apply(n, &mut idx, &res, &slack, &flags, &links, &drv);
    }
    project.forecast = Some(pf);
    let mut diags = diags;
    diags.sort_by_key(|d| d.line);
    project.diagnostics.extend(diags);
}

/// Give every interface the forecast of its milestones: each file scheduled once against the
/// declared dates of the others, so a dated deliverable sees its target's forecast one hop deep.
pub fn add_forecasts(ifs: &mut Interfaces, projects: &[(String, &Project)], today: Civil) {
    let declared = ifs.clone();
    for (key, p) in projects {
        let mut c = (*p).clone();
        schedule(&mut c, today, &declared);
        let mut all = BTreeMap::new();
        let mut dn = BTreeMap::new();
        fn walk(ns: &[Node], all: &mut BTreeMap<String, Civil>, dn: &mut BTreeMap<String, Civil>) {
            for n in ns {
                if let (true, Some(id), Some(f)) = (n.checkbox != Checkbox::Cancelled, &n.id, &n.forecast) {
                    if let Some(e) = f.end {
                        all.entry(id.clone()).or_insert(e);
                        if n.checkbox == Checkbox::Done {
                            dn.entry(id.clone()).or_insert(n.done_date().unwrap_or(e));
                        }
                    }
                }
                walk(&n.children, all, dn);
            }
        }
        walk(&c.nodes, &mut all, &mut dn);
        if let Some(i) = ifs.get_mut(key) {
            i.item_forecasts = all;
            i.item_done = dn;
        }
    }
}

/// FORMAT §7.3 rule 0: find `after:` cycles (an item may also wait for its own ancestor or
/// descendant, since a parent's end follows its children). Every item in a cycle gets `E-CYCLE`
/// and every `after:` edge inside the cycle is dropped, so the result is order independent.
fn cut_cycles(infos: &mut [Info], canc: &[bool]) -> Vec<Diagnostic> {
    let n = infos.len();
    let is_desc = |mut j: usize, i: usize| {
        while let Some(p) = infos[j].parent {
            if p == i {
                return true;
            }
            j = p;
        }
        false
    };
    let mut adj: Vec<Vec<usize>> = (0..n)
        .map(|i| {
            if canc[i] {
                return Vec::new();
            }
            let after = infos[i].deps.iter().filter_map(|d| if let Dep::Local(j) = d { Some(*j) } else { None });
            after.chain(infos[i].children.iter().copied()).filter(|&j| !canc[j]).collect()
        })
        .collect();
    // waiting for one's own descendant is a cycle: the parent's span contains it
    for i in 0..n {
        for d in &infos[i].deps {
            if let Dep::Local(j) = d {
                if !canc[i] && !canc[*j] && is_desc(*j, i) {
                    adj[*j].push(i);
                }
            }
        }
    }
    // Tarjan's strongly connected components
    struct T<'a> {
        adj: &'a [Vec<usize>],
        idx: Vec<usize>,
        low: Vec<usize>,
        on: Vec<bool>,
        st: Vec<usize>,
        n: usize,
        sccs: Vec<Vec<usize>>,
    }
    fn go(t: &mut T, v: usize) {
        t.n += 1;
        t.idx[v] = t.n;
        t.low[v] = t.n;
        t.st.push(v);
        t.on[v] = true;
        for k in 0..t.adj[v].len() {
            let w = t.adj[v][k];
            if t.idx[w] == 0 {
                go(t, w);
                t.low[v] = t.low[v].min(t.low[w]);
            } else if t.on[w] {
                t.low[v] = t.low[v].min(t.idx[w]);
            }
        }
        if t.low[v] == t.idx[v] {
            let mut c = Vec::new();
            while let Some(w) = t.st.pop() {
                t.on[w] = false;
                c.push(w);
                if w == v {
                    break;
                }
            }
            t.sccs.push(c);
        }
    }
    let mut t = T { adj: &adj, idx: vec![0; n], low: vec![0; n], on: vec![false; n], st: Vec::new(), n: 0, sccs: Vec::new() };
    for v in 0..n {
        if t.idx[v] == 0 {
            go(&mut t, v);
        }
    }
    let mut diags = Vec::new();
    for mut c in t.sccs {
        let has_after = |i: usize, c: &[usize]| infos[i].deps.iter().any(|d| matches!(d, Dep::Local(j) if c.contains(j)));
        if !c.iter().any(|&i| has_after(i, &c)) {
            continue;
        }
        c.sort();
        let names: Vec<String> = c.iter().map(|&i| format!("`{}`", infos[i].node.title)).collect();
        for &i in &c {
            diags.push(Diagnostic::new(Severity::E, "E-CYCLE", infos[i].node.line, format!("{} wait for each other through `after:`; these `after:` are ignored", names.join(", "))));
        }
        for &i in &c {
            infos[i].deps.retain(|d| !matches!(d, Dep::Local(j) if c.contains(j)));
        }
    }
    diags
}
