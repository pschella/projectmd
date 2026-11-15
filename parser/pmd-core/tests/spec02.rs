//! Format 0.2 (D47, D50): interface membership, W-KEYLIKE, scheduling rules.
use pmd_core::*;

#[allow(dead_code)]
fn today() -> Civil {
    Civil::new(2026, 11, 15)
}

#[allow(dead_code)]
fn graph_of(files: &[(&str, &str)]) -> Graph {
    let mut g = Graph::new(files[0].0);
    for (path, src) in files {
        g.add(parse(path, src));
    }
    g
}

fn ids(src: &str) -> Vec<String> {
    let p = parse("a.md", src);
    interface_items(&p).into_iter().filter_map(|i| i.id).collect()
}

// ---- interface membership (D45) ---------------------------------------------------------------

#[test]
fn milestones_are_public_unless_private() {
    let d = ids("# A\n\n- [ ] 2027-01-01 One  ^one\n- [ ] 2027-02-01 Two  #private  ^two\n- [ ] Plain  2w  ^plain\n");
    assert_eq!(d, ["one"]);
}

#[test]
fn only_milestones_are_referable() {
    let g = graph_of(&[
        ("r.md", "# R\n\n- Owner: @a\n\n- [ ] [a](s.md#m)\n- [ ] [b](s.md#p)\n- [ ] [c](s.md#t)\n"),
        ("s.md", "# S\n\n- Owner: @a\n\n- [ ] 2027-01-01 M  ^m\n- [ ] 2027-01-01 P  #private  ^p\n- [ ] T  2w  ^t\n"),
    ]);
    let bad: Vec<u32> = g.check().iter().filter(|d| d.code == "E-REF-PUBLIC").map(|d| d.line).collect();
    assert_eq!(bad, [6, 7], "private milestone and plain task are not in the interface");
}

// ---- headings carry no dates (D47) ------------------------------------------------------------

fn sched(src: &str) -> Project {
    let mut p = parse("t.md", src);
    schedule(&mut p, Civil::new(2027, 1, 1), &Interfaces::new());
    p
}

#[test]
fn heading_dates_are_ignored() {
    let p = sched("# T\n\n- Commitment: 2027-03-31\n\n## Phase  2027-01 .. 2027-12\n\n- [ ] X  2w  ^x\n");
    assert!(!p.diagnostics.iter().any(|d| d.code == "E-CONFLICT"), "{:?}", p.diagnostics);
    assert_eq!(p.find_id("x").unwrap().forecast.unwrap().start.unwrap().to_string(), "2027-01-01");
}

// ---- parsing 0.2 -------------------------------------------------------------------------------

#[test]
fn deliverable_and_done_key() {
    let p = parse("a.md", "# A\n\n- [x] [Pack B](b.md#b-samples)  by: 2027-09-30\n- [x] 2026-11-06 Live  done: 2026-11-06  ^l\n");
    assert_eq!(p.nodes[0].kind, NodeKind::Deliverable);
    assert_eq!(p.nodes[0].title, "Pack B");
    assert_eq!(p.nodes[1].done_date().unwrap().to_string(), "2026-11-06");
    let bad = parse("a.md", "# A\n\n- [x] Live  done: Friday\n");
    assert_eq!(bad.nodes[0].title, "Live  done: Friday");
}

#[test]
fn header_keys_done_and_other_keys_kept() {
    let src = "# A\n\n- Owner: @a\n- Done: 2026-11-01\n- Phase: concept\n";
    let p = parse("a.md", src);
    assert_eq!(p.done.as_ref().unwrap().text, "2026-11-01");
    assert_eq!(p.header.get("phase").map(|s| s.as_str()), Some("concept"));
}

#[test]
fn keylike_near_misses_warn_and_stay_in_the_title() {
    let p = parse("t.md", "# T\n\n- Owner: @a\n\n- [ ] Review  by: Friday\n- [ ] Fine  by: 2027-01-01\n- [ ] Fix issue: retry  2w\n");
    assert_eq!(p.nodes[0].title, "Review  by: Friday");
    let k: Vec<(&str, u32)> = p.diagnostics.iter().filter(|d| d.code == "W-KEYLIKE").map(|d| (d.code, d.line)).collect();
    assert_eq!(k, [("W-KEYLIKE", 5)]);
    assert!(p.diagnostics.iter().find(|d| d.code == "W-KEYLIKE").unwrap().message.contains("`by: Friday`"));
    assert_eq!(p.diagnostics.iter().filter(|d| d.severity == Severity::W).count(), 1);
}

#[test]
fn own_commitment_is_a_declared_end_of_the_file() {
    let p = sched("# T\n\n- Commitment: 2027-01-31\n\n- [ ] 2027-01-15 M  ^m\n- [ ] Work  8w  after: ^m  ^w\n");
    assert!(p.diagnostics.iter().any(|d| d.code == "E-CONFLICT"), "{:?}", p.diagnostics);
}

#[test]
fn milestone_never_ends_before_its_start() {
    let p = sched("# T\n\n- [ ] Work  8w  ^w\n- [ ] 2027-01-15 M  after: ^w  ^m\n");
    let m = p.find_id("m").unwrap().forecast.unwrap();
    assert_eq!(m.start.unwrap().to_string(), "2027-02-25", "a milestone may fall on the day its predecessor ends");
    assert_eq!(m.end.unwrap().to_string(), "2027-02-25");
    assert!(m.conflict);
}

#[test]
fn conflict_is_reported_once_against_the_commitment() {
    let p = sched("# T\n\n- Commitment: 2027-01-31\n\n- [ ] A  4w  ^a\n- [ ] B  4w  after: ^a  ^b\n- [ ] C  4w  after: ^b\n");
    let n = p.diagnostics.iter().filter(|d| d.code == "E-CONFLICT").count();
    assert_eq!(n, 1, "{:?}", p.diagnostics);
}

#[test]
fn sub_project_and_deliverable_end_at_declared_dates_only() {
    let mut ifs = Interfaces::new();
    ifs.insert("c.md".to_string(), Interface::from_texts(Some("2027-06-30"), [("final", "2027-06-30"), ("early", "2027-02-01")]));
    let mut p = parse("a.md", "# A\n\n- [ ] [C](c.md)  by: 2027-07-31\n- [ ] [Final](c.md#final)\n- [ ] [Early](c.md#early)\n");
    schedule(&mut p, Civil::new(2027, 1, 1), &ifs);
    let end = |t: &str| {
        let mut r = None;
        p.walk(|n| if n.title == t { r = n.forecast.unwrap().end });
        r.unwrap().to_string()
    };
    assert_eq!(end("C"), "2027-06-30");
    assert_eq!(end("Final"), "2027-06-30");
    assert_eq!(end("Early"), "2027-02-01");
}
