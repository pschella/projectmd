use pmd_core::*;

fn run(src: &str, today: &str, ifs: &[(&str, Interface)]) -> Project {
    let mut p = parse("dir/a.md", src);
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
        if n.title == title || n.link.as_ref().is_some_and(|l| l.text == title) {
            let f = n.forecast.unwrap();
            let s = |d: Option<Civil>| d.map(|d| d.to_string()).unwrap_or("null".into());
            r = Some((s(f.start), s(f.end), f.conflict));
        }
    });
    r.unwrap_or_else(|| panic!("no node {}", title))
}
fn codes(p: &Project) -> Vec<&'static str> {
    p.diagnostics.iter().map(|d| d.code).filter(|c| c.starts_with("E-C")).collect()
}
fn iface(c: &str, ms: &[(&str, &str)]) -> Interface {
    Interface::from_texts(Some(c), ms.iter().copied())
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
    let p = run("# T\n\n- [ ] A  1m  after: 2027-01-31\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "A").1, "2027-02-27");
}

#[test]
fn after_chain_and_unfinished_task_without_info() {
    let p = run("# T\n\n- [ ] 2027-03-01 M  ^m\n- [ ] T  10d  after: ^m\n- [ ] Nothing\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "T"), ("2027-03-02".into(), "2027-03-15".into(), false));
    // an open item with nothing to date it is due today (its successors wait for it)
    assert_eq!(fc(&p, "Nothing"), ("2027-01-01".into(), "2027-01-01".into(), false));
}

#[test]
fn deliverable_dates_come_from_the_interface_and_local_after_follows() {
    let src = "# T\n\n- [ ] [Ready](b.md#ready)  by: 2027-07-31  ^ready\n- [ ] X  2w  after: ^ready\n";
    let b = iface("2027-12-31", &[("ready", "2027-06-30")]);
    let p = run(src, "2027-01-01", &[("dir/b.md", b)]);
    assert_eq!(fc(&p, "X"), ("2027-07-01".into(), "2027-07-14".into(), false));
}

#[test]
fn cross_file_after_is_not_a_key() {
    let p = parse("a.md", "# T\n\n- [ ] X  2w  after: [p](b.md#ready)\n- [ ] Y  1w  after: b.md\n");
    assert!(!p.nodes[0].keys.contains_key("after"));
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "W-KEYLIKE").count(), 2, "{:?}", p.diagnostics);
}

#[test]
fn milestone_may_fall_on_its_predecessors_end_day() {
    let p = run("# T\n\n- [ ] A  2w  ^a\n- [ ] 2027-01-14 M  after: ^a\n- [ ] 2027-01-13 L  after: ^a\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "M"), ("2027-01-14".into(), "2027-01-14".into(), false));
    assert_eq!(fc(&p, "L"), ("2027-01-14".into(), "2027-01-14".into(), true));
}

#[test]
fn subproject_interface_vs_own_by() {
    let src = "# T\n\n- [ ] [S](s.md)  by: 2027-06-30\n- [ ] [U](u.md)  by: 2027-08-31\n";
    let s = iface("2027-05-31", &[]);
    let p = run(src, "2027-01-01", &[("dir/s.md", s)]);
    assert_eq!(fc(&p, "S"), ("2027-05-31".into(), "2027-05-31".into(), false));
    assert_eq!(fc(&p, "U"), ("null".into(), "2027-08-31".into(), false));
    let mut n = None;
    p.walk(|x| if x.link.as_ref().is_some_and(|l| l.text == "S") { n = Some(x.link_resolved.clone()) });
    assert_eq!(n.unwrap().as_deref(), Some("dir/s.md"));
}

#[test]
fn bound_conflict_on_subproject() {
    let p = run("# T\n\n- [ ] [S](s.md)  by: 2027-06-30\n", "2027-01-01", &[("dir/s.md", iface("2027-07-31", &[]))]);
    // a promise later than `by:` is a negotiation (W-DISPUTED from the contracts), not a conflict
    assert!(codes(&p).is_empty(), "{:?}", codes(&p));
    assert!(!fc(&p, "S").2);
}

#[test]
fn conflict_detection() {
    let src = "# T\n\n- Commitment: 2027-06-30\n\n- [ ] 2027-03-01 Gate  ^g\n- [ ] Work  8w  after: 2027-01-04  ^w\n- [ ] 2027-02-01 Late gate  after: ^w\n- [ ] [S](s.md)  by: 2027-07-31\n";
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
    let mut p = parse("/examples/car/aurora.md", "# T\n\n- [ ] [S](sub/s.md)  by: 2027-06-30\n- [ ] [D](sub/s.md#m)  by: 2027-06-30  ^d\n- [ ] X  1w  after: ^d\n");
    let mut i = Interfaces::new();
    i.insert("/examples/car/sub/s.md".into(), Interface::from_texts(Some("2027-05-31"), [("m", "2027-04-01")]));
    schedule(&mut p, Civil::new(2027, 1, 1), &i);
    assert_eq!(p.nodes[0].link_resolved.as_deref(), Some("/examples/car/sub/s.md"));
    assert_eq!(p.nodes[0].forecast.unwrap().end, Some(Civil::new(2027, 5, 31)));
    assert_eq!(p.nodes[2].forecast.unwrap().start, Some(Civil::new(2027, 4, 2)));
}

#[test]
fn six_weeks_is_inclusive() {
    let p = run("# T\n\n- [ ] A  6w  after: 2026-11-14\n- [ ] B  1d  after: A\n", "2026-11-14", &[]);
    assert_eq!(fc(&p, "A"), ("2026-11-14".into(), "2026-12-25".into(), false));
    let p = run("# T\n\n- [ ] A  6w  ^a\n- [ ] B  1d  after: ^a\n", "2026-11-14", &[]);
    assert_eq!(fc(&p, "A").1, "2026-12-25");
    assert_eq!(fc(&p, "B"), ("2026-12-28".into(), "2026-12-28".into(), false));
}

#[test]
fn parent_ends_at_its_sub_items_and_its_range_end_only_gives_slack() {
    let src = "# T\n\n- [ ] Concept  2026-11-16 .. 2027-01-22\n  - [ ] Clinic  2027-01-11 .. 2027-01-15\n- [ ] Early  2027-02-01 .. 2027-02-10\n  - [ ] Kid  2027-01-20 .. 2027-02-20\n";
    let p = run(src, "2026-11-14", &[]);
    // D68: a range with sub-items ends at its latest sub-item; its own end (01-22, 02-10) only gives slack/conflict
    assert_eq!(fc(&p, "Concept"), ("2026-11-16".into(), "2027-01-15".into(), false));
    assert_eq!(fc(&p, "Early"), ("2027-01-20".into(), "2027-02-20".into(), true));
}

#[test]
fn coarse_range_with_after() {
    let src = "# T\n\n- [ ] 2028-03-31 Tool  ^tool\n- [ ] Ready  2028-Q2 .. 2028-Q3  after: ^tool\n";
    let p = run(src, "2026-11-14", &[]);
    // a range is a window: start = max(range start, after:), end = the range end
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
fn promotion_schedules_from_the_estimate_until_the_child_promises() {
    let src = "# T\n\n- [ ] 2027-01-10 Gate  ^g\n- [ ] [S](s.md)  6w  after: ^g  ^s\n- [ ] Next  1w  after: ^s\n";
    // no Commitment in the child: the line's own estimate
    let p = run(src, "2027-01-01", &[("dir/s.md", Interface::default())]);
    assert_eq!(fc(&p, "S"), ("2027-01-11".into(), "2027-02-21".into(), false));
    assert_eq!(fc(&p, "Next").0, "2027-02-22");
    // once the child promises, the promise is the end
    let p = run(src, "2027-01-01", &[("dir/s.md", iface("2027-03-31", &[]))]);
    assert_eq!(fc(&p, "S").1, "2027-03-31");
    assert_eq!(fc(&p, "Next").0, "2027-04-01");
}

#[test]
fn dated_deliverable_forecast_slack_and_conflict() {
    let ifs = [("dir/c.md", iface("2027-12-31", &[("m", "2027-05-01")]))];
    let p = run("# T\n\n- [ ] 2027-06-01 [D](c.md#m)  ^d\n", "2027-01-01", &ifs);
    assert_eq!(fc(&p, "D"), ("2027-05-01".into(), "2027-05-01".into(), false));
    let j = serde_json::to_value(&p).unwrap();
    assert_eq!(j["nodes"][0]["kind"], "milestone");
    assert_eq!(j["nodes"][0]["forecast"]["slack_days"], 31);
    let p = run("# T\n\n- [ ] 2027-04-01 [D](c.md#m)\n", "2027-01-01", &ifs);
    assert!(fc(&p, "D").2);
    assert_eq!(codes(&p), vec!["E-CONFLICT"]);
    let j = serde_json::to_value(&p).unwrap();
    assert_eq!(j["nodes"][0]["forecast"]["slack_days"], -30);
}

fn slack(p: &Project, title: &str) -> Option<i64> {
    let mut r = None;
    p.walk(|n| {
        if n.title == title {
            r = n.forecast.unwrap().slack_days;
        }
    });
    r
}

#[test]
fn r2a_slack_is_declared_end_minus_forecast_end() {
    let p = run("# T\n\n- [ ] Pred 2w ^a\n- [ ] 2027-02-01 Ship  after: ^a\n", "2027-01-01", &[]);
    // D68: Ship is a point on its input's end (01-14); its declared 02-01 gives slack 18
    assert_eq!(slack(&p, "Ship"), Some(18));
    let p = run("# T\n\n- [ ] Pred 3w ^a\n- [ ] 2027-01-10 Ship  after: ^a\n", "2027-01-01", &[]);
    assert_eq!(slack(&p, "Ship"), Some(-11));
}

#[test]
fn r2c_dated_item_with_estimate_is_work_with_a_deadline() {
    let p = run("# T\n\n- [ ] 2027-03-01 Qualify  4w\n- [ ] 2027-01-10 Late  4w\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "Qualify"), ("2027-01-01".into(), "2027-01-28".into(), false));
    assert_eq!(slack(&p, "Qualify"), Some(32));
    assert!(fc(&p, "Late").2);
    assert_eq!(codes(&p), vec!["E-CONFLICT"]);
}

#[test]
fn done_item_ends_at_done_date_and_successors_follow() {
    let src = "# T\n\n- [x] A  2w  done: 2027-01-20  ^a\n- [x] B  2w  ^b\n- [ ] C  1w  after: ^a\n- [ ] D  1w  after: ^b\n";
    let p = run(src, "2027-01-01", &[]);
    assert_eq!(fc(&p, "A").1, "2027-01-20");
    assert_eq!(fc(&p, "C").0, "2027-01-21");
    assert_ne!(fc(&p, "B").1, "2027-01-20");
}

#[test]
fn link_without_commitment_ends_at_estimate_or_by_and_can_be_followed() {
    let src = "# T\n\n- [ ] [S](s.md)  by: 2027-06-30  ^s\n- [ ] [E](e.md)  2w  ^e\n- [ ] X  1w  after: ^s\n";
    let p = run(src, "2027-01-01", &[]);
    assert_eq!(fc(&p, "S").1, "2027-06-30");
    assert_eq!(fc(&p, "E").1, "2027-01-14");
    assert_eq!(fc(&p, "X").0, "2027-07-01");
}

#[test]
fn deliverable_link_without_interface_ends_at_by() {
    let src = "# T\n\n- [ ] [U](u.md#m)  by: 2027-07-31  ^u\n- [ ] Y  1w  after: ^u\n";
    let p = run(src, "2027-01-01", &[]);
    assert_eq!(fc(&p, "U").1, "2027-07-31");
    assert_eq!(fc(&p, "Y").0, "2027-08-01");
    let p = run(src, "2027-01-01", &[("dir/u.md", iface("2027-05-31", &[]))]);
    assert_eq!(fc(&p, "U").1, "2027-07-31");
}

fn msgs(p: &Project) -> Vec<String> {
    p.diagnostics.iter().filter(|d| d.code == "E-CONFLICT").map(|d| d.message.clone()).collect()
}
#[test]
fn r4a_overdue_wording() {
    let p = run("# T\n\n- [ ] 2026-10-01 Late\n- [ ] Task  1w  by: 2026-10-05\n- [ ] 2027-02-01 Future  2w  after: 2027-03-01\n", "2026-11-15", &[]);
    let m = msgs(&p);
    assert!(m.contains(&"`Late` is overdue: not marked done by 2026-10-01".to_string()), "{:?}", m);
    assert!(m.contains(&"`Task` is overdue: not marked done by 2026-10-05".to_string()), "{:?}", m);
    assert!(m.iter().any(|s| s.starts_with("`Future` is forecast to end")), "{:?}", m);
}

#[test]
fn r4b_done_without_date_never_clamped_to_today() {
    let p = run("# T\n\n- [x] A  2w  after: 2026-01-01  ^a\n- [ ] B  1w  after: ^a\n", "2026-11-15", &[]);
    assert_eq!(fc(&p, "A"), ("2026-01-01".into(), "2026-01-14".into(), false));
}

#[test]
fn r7a_date_after_is_inclusive_earliest_start_items_are_day_after() {
    let p = run("# T\n\n- [ ] A  1w  after: 2027-03-01  ^a\n- [ ] B  1w  after: ^a\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "A"), ("2027-03-01".into(), "2027-03-07".into(), false));
    assert_eq!(fc(&p, "B").0, "2027-03-08");
}

#[test]
fn r4c_done_applies_to_milestones_and_dated_estimates() {
    let p = run("# T\n\n- [x] 2027-06-30 Gate  done: 2027-06-10  ^g\n- [x] 2027-06-30 Work  4w  after: 2027-05-01  done: 2027-06-01\n- [ ] Next  1w  after: ^g\n", "2027-07-15", &[]);
    assert_eq!(fc(&p, "Gate").1, "2027-06-10");
    assert_eq!(fc(&p, "Work").1, "2027-06-01");
    assert_eq!(slack(&p, "Gate"), Some(20));
    assert_eq!(slack(&p, "Work"), Some(29));
}

#[test]
fn r4d_cancelled_item_cancels_descendants() {
    let src = "# T\n\n- [-] Group  ^g\n  - [ ] Child  2w  ^c\n  - [ ] 2026-01-01 Old\n- [ ] After  1w  after: ^c\n";
    let p = run(src, "2026-11-15", &[]);
    assert!(msgs(&p).is_empty(), "{:?}", msgs(&p));
    assert_eq!(fc(&p, "After").0, "2026-11-15");
}

#[test]
fn done_project_ends_at_done() {
    let p = run("# T\n\n- Owner: @a\n- Done: 2026-12-01\n\n- [ ] A  2w\n", "2027-01-01", &[]);
    assert_eq!(p.forecast.unwrap().end.unwrap().to_string(), "2026-12-01");
    let c = Interface { commitment: Some((parse_c("2026-09-01"), parse_c("2027-03-01"))), done: Some(parse_c("2026-12-01")), ..Default::default() };
    let p = run("# T\n\n- [ ] [C](c.md)\n", "2027-01-01", &[("dir/c.md", c)]);
    assert_eq!(fc(&p, "C").1, "2026-12-01");
}
fn parse_c(s: &str) -> Civil {
    match parse_date(s) {
        DateParse::Ok(c, _) => c,
        _ => panic!(),
    }
}

#[test]
fn done_without_date_does_not_constrain_successors() {
    let p = run("# T\n\n- [x] A  2w  ^a\n- [ ] B  1w  after: ^a\n", "2027-01-01", &[]);
    assert_eq!(fc(&p, "B"), ("2027-01-01".into(), "2027-01-07".into(), false));
}

#[test]
fn r8b_done_deliverable_ends_at_done_date_without_parent_tick() {
    let mut c = Interface::default();
    c.milestones.insert("m".into(), parse_c("2027-03-01"));
    c.done_milestones.insert("m".into(), parse_c("2027-02-10"));
    let p = run("# T\n\n- [ ] [D](c.md#m)  by: 2027-03-01  ^d\n- [ ] X  1w  after: ^d\n", "2027-06-01", &[("dir/c.md", c)]);
    assert_eq!(fc(&p, "D").1, "2027-02-10");
}

#[test]
fn r9a_done_file_has_no_conflicts_and_no_clamp() {
    let p = run("# T\n\n- Owner: @a\n- Done: 2026-12-01\n\n- [ ] 2026-03-01 Late  ^m\n- [ ] B  2w  by: 2026-02-01\n", "2027-01-01", &[]);
    assert!(codes(&p).is_empty(), "{:?}", p.diagnostics);
    assert!(!p.diagnostics.iter().any(|d| d.message.contains("overdue")));
    assert_ne!(fc(&p, "B").0, "2027-01-01");
    assert_ne!(fc(&p, "Late").1, "2027-01-01");
}

#[test]
fn r9b_dated_deliverable_follows_forecast_only_within_one_owner() {
    // another owner's forecast never crosses the boundary (D69): the declared date does
    let mut c = Interface::default();
    c.milestones.insert("m".into(), parse_c("2027-03-01"));
    c.item_forecasts.insert("m".into(), parse_c("2027-04-10"));
    let p = run("# T\n\n- [ ] 2027-03-15 [M](c.md#m)\n", "2027-01-01", &[("dir/c.md", c.clone())]);
    assert_eq!(fc(&p, "M").1, "2027-03-01");
    assert!(codes(&p).is_empty());
    // the same owner: the forecast flows, and the promise is broken by the forecast
    c.owner = Some("Ann".into());
    let p = run("# T\n\n- Owner: Ann\n\n- [ ] 2027-03-15 [M](c.md#m)\n", "2027-01-01", &[("dir/c.md", c.clone())]);
    assert_eq!(fc(&p, "M").1, "2027-04-10");
    assert_eq!(codes(&p), vec!["E-CONFLICT"]);
    c.item_forecasts.insert("m".into(), parse_c("2027-02-20"));
    let p = run("# T\n\n- Owner: Ann\n\n- [ ] 2027-03-15 [M](c.md#m)\n", "2027-01-01", &[("dir/c.md", c)]);
    assert!(codes(&p).is_empty());
    assert_eq!(fc(&p, "M").1, "2027-02-20");
}

#[test]
fn finished_late_work_is_not_a_commitment_conflict() {
    let p = run("# T\n\n- Commitment: 2027-01-31\n\n- [x] 2027-01-31 Delivered  done: 2027-02-05  ^d\n", "2027-03-01", &[]);
    assert!(!p.diagnostics.iter().any(|d| d.code == "E-CONFLICT"), "{:?}", p.diagnostics);
}
