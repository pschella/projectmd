#![allow(dead_code)]
use pmd_core::*;

pub fn parse_str(src: &str) -> Project {
    parse("t.md", src)
}

/// Parse a single list line placed after an H1 and return the first top-level node.
pub fn item(line: &str) -> Option<Node> {
    let p = parse_str(&format!("# T\n\n{}\n", line));
    p.nodes.into_iter().next()
}

/// Suffix of a node as a sorted list of `name=value` strings.
pub fn suffix_set(n: &Node) -> Vec<String> {
    let mut v = Vec::new();
    if let Some(d) = &n.duration {
        v.push(format!("duration={}", d.text));
    }
    if let Some(i) = &n.id {
        v.push(format!("id={}", i));
    }
    for t in &n.tags {
        v.push(format!("tag={}", t));
    }
    if n.kind != NodeKind::Milestone {
        if let Some(d) = &n.date {
            v.push(format!("date={}", d.text));
        }
    }
    if let Some(r) = &n.range {
        v.push(format!("range={}", r.text));
    }
    for (k, vals) in &n.keys {
        for x in vals {
            v.push(format!("{}={}", k, x.text));
        }
    }
    v.sort();
    v
}
