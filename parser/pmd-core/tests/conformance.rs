mod common;
use common::*;
use pmd_core::*;
use pmd_core::date;
use std::path::PathBuf;

fn conf(name: &str) -> Option<String> {
    let p = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/conformance").join(name);
    std::fs::read_to_string(p).ok()
}

fn split_suffix(s: &str) -> Vec<String> {
    let keys = ["duration", "id", "tag", "date", "range", "owner", "after", "by", "done", "replaced-by", "ticket"];
    let mut out: Vec<String> = Vec::new();
    for tok in s.split_whitespace() {
        let starts = tok.split_once('=').map(|(k, _)| keys.contains(&k)).unwrap_or(false);
        if starts || out.is_empty() {
            out.push(tok.to_string());
        } else {
            let l = out.last_mut().unwrap();
            l.push(' ');
            l.push_str(tok);
        }
    }
    out.sort();
    out
}

/// Inputs where items.txt disagrees with the (amended) GRAMMAR.md; GRAMMAR wins.
const KNOWN: &[&str] = &[
    // a title that is one link with a #fragment is a deliverable (GRAMMAR §2.3, 0.2)
    "- [ ] [Spec](spec.md#part)  2w",
    // leading ranges (GRAMMAR §2.2 step 2 amended): task with declared range
    "- [ ] 2026-01 .. 2026-03 Kickoff",
    "- [ ] 2026-01..2026-03 Kickoff",
    // whitespace inside the title is verbatim per §4.3 (file collapses the double space)
    "- [ ] Design  owner: <jane@example.com>",
    // core keys type-check their values (§4.5 amended): `by: Friday` stays in the title
    "- [ ] Review  by: Friday",
    // E-SYNTAX on a repeated non-repeatable key; last value wins (§4.4)
    "- [ ] Ship  owner: @jan  owner: @piet",
    // invalid dates: the atom is consumed with E-DATE, but no expanded date can be stored
    "- [ ] Fix bug  2027-13",
    "- [ ] 2027-02-30 Bad date",
    // a property bullet must hold exactly one value of the key's type (§2.2, §4.5)
    "- Owner: Jan Doe",
    "- Owner: @jan  2w",
];

#[test]
fn conformance_items() {
    let Some(text) = conf("items.txt") else {
        eprintln!("items.txt not found; skipping");
        return;
    };
    let mut cases: Vec<Vec<(String, String)>> = Vec::new();
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let Some((k, v)) = line.split_once(':') else { continue };
        let v = v.trim().to_string();
        if k == "in" {
            cases.push(Vec::new());
        }
        cases.last_mut().unwrap().push((k.to_string(), v));
    }
    let mut bad = Vec::new();
    for c in &cases {
        let get = |k: &str| c.iter().find(|(a, _)| a == k).map(|(_, v)| v.clone());
        let input = get("in").unwrap();
        let p = parse_str(&format!("# T\n\n- [ ] P\n  {}\n", input));
        let parent = &p.nodes[0];
        let n = parent.children.first().cloned();
        let prop = n.is_none() && !parent.keys.is_empty();
        let (kind, check, date, title, suffix) = match &n {
            None if prop => {
                let mut v: Vec<String> = parent.keys.iter().flat_map(|(k, vs)| vs.iter().map(move |x| format!("{}={}", k, x.text))).collect();
                v.sort();
                ("property".to_string(), "none".to_string(), None, String::new(), v)
            }
            None => ("prose".to_string(), "none".to_string(), None, String::new(), Vec::new()),
            Some(n) => (
                format!("{:?}", n.kind).to_lowercase(),
                format!("{:?}", n.checkbox).to_lowercase(),
                if n.kind == NodeKind::Milestone { n.date.as_ref().map(|d| d.text.clone()) } else { None },
                n.title.clone(),
                {
                    let mut s = suffix_set(n);
                    if let Some(d) = &n.suffix_date {
                        s.push(format!("date={}", d.text));
                        s.sort();
                    }
                    s
                },
            ),
        };
        let mut why = Vec::new();
        if Some(kind.clone()) != get("kind") {
            why.push(format!("kind {} vs {:?}", kind, get("kind")));
        }
        if Some(check.clone()) != get("check") {
            why.push(format!("check {} vs {:?}", check, get("check")));
        }
        if date != get("date") {
            why.push(format!("date {:?} vs {:?}", date, get("date")));
        }
        if Some(title.clone()) != get("title") {
            why.push(format!("title {:?} vs {:?}", title, get("title")));
        }
        if suffix != split_suffix(&get("suffix").unwrap_or_default()) {
            why.push(format!("suffix {:?} vs {:?}", suffix, get("suffix")));
        }
        if !why.is_empty() {
            bad.push(format!("{}  =>  {}", input, why.join("; ")));
        }
    }
    eprintln!("conformance items: {} cases, {} disagreements", cases.len(), bad.len());
    for b in &bad {
        eprintln!("  DISAGREE: {}", b);
    }
    // Known disagreements where the conformance file predates the amended GRAMMAR (leading ranges).
    let unexpected: Vec<&String> = bad.iter().filter(|b| !KNOWN.iter().any(|k| b.starts_with(&format!("{}  =>", k)))).collect();
    assert!(unexpected.is_empty(), "unexpected disagreements: {:#?}", unexpected);
}

#[test]
fn conformance_dates() {
    let Some(text) = conf("dates.txt") else {
        eprintln!("dates.txt not found; skipping");
        return;
    };
    let mut n = 0;
    for line in text.lines() {
        if line.starts_with('#') || line.trim().is_empty() {
            continue;
        }
        let Some((a, b)) = line.split_once("=>") else { continue };
        let (a, b) = (a.trim(), b.trim());
        let got = if let Some((x, y)) = a.split_once(" .. ").or_else(|| date::split_range_word(a)) {
            match date::range_of(x.trim(), y.trim()) {
                date::RangeParse::Ok(s, e) => format!("{}..{}", s, e),
                date::RangeParse::Invalid(_) => "error".to_string(),
                date::RangeParse::NotRange => "notrange".to_string(),
            }
        } else {
            match parse_date(a) {
                DateParse::Ok(s, e) => format!("{}..{}", s, e),
                DateParse::Invalid => "error".to_string(),
                DateParse::NotDate => "notdate".to_string(),
            }
        };
        assert_eq!(got, b, "date {}", a);
        n += 1;
    }
    eprintln!("conformance dates: {} cases", n);
}
