//! Contract states (FORMAT §9), computed live from a parent's `by:` and one child's own file.
//! Nothing is stored. Pure: the caller supplies the children it could read.

use crate::date::Civil;
use crate::graph::{resolve, split_fragment};
use crate::iface::{interface_items, is_empty_promise};
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

/// State of a project contract, tested in this order: delivered, requested, disputed, agreed. `contracts_of` then turns
/// disputed or agreed into `overdue` when the need-by is before today.
pub fn project_state(by: Civil, child: &Project) -> State {
    if child.done.is_some() {
        State::Delivered
    } else {
        match child.commitment.as_ref().map(|c| c.end) {
            None => State::Requested,
            Some(c) if c > by => State::Disputed,
            Some(_) => State::Agreed,
        }
    }
}

/// State of a deliverable contract against milestone `frag` of `child`.
pub fn deliverable_state(by: Civil, child: &Project, frag: &str) -> (State, Option<Civil>) {
    let items = interface_items(child);
    match items.iter().find(|i| i.id.as_deref() == Some(frag)) {
        Some(i) if i.checkbox == 'x' => (State::Delivered, Some(i.end)),
        None if is_empty_promise(child) => (State::Requested, None),
        None => (State::Disputed, None),
        Some(i) if i.checkbox == '-' => (State::Disputed, None),
        Some(i) if i.end > by => (State::Disputed, Some(i.end)),
        Some(i) => (State::Agreed, Some(i.end)),
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
        let (state, promise) = match child {
            Child::Unknown => (State::Unknown, None),
            Child::Read(c) => match (n.kind, frag) {
                (NodeKind::Deliverable, Some(f)) => deliverable_state(by, c, f),
                _ => (project_state(by, c), c.commitment.as_ref().map(|c| c.end)),
            },
        };
        let state = if by < today && matches!(state, State::Disputed | State::Agreed) { State::Overdue } else { state };
        out.push(Contract { line: n.line, title: n.title.clone(), target, kind: n.kind, by, by_text, promise, state });
    });
    out
}

/// W-OVERDUE, W-DISPUTED, I-REQUESTED and W-UNKNOWN for `proj`'s contracts.
pub fn contract_diags<'a>(proj: &Project, today: Civil, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Vec<Diagnostic> {
    contracts_of(proj, today, lookup)
        .into_iter()
        .filter_map(|c| {
            let t = &c.title;
            let (sev, code, msg) = match c.state {
                State::Overdue => (Severity::W, "W-OVERDUE", format!("`{}` is overdue: needed by {}, not delivered", t, c.by_text)),
                State::Disputed => match c.promise {
                    Some(d) => (Severity::W, "W-DISPUTED", format!("`{}` promises {}, after `by: {}`", t, d, c.by_text)),
                    None => (Severity::W, "W-DISPUTED", format!("`{}` was withdrawn or is not offered; `by: {}`", t, c.by_text)),
                },
                State::Requested => (Severity::I, "I-REQUESTED", format!("`{}` has not answered `by: {}`", t, c.by_text)),
                State::Unknown => (Severity::W, "W-UNKNOWN", format!("`{}` could not be read; shown as unknown", t)),
                _ => return None,
            };
            let mut d = Diagnostic::new(sev, code, c.line, msg);
            d.path = proj.path.clone();
            Some(d)
        })
        .collect()
}
