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
    /// Public milestone id -> declared (end) date.
    pub milestones: BTreeMap<String, Civil>,
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
        let mut i = Interface { commitment: p.commitment.as_ref().map(|c| (c.start, c.end)), ..Default::default() };
        for it in crate::iface::interface_items(p) {
            if it.checkbox != '-' {
                if let Some(id) = it.id {
                    i.milestones.entry(id).or_insert(it.end);
                }
            }
        }
        i
    }
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
    today: Civil,
    diags: Vec<Diagnostic>,
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
        // A task starts the day after its predecessor ends. A milestone or deliverable is a point
        // in time and may fall on the day its predecessor ends.
        let point = matches!(node.kind, NodeKind::Milestone | NodeKind::Deliverable);
        let mut after: Option<Civil> = None;
        let step = |c: Civil| if point { c } else { Civil::from_days(c.to_days() + 1) };
        for d in deps {
            match d {
                Dep::Fixed(c) => after = mx(after, Some(step(c))),
                Dep::Local(j) => {
                    if self.eval_dep(i, j) {
                        if let Some(e) = self.res[j].and_then(|r| r.end) {
                            after = mx(after, Some(step(e)));
                        }
                    }
                }
            }
        }
        self.state[i] = 2;
        if node.checkbox == Checkbox::Cancelled {
            self.res[i] = None;
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
        if node.kind == NodeKind::Heading {
            end = kid_end;
            start = mn(start, kid_start);
        } else if let Some(d) = info.deliv {
            // deliverable: the milestone's date
            end = Some(d);
            if start.is_none() {
                start = Some(d);
            }
        } else if let Some(iface) = info.iface.as_ref().filter(|f| f.commitment.is_some() && node.kind == NodeKind::Subproject) {
            // sub-project: the child's declared Commitment
            let (cs, ce) = iface.commitment.unwrap();
            end = Some(ce);
            if start.is_none() {
                start = Some(if cs != ce { cs } else { ce });
            }
        } else if let Some(dur) = &node.duration {
            if unfinished(node) && info.decl_start.is_none() && (start.is_none() || info.deps.is_empty()) {
                start = mx(start, Some(today));
            }
            end = start.map(|s| dur.end_from(s));
        } else if kid_end.is_some() {
            end = mx(kid_end, info.decl_end);
            start = mn(start, kid_start);
        } else if node.is_link_node() {
            // a link to a child with no promise yet: the line's own estimate (promotion rule, D47)
            end = info.decl_end.or(info.own_by);
        } else {
            end = info.decl_end;
        }
        if let (Some(s), Some(e)) = (start, end) {
            if e < s {
                end = Some(s);
            }
        }
        if end.is_none() {
            end = start;
        }
        if unfinished(node) {
            end = end.map(|e| e.max(today));
        }
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
    });
    for c in &n.children {
        let ci = collect(c, Some(idx), infos);
        infos[idx].children.push(ci);
    }
    idx
}

fn apply<'a>(n: &mut Node, idx: &mut usize, res: &[Option<Res>], flags: &[(Option<Civil>, bool)], links: &[Option<String>]) {
    let i = *idx;
    *idx += 1;
    let r = res[i].unwrap_or_default();
    n.forecast = Some(Forecast { start: r.start, end: r.end, declared_end: flags[i].0, conflict: flags[i].1 });
    n.link_resolved = links[i].clone();
    for c in &mut n.children {
        apply(c, idx, res, flags, links);
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
    let (diags, res, flags, links, pf) = {
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
                if n.kind == NodeKind::Deliverable {
                    let frag = split_fragment(&l.target).1.unwrap_or("");
                    infos[i].deliv = infos[i].iface.as_ref().and_then(|f| f.milestones.get(frag).copied());
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
        let mut cx = Ctx { infos, res: vec![None; n], state: vec![0; n], today, diags: Vec::new() };
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
            eff[i] = if n_.checkbox == Checkbox::Cancelled {
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
            if node.checkbox == Checkbox::Cancelled || node.checkbox == Checkbox::Done {
                continue;
            }
            // (bound, is the sub-project's own `by:`)
            let mut bounds: Vec<(Civil, &'static str)> = Vec::new();
            if let Some(d) = inf.own_date {
                bounds.push((d, "own"));
            }
            // A link whose interface is known has a contract, not a bound: `by:` against the
            // promise is a negotiation (W-DISPUTED from the contracts), never a conflict.
            let contract = inf.deliv.is_some() || (inf.node.kind == NodeKind::Subproject && inf.iface.as_ref().is_some_and(|f| f.commitment.is_some()));
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
                let d = Diagnostic::new(
                    Severity::E,
                    "E-CONFLICT",
                    node.line,
                    format!("`{}` is forecast to end {}, after {}", node.title, end, bound_name),
                );
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
        let peff = tops.iter().fold(None, |a, &t| mx(a, eff[t]));
        let pconf = matches!((peff, proj_end), (Some(e), Some(c)) if e > c);
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
        let pf = Forecast { start: ps, end: pe, declared_end: proj_end, conflict: pconf };
        (cx.diags, cx.res, flags, links, pf)
    };
    let mut idx = 0;
    for n in &mut project.nodes {
        apply(n, &mut idx, &res, &flags, &links);
    }
    project.forecast = Some(pf);
    let mut diags = diags;
    diags.sort_by_key(|d| d.line);
    project.diagnostics.extend(diags);
}
