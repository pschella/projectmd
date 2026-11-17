use pmd_core::*;

fn d(s: &str) -> Civil {
    match parse_date(s) {
        DateParse::Ok(c, _) => c,
        _ => panic!("{}", s),
    }
}

const TODAY: &str = "2027-01-11";

/// Replan `src` (the file `parent.md`) against children given as (path, text), as the CLI does.
fn replan_with(src: &str, kids: &[(&str, &str)]) -> (String, Vec<Edit>) {
    replan_on(src, kids, d(TODAY))
}

fn replan_on(src: &str, kids: &[(&str, &str)], today: Civil) -> (String, Vec<Edit>) {
    let children: Vec<Project> = kids.iter().map(|(p, s)| parse(p, s)).collect();
    let mut ifs = Interfaces::new();
    for c in &children {
        ifs.insert(normalize(&c.path), Interface::from_project(c));
    }
    let ps: Vec<(String, &Project)> = children.iter().map(|c| (normalize(&c.path), c)).collect();
    add_forecasts(&mut ifs, &ps, today);
    let sched = |text: &str| {
        let mut p = parse("parent.md", text);
        schedule(&mut p, today, &ifs);
        let scheduled: Vec<Project> = children
            .iter()
            .map(|c| {
                let mut k = c.clone();
                schedule(&mut k, today, &ifs);
                k
            })
            .collect();
        let cs = contracts_of(&p, today, &|t: &str| scheduled.iter().find(|k| normalize(&k.path) == t).map(Child::Read));
        (p, cs)
    };
    replan(src, &sched)
}

fn line_of(text: &str, needle: &str) -> u32 {
    text.lines().position(|l| l.contains(needle)).map(|i| i as u32 + 1).unwrap()
}

const CHILD_LATE: &str = "# Child\n\n- Owner: Cy\n- Commitment: 2027-02-01\n\n- [ ] Build  5d  after: 2027-01-20  ^build\n";

#[test]
fn accept_a_disputed_promise_moves_by_only() {
    let src = "# Parent\n\n- Owner: Pat\n- Commitment: 2027-03-01\n\n- [ ] [Impl](child.md)  by: 2027-01-20  ^impl\n";
    let (out, edits) = replan_with(src, &[("child.md", CHILD_LATE)]);
    assert_eq!(edits.len(), 1);
    let e = &edits[0];
    assert_eq!((e.line, e.what, e.old.as_str(), e.new.as_str(), e.delta_days, e.promise), (6, "accept dispute", "2027-01-20", "2027-02-01", 12, false));
    // only the date token changed: the rest of the line is byte-identical
    assert_eq!(out, src.replace("by: 2027-01-20", "by: 2027-02-01"));
}

#[test]
fn disputed_promise_is_accepted_even_when_by_has_passed() {
    // the promise (2027-01-20) is later than `by:` (2027-01-13), and `by:` is already past: still disputed,
    // so the parent's move, and replan accepts the promise
    let today = d("2027-01-19");
    let src = "# Parent\n\n- Owner: Pat\n\n- [ ] [Impl](child.md)  by: 2027-01-13  ^impl\n";
    let mut child = parse("child.md", "# Child\n\n- Owner: Cy\n- Commitment: 2027-01-20\n");
    schedule(&mut child, today, &Interfaces::new());
    let parent = parse("parent.md", src);
    let cs = contracts_of(&parent, today, &|t: &str| (t == "child.md").then_some(Child::Read(&child)));
    assert_eq!((cs[0].state, cs[0].state.actor(), cs[0].state.move_name()), (State::Disputed, Some("parent"), Some("decide")));
    let (out, edits) = replan_on(src, &[("child.md", "# Child\n\n- Owner: Cy\n- Commitment: 2027-01-20\n")], today);
    assert_eq!(edits.len(), 1);
    assert_eq!((edits[0].what, edits[0].old.as_str(), edits[0].new.as_str()), ("accept dispute", "2027-01-13", "2027-01-20"));
    assert_eq!(out, src.replace("by: 2027-01-13", "by: 2027-01-20"));
}

#[test]
fn overrun_milestone_moves_and_commitment_follows() {
    let src = "# P\n\n- Owner: Pat\n- Commitment: 2027-01-26\n\n- [ ] Design  20d  ^design\n- [ ] 2027-01-25 Alpha  after: ^design  ^alpha\n";
    let (out, edits) = replan_with(src, &[]);
    let alpha = edits.iter().find(|e| e.what == "move declared end").expect("milestone moves");
    let commit = edits.iter().find(|e| e.what == "move Commitment").expect("commitment moves");
    assert_eq!(alpha.line, line_of(src, "Alpha"));
    assert_eq!(alpha.old, "2027-01-25");
    assert!(alpha.promise && commit.promise);
    assert_eq!(commit.new, alpha.new, "the Commitment follows the milestone that sets the end");
    assert!(out.contains(&format!("Commitment: {}", commit.new)));
    // a second run finds nothing left to do
    assert!(replan_with(&out, &[]).1.is_empty());
}

#[test]
fn fixpoint_over_a_chain_moves_the_successor_too() {
    let src = "# Parent\n\n- Owner: Pat\n- Commitment: 2027-03-01\n\n- [ ] [Impl](child.md)  by: 2027-01-20  ^impl\n- [ ] 2027-01-25 Ship  after: ^impl  ^ship\n";
    let (out, edits) = replan_with(src, &[("child.md", CHILD_LATE)]);
    assert_eq!(edits.len(), 2, "{:?}", edits);
    assert_eq!(edits[0].what, "accept dispute");
    let ship = &edits[1];
    assert_eq!((ship.what, ship.old.as_str(), ship.promise), ("move declared end", "2027-01-25", true));
    assert_eq!(ship.line, line_of(src, "Ship"));
    assert!(ship.delta_days > 0);
    assert!(replan_with(&out, &[("child.md", CHILD_LATE)]).1.is_empty());
}

#[test]
fn nothing_to_do_returns_the_text_unchanged() {
    let src = "# P\n\n- Owner: Pat\n- Commitment: 2027-03-01\n\n- [ ] Design  5d  ^design\n- [ ] 2027-03-01 Done line  after: ^design\n";
    let (out, edits) = replan_with(src, &[]);
    assert!(edits.is_empty());
    assert_eq!(out, src);
}

#[test]
fn done_items_and_coarse_dates_are_handled() {
    // a finished milestone past its date is never touched; an open coarse `by:` is rewritten exactly
    let src = "# P\n\n- Owner: Pat\n- Commitment: 2027-03-01\n\n- [x] 2027-01-05 Early  done: 2027-01-09  ^early\n- [ ] Long  30d  by: 2027-01-10  ^long\n";
    let (out, edits) = replan_with(src, &[]);
    assert_eq!(edits.len(), 1);
    assert_eq!(edits[0].what, "move declared end");
    assert_eq!(edits[0].line, line_of(src, "Long"));
    assert!(out.contains("Early  done: 2027-01-09"));
    assert!(!out.contains("by: 2027-01-10"));
}

#[test]
fn write_changes_only_date_tokens_per_line() {
    let src = "# Parent\n\n- Owner: Pat\n- Commitment: 2027-03-01\n\n- [ ] [Impl](child.md)  by: 2027-01-20  ^impl  owner: Bob\n";
    let (out, _) = replan_with(src, &[("child.md", CHILD_LATE)]);
    for (a, b) in src.lines().zip(out.lines()) {
        if a != b {
            assert_eq!(a.replace("2027-01-20", "2027-02-01"), b);
        }
    }
    assert_eq!(src.lines().count(), out.lines().count());
}
