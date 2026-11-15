//! Lock files (GRAMMAR §8, FORMAT §9) and their construction. Pure: the caller reads the files.

use crate::date::Civil;
use crate::graph::{is_remote, normalize, project_links, resolve, Graph};
use crate::model::{Node, NodeKind, Project, ValueKind};
use crate::schedule::{project_start, schedule, Interface, Interfaces};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fmt;

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct LockEntry {
    pub location: String,
    /// key=value fields in order (unknown fields are kept).
    pub fields: Vec<(String, String)>,
    /// (milestone id, date expression or `-> new-id`).
    pub milestones: Vec<(String, String)>,
}

impl LockEntry {
    pub fn field(&self, k: &str) -> Option<&str> {
        self.fields.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str())
    }
    pub fn set(&mut self, k: &str, v: &str) {
        match self.fields.iter_mut().find(|(a, _)| a == k) {
            Some(f) => f.1 = v.to_string(),
            None => self.fields.push((k.to_string(), v.to_string())),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize)]
pub struct Lock {
    /// Leading comment lines (including the `#`).
    pub comments: Vec<String>,
    pub entries: Vec<LockEntry>,
}

impl Lock {
    pub fn parse(src: &str) -> Lock {
        let mut lock = Lock::default();
        for line in src.lines() {
            let line = line.trim_end();
            if line.trim().is_empty() {
                continue;
            }
            if line.starts_with('#') {
                if lock.entries.is_empty() {
                    lock.comments.push(line.to_string());
                }
                continue;
            }
            if line.starts_with(' ') || line.starts_with('\t') {
                if let Some(e) = lock.entries.last_mut() {
                    let t = line.trim();
                    let (id, rest) = match t.find(char::is_whitespace) {
                        Some(i) => (&t[..i], t[i..].trim()),
                        None => (t, ""),
                    };
                    e.milestones.push((id.to_string(), rest.to_string()));
                }
                continue;
            }
            let mut parts = line.split_whitespace();
            let loc = parts.next().unwrap_or("").to_string();
            let mut e = LockEntry { location: loc, ..Default::default() };
            for p in parts {
                if let Some((k, v)) = p.split_once('=') {
                    e.fields.push((k.to_string(), v.to_string()));
                }
            }
            lock.entries.push(e);
        }
        lock
    }

    pub fn entry(&self, location: &str) -> Option<&LockEntry> {
        self.entries.iter().find(|e| e.location == location)
    }

    pub fn set_seen(&mut self, date: &str) {
        for e in &mut self.entries {
            e.set("seen", date);
        }
    }
}

impl fmt::Display for Lock {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for c in &self.comments {
            writeln!(f, "{}", c)?;
        }
        let mut idx: Vec<usize> = (0..self.entries.len()).collect();
        idx.sort_by(|a, b| self.entries[*a].location.cmp(&self.entries[*b].location));
        for i in idx {
            let e = &self.entries[i];
            write!(f, "{}", e.location)?;
            for (k, v) in &e.fields {
                write!(f, "  {}={}", k, v)?;
            }
            writeln!(f)?;
            let mut ms: Vec<&(String, String)> = e.milestones.iter().collect();
            ms.sort_by(|a, b| a.0.cmp(&b.0));
            for (id, d) in ms {
                writeln!(f, "    {}  {}", id, d)?;
            }
        }
        Ok(())
    }
}

/// Directory of a normalised path (`""` for a bare name, `/` for a root-level absolute path).
fn dir_of(p: &str) -> String {
    match p.rfind('/') {
        Some(0) => "/".to_string(),
        Some(i) => p[..i].to_string(),
        None => String::new(),
    }
}

/// `dir/p` normalised; an absolute `p` stands alone.
fn join(dir: &str, p: &str) -> String {
    if p.starts_with('/') || dir.is_empty() {
        normalize(p)
    } else {
        normalize(&format!("{}/{}", dir, p))
    }
}

/// `target` relative to the folder `base` (both normalised, same root).
fn rel_from(base: &str, target: &str) -> String {
    if base.starts_with('/') != target.starts_with('/') {
        return target.to_string();
    }
    let bs: Vec<&str> = base.split('/').filter(|s| !s.is_empty()).collect();
    let ts: Vec<&str> = target.split('/').filter(|s| !s.is_empty()).collect();
    let common = bs.iter().zip(&ts).take_while(|(a, b)| a == b).count();
    let mut parts: Vec<&str> = vec![".."; bs.len() - common];
    parts.extend_from_slice(&ts[common..]);
    parts.join("/")
}

/// A freshly read project's entry: id, hash, by, Commitment, start, late, seen, public milestones.
fn fresh_entry(location: String, p: &Project, hash: Option<&str>, by: Option<&str>, late: Option<Civil>, today: Civil) -> LockEntry {
    let mut e = LockEntry { location, ..Default::default() };
    if let Some(id) = &p.id {
        e.set("id", id);
    }
    if let Some(h) = hash {
        e.set("hash", h);
    }
    if let Some(b) = by {
        e.set("by", b);
    }
    if let Some(c) = &p.commitment {
        e.set("commitment", &c.text);
    }
    if let Some(st) = project_start(p) {
        e.set("start", &st.to_string());
    }
    if let Some(l) = late {
        e.set("late", &l.to_string());
    }
    p.walk(|n| {
        if n.kind == NodeKind::Milestone && n.has_tag("public") {
            if let Some(id) = &n.id {
                let d = if let Some(r) = n.keys.get("replaced-by").and_then(|v| v.first()) {
                    format!("-> {}", r.text.trim_start_matches('^'))
                } else {
                    n.date.as_ref().map(|d| d.text.clone()).unwrap_or_else(|| "-".to_string())
                };
                e.milestones.push((id.clone(), d));
            }
        }
    });
    e.milestones.sort();
    e.set("seen", &today.to_string());
    e
}

/// The need-by date a link item declares (`by:` on a sub-project item linking `child`), as
/// written for literal dates and as an exact date for `^id` of this file. The earliest wins.
fn need_by(proj: &Project, child: &str) -> Option<String> {
    let mut best: Option<(Civil, String)> = None;
    proj.walk(|n: &Node| {
        let Some(l) = &n.link else { return };
        if n.kind != NodeKind::Subproject || resolve(&proj.path, &l.target) != child {
            return;
        }
        for v in n.keys.get("by").into_iter().flatten() {
            let cand = match (v.end, v.kind) {
                (Some(e), _) => Some((e, v.text.clone())),
                (None, ValueKind::Word) | (None, ValueKind::Link) => {
                    let t = v.target.as_deref().unwrap_or(v.text.as_str());
                    t.strip_prefix('^')
                        .and_then(|id| proj.find_id(id))
                        .and_then(|m| m.date.as_ref())
                        .map(|d| (d.end, d.end.to_string()))
                }
                _ => None,
            };
            if let Some(c) = cand {
                if best.as_ref().is_none_or(|b| c.0 < b.0) {
                    best = Some(c);
                }
            }
        }
    });
    best.map(|b| b.1)
}

/// `child`'s forecast end when it is later than its Commitment end, scheduled against its own
/// lock (its children's `late` dates are honoured there). `None` when on time or undefined.
fn late_of(child: &Project, lock: Option<&Lock>, today: Civil) -> Option<Civil> {
    let cpath = normalize(&child.path);
    let cdir = dir_of(&cpath);
    let mut ifs = Interfaces::new();
    for e in lock.map(|l| l.entries.as_slice()).unwrap_or(&[]) {
        let key = if is_remote(&e.location) { e.location.clone() } else { join(&cdir, &e.location) };
        ifs.insert(key, Interface::from_lock_entry(e));
    }
    let mut c = child.clone();
    schedule(&mut c, today, &ifs);
    let end = c.forecast.as_ref()?.end?;
    let ce = c.commitment.as_ref()?.end;
    (end > ce).then_some(end)
}

/// Build the lock of `file` (FORMAT §9, GRAMMAR §8): one entry per directly linked project.
///
/// The caller supplies:
/// - `graph`: `file` and each direct local link that could be read (hash included); a direct
///   link missing from the graph is unreadable;
/// - `child_locks`: the lock (`X.lock`) of each direct local link that has one, keyed by link path;
/// - `prev`: `file`'s previous lock, used only for unreadable direct links;
/// - `today`: the `seen` date of fresh entries and the date `late` is computed for.
///
/// Nothing deeper than the direct links is read. Remote links get location, `by` and `seen`
/// only. `file` itself is never listed.
pub fn build_lock(file: &str, graph: &Graph, child_locks: &HashMap<String, Lock>, prev: Option<&Lock>, today: Civil) -> Lock {
    let me = normalize(file);
    let base = dir_of(&me);
    let name = me.rsplit('/').next().unwrap_or(&me).to_string();
    let stem = name.rsplit_once('.').map(|(s, _)| s.to_string()).unwrap_or_else(|| name.clone());
    let mut entries = Vec::new();
    if let Some(proj) = graph.get(&me) {
        let mut direct: BTreeMap<String, bool> = BTreeMap::new();
        for l in project_links(proj) {
            if l.path != me {
                direct.insert(l.path.clone(), l.remote);
            }
        }
        for (path, &remote) in &direct {
            let by = need_by(proj, path);
            if remote {
                let mut e = LockEntry { location: path.clone(), ..Default::default() };
                if let Some(b) = &by {
                    e.set("by", b);
                }
                e.set("seen", &today.to_string());
                entries.push(e);
            } else if let Some(child) = graph.get(path) {
                let late = late_of(child, child_locks.get(path), today);
                entries.push(fresh_entry(rel_from(&base, path), child, graph.hash(path), by.as_deref(), late, today));
            } else if let Some(old) = prev.and_then(|p| p.entries.iter().find(|e| !is_remote(&e.location) && join(&base, &e.location) == *path)) {
                entries.push(old.clone());
            }
        }
    }
    entries.sort_by(|a, b| a.location.cmp(&b.location));
    Lock { comments: vec![format!("# {}.lock — tool-generated snapshot; rebuild with `pmd lock {}`.", stem, name)], entries }
}
