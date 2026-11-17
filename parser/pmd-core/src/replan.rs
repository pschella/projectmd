//! `replan` (D62): the smallest set of edits to one file's own declared dates that makes it consistent:
//! accept each disputed promise (`by:` := the promise), move each own declared end that its forecast
//! overruns, move the Commitment when the project overruns it. Pure: the caller supplies, for each
//! pass, the file scheduled against its links and its contracts. Only date tokens are rewritten.

use crate::contract::{Contract, State};
use crate::date::Civil;
use crate::iface::in_interface;
use crate::model::*;
use serde::Serialize;
use std::collections::BTreeMap;

const MAX_PASSES: usize = 20;

/// One merged edit: the first declared value of a line and its last proposed value.
#[derive(Clone, Debug, Serialize)]
pub struct Edit {
    pub line: u32,
    pub what: &'static str,
    pub old: String,
    pub new: String,
    pub delta_days: i64,
    /// A Commitment or an exposed milestone moved: the parents will see it.
    pub promise: bool,
}

/// Which date token of a line is rewritten.
#[derive(Clone, Copy, Debug)]
enum Tok {
    /// The leading date of a milestone.
    Leading,
    /// The end of a declared range `A .. B`.
    RangeEnd,
    /// The value of `by:` (on a link line or inline).
    By,
    /// The value of the header `Commitment:` (its end, when it is a range).
    Commitment,
}

struct Prop {
    line: u32,
    what: &'static str,
    tok: Tok,
    /// The declared day the token currently means.
    old_day: Civil,
    new: Civil,
    promise: bool,
}

/// Byte span of the token after the first `anchor` (lower case, matched ignoring ASCII case).
fn span_after(line: &str, anchor: &str) -> Option<(usize, usize)> {
    let at = line.to_ascii_lowercase().find(anchor)? + anchor.len();
    let start = at + line[at..].find(|c: char| !c.is_whitespace())?;
    let end = line[start..].find(char::is_whitespace).map_or(line.len(), |i| start + i);
    Some((start, end))
}

fn commitment_span(line: &str) -> Option<(usize, usize)> {
    let at = line.to_ascii_lowercase().find("commitment:")?;
    if line[at..].contains("..") {
        span_after(line, "..")
    } else {
        span_after(line, "commitment:")
    }
}

fn token_span(line: &str, tok: Tok) -> Option<(usize, usize)> {
    match tok {
        Tok::Leading => span_after(line, "] "),
        Tok::RangeEnd => span_after(line, ".."),
        Tok::By => span_after(line, "by:"),
        Tok::Commitment => commitment_span(line),
    }
}

/// Line index (0-based) of the header `Commitment:` item, if any.
fn commitment_line(lines: &[String]) -> Option<usize> {
    lines.iter().position(|l| l.trim_start().trim_start_matches(['-', '*']).trim_start().to_ascii_lowercase().starts_with("commitment:"))
}

/// Every edit this scheduled file asks for, in one pass.
fn propose(p: &Project, cs: &[Contract], lines: &[String]) -> Vec<Prop> {
    let mut out = Vec::new();
    // accept: a disputed link line's `by:` moves to the promise
    for c in cs.iter().filter(|c| c.state == State::Disputed) {
        if let Some(d) = c.promise {
            out.push(Prop { line: c.line, what: "accept dispute", tok: Tok::By, old_day: c.by, new: d, promise: false });
        }
    }
    // an own declared end that the forecast overruns moves to the forecast
    p.walk_live(|n| {
        let Some(f) = n.forecast else { return };
        if matches!(n.checkbox, Checkbox::Done | Checkbox::Cancelled) || n.is_link_node() || n.kind == NodeKind::Heading {
            return;
        }
        let (Some(fe), Some(de)) = (f.end, f.declared_end) else { return };
        if fe <= de {
            return;
        }
        let tok = if n.kind == NodeKind::Milestone && n.date.is_some() {
            Tok::Leading
        } else if n.range.is_some() {
            Tok::RangeEnd
        } else if n.keys.get("by").and_then(|v| v.first()).is_some_and(|v| v.end.is_some()) {
            Tok::By
        } else {
            return;
        };
        out.push(Prop { line: n.line, what: "move declared end", tok, old_day: de, new: fe, promise: in_interface(n) });
    });
    // the project overruns its Commitment: move it (a `^id` Commitment follows its milestone instead)
    if let (Some(fc), Some(i)) = (p.forecast, commitment_line(lines)) {
        if let (Some(fe), Some(de)) = (fc.end, fc.declared_end) {
            if fe > de && !lines[i].contains('^') {
                out.push(Prop { line: i as u32 + 1, what: "move Commitment", tok: Tok::Commitment, old_day: de, new: fe, promise: true });
            }
        }
    }
    out
}

/// Replan `src` to a fixpoint (at most 20 passes). `sched` schedules one text of this file with its
/// links and returns it with its contracts. Returns the new text and the merged edits in line order.
pub fn replan(src: &str, sched: &dyn Fn(&str) -> (Project, Vec<Contract>)) -> (String, Vec<Edit>) {
    let mut text = src.to_string();
    let mut merged: BTreeMap<u32, (Edit, Civil)> = BTreeMap::new();
    for _ in 0..MAX_PASSES {
        let (p, cs) = sched(&text);
        let mut lines: Vec<String> = text.split('\n').map(String::from).collect();
        let mut changed = false;
        for pr in propose(&p, &cs, &lines) {
            let i = pr.line as usize - 1;
            let Some((a, b)) = token_span(&lines[i], pr.tok) else { continue };
            let old = lines[i][a..b].to_string();
            let new = pr.new.to_string();
            if old.starts_with('^') || old == new {
                continue;
            }
            lines[i].replace_range(a..b, &new);
            changed = true;
            let slot = merged.entry(pr.line).or_insert_with(|| (Edit { line: pr.line, what: pr.what, old: old.clone(), new: String::new(), delta_days: 0, promise: pr.promise }, pr.old_day));
            slot.0.delta_days = pr.new.to_days() - slot.1.to_days();
            slot.0.new = new;
        }
        if !changed {
            break;
        }
        text = lines.join("\n");
    }
    let edits = merged.into_values().map(|(e, _)| e).filter(|e| e.old != e.new).collect();
    (text, edits)
}
