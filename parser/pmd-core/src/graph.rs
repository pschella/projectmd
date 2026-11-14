//! Cross-file graph checks (E-REF-MISSING, E-REF-PUBLIC, E-CYCLE). Pure: callers supply projects.

use crate::model::*;
use std::collections::HashMap;

pub fn is_remote(t: &str) -> bool {
    let l = t.as_bytes();
    let starts = |p: &str| l.len() >= p.len() && l[..p.len()].eq_ignore_ascii_case(p.as_bytes());
    starts("http://") || starts("https://")
}

/// Normalise a path: `/` separators, no `.`/empty segments, `..` folded where possible.
pub fn normalize(p: &str) -> String {
    let abs = p.starts_with('/');
    let mut out: Vec<&str> = Vec::new();
    for seg in p.split('/') {
        match seg {
            "" | "." => {}
            ".." => match out.last() {
                Some(l) if *l != ".." => {
                    out.pop();
                }
                _ => {
                    if !abs {
                        out.push("..");
                    }
                }
            },
            s => out.push(s),
        }
    }
    let j = out.join("/");
    if abs {
        format!("/{}", j)
    } else {
        j
    }
}

pub fn split_fragment(t: &str) -> (&str, Option<&str>) {
    match t.find('#') {
        Some(i) => (&t[..i], Some(&t[i + 1..])),
        None => (t, None),
    }
}

/// Resolve a link target (fragment stripped) relative to the file `from`.
pub fn resolve(from: &str, target: &str) -> String {
    let (path, _) = split_fragment(target);
    if is_remote(path) {
        return path.to_string();
    }
    if is_remote(from) {
        // keep `scheme://host` intact; resolve the rest like an absolute path
        let after = from.find("://").map(|i| i + 3).unwrap_or(0);
        let hostend = from[after..].find('/').map(|i| i + after).unwrap_or(from.len());
        let (origin, rest) = from.split_at(hostend);
        let rest = if rest.is_empty() { "/" } else { rest };
        let joined = if path.starts_with('/') { normalize(path) } else { resolve(rest, path) };
        return format!("{}{}", origin, joined);
    }
    if path.starts_with('/') {
        return normalize(path);
    }
    match from.rfind('/') {
        Some(i) => normalize(&format!("{}/{}", &from[..i], path)),
        None => normalize(path),
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LinkKind {
    Subproject,
    Reference,
}

#[derive(Clone, Debug)]
pub struct Link {
    pub kind: LinkKind,
    pub line: u32,
    /// Target as written (resolved through reference definitions).
    pub target: String,
    /// Normalised path or URL, fragment stripped.
    pub path: String,
    pub fragment: Option<String>,
    pub remote: bool,
}

fn ends_md(w: &str) -> bool {
    let b = w.as_bytes();
    b.len() >= 3 && b[b.len() - 3..].eq_ignore_ascii_case(b".md")
}

/// All outgoing file links of a project: sub-projects and cross-file references.
pub fn project_links(p: &Project) -> Vec<Link> {
    let mut out = Vec::new();
    let mut add = |kind: LinkKind, line: u32, target: &str| {
        let (path, frag) = split_fragment(target);
        if path.is_empty() {
            return;
        }
        let remote = is_remote(path);
        if !remote && !ends_md(path) {
            return;
        }
        out.push(Link {
            kind,
            line,
            target: target.to_string(),
            path: resolve(&p.path, path),
            fragment: frag.filter(|f| !f.is_empty()).map(|f| f.to_string()),
            remote,
        });
    };
    for (_, vals) in &p.suffix.keys {
        for v in vals {
            ref_target(v, |t| add(LinkKind::Reference, 1, t));
        }
    }
    p.walk(|n| {
        if n.kind == NodeKind::Subproject {
            if let Some(l) = &n.link {
                add(LinkKind::Subproject, n.line, &l.target);
            }
        }
        for vals in n.keys.values() {
            for v in vals {
                ref_target(v, |t| add(LinkKind::Reference, n.line, t));
            }
        }
    });
    out
}

fn ref_target(v: &Value, mut f: impl FnMut(&str)) {
    match v.kind {
        ValueKind::Link => {
            if let Some(t) = &v.target {
                f(t)
            }
        }
        ValueKind::Word => {
            if !v.text.starts_with('^') && !v.text.starts_with('@') {
                f(&v.text)
            }
        }
        _ => {}
    }
}

#[derive(Default)]
pub struct Graph {
    root: String,
    projects: Vec<Project>,
    index: HashMap<String, usize>,
    hashes: HashMap<String, String>,
}

impl Graph {
    pub fn new(root: &str) -> Graph {
        Graph { root: normalize(root), ..Default::default() }
    }
    pub fn root(&self) -> &str {
        &self.root
    }
    pub fn add(&mut self, project: Project) {
        let key = normalize(&project.path);
        match self.index.get(&key) {
            Some(&i) => self.projects[i] = project,
            None => {
                self.index.insert(key, self.projects.len());
                self.projects.push(project);
            }
        }
    }
    /// Add with the file's content hash (see `hash_prefix`), for lock building.
    pub fn add_hashed(&mut self, project: Project, hash: String) {
        self.hashes.insert(normalize(&project.path), hash);
        self.add(project);
    }
    pub fn get(&self, path: &str) -> Option<&Project> {
        self.index.get(&normalize(path)).map(|&i| &self.projects[i])
    }
    pub fn hash(&self, path: &str) -> Option<&str> {
        self.hashes.get(&normalize(path)).map(|s| s.as_str())
    }
    pub fn projects(&self) -> &[Project] {
        &self.projects
    }
    pub fn len(&self) -> usize {
        self.projects.len()
    }
    pub fn is_empty(&self) -> bool {
        self.projects.is_empty()
    }

    /// Distinct remote (URL) targets linked from loaded projects, sorted.
    pub fn remote_links(&self) -> Vec<String> {
        let mut v: Vec<String> = self.projects.iter().flat_map(project_links).filter(|l| l.remote).map(|l| l.path).collect();
        v.sort();
        v.dedup();
        v
    }

    /// E-REF-MISSING, E-REF-PUBLIC, E-CYCLE.
    pub fn check(&self) -> Vec<Diagnostic> {
        let mut diags = Vec::new();
        let mut frag_cache: HashMap<usize, HashMap<String, bool>> = HashMap::new();
        let mut edges: Vec<Vec<(usize, u32)>> = vec![Vec::new(); self.projects.len()];
        for (i, p) in self.projects.iter().enumerate() {
            for l in project_links(p) {
                if l.remote {
                    continue;
                }
                let mut d = |code: &'static str, msg: String| {
                    let mut x = Diagnostic::new(Severity::E, code, l.line, msg);
                    x.path = p.path.clone();
                    diags.push(x);
                };
                let ti = match self.index.get(&l.path) {
                    Some(&t) => t,
                    None => {
                        d("E-REF-MISSING", format!("linked project not found: {}", l.path));
                        continue;
                    }
                };
                if l.kind == LinkKind::Subproject {
                    edges[i].push((ti, l.line));
                }
                if let Some(frag) = &l.fragment {
                    let idx = frag_cache.entry(ti).or_insert_with(|| {
                        let mut m = HashMap::new();
                        self.projects[ti].walk(|n| {
                            if let Some(id) = &n.id {
                                m.entry(id.clone()).or_insert(n.has_tag("public"));
                            }
                        });
                        m
                    });
                    match idx.get(frag.as_str()) {
                        None => d("E-REF-MISSING", format!("no item ^{} in {}", frag, l.path)),
                        Some(false) => d("E-REF-PUBLIC", format!("^{} in {} is not #public", frag, l.path)),
                        Some(true) => {}
                    }
                }
            }
        }
        // include cycles: iterative DFS
        let n = self.projects.len();
        let mut color = vec![0u8; n];
        for s in 0..n {
            if color[s] != 0 {
                continue;
            }
            let mut stack: Vec<(usize, usize)> = vec![(s, 0)];
            color[s] = 1;
            while let Some(&(u, ei)) = stack.last() {
                if ei < edges[u].len() {
                    stack.last_mut().unwrap().1 += 1;
                    let (v, line) = edges[u][ei];
                    if color[v] == 1 {
                        let from = stack.iter().position(|(x, _)| *x == v).unwrap();
                        let mut chain: Vec<&str> = stack[from..].iter().map(|(x, _)| self.projects[*x].path.as_str()).collect();
                        chain.push(self.projects[v].path.as_str());
                        let mut d = Diagnostic::new(Severity::E, "E-CYCLE", line, format!("include cycle: {}", chain.join(" -> ")));
                        d.path = self.projects[u].path.clone();
                        diags.push(d);
                    } else if color[v] == 0 {
                        color[v] = 1;
                        stack.push((v, 0));
                    }
                } else {
                    color[u] = 2;
                    stack.pop();
                }
            }
        }
        diags
    }
}
