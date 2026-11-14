//! Lock files (GRAMMAR §8) and their construction from a graph.

use crate::graph::{project_links, Graph, LinkKind};
use crate::model::NodeKind;
use serde::Serialize;
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

/// Build the lock for `root`: the root itself plus every project directly linked from it
/// (sub-projects and cross-file references), local or remote.
pub fn build_lock(root: &str, graph: &Graph) -> Lock {
    let root_key = crate::graph::normalize(root);
    let base = match root_key.rfind('/') {
        Some(i) => root_key[..=i].to_string(),
        None => String::new(),
    };
    let rel = |p: &str| -> String { p.strip_prefix(base.as_str()).unwrap_or(p).to_string() };
    let mut lock = Lock {
        comments: vec!["# pmd.lock — tool-generated snapshot; rebuild with `pmd lock`.".to_string()],
        entries: Vec::new(),
    };
    // the root plus every project reachable from it (transitively), local or remote
    let mut targets: Vec<String> = vec![root_key.clone()];
    let mut i = 0;
    while i < targets.len() {
        if let Some(p) = graph.get(&targets[i]) {
            for l in project_links(p) {
                let _ = LinkKind::Subproject;
                if !targets.contains(&l.path) {
                    targets.push(l.path.clone());
                }
            }
        }
        i += 1;
    }
    for t in targets {
        let mut e = LockEntry { location: rel(&t), ..Default::default() };
        if let Some(p) = graph.get(&t) {
            if let Some(id) = &p.id {
                e.set("id", id);
            }
            if let Some(h) = graph.hash(&t) {
                e.set("hash", h);
            }
            if let Some(c) = &p.commitment {
                e.set("commitment", &c.text);
            }
            if let Some(st) = crate::schedule::project_start(p) {
                e.set("start", &st.to_string());
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
        }
        lock.entries.push(e);
    }
    lock.entries.sort_by(|a, b| a.location.cmp(&b.location));
    lock
}
