//! Contract states (FORMAT §9, one table in decision order), computed live from a parent's `by:` and one child's own file.
//! Nothing is stored. Pure: the caller supplies the children it could read.

use crate::date::Civil;
use crate::graph::{resolve, split_fragment};
use crate::model::*;

/// What the caller knows about a directly linked file.
#[derive(Clone, Copy)]
pub enum Child<'a> {
    /// Read and parsed.
    Read(&'a Project),
    /// Not readable here (remote and not fetched, no access): shown as unknown.
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Requested,
    Agreed,
    Overdue,
    Disputed,
    Delivered,
    Unknown,
}

impl State {
    /// Whose move it is: `parent`, `child`, or nobody.
    pub fn actor(self) -> Option<&'static str> {
        match self {
            State::Delivered => None,
            State::Unknown | State::Disputed => Some("parent"),
            State::Requested | State::Agreed | State::Overdue => Some("child"),
        }
    }
    /// The move itself: `fix` the link, `decide` (accept, re-plan or talk), `answer`, `deliver`.
    pub fn move_name(self) -> Option<&'static str> {
        match self {
            State::Delivered => None,
            State::Unknown => Some("fix"),
            State::Disputed => Some("decide"),
            State::Requested => Some("answer"),
            State::Agreed | State::Overdue => Some("deliver"),
        }
    }
    pub fn as_str(self) -> &'static str {
        match self {
            State::Requested => "requested",
            State::Agreed => "agreed",
            State::Overdue => "overdue",
            State::Disputed => "disputed",
            State::Delivered => "delivered",
            State::Unknown => "unknown",
        }
    }
}

/// The contract on one link line that has a `by:`.
#[derive(Clone, Debug)]
pub struct Contract {
    pub line: u32,
    pub title: String,
    /// Normalised path or URL of the child.
    pub target: String,
    pub kind: NodeKind,
    pub by: Civil,
    pub by_text: String,
    /// The child's promise (end date) when it has one.
    pub promise: Option<Civil>,
    pub state: State,
    /// Disputed because the target milestone was cancelled `[-]`.
    pub cancelled: bool,
}

/// The `by:` of one item: a literal date, or the date of a local `^id` milestone. Earliest wins.
fn node_by(proj: &Project, n: &Node) -> Option<(Civil, String)> {
    let mut best: Option<(Civil, String)> = None;
    for v in n.keys.get("by").into_iter().flatten() {
        let cand = match (v.end, v.kind) {
            (Some(e), _) => Some((e, v.text.clone())),
            (None, ValueKind::Word) => v.text.strip_prefix('^').and_then(|id| proj.find_id(id)).and_then(|m| m.date.as_ref()).map(|d| (d.end, d.end.to_string())),
            _ => None,
        };
        if let Some(c) = cand {
            if best.as_ref().is_none_or(|b| c.0 < b.0) {
                best = Some(c);
            }
        }
    }
    best
}

/// The contract table of FORMAT §9, first match wins: delivered, then disputed (cancelled, or a promise
/// later than `by`, even when `by` has passed), then overdue (`by` passed with no promise after it), then
/// requested or agreed. `unknown` is decided by the caller, which has no child to classify.
fn classify(by: Civil, today: Civil, delivered: bool, cancelled: bool, promise: Option<Civil>) -> State {
    if delivered {
        State::Delivered
    } else if cancelled || promise.is_some_and(|p| p > by) {
        State::Disputed
    } else if by < today {
        State::Overdue
    } else {
        match promise {
            None => State::Requested,
            Some(_) => State::Agreed,
        }
    }
}

/// State of a project contract (tested against `by` as if today were `by`).
pub fn project_state(by: Civil, child: &Project) -> State {
    classify(by, by, child.done.is_some(), false, child.commitment.as_ref().map(|c| c.end))
}

/// What a deliverable line sees of its target: `None` when there is no contract (missing or not
/// exposed: the graph check reports it), else (delivered, cancelled, promise).
fn target_of(child: &Project, frag: &str, same_owner: bool) -> Option<(bool, bool, Option<Civil>)> {
    let (n, cancelled) = crate::iface::find_target(child, frag)?;
    if !same_owner && !crate::iface::is_referable(n) {
        return None;
    }
    let promise = match (&n.date, n.forecast.and_then(|f| f.end)) {
        (Some(d), _) if crate::iface::in_interface(n) => Some(d.end),
        (_, f) => f,
    };
    Some((n.checkbox == Checkbox::Done, cancelled, if cancelled { None } else { promise }))
}

/// State of a deliverable contract against milestone `frag` of `child` (milestones only).
pub fn deliverable_state(by: Civil, child: &Project, frag: &str) -> (State, Option<Civil>) {
    match target_of(child, frag, false) {
        Some((d, c, p)) => (classify(by, by, d, c, p), p),
        None => (State::Requested, None),
    }
}

/// Contracts of `proj`: every unfinished sub-project or deliverable line with a `by:`. A link line
/// without `by:` has no contract. `lookup` gets the normalised target; `None` means "skip" (for
/// example a local file that does not exist, which is `E-REF-MISSING`, not a contract matter).
pub fn contracts_of<'a>(proj: &Project, today: Civil, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Vec<Contract> {
    let mut out = Vec::new();
    proj.walk_live(|n| {
        let Some(l) = &n.link else { return };
        if !matches!(n.kind, NodeKind::Subproject | NodeKind::Deliverable) || matches!(n.checkbox, Checkbox::Cancelled | Checkbox::Done) {
            return;
        }
        let Some((by, by_text)) = node_by(proj, n) else { return };
        let (path, frag) = split_fragment(&l.target);
        let target = resolve(&proj.path, path);
        let Some(child) = lookup(&target) else { return };
        let (state, promise, cancelled) = match child {
            Child::Unknown => (State::Unknown, None, false),
            Child::Read(c) => match (n.kind, frag) {
                (NodeKind::Deliverable, Some(f)) => {
                    let same = matches!((&proj.owner, &c.owner), (Some(a), Some(b)) if same_owner(&a.text, &b.text));
                    let Some((d, can, p)) = target_of(c, f, same) else { return };
                    (classify(by, today, d, can, p), p, can)
                }
                _ => {
                    let p = c.commitment.as_ref().map(|c| c.end);
                    (classify(by, today, c.done.is_some(), false, p), p, false)
                }
            },
        };
        out.push(Contract { line: n.line, title: n.title.clone(), target, kind: n.kind, by, by_text, promise, state, cancelled });
    });
    out
}

/// W-OVERDUE, W-DISPUTED, I-REQUESTED and W-UNKNOWN for `proj`'s contracts.
pub fn contract_diags<'a>(proj: &Project, today: Civil, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Vec<Diagnostic> {
    contracts_of(proj, today, lookup)
        .into_iter()
        .filter_map(|c| {
            let t = &c.title;
            let (sev, code, msg) = match (c.state, c.promise) {
                (State::Overdue, _) => (Severity::W, "W-OVERDUE", format!("`{}` is overdue: needed by {}, not delivered", t, c.by_text)),
                (State::Disputed, _) if c.cancelled => (Severity::W, "W-DISPUTED", format!("`{}` was cancelled; it was needed by {}", t, c.by_text)),
                (State::Disputed, Some(d)) => (Severity::W, "W-DISPUTED", format!("`{}` promises {}, after `by: {}`", t, d, c.by_text)),
                (State::Requested, _) => (Severity::I, "I-REQUESTED", format!("`{}` has not answered `by: {}`", t, c.by_text)),
                (State::Unknown, _) => (Severity::W, "W-UNKNOWN", format!("`{}` could not be read; shown as unknown", t)),
                _ => return None,
            };
            let mut d = Diagnostic::new(sev, code, c.line, msg);
            d.path = proj.path.clone();
            Some(d)
        })
        .collect()
}
