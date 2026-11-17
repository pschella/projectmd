//! Format 0.9 consolidation: one test per audit finding (dev/reviews/beta-audit.md).
mod common;
use common::*;
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
fn span(p: &Project, title: &str) -> String {
    let mut r = None;
    p.walk(|n| {
        if n.title == title {
            let f = n.forecast.unwrap();
            let s = |d: Option<Civil>| d.map(|d| d.to_string()).unwrap_or("?".into());
            r = Some(format!("{}..{}", s(f.start), s(f.end)));
        }
    });
    r.unwrap_or_else(|| panic!("no node {}", title))
}
fn codes(p: &Project) -> Vec<&'static str> {
    p.diagnostics.iter().map(|d| d.code).collect()
}
fn has(p: &Project, code: &str, needle: &str) -> bool {
    p.diagnostics.iter().any(|d| d.code == code && d.message.contains(needle))
}
fn iface(c: &str, ms: &[(&str, &str)]) -> Interface {
    Interface::from_texts(Some(c), ms.iter().copied())
}
fn parse_c(s: &str) -> Civil {
    match parse_date(s) {
        DateParse::Ok(c, _) => c,
        _ => panic!(),
    }
}

// ---- §1 arithmetic and coarse dates -------------------------------------------------------------

#[test]
fn a1_arithmetic_examples() {
    let t = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/../../testdata/conformance/arithmetic.txt")).unwrap();
    let mut n = 0;
    for l in t.lines().filter(|l| !l.starts_with('#') && !l.trim().is_empty()) {
        let (a, end) = l.split_once("=>").unwrap();
        let (est, start) = a.trim().split_once(" from ").unwrap();
        let p = run(&format!("# T\n\n- [ ] A  {}  after: {}\n", est.trim(), start.trim()), "2020-01-01", &[]);
        assert_eq!(span(&p, "A"), format!("{}..{}", start.trim(), end.trim()), "{}", l);
        n += 1;
    }
    assert!(n >= 4);
}

#[test]
fn a4_coarse_after_starts_on_the_first_day() {
    let p = run("# T\n\n- [ ] A  1w  after: 2027-Q3\n- [ ] B  2027-Q3 .. 2027-Q4\n", "2027-01-01", &[]);
    assert_eq!(span(&p, "A"), "2027-07-01..2027-07-07");
    assert_eq!(span(&p, "B"), "2027-07-01..2027-12-31"); // range start first day, range end last day
}

// ---- §4 algorithm ---------------------------------------------------------------------------------

#[test]
fn a2_unfinished_estimated_work_starts_today_unless_started() {
    let src = "# T\n\n- [x] Pred  2026-09-01 .. 2026-09-01\n";
    let _ = src;
    let p = run("# T\n\n- [x] Pred  done: 2026-09-01  ^p\n- [ ] Follows  2w  after: ^p\n- [ ] Plain  2w\n- [ ] Started  2w  after: 2026-10-01\n", "2026-11-16", &[]);
    assert_eq!(span(&p, "Follows"), "2026-11-16..2026-11-29");
    assert_eq!(span(&p, "Plain"), "2026-11-16..2026-11-29");
    // a recorded start keeps its start; the end never precedes today
    assert_eq!(span(&p, "Started"), "2026-10-01..2026-11-16");
}

#[test]
fn a3_declared_end_combinations() {
    let bad = |l: &str| {
        let p = parse_str(&format!("# T\n\n{}\n", l));
        assert!(has(&p, "E-SYNTAX", "more than one declared end"), "{}: {:?}", l, p.diagnostics);
    };
    bad("- [ ] A  2027-01 .. 2027-02  2w");
    bad("- [ ] A  2027-01 .. 2027-02  by: 2027-03-01");
    bad("- [ ] 2027-06-30 A  by: 2027-05-01");
    bad("- [ ] 2027-06-30 A  2027-01 .. 2027-02");
    // estimate with `by:` is fine; so is a dated estimate
    let p = parse_str("# T\n\n- [ ] A  2w  by: 2027-03-01\n- [ ] 2027-06-30 B  26w\n");
    assert!(!codes(&p).contains(&"E-SYNTAX"), "{:?}", p.diagnostics);
}

#[test]
fn b_trailing_date_is_title_text_with_a_hint() {
    let p = parse_str("# T\n\n- [ ] Ship  2027-06-30\n");
    assert_eq!(p.nodes[0].title, "Ship  2027-06-30");
    assert!(p.nodes[0].date.is_none() && p.nodes[0].range.is_none());
    assert!(has(&p, "W-KEYLIKE", "by: DATE"));
}

#[test]
fn b6_children_win_over_estimate_and_max_with_declared_end() {
    let p = run("# T\n\n- [ ] Parent  4w\n  - [ ] Kid  1w  after: 2027-01-20\n- [ ] Phase  2026-11 .. 2026-12\n  - [ ] Small  2w\n", "2026-11-20", &[]);
    // own start (today, step 2) vs earliest sub-item start: the earlier
    assert_eq!(span(&p, "Parent"), "2026-11-20..2027-01-26");
    // D68: the range's end (12-31) is not reached: Phase ends at its latest sub-item (Small, 12-03)
    assert_eq!(span(&p, "Phase").split("..").nth(1), Some("2026-12-03"));
}

#[test]
fn b8_nothing_to_date_it() {
    let p = run("# T\n\n- [ ] Decide\n- [ ] Follow  1w  after: ^d\n- [ ] Cancelled parent\n  - [-] Gone\n- [ ] [Link](c.md)\n- [ ] Dated only  after: 2027-03-01\n## H\n- [x] Old  done: 2026-01-01\n", "2027-01-01", &[("dir/c.md", Interface::default())]);
    assert_eq!(span(&p, "Decide"), "2027-01-01..2027-01-01");
    assert_eq!(span(&p, "Cancelled parent"), "2027-01-01..2027-01-01");
    assert_eq!(span(&p, "Link"), "2027-01-01..2027-01-01");
    assert_eq!(span(&p, "Dated only"), "2027-03-01..2027-03-01");
    // a heading whose children are all done is not clamped to today
    assert_eq!(span(&p, "H"), "?..2026-01-01");
}

#[test]
fn b9_headings_carry_nothing() {
    let p = run("# T  by: 2026-01-01\n\n## Phase A  by: 2026-12-01  ^pa\n\n- [ ] Task  2w  after: ^pa\n", "2027-01-01", &[]);
    assert_eq!(p.title, "T  by: 2026-01-01");
    assert!(has(&p, "W-KEYLIKE", "heading"));
    assert!(!codes(&p).contains(&"E-CONFLICT"), "{:?}", p.diagnostics);
    assert!(p.nodes[0].id.is_none() && p.nodes[0].keys.is_empty());
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "E-REF-LOCAL").count(), 1); // ^pa is no id any more
}

#[test]
fn c8_closing_hash_rule() {
    let p = parse_str("# T\n\n## Use C#\n- [ ] x\n## Closed ##\n- [ ] y\n");
    let t: Vec<&str> = p.nodes.iter().map(|n| n.title.as_str()).collect();
    assert_eq!(t, ["Use C#", "Closed"]);
}

#[test]
fn b11_after_cycles_are_order_independent() {
    for src in ["- [ ] A  1w  after: ^b  ^a\n- [ ] B  1w  after: ^a  ^b\n", "- [ ] B  1w  after: ^a  ^b\n- [ ] A  1w  after: ^b  ^a\n"] {
        let p = run(&format!("# T\n\n{}", src), "2027-01-01", &[]);
        assert_eq!(p.diagnostics.iter().filter(|d| d.code == "E-CYCLE").count(), 2);
        assert!(p.diagnostics.iter().all(|d| d.code != "E-CYCLE" || d.message.contains("`after:`")));
        assert_eq!(span(&p, "A"), "2027-01-01..2027-01-07");
        assert_eq!(span(&p, "B"), "2027-01-01..2027-01-07");
    }
    // an item after its own descendant
    let p = run("# T\n\n- [ ] Par  after: ^kid\n  - [ ] Kid  1w  ^kid\n", "2027-01-01", &[]);
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "E-CYCLE").count(), 2, "{:?}", p.diagnostics);
    let d = p.diagnostics.iter().find(|d| d.code == "E-CYCLE").unwrap();
    assert_eq!(fix_for(d), Some("Remove one `after:`"));
}

// ---- §5, §6 link nodes and contracts ----------------------------------------------------------------

#[test]
fn b7_after_never_moves_the_end_of_a_promise() {
    let p = run("# T\n\n- [ ] [Sub](s.md)  by: 2027-04-01  after: 2027-06-01\n- [ ] 2027-02-01 [Dm](c.md#m)  after: 2027-06-01  ^m\n", "2027-01-01", &[("dir/s.md", iface("2027-03-31", &[])), ("dir/c.md", iface("2027-12-31", &[("m", "2027-02-01")]))]);
    assert_eq!(span(&p, "Sub"), "2027-03-31..2027-03-31");
    assert_eq!(span(&p, "Dm").split("..").nth(1), Some("2027-02-01"));
    assert!(!codes(&p).contains(&"E-CONFLICT"), "{:?}", p.diagnostics);
}

#[test]
fn b10_contract_states_decision_order() {
    let child = parse("dir/c.md", "# C\n\n- Owner: @c\n- Commitment: 2027-06-30\n\n- [ ] 2027-05-01 M  ^m\n- [-] 2027-05-02 Gone  ^gone\n- [ ] Plain  ^plain\n");
    let silent = parse("dir/s.md", "# S\n\n- Owner: @c\n");
    let a = parse("dir/a.md", "# A\n\n- Owner: @a\n\n- [ ] [Late](s.md)  by: 2027-01-05\n- [ ] [Req](s.md)  by: 2027-03-01\n- [ ] [Can](c.md#gone)  by: 2027-03-01\n- [ ] [Dis](c.md#m)  by: 2027-04-01\n- [ ] [Agr](c.md#m)  by: 2027-05-01\n- [ ] [Priv](c.md#plain)  by: 2027-05-01\n- [ ] [Miss](c.md#nope)  by: 2027-05-01\n");
    let look = |t: &str| match t {
        "dir/c.md" => Some(Child::Read(&child)),
        "dir/s.md" => Some(Child::Read(&silent)),
        _ => None,
    };
    let cs = contracts_of(&a, parse_c("2027-01-10"), &look);
    let s: Vec<(&str, &str)> = cs.iter().map(|c| (c.title.as_str(), c.state.as_str())).collect();
    // overdue beats requested; a missing or unexposed target has no contract
    assert_eq!(s, [("Late", "overdue"), ("Req", "requested"), ("Can", "disputed"), ("Dis", "disputed"), ("Agr", "agreed")]);
    assert_eq!(cs[0].state.actor(), Some("child"));
    assert_eq!(cs[3].state.actor(), Some("parent"));
    assert_eq!(cs[3].state.move_name(), Some("decide"));
    let d = contract_diags(&a, parse_c("2027-01-10"), &look);
    assert!(d.iter().any(|d| d.code == "W-DISPUTED" && d.message.contains("was cancelled")), "{:?}", d);
}

#[test]
fn b10_same_owner_item_targets() {
    let child = parse("dir/c.md", "# C\n\n- Owner: Ann Lee <ann@x.org>\n\n- [ ] Work  4w  ^w\n");
    let a = parse("dir/a.md", "# A\n\n- Owner: ann   LEE <ANN@x.org>\n\n- [ ] [W](c.md#w)  by: 2027-01-15\n");
    let look = |t: &str| (t == "dir/c.md").then_some(Child::Read(&child));
    let mut sc = child.clone();
    schedule(&mut sc, parse_c("2027-01-01"), &Interfaces::new());
    let look2 = |t: &str| (t == "dir/c.md").then_some(Child::Read(&sc));
    assert_eq!(contracts_of(&a, parse_c("2027-01-01"), &look)[0].state, State::Requested); // not scheduled: no promise yet
    let c = &contracts_of(&a, parse_c("2027-01-01"), &look2)[0];
    assert_eq!((c.state, c.promise), (State::Disputed, Some(parse_c("2027-01-28"))));
}

#[test]
fn b15_one_missing_diagnostic_on_the_link_line() {
    let a = parse("a.md", "# A\n\n- Owner: @a\n\n- [ ] [Gone](gone.md)\n");
    let mut g = Graph::new("a.md");
    g.add(a);
    let d: Vec<_> = g.check().into_iter().filter(|d| d.code == "E-REF-MISSING").collect();
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].path.as_str(), d[0].line), ("a.md", 5));
}

// ---- §2, §7 diagnostics and recognition ----------------------------------------------------------

#[test]
fn b13_dated_link_needs_an_id() {
    let p = parse_str("# T\n\n- [ ] 2027-02-10 [Dated sub](c/child.md)\n");
    assert!(has(&p, "E-SYNTAX", "#id"), "{:?}", p.diagnostics);
    let p = parse_str("# T\n\n- [ ] 2027-02-10 [D](c/child.md#m)\n");
    assert!(!codes(&p).contains(&"E-SYNTAX"));
    assert_eq!(p.nodes[0].link.as_ref().unwrap().text, "D");
}

#[test]
fn b4_vertical_owner_runs_to_the_end_of_the_line() {
    let p = parse_str("# T\n\n- [ ] A\n  - owner: Henrik Osei\n- [ ] B\n  - by: 2027-01-01 stray\n");
    assert_eq!(p.nodes[0].keys["owner"][0].text, "Henrik Osei");
    assert!(!p.nodes[1].keys.contains_key("by"));
    assert!(has(&p, "W-KEYLIKE", "one value"));
}

#[test]
fn b5_key_as_last_word_and_by_to_undated_id() {
    let p = parse_str("# T\n\n- [ ] Key  owner:\n- [ ] Range after  1w  after: 2027-01 .. 2027-03\n- [ ] Plain  ^u\n- [ ] Dated  by: ^u\n");
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "W-KEYLIKE" && d.message.contains("no value")).count(), 2, "{:?}", p.diagnostics);
    assert!(has(&p, "E-REF-LOCAL", "no date"));
}

#[test]
fn c9_done_on_open_item_and_case_insensitivity() {
    let p = parse_str("# T\n\n- [ ] A  done: 2027-01-01\n- [ ] B\n  - done: 2027-01-01\n- [ ] 2027-01-01 Gate  #Private  Owner: @x  ^g\n");
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "W-KEYLIKE" && d.message.contains("tick the box")).count(), 2);
    assert!(!in_interface(&p.nodes[2]));
    assert_eq!(p.nodes[2].keys["owner"][0].text, "@x");
    assert_eq!(pmd_core::fmt::format("# T\n\n- [X] 2027-01-01 Gate  Owner: @x  #Private  ^g\n* [ ] Z  By: 2027-01-01\n"), "# T\n\n- [x] 2027-01-01 Gate  #private  owner: @x  ^g\n- [ ] Z  by: 2027-01-01\n");
}

#[test]
fn b12_line_recognition() {
    let p = parse_str("# T\n\n- note\n  - [ ] Nested  ^n\n1. [ ] ordered\n+ [ ] Plus\n* [X] Star\n");
    let t: Vec<(&str, Checkbox)> = p.nodes.iter().map(|n| (n.title.as_str(), n.checkbox)).collect();
    assert_eq!(t, [("Nested", Checkbox::Open), ("Plus", Checkbox::Open), ("Star", Checkbox::Done)]);
    let p = parse_str("# T\n\n- [ ] Parent\n  - prose\n    - [ ] Deep\n");
    assert_eq!(p.nodes[0].children[0].title, "Deep");
}

#[test]
fn headers_leading_range_and_duplicates() {
    let p = parse_str("# T\n\n- Owner: @a\n- Owner: @b\n\n- [ ] 2027-02 .. 2027-03 Lead\n");
    assert!(has(&p, "E-SYNTAX", "set twice"));
    assert!(has(&p, "W-KEYLIKE", "after the title"));
    assert_eq!(pmd_core::fmt::format("# T\n\n- owner: @a\n- COMMITMENT: 2027-01-01\n\n- [ ] A\n"), "# T\n\n- Owner: @a\n- Commitment: 2027-01-01\n\n- [ ] A\n");
}

#[test]
fn b1_unclosed_brackets_are_text() {
    let p = parse_str("# T\n\n- [ ] Link bad [x](y.md  1w\n- [ ] A  owner: Ann <a@b\n");
    assert!(!p.diagnostics.iter().any(|d| d.code == "E-SYNTAX"));
}
