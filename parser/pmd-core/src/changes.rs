//! `changes` (D61): the events between two versions of one project file, from their declared dates,
//! links and checkboxes. Pure: no link is followed, nothing is scheduled.

use crate::date::Civil;
use crate::graph::split_fragment;
use crate::iface::in_interface;
use crate::model::*;
use serde::Serialize;
use std::collections::{HashMap, HashSet};

/// One event. Kinds: `promise added|moved|removed|done|cancelled` (audience `parents`), `ask added|moved|removed`
/// (audience `child PATH`), `own done|cancelled` and `owner changed` (audience `self`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct Change {
    pub kind: &'static str,
    pub subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The link target as written (asks).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub old: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub new: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub delta_days: Option<i64>,
    pub audience: String,
}

/// A value as shown (`text`) and, when it is a date, its day (for the delta).
type Val = (String, Option<Civil>);

fn day(c: Civil) -> Option<Val> {
    Some((c.to_string(), Some(c)))
}

/// `by:` of a link line (or inline): the text and the day when it is a literal date.
fn by_val(n: &Node) -> Option<Val> {
    n.keys.get("by").and_then(|v| v.first()).map(|v| (format!("by: {}", v.text), v.end))
}

fn owner_of(p: &Project) -> Option<String> {
    p.owner.as_ref().map(|v| v.text.trim().to_string())
}

/// Shown name of an item: `^id Title` (or the title alone).
fn label(n: &Node) -> String {
    let title = n.link.as_ref().map_or(n.title.as_str(), |l| l.text.as_str());
    match &n.id {
        Some(i) => format!("^{} {}", i, title),
        None => title.to_string(),
    }
}

fn ev(kind: &'static str, subject: String, id: Option<&str>, target: Option<String>, old: Option<Val>, new: Option<Val>, audience: &str) -> Change {
    let delta_days = match (&old, &new) {
        (Some((_, Some(a))), Some((_, Some(b)))) => Some(b.to_days() - a.to_days()),
        _ => None,
    };
    Change { kind, subject, id: id.map(str::to_string), target, old: old.map(|v| v.0), new: new.map(|v| v.0), delta_days, audience: audience.to_string() }
}

/// The event for a value that appeared, changed or disappeared; `kinds` is `[added, moved, removed]`.
fn when(kinds: [&'static str; 3], subject: &str, id: Option<&str>, target: Option<String>, old: Option<Val>, new: Option<Val>, audience: &str) -> Option<Change> {
    let [add, mv, rm] = kinds;
    match (old, new) {
        (Some(o), Some(n)) if o.0 != n.0 => Some(ev(mv, subject.to_string(), id, target, Some(o), Some(n), audience)),
        (None, Some(n)) => Some(ev(add, subject.to_string(), id, target, None, Some(n), audience)),
        (Some(o), None) => Some(ev(rm, subject.to_string(), id, target, Some(o), None, audience)),
        _ => None,
    }
}

/// Events from `old` to `new` (two versions of one file), in NEW's line order, then removals.
pub fn changes(old: &Project, new: &Project) -> Vec<Change> {
    let mut out = Vec::new();
    let (oc, nc) = (old.commitment.as_ref().map(|c| c.end), new.commitment.as_ref().map(|c| c.end));
    out.extend(when(["promise added", "promise moved", "promise removed"], "Commitment", None, None, oc.and_then(day), nc.and_then(day), "parents"));
    let (od, nd) = (old.done.as_ref().map(|d| d.end), new.done.as_ref().map(|d| d.end));
    out.extend(when(["promise done", "promise moved", "promise removed"], "Done", None, None, od.and_then(day), nd.and_then(day), "parents"));
    let (oo, no) = (owner_of(old), owner_of(new));
    if oo != no {
        out.push(ev("owner changed", "Owner".into(), None, None, oo.map(|t| (t, None)), no.map(|t| (t, None)), "self"));
    }

    let mut onodes = Vec::new();
    old.walk(|n| onodes.push(n));
    let mut nodes = Vec::new();
    new.walk(|n| nodes.push(n));
    let by_id: HashMap<&str, &Node> = onodes.iter().filter_map(|n| n.id.as_deref().map(|i| (i, *n))).collect();
    let by_title: HashMap<&str, &Node> = onodes.iter().map(|n| (n.title.as_str(), *n)).collect();
    let by_link: HashMap<&str, &Node> = onodes.iter().filter_map(|n| n.link.as_ref().map(|l| (l.target.as_str(), *n))).collect();
    let new_ids: HashSet<&str> = nodes.iter().filter(|n| in_interface(n)).filter_map(|n| n.id.as_deref()).collect();
    let new_links: HashSet<&str> = nodes.iter().filter_map(|n| n.link.as_ref().map(|l| l.target.as_str())).collect();

    for n in &nodes {
        if let (true, Some(id), Some(d)) = (in_interface(n), n.id.as_deref(), n.date.as_ref().map(|d| d.end)) {
            // exposed milestone (promise), matched by its id
            let subj = label(n);
            match by_id.get(id).filter(|o| in_interface(o)).copied() {
                None => out.push(ev("promise added", subj, Some(id), None, None, day(d), "parents")),
                Some(o) => {
                    let od = o.date.as_ref().map(|x| x.end);
                    if n.checkbox == Checkbox::Cancelled {
                        if o.checkbox != Checkbox::Cancelled {
                            out.push(ev("promise cancelled", subj, Some(id), None, od.and_then(day), None, "parents"));
                        }
                    } else {
                        out.extend(when(["promise added", "promise moved", "promise removed"], &subj, Some(id), None, od.and_then(day), day(d), "parents"));
                        if o.checkbox != Checkbox::Done && n.checkbox == Checkbox::Done {
                            out.push(ev("promise done", subj, Some(id), None, None, day(n.done_date().unwrap_or(d)), "parents"));
                        }
                    }
                }
            }
        }
        if let Some(l) = &n.link {
            // ask: a link line, matched by its target incl. fragment; its `by:` is the ask's date
            let aud = format!("child {}", split_fragment(&l.target).0);
            match by_link.get(l.target.as_str()).copied() {
                None => out.push(ev("ask added", l.target.clone(), None, Some(l.target.clone()), None, by_val(n), &aud)),
                Some(o) => out.extend(when(["ask added", "ask moved", "ask removed"], &l.target, None, Some(l.target.clone()), by_val(o), by_val(n), &aud)),
            }
        }
        if !in_interface(n) {
            // own item: matched by ^id, else by title
            let o = n.id.as_deref().and_then(|i| by_id.get(i)).or_else(|| by_title.get(n.title.as_str())).copied();
            if let Some(o) = o.filter(|o| !in_interface(o)) {
                if n.checkbox == Checkbox::Cancelled && o.checkbox != Checkbox::Cancelled {
                    out.push(ev("own cancelled", label(n), n.id.as_deref(), None, None, None, "self"));
                } else if n.checkbox == Checkbox::Done && o.checkbox != Checkbox::Done {
                    out.push(ev("own done", label(n), n.id.as_deref(), None, None, n.done_date().and_then(day), "self"));
                }
            }
        }
    }

    // removals: exposed milestones and link lines of OLD that NEW no longer has
    for o in &onodes {
        if let (true, Some(id)) = (in_interface(o), o.id.as_deref()) {
            if !new_ids.contains(id) {
                out.push(ev("promise removed", label(o), Some(id), None, o.date.as_ref().and_then(|d| day(d.end)), None, "parents"));
            }
        }
        if let Some(l) = &o.link {
            if !new_links.contains(l.target.as_str()) {
                let aud = format!("child {}", split_fragment(&l.target).0);
                out.push(ev("ask removed", l.target.clone(), None, Some(l.target.clone()), by_val(o), None, &aud));
            }
        }
    }
    out
}
