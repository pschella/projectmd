//! Golden conformance cases (testdata/conformance/cases/<nn-name>/), see testdata/conformance/README.md.
//!
//! Each case directory holds `today`, `main.md` (the root) with any children it links to, and
//! `expected.json`: the normative projection of `main.md`. The projection is computed here with
//! the same library calls the CLI makes for `check` and `schedule`, resolving links relative to
//! the case directory.
//!
//! Comparison rules (see the README): `nodes` lists the nodes to assert, matched by line; a field
//! given there must equal the computed value, and `null` means "must be absent". Fields not listed
//! are not asserted. `project` is compared the same way. `contracts` and `diagnostics` are exact.
//!
//! `PMD_BLESS=1 cargo test golden` rewrites each (non-skipped) expected.json with the full
//! computed projection. Only use it to format a file whose values were derived by hand first.

use pmd_core::*;
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Cases held out of the run. Empty: every case is active. A case that disagrees with the tool is
/// recorded in testdata/conformance/DISAGREEMENTS.md, and a human decides which side is wrong.
const SKIPPED: &[&str] = &[];

fn cases_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testdata/conformance/cases")
}

fn case_dirs() -> Vec<PathBuf> {
    let mut v: Vec<PathBuf> = fs::read_dir(cases_dir())
        .expect("testdata/conformance/cases")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir())
        .collect();
    v.sort();
    v
}

fn read_today(dir: &Path) -> Civil {
    let s = fs::read_to_string(dir.join("today")).expect("today file");
    match parse_date(s.trim()) {
        DateParse::Ok(c, _) => c,
        _ => panic!("bad today in {}", dir.display()),
    }
}

/// The root and its direct links that can be read (one hop), like `pmd schedule` and `check`.
fn load(dir: &Path) -> Graph {
    let root = "main.md";
    let src = fs::read_to_string(dir.join(root)).expect("main.md");
    let p = parse(root, &src);
    let links = project_links(&p);
    let mut g = Graph::new(root);
    g.add(p);
    for l in links {
        if l.remote || g.get(&l.path).is_some() {
            continue;
        }
        if let Ok(s) = fs::read_to_string(dir.join(&l.path)) {
            g.add(parse(&l.path, &s));
        }
    }
    g
}

/// Contract lookup over the scheduled graph, as in pmd-cli `lookup_in`.
fn lookup_in<'a>(g: &'a Graph) -> impl Fn(&str) -> Option<Child<'a>> + 'a {
    move |t: &str| if is_remote(t) { Some(Child::Unknown) } else { g.get(t).map(Child::Read) }
}

fn opt_date(d: Option<Civil>) -> Value {
    d.map(|d| json!(d.to_string())).unwrap_or(Value::Null)
}

fn node_json(n: &Node) -> Value {
    let kind = match n.kind {
        NodeKind::Task => "task",
        NodeKind::Milestone => "milestone",
        NodeKind::Subproject => "subproject",
        NodeKind::Deliverable => "deliverable",
        NodeKind::Heading => "heading",
    };
    let cb = match n.checkbox {
        Checkbox::None => "none",
        Checkbox::Open => "open",
        Checkbox::Done => "done",
        Checkbox::Cancelled => "cancelled",
    };
    let mut v = json!({ "line": n.line, "kind": kind, "checkbox": cb, "title": n.title });
    if let Some(id) = &n.id {
        v["id"] = json!(id);
    }
    if let Some(f) = n.forecast {
        v["start"] = opt_date(f.start);
        v["end"] = opt_date(f.end);
        v["slack"] = f.slack_days.map(|s| json!(s)).unwrap_or(Value::Null);
    }
    v
}

/// The projection of `main.md` in a case directory.
fn evaluate(dir: &Path) -> Value {
    let today = read_today(dir);
    let g = load(dir);
    let root = normalize("main.md");
    let p0 = g.get(&root).expect("root loaded").clone();

    // Interfaces exactly as pmd-cli `interfaces_for`.
    let mut ifs = Interfaces::new();
    for p in g.projects() {
        ifs.insert(normalize(&p.path), Interface::from_project(p));
    }
    let ps: Vec<(String, &Project)> = g.projects().iter().map(|p| (normalize(&p.path), p)).collect();
    add_forecasts(&mut ifs, &ps, today);

    // Every loaded file scheduled, for contract lookups (as pmd-cli `scheduled`).
    let mut sg = Graph::new(&root);
    for p in g.projects() {
        let mut c = p.clone();
        schedule(&mut c, today, &ifs);
        sg.add(c);
    }

    // Diagnostics of the root as `pmd check` reports them: parse, graph, contracts, schedule.
    let mut diags: Vec<Diagnostic> = p0.diagnostics.clone();
    diags.extend(g.check().into_iter().filter(|d| normalize(&d.path) == root));
    diags.extend(contract_diags(&p0, today, &lookup_in(&sg)));
    let mut c = p0.clone();
    let before = c.diagnostics.len();
    schedule(&mut c, today, &ifs);
    diags.extend(c.diagnostics[before..].iter().cloned());
    diags.sort_by(|a, b| (a.line, a.code).cmp(&(b.line, b.code)));

    let mut nodes = Vec::new();
    c.walk(|n| nodes.push(node_json(n)));
    let f = c.forecast.unwrap_or_default();
    let project = json!({
        "start": opt_date(f.start),
        "end": opt_date(f.end),
        "commitment_end": c.commitment.as_ref().map(|d| json!(d.end.to_string())).unwrap_or(Value::Null),
    });
    let contracts: Vec<Value> = contracts_of(&p0, today, &lookup_in(&sg))
        .iter()
        .map(|k| json!({ "line": k.line, "state": k.state.as_str(), "actor": k.state.actor() }))
        .collect();
    let diagnostics: Vec<Value> = diags.iter().map(|d| json!({ "line": d.line, "code": d.code })).collect();
    json!({ "nodes": nodes, "project": project, "contracts": contracts, "diagnostics": diagnostics })
}

/// Differences between an expectation and the computed projection (empty when they agree).
fn compare(exp: &Value, act: &Value) -> Vec<String> {
    let mut errs = Vec::new();
    let empty = Vec::new();
    let act_nodes = act["nodes"].as_array().unwrap_or(&empty);
    for e in exp["nodes"].as_array().unwrap_or(&empty) {
        let line = e["line"].clone();
        let Some(a) = act_nodes.iter().find(|a| a["line"] == line) else {
            errs.push(format!("node at line {} is not in the projection", line));
            continue;
        };
        for (k, v) in e.as_object().into_iter().flatten() {
            if k == "line" {
                continue;
            }
            let av = a.get(k).cloned().unwrap_or(Value::Null);
            if *v != av {
                errs.push(format!("node line {}: {} expected {} got {}", line, k, v, av));
            }
        }
    }
    for (k, v) in exp["project"].as_object().into_iter().flatten() {
        let av = act["project"].get(k).cloned().unwrap_or(Value::Null);
        if *v != av {
            errs.push(format!("project {}: expected {} got {}", k, v, av));
        }
    }
    if exp["contracts"] != act["contracts"] {
        errs.push(format!("contracts expected {} got {}", exp["contracts"], act["contracts"]));
    }
    let mut ed = exp["diagnostics"].as_array().cloned().unwrap_or_default();
    ed.sort_by(|a, b| (a["line"].as_u64(), a["code"].as_str()).cmp(&(b["line"].as_u64(), b["code"].as_str())));
    if Value::Array(ed.clone()) != act["diagnostics"] {
        errs.push(format!("diagnostics expected {} got {}", Value::Array(ed), act["diagnostics"]));
    }
    errs
}

#[test]
fn golden_cases() {
    let bless = std::env::var("PMD_BLESS").is_ok();
    let mut failures: Vec<String> = Vec::new();
    let mut ran = 0;
    for dir in case_dirs() {
        let name = dir.file_name().unwrap().to_string_lossy().to_string();
        if SKIPPED.contains(&name.as_str()) {
            continue;
        }
        let act = evaluate(&dir);
        if bless {
            let s = serde_json::to_string_pretty(&act).unwrap() + "\n";
            fs::write(dir.join("expected.json"), s).unwrap();
            continue;
        }
        let exp: Value = serde_json::from_str(&fs::read_to_string(dir.join("expected.json")).expect("expected.json"))
            .unwrap_or_else(|e| panic!("{}: bad expected.json: {}", name, e));
        ran += 1;
        let errs = compare(&exp, &act);
        if !errs.is_empty() {
            let mut m = format!("--- {}\n", name);
            for e in &errs {
                m.push_str(&format!("    {}\n", e));
            }
            m.push_str(&format!("    computed: {}\n", serde_json::to_string(&act).unwrap()));
            failures.push(m);
        }
    }
    if bless {
        return;
    }
    assert!(ran >= 40, "expected at least 40 golden cases, ran {}", ran);
    assert!(failures.is_empty(), "{} golden case(s) disagree with the tool:\n{}", failures.len(), failures.join("\n"));
}

fn dated_by_id(v: &Value) -> BTreeMap<String, (Value, Value)> {
    let mut m = BTreeMap::new();
    for n in v["nodes"].as_array().into_iter().flatten() {
        if let Some(id) = n["id"].as_str() {
            m.insert(id.to_string(), (n["start"].clone(), n["end"].clone()));
        }
    }
    m
}

/// The same items in a different file order give the same dates (FORMAT §7.3).
#[test]
fn order_independence_matches_by_id() {
    let a = dated_by_id(&evaluate(&cases_dir().join("33-order-independence-a")));
    let b = dated_by_id(&evaluate(&cases_dir().join("34-order-independence-b")));
    assert_eq!(a.len(), 3, "three items with ids");
    assert_eq!(a, b, "dates by id must not depend on file order");
}
