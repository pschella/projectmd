//! Interface (FORMAT §8): the part of a project's own file that other projects may rely on.
//! Derived, never written: header `Owner`, `Commitment`, `Done` and the milestones that
//! are not `#private`. Pure.

use crate::date::Civil;
use crate::model::*;

/// One interface item: a milestone.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IfaceItem {
    /// `' '` open, `'x'` done, `'-'` cancelled.
    pub checkbox: char,
    /// Date text as written.
    pub date: String,
    pub start: Civil,
    pub end: Civil,
    pub title: String,
    pub id: Option<String>,
    pub done: Option<String>,
}

/// `#private` wins over everything.
pub fn is_private(n: &Node) -> bool {
    n.has_tag("private")
}

/// Is `n` in its project's interface? Every dated milestone, unless `#private`.
pub fn in_interface(n: &Node) -> bool {
    n.kind == NodeKind::Milestone && n.date.is_some() && !is_private(n)
}

/// Can another file reference this item (`file#id`)? Exactly the interface milestones.
pub fn is_referable(n: &Node) -> bool {
    n.kind == NodeKind::Milestone && !is_private(n)
}

/// The interface milestones of a project in file order.
pub fn interface_items(p: &Project) -> Vec<IfaceItem> {
    let mut out = Vec::new();
    p.walk_live(|n| {
        if !in_interface(n) {
            return;
        }
        let d = n.date.as_ref().unwrap();
        out.push(IfaceItem {
            checkbox: match n.checkbox {
                Checkbox::Done => 'x',
                Checkbox::Cancelled => '-',
                _ => ' ',
            },
            date: d.text.clone(),
            start: d.start,
            end: d.end,
            title: n.title.clone(),
            id: n.id.clone(),
            done: n.done_date().map(|d| d.to_string()),
        });
    });
    out
}

/// A project with nothing to promise: no Commitment, no milestones, no `Done`.
pub fn is_empty_promise(p: &Project) -> bool {
    p.commitment.is_none() && p.done.is_none() && interface_items(p).is_empty()
}

/// The item with `id` and whether it is cancelled (itself or an ancestor).
pub fn find_target<'a>(p: &'a Project, id: &str) -> Option<(&'a Node, bool)> {
    fn go<'a>(ns: &'a [Node], id: &str, canc: bool) -> Option<(&'a Node, bool)> {
        ns.iter().find_map(|n| {
            let c = canc || n.checkbox == Checkbox::Cancelled;
            if n.id.as_deref() == Some(id) { Some((n, c)) } else { go(&n.children, id, c) }
        })
    }
    go(&p.nodes, id, false)
}
