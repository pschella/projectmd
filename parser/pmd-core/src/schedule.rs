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
    /// The `Done:` date: the project is finished.
    pub done: Option<Civil>,
    /// Public milestone id -> declared (end) date.
    pub milestones: BTreeMap<String, Civil>,
    /// Public milestones that are done -> the day they were done (their date if no `done:`).
    pub done_milestones: BTreeMap<String, Civil>,
    /// Public milestone id -> (forecast end, earliest end) in its own file, when scheduled.
    pub forecasts: BTreeMap<String, (Civil, Option<Civil>)>,
    /// Header `Owner:` text (trimmed); references between same-owner files may target any item.
    pub owner: Option<String>,
    /// Any item id -> (forecast end, earliest end), for same-owner references.
    pub item_forecasts: BTreeMap<String, (Civil, Option<Civil>)>,
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
        let mut i = Interface { commitment: p.commitment.as_ref().map(|c| (c.start, c.end)), done: p.done.as_ref().map(|d| d.end), owner: owner_text(p), ..Default::default() };
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
    /// The linked milestone's own (forecast end, earliest end).
    deliv_fc: Option<(Civil, Option<Civil>)>,
}

#[derive(Clone, Copy, Default)]
struct Res {
    start: Option<Civil>,
    end: Option<Civil>,
}

struct Ctx<'a> {
    infos: Vec<Info<'a>>,
    res: Vec<Option<Res>>,
    /// Earliest possible end from predecessors, estimate and children (not clamped to a declared date).
    early: Vec<Option<Civil>>,
    state: Vec<u8>,
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
    fn eval(&mut self, i: usize) {
        if self.state[i] != 0 {
            return;
        }
        self.state[i] = 1;
        let node = self.infos[i].node;
        // children first (they never depend on us unless that is a cycle, which is broken below)
        let kids = self.infos[i].children.clone();
        for k in kids.iter().copied() {
            self.eval_dep(i, k);
        }
        let deps = self.infos[i].deps.clone();
        // A task starts the day after an item it follows ends; a date in `after:` is the earliest start (inclusive). A milestone or deliverable is a point
        // in time and may fall on the day its predecessor ends.
        // A point (milestone, deliverable, or a task with no estimate and no children) may fall on
        // the day its predecessor ends; work with an estimate starts the day after.
        let point = node.duration.is_none()
            && (matches!(node.kind, NodeKind::Milestone | NodeKind::Deliverable)
                || (node.kind == NodeKind::Task && self.infos[i].children.is_empty()));
        let mut after: Option<Civil> = None;
        let mut pred_end: Option<Civil> = None;
        let step = |c: Civil| if point { c } else { Civil::from_days(c.to_days() + 1) };
        for d in deps {
            match d {
                Dep::Fixed(c) => {
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
                            after = mx(after, Some(step(e)));
                            pred_end = mx(pred_end, Some(e));
                        }
                    }
                }
            }
        }
        self.state[i] = 2;
        if self.canc[i] {
            self.res[i] = None;
            self.early[i] = None;
            return;
        }
        let today = self.today;
        let info = &self.infos[i];
        let mut start = mx(info.decl_start, after);
        let mut kid_start = None;
        let mut kid_end = None;
        for &k in &info.children {
            if let Some(r) = self.res[k] {
                kid_start = mn(kid_start, r.start);
                kid_end = mx(kid_end, r.end);
            }
        }
        let mut end: Option<Civil>;
        let mut early: Option<Civil>;
        if node.kind == NodeKind::Heading {
            end = kid_end;
            early = None;
            start = mn(start, kid_start);
        } else if let Some(d) = info.deliv {
            // deliverable: the milestone's date
            end = Some(info.deliv_done.unwrap_or(d));
            early = end;
            if let (None, true, Some((fe, ee))) = (info.deliv_done, node.kind == NodeKind::Milestone, info.deliv_fc) {
                // dated deliverable: forecast from the linked milestone's own forecast
                end = Some(fe);
                early = Some(ee.unwrap_or(fe));
            }
            if let Some(dd) = info.deliv_done {
                start = Some(start.map_or(dd, |s| s.min(dd)));
            } else if node.kind == NodeKind::Milestone {
                // dated deliverable: the declared date is the promise, the linked milestone the forecast
                start = Some(end.map_or(d, |e| e.min(d)));
            } else if start.is_none() {
                start = Some(d);
            }
        } else if let Some(iface) = info.iface.as_ref().filter(|f| (f.commitment.is_some() || f.done.is_some()) && node.kind == NodeKind::Subproject) {
            // sub-project: the child's declared Commitment; done: it ended at `Done`
            let (cs, ce) = iface.commitment.or(iface.done.map(|d| (d, d))).unwrap();
            end = Some(iface.done.unwrap_or(ce));
            early = end;
            if start.is_none() {
                start = Some(if cs != ce { cs } else { ce });
            }
        } else if let Some(dur) = &node.duration {
            if unfinished(node) && !self.finished && info.decl_start.is_none() && (start.is_none() || info.deps.is_empty()) {
                start = mx(start, Some(today));
            }
            end = start.map(|s| dur.end_from(s));
            if end.is_none() && node.kind == NodeKind::Milestone {
                // a done milestone with an estimate: its date stands
                end = info.decl_end;
                start = end;
            }
            early = end;
        } else if kid_end.is_some() {
            early = kid_end;
            end = mx(kid_end, info.decl_end);
            start = mn(start, kid_start);
        } else if node.is_link_node() {
            // a link to a child with no promise yet: the line's own estimate (promotion rule, D47)
            end = info.decl_end.or(info.own_by);
            early = None;
        } else {
            end = info.decl_end;
            early = pred_end;
        }
        if node.checkbox == Checkbox::Done {
            // actuals drive successors: a done item ends when it was done
            if let Some(d) = node.done_date() {
                end = Some(d);
                early = end;
                if start.map_or(false, |s| s > d) {
                    start = Some(d);
                }
            }
        }
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                end = Some(s);
            }
        }
        if end.is_none() {
            end = start;
        }
        let child_done = node.kind == NodeKind::Subproject && info.iface.as_ref().is_some_and(|f| f.done.is_some());
        let unfinished_ = unfinished(node) && !self.finished && !child_done && info.deliv_done.is_none();
        if unfinished_ && end.is_none() && matches!(node.kind, NodeKind::Task | NodeKind::Milestone) && !node.is_link_node() {
            // an open item with nothing to date it (e.g. a decision someone owes) is due now:
            // it ends today at the earliest, so its successors wait for it
            start = Some(today);
            end = start;
        }
        if unfinished_ {
            end = end.map(|e| e.max(today));
        }
        self.early[i] = if unfinished_ { early.map(|e| e.max(today)) } else { early };
        self.res[i] = Some(Res { start, end });
    }

    /// Evaluate `j` as a dependency of `i`; false when that would close a cycle (edge dropped).
    fn eval_dep(&mut self, i: usize, j: usize) -> bool {
        if self.state[j] == 1 {
            let (a, b) = (self.infos[i].node, self.infos[j].node);
            self.diags.push(Diagnostic::new(
                Severity::E,
                "E-CYCLE",
                a.line,
                format!("`{}` and `{}` depend on each other; ignoring the dependency of `{}` on `{}`", a.title, b.title, a.title, b.title),
            ));
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
        own_date: if heading { None } else { n.date.as_ref().map(|d| d.end) },
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

fn apply<'a>(n: &mut Node, idx: &mut usize, res: &[Option<Res>], early: &[Option<Civil>], flags: &[(Option<Civil>, bool)], links: &[Option<String>]) {
    let i = *idx;
    *idx += 1;
    let r = res[i].unwrap_or_default();
    n.forecast = Some(Forecast { start: r.start, end: r.end, declared_end: flags[i].0, conflict: flags[i].1, slack_days: match (flags[i].0, early[i]) { (Some(d), Some(e)) => Some(d.to_days() - e.to_days()), _ => None } });
    n.link_resolved = links[i].clone();
    for c in &mut n.children {
        apply(c, idx, res, early, flags, links);
    }
}

/// Compute forecasts (FORMAT §7.1) and conflicts (7.2) for one file.
///
/// `interfaces` holds the declared dates of other files (7.3). Sets `forecast` on every node and on
/// the project, `link_resolved` on linked nodes, and appends `E-CONFLICT` and `E-CYCLE`
/// diagnostics.
pub fn schedule(project: &mut Project, today: Civil, interfaces: &Interfaces) {
    let path = project.path.clone();
    let proj_end = project.commitment.as_ref().map(|c| c.end);
    let mine = owner_text(project);
    let (diags, res, early, flags, links, pf) = {
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
                    infos[i].deliv_fc = infos[i].iface.as_ref().and_then(|f| f.forecasts.get(frag).copied());
                    if let Some(f) = infos[i].iface.as_ref().filter(|f| mine.is_some() && f.owner == mine) {
                        // same owner: any item with that id; its end is its forecast end
                        if let Some(&(fe, ee)) = f.item_forecasts.get(frag) {
                            let (d, dd) = (infos[i].deliv.unwrap_or(fe), f.item_done.get(frag).copied());
                            infos[i].deliv = Some(d);
                            infos[i].deliv_done = infos[i].deliv_done.or(dd);
                            infos[i].deliv_fc = Some((fe, ee));
                        }
                    }
                }
                links[i] = Some(r);
            }
            let local = |t: &str| -> Option<usize> { ids.get(t.trim_start_matches('^')).copied() };
            let mut deps = Vec::new();
            if let Some(vs) = n.keys.get("after") {
                for v in vs {
                    if let Some(e) = v.end {
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
        for i in 0..n {
            let mut a = Some(i);
            while let Some(j) = a {
                if cx_cancelled(&infos[j]) {
                    canc[i] = true;
                    break;
                }
                a = infos[j].parent;
            }
        }
        let mut cx = Ctx { infos, res: vec![None; n], early: vec![None; n], state: vec![0; n], canc, today, finished: project.done.is_some(), diags: Vec::new() };
        for i in 0..n {
            cx.eval(i);
        }
        // conflicts, children before parents
        let mut flags: Vec<(Option<Civil>, bool)> = vec![(None, false); n];
        // End used against inherited bounds (container `by:`, Commitment). An unmoved coarse
        // range estimate (`2027-Q3`) is an estimate, not a forecast: it is not checked there.
        let mut eff: Vec<Option<Civil>> = vec![None; n];
        for i in (0..n).rev() {
            let inf = &cx.infos[i];
            let n_ = inf.node;
            let end = cx.res[i].and_then(|r| r.end);
            eff[i] = if cx.canc[i] {
                None
            } else if n_.duration.is_none() && !inf.children.is_empty() && inf.iface.is_none() {
                inf.children.iter().fold(None, |a, &k| mx(a, eff[k]))
            } else {
                end
            };
        }
        for i in (0..n).rev() {
            let inf = &cx.infos[i];
            let node = inf.node;
            let end = match cx.res[i].and_then(|r| r.end) {
                Some(e) => e,
                None => {
                    flags[i].0 = inf.own_date.or(inf.own_by);
                    continue;
                }
            };
            let own = mn(inf.own_date, inf.own_by);
            flags[i].0 = own;
            if cx.canc[i] || cx.finished || node.checkbox == Checkbox::Done {
                continue;
            }
            // (bound, is the sub-project's own `by:`)
            let mut bounds: Vec<(Civil, &'static str)> = Vec::new();
            if let Some(d) = inf.own_date {
                bounds.push((d, "own"));
            }
            // A link whose interface is known has a contract, not a bound: `by:` against the
            // promise is a negotiation (W-DISPUTED from the contracts), never a conflict.
            let contract = inf.deliv.is_some() || (inf.node.kind == NodeKind::Subproject && inf.iface.as_ref().is_some_and(|f| f.commitment.is_some() || f.done.is_some()));
            if let (Some(b), false) = (inf.own_by, contract) {
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
                .filter(|(b, why)| if matches!(*why, "container" | "commitment") { inh.map_or(false, |e| end > *b && e > *b) } else { end > *b })
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
        (cx.diags, cx.res, cx.early, flags, links, pf)
    };
    let mut idx = 0;
    for n in &mut project.nodes {
        apply(n, &mut idx, &res, &early, &flags, &links);
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
        let mut fc = BTreeMap::new();
        let mut all = BTreeMap::new();
        let mut dn = BTreeMap::new();
        fn walk(ns: &[Node], fc: &mut BTreeMap<String, (Civil, Option<Civil>)>, all: &mut BTreeMap<String, (Civil, Option<Civil>)>, dn: &mut BTreeMap<String, Civil>) {
            for n in ns {
                if let (true, Some(id), Some(f)) = (n.checkbox != Checkbox::Cancelled, &n.id, &n.forecast) {
                    if let Some(e) = f.end {
                        let early = match (f.declared_end, f.slack_days) {
                            (Some(d), Some(s)) => Some(Civil::from_days(d.to_days() - s)),
                            _ => None,
                        };
                        all.entry(id.clone()).or_insert((e, early));
                        if n.checkbox == Checkbox::Done {
                            dn.entry(id.clone()).or_insert(n.done_date().unwrap_or(e));
                        }
                    }
                }
                if let (true, Some(id), Some(f)) = (n.kind == NodeKind::Milestone && n.checkbox != Checkbox::Cancelled, &n.id, &n.forecast) {
                    if let Some(e) = f.end {
                        let early = match (f.declared_end, f.slack_days) {
                            (Some(d), Some(s)) => Some(Civil::from_days(d.to_days() - s)),
                            _ => None,
                        };
                        fc.entry(id.clone()).or_insert((e, early));
                    }
                }
                walk(&n.children, fc, all, dn);
            }
        }
        walk(&c.nodes, &mut fc, &mut all, &mut dn);
        if let Some(i) = ifs.get_mut(key) {
            i.forecasts = fc;
            i.item_forecasts = all;
            i.item_done = dn;
        }
    }
}
