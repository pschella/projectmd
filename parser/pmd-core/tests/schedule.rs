use pmd_core::*;

fn run(src: &str, today: &str, ifs: &[(&str, Interface)]) -> Project {
    let mut p = parse("dir/a.md", src, &Options::default());
    let t = match parse_date(today) {
        DateParse::Ok(c, _) => c,
        _ => panic!(),
    };
    let i: Interfaces = ifs.iter().map(|(k, v)| (k.to_string(), v.clone())).collect();
    schedule(&mut p, t, &i);
    p
}
fn fc(p: &Project, title: &str) -> (String, String, bool) {
    let mut r = None;
    p.walk(|n| {
        if n.title == title || n.link.as_ref().map_or(false, |l| l.text == title) {
            let f = n.forecast.unwrap();
            let s = |d: Option<Civil>| d.map(|d| d.to_string()).unwrap_or("null".into());
            r = Some((s(f.start), s(f.end), f.conflict));
        }
    });
    r.unwrap_or_else(|| panic!("no node {}", title))
}
fn codes(p: &Project) -> Vec<&'static str> {
    p.diagnostics.iter().map(|d| d.code).filter(|c| c.starts_with("E-C") || *c == "E-BOUND").collect()
}
fn iface(c: &str, ms: &[(&str, &str)]) -> Interface {
    Interface::from_texts(Some(c), None, ms.iter().copied())
}

#[test]
fn duration_chain_starts_today() {
    let p = run("# T\n\n- [ ] A  2w  ^a\n- [ ] B  1m  after: ^a\n- [ ] C  3d  after: ^b  ^c\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "A"), ("2027-01-01".into(), "2027-01-14".into(), false));
    assert_eq!(fc(&p, "B"), ("2027-01-15".into(), "2027-02-14".into(), false));
    let j = serde_json::to_value(&p).unwrap();
    assert_eq!(j["nodes"][0]["forecast"]["end"], "2027-01-14");
    assert_eq!(j["forecast"]["start"], "2027-01-01");
}

#[test]
fn months_are_calendar() {
    let p = run("# T\n\n- [ ] A  2027-01-31 .. 2027-01-31  1m\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "A").1, "2027-02-27");
}

#[test]
fn after_chain_and_unfinished_task_without_info() {
    let p = run("# T\n\n- [ ] 2027-03-01 M  ^m\n- [ ] T  10d  after: ^m\n- [ ] Nothing\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "T"), ("2027-03-02".into(), "2027-03-11".into(), false));
    assert_eq!(fc(&p, "Nothing"), ("null".into(), "null".into(), false));
}

#[test]
fn cross_file_after_uses_interface() {
    let src = "# T\n\n- [ ] X  2w  after: [p](b.md#ready)\n- [ ] Y  1w  after: [q](b.md)\n- [ ] Z  1w  after: [q](unknown.md#a)\n";
    let b = iface("2027-12-31", &[("ready", "2027-06-30")]);
    let p = run(src, "2027-01-01", &[("dir/b.md", b)]);
    assert_eq!(fc(&p, "X"), ("2027-07-01".into(), "2027-07-14".into(), false));
    assert_eq!(fc(&p, "Y"), ("2028-01-01".into(), "2028-01-07".into(), false));
    assert_eq!(fc(&p, "Z").0, "2027-01-01"); // unknown target ignored
}

#[test]
fn subproject_interface_vs_own_by() {
    let src = "# T\n\n- [ ] [S](s.md)  by: 2027-06-30\n- [ ] [U](u.md)  by: 2027-08-31\n";
    let s = iface("2027-05-31", &[]);
    let p = run(src, "2027-01-01", &[("dir/s.md", s)]);
    assert_eq!(fc(&p, "S"), ("null".into(), "2027-05-31".into(), false));
    assert_eq!(fc(&p, "U"), ("null".into(), "2027-08-31".into(), false));
    let mut n = None;
    p.walk(|x| if x.link.as_ref().map_or(false, |l| l.text == "S") { n = Some(x.link_resolved.clone()) });
    assert_eq!(n.unwrap().as_deref(), Some("dir/s.md"));
}

#[test]
fn bound_conflict_on_subproject() {
    let p = run("# T\n\n- [ ] [S](s.md)  by: 2027-06-30\n", "2027-01-01", &[("dir/s.md", iface("2027-07-31", &[]))]);
    assert_eq!(codes(&p), vec!["E-BOUND"]);
    assert!(fc(&p, "S").2);
}

#[test]
fn conflict_detection() {
    let src = "# T\n\n- Commitment: 2027-06-30\n\n- [ ] 2027-03-01 Gate  ^g\n- [ ] Work  8w  2027-01-04 .. 2027-01-04  ^w\n- [ ] Late gate  after: ^w  2027-02-01\n- [ ] [S](s.md)  by: 2027-07-31\n";
    let p = run(src, "2027-01-01", &[("dir/s.md", iface("2027-07-31", &[]))]);
    // Work ends 2027-03-01 (ok); Late gate ends 2027-03-01 > 2027-02-01
    assert!(!fc(&p, "Work").2);
    assert!(fc(&p, "Late gate").2);
    // S ends after the project Commitment
    assert!(fc(&p, "S").2);
    assert!(p.forecast.unwrap().conflict);
    assert!(codes(&p).contains(&"E-CONFLICT"));
}

#[test]
fn container_by_and_overdue() {
    let src = "# T\n\n- [ ] Box  by: 2027-02-01\n  - [ ] Inner  6w  ^i\n- [ ] 2026-12-01 Overdue\n";
    let p = run(src, "2027-01-01", &[]);
    assert!(fc(&p, "Inner").2);
    assert!(fc(&p, "Box").2);
    assert_eq!(fc(&p, "Overdue").1, "2027-01-01");
    assert!(fc(&p, "Overdue").2);
    let n = codes(&p).len();
    assert_eq!(n, 2, "{:?}", p.diagnostics); // Inner and Overdue (Box is explained by its child)
}

#[test]
fn cancelled_not_waited_for() {
    let p = run("# T\n\n- [-] Gone  10w  ^g\n- [ ] Next  1w  after: ^g\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "Gone"), ("null".into(), "null".into(), false));
    assert_eq!(fc(&p, "Next"), ("2027-01-01".into(), "2027-01-07".into(), false));
}

#[test]
fn today_clamp() {
    let p = run("# T\n\n- [ ] Open  2026-01-01 .. 2026-02-01\n- [x] Done  2026-01-01 .. 2026-02-01\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "Open").1, "2027-01-01");
    assert_eq!(fc(&p, "Done").1, "2026-02-01");
}

#[test]
fn heading_spans_children() {
    let p = run("# T\n\n## Phase\n\n- [ ] A  2027-02-01 .. 2027-02-10\n- [ ] B  2027-03-01 .. 2027-03-20\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "Phase"), ("2027-02-01".into(), "2027-03-20".into(), false));
}

#[test]
fn cycle_is_reported_and_broken() {
    let p = run("# T\n\n- [ ] A  1w  after: ^b  ^a\n- [ ] B  1w  after: ^a  ^b\n", "2027-01-01", &[]);
    assert!(p.diagnostics.iter().any(|d| d.code == "E-CYCLE"));
    assert!(fc(&p, "A").1 != "null");
    assert!(fc(&p, "B").1 != "null");
}

#[test]
fn absolute_url_paths_keep_leading_slash() {
    assert_eq!(resolve("/examples/car/aurora.md", "powertrain/powertrain.md"), "/examples/car/powertrain/powertrain.md");
    assert_eq!(resolve("/a/b/c.md", "../x.md"), "/a/x.md");
    assert_eq!(resolve("https://h.example/p/q.md", "../r.md#id"), "https://h.example/r.md");
    assert_eq!(resolve("https://h.example/p/q.md", "/r.md"), "https://h.example/r.md");
    assert_eq!(resolve("https://h.example/p/q.md", "https://o.example/z.md"), "https://o.example/z.md");
    let mut p = parse("/examples/car/aurora.md", "# T\n\n- [ ] [S](sub/s.md)  by: 2027-06-30\n- [ ] X  1w  after: [d](sub/s.md#m)\n", &Options::default());
    let mut i = Interfaces::new();
    i.insert("/examples/car/sub/s.md".into(), Interface::from_texts(Some("2027-05-31"), None, [("m", "2027-04-01")]));
    schedule(&mut p, Civil::new(2027, 1, 1), &i);
    assert_eq!(p.nodes[0].link_resolved.as_deref(), Some("/examples/car/sub/s.md"));
    assert_eq!(p.nodes[0].forecast.unwrap().end, Some(Civil::new(2027, 5, 31)));
    assert_eq!(p.nodes[1].forecast.unwrap().start, Some(Civil::new(2027, 4, 2)));
}

#[test]
fn six_weeks_is_inclusive() {
    let p = run("# T\n\n- [ ] A  6w  2026-11-14 .. 2026-11-14\n- [ ] B  1d  after: A\n", "2026-11-14", &[]);
    assert_eq!(fc(&p, "A"), ("2026-11-14".into(), "2026-12-25".into(), false));
    let p = run("# T\n\n- [ ] A  6w  ^a\n- [ ] B  1d  after: ^a\n", "2026-11-14", &[]);
    assert_eq!(fc(&p, "A").1, "2026-12-25");
    assert_eq!(fc(&p, "B"), ("2026-12-26".into(), "2026-12-26".into(), false));
}

#[test]
fn parent_spans_own_range_and_children() {
    let src = "# T\n\n- [ ] Concept  2026-11-16 .. 2027-01-22\n  - [ ] Clinic  2027-01-11 .. 2027-01-15\n- [ ] Early  2027-02-01 .. 2027-02-10\n  - [ ] Kid  2027-01-20 .. 2027-02-20\n";
    let p = run(src, "2026-11-14", &[]);
    assert_eq!(fc(&p, "Concept"), ("2026-11-16".into(), "2027-01-22".into(), false));
    assert_eq!(fc(&p, "Early"), ("2027-01-20".into(), "2027-02-20".into(), false));
}

#[test]
fn coarse_range_with_after() {
    let src = "# T\n\n- [ ] 2028-03-31 Tool  ^tool\n- [ ] Ready  2028-Q2 .. 2028-Q3  after: ^tool\n";
    let p = run(src, "2026-11-14", &[]);
    assert_eq!(fc(&p, "Ready"), ("2028-04-01".into(), "2028-09-30".into(), false));
}

#[test]
fn coarse_estimates_are_checked_against_bounds() {
    let src = "# T\n\n- [ ] Box  by: 2028-03-31\n  - [ ] Est  2028-Q2 .. 2028-Q3\n";
    let p = run(src, "2026-11-14", &[]);
    assert!(fc(&p, "Est").2);
    assert!(codes(&p).contains(&"E-CONFLICT"));
}

#[test]
fn subproject_start_from_interface() {
    let src = "# T\n\n- [ ] [S](s.md)  by: 2027-06-30\n";
    let mut s = iface("2027-05-31", &[]);
    s.start = Some(Civil::new(2027, 2, 1));
    let p = run(src, "2027-01-01", &[("dir/s.md", s)]);
    assert_eq!(fc(&p, "S"), ("2027-02-01".into(), "2027-05-31".into(), false));
    assert_eq!(p.nodes[0].title, "S");
}

#[test]
fn project_start_and_lock_field() {
    let a = parse("a.md", "# T\n\n- Commitment: 2027-Q3\n\n- [ ] 2027-02-01 M\n- [ ] X  2027-01-10 .. 2027-03-01\n", &Options::default());
    assert_eq!(Interface::from_project(&a).start, Some(Civil::new(2027, 1, 10)));
    let b = parse("b.md", "# T\n\n- Commitment: 2026-09-01..2027-11-30\n\n- [ ] 2026-01-01 M\n", &Options::default());
    assert_eq!(Interface::from_project(&b).start, Some(Civil::new(2026, 9, 1)));
    let l = Lock::parse("a.md  id=x  commitment=2027-Q3  start=2027-01-10  seen=2026-11-14\n");
    assert_eq!(Interface::from_lock_entry(&l.entries[0]).start, Some(Civil::new(2027, 1, 10)));
}
