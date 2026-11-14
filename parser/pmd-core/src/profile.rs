//! Profile files (GRAMMAR §7).

use crate::model::{Options, Person};

fn parse_person(s: &str) -> Person {
    let s = s.trim();
    if let (Some(a), true) = (s.rfind('<'), s.ends_with('>')) {
        Person { name: s[..a].trim().to_string(), email: Some(s[a + 1..s.len() - 1].to_string()) }
    } else {
        Person { name: s.to_string(), email: None }
    }
}

/// Candidate profile locations for a project file, nearest first (the caller reads the files).
pub fn profile_candidates(path: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut dir = match path.rfind('/') {
        Some(i) => &path[..i],
        None => "",
    };
    loop {
        if dir.is_empty() {
            out.push(".pmd/profile.md".to_string());
            break;
        }
        out.push(format!("{}/.pmd/profile.md", dir));
        dir = match dir.rfind('/') {
            Some(0) => {
                out.push("/.pmd/profile.md".to_string());
                break;
            }
            Some(i) => &dir[..i],
            None => "",
        };
    }
    out
}

#[derive(PartialEq, Clone, Copy)]
enum Sec {
    None,
    Keys,
    States,
    People,
}

pub fn parse_profile(src: &str) -> Options {
    let mut o = Options::default();
    let mut sec = Sec::None;
    let mut fence: Option<char> = None;
    for line in src.lines() {
        let t = line.trim_start();
        if t.starts_with("```") || t.starts_with("~~~") {
            let c = t.chars().next().unwrap();
            fence = match fence {
                Some(f) if f == c => None,
                None => Some(c),
                f => f,
            };
            continue;
        }
        if fence.is_some() {
            continue;
        }
        if line.starts_with('#') {
            let h = line.trim_start_matches('#').trim().to_ascii_lowercase();
            sec = if h.contains("key") {
                Sec::Keys
            } else if h.contains("state") {
                Sec::States
            } else if h.contains("people") || h.contains("registry") {
                Sec::People
            } else {
                Sec::None
            };
            continue;
        }
        let b = line.as_bytes();
        let i = b.iter().position(|c| *c != b' ' && *c != b'\t').unwrap_or(b.len());
        if i + 1 >= b.len() || !matches!(b[i], b'-' | b'*' | b'+') || b[i + 1] != b' ' {
            continue;
        }
        let body = line[i + 2..].trim();
        match sec {
            Sec::Keys => {
                let name: String = body
                    .trim_start_matches('`')
                    .chars()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '-')
                    .collect::<String>()
                    .to_ascii_lowercase();
                if name.is_empty() {
                    continue;
                }
                if body.to_ascii_lowercase().contains("(repeatable)") {
                    o.repeatable.push(name.clone());
                }
                o.extra_keys.push(name);
            }
            Sec::States => {
                let s = if let Some(r) = body.strip_prefix('`') {
                    r.split('`').next().unwrap_or("").to_string()
                } else {
                    body.split(" — ").next().unwrap_or(body).split(" - ").next().unwrap_or(body).trim().to_string()
                };
                if !s.is_empty() {
                    o.states.push(s);
                }
            }
            Sec::People => {
                if !body.starts_with('@') {
                    continue;
                }
                let end = body.find(|c: char| c.is_whitespace() || c == ':' || c == '→' || c == '=').unwrap_or(body.len());
                let handle = body[..end].to_string();
                let rest = body[end..].trim_start_matches(|c: char| c.is_whitespace() || c == ':' || c == '→' || c == '=' || c == '-' || c == '>');
                o.people.insert(handle, parse_person(rest));
            }
            Sec::None => {}
        }
    }
    o
}
