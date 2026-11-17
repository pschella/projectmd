//! Track record (D64): how a project's promises and estimates went, from declared and delivery dates
//! only. Informative: a record is shown next to a forecast, never fed into one. Pure.

use crate::iface::interface_items;
use crate::model::*;
use crate::schedule::span_of_text;
use serde::Serialize;

/// Promise record of a project: over its finished exposed milestones (declared and `done:` dates).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
pub struct PromiseRecord {
    pub total: usize,
    pub kept: usize,
    /// Median lateness in days of the late ones; absent when none was late.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub median_late: Option<i64>,
}

/// Estimate record of a file: over done items with an estimate, a recorded start (`after: DATE`) and a `done:` date.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct EstimateRecord {
    pub n: usize,
    /// Median of actual days (inclusive) over estimated days.
    pub median_ratio: f64,
}

fn median_i(v: &mut [i64]) -> Option<i64> {
    v.sort_unstable();
    let n = v.len();
    match n {
        0 => None,
        _ if n % 2 == 1 => Some(v[n / 2]),
        _ => Some((v[n / 2 - 1] + v[n / 2]) / 2),
    }
}

/// Promise record of a project (its interface), or `None` with fewer than 3 finished promises.
pub fn promise_record(p: &Project) -> Option<PromiseRecord> {
    let (mut total, mut kept, mut late) = (0, 0, Vec::new());
    for it in interface_items(p).iter().filter(|i| i.checkbox == 'x') {
        let Some((_, done)) = it.done.as_deref().and_then(span_of_text) else { continue };
        total += 1;
        if done <= it.end {
            kept += 1;
        } else {
            late.push(done.to_days() - it.end.to_days());
        }
    }
    (total >= 3).then(|| PromiseRecord { total, kept, median_late: median_i(&mut late) })
}

/// The promise record as shown: `record 3/5, late +6d` (the lateness only when some were late).
pub fn promise_text(r: &PromiseRecord) -> String {
    let late = r.median_late.map(|m| format!(", late {:+}d", m)).unwrap_or_default();
    format!("record {}/{}{}", r.kept, r.total, late)
}

/// Estimate record of a file, or `None` with fewer than 3 usable items.
pub fn estimate_record(p: &Project) -> Option<EstimateRecord> {
    let mut ratios = Vec::new();
    p.walk(|n| {
        let (Some(dur), Some(done), true) = (&n.duration, n.done_date(), n.checkbox == Checkbox::Done) else { return };
        let Some(start) = n.keys.get("after").into_iter().flatten().filter_map(|v| v.end).max() else { return };
        let est = dur.end_from(start).to_days() - start.to_days() + 1;
        let act = done.to_days() - start.to_days() + 1;
        if est > 0 && act > 0 {
            ratios.push(act as f64 / est as f64);
        }
    });
    if ratios.len() < 3 {
        return None;
    }
    ratios.sort_by(|a, b| a.total_cmp(b));
    let n = ratios.len();
    let median_ratio = if n % 2 == 1 { ratios[n / 2] } else { (ratios[n / 2 - 1] + ratios[n / 2]) / 2.0 };
    Some(EstimateRecord { n, median_ratio })
}
