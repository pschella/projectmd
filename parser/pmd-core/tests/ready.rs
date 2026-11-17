use pmd_core::*;

fn titles(src: &str, owner: Option<&str>) -> Vec<String> {
    run(src, owner, &|_| None)
}
fn run<'a>(src: &str, owner: Option<&str>, lookup: &dyn Fn(&str) -> Option<Child<'a>>) -> Vec<String> {
    let mut p = parse("dir/a.md", src);
    let t = match parse_date("2027-01-07") {
        DateParse::Ok(c, _) => c,
        _ => panic!(),
    };
    schedule(&mut p, t, &Interfaces::new());
    ready(&p, t, owner, lookup).into_iter().map(|r| format!("{}|{}", r.title, r.owner)).collect()
}

#[test]
fn after_id_done_vs_open() {
    let s = "# T\n\n- Owner: Ann\n\n- [ ] A  ^a\n- [ ] B  after: ^a\n- [x] C  ^c  done: 2027-01-01\n- [ ] D  after: ^c\n";
    assert_eq!(titles(s, None), ["A|Ann", "D|Ann"]);
}

#[test]
fn after_date_future() {
    let s = "# T\n\n- [ ] A  after: 2027-02-01\n- [ ] B  after: 2027-01-07\n";
    assert_eq!(titles(s, None), ["B|"]);
}

#[test]
fn container_and_ancestor_after() {
    let s = "# T\n\n- [ ] Parent\n  - [ ] Child\n- [ ] Blocked  after: 2027-03-01\n  - [ ] Kid\n- [-] Gone\n  - [ ] Ghost\n- [-] Cx  ^x\n- [ ] AfterCx  after: ^x\n";
    assert_eq!(titles(s, None), ["Child|", "AfterCx|"]);
}

#[test]
fn owner_inheritance_and_filter() {
    let s = "# T\n\n- Owner: Ann\n\n- [ ] P  owner: Bob\n  - [ ] Kid\n- [ ] Own\n";
    assert_eq!(titles(s, None), ["Kid|Bob", "Own|Ann"]);
    assert_eq!(titles(s, Some("bo")), ["Kid|Bob"]);
}

#[test]
fn delivered_deliverable_unblocks_successor() {
    let parent = "# P\n\n- [ ] [Part](c.md#m)  by: 2027-02-01  ^d\n- [ ] Next  after: ^d\n";
    let mk = |tick: &str| parse("dir/c.md", &format!("# C\n\n- [{}] 2027-01-05 M  ^m\n", tick));
    let (open, done) = (mk(" "), mk("x"));
    let with = |c: &Project| run(parent, None, &|t| (t == "dir/c.md").then_some(Child::Read(c)));
    assert_eq!(with(&open), Vec::<String>::new());
    assert_eq!(with(&done), ["Next|"]);
}

#[test]
fn started_and_dated_milestones() {
    let s = "# T\n\n- [ ] A  after: 2027-01-03\n- [ ] 2027-02-01 Future M\n- [ ] 2027-01-05 Past M\n- [ ] 2027-02-01 Gated M  after: 2027-01-01\n";
    assert_eq!(titles(s, None), ["A|", "Past M|", "Gated M|"]);
    let mut p = parse("a.md", s);
    let d = match parse_date("2027-01-07") { DateParse::Ok(c, _) => c, _ => panic!() };
    schedule(&mut p, d, &Interfaces::new());
    let r = ready(&p, d, None, &|_| None);
    assert_eq!(r[0].started.unwrap().to_string(), "2027-01-03");
    assert!(r[1].started.is_none());
}

fn today() -> Civil {
    match parse_date("2027-01-07") {
        DateParse::Ok(c, _) => c,
        _ => panic!(),
    }
}

/// `do`/`decide`/`fix` of the parent, then the children's `answer`/`deliver`, as `kind|path:line|owner|detail`.
fn queue(parent: &str, child: Option<&str>, owner: Option<&str>) -> Vec<String> {
    let p = parse("dir/a.md", parent);
    let c = child.map(|s| parse("dir/c.md", s));
    let lookup = |t: &str| (t == "dir/c.md").then(|| c.as_ref().map_or(Child::Unknown, Child::Read));
    let mut sp = p.clone();
    schedule(&mut sp, today(), &Interfaces::new());
    let mut all = ready(&sp, today(), owner, &lookup);
    all.extend(merge_asks(child_moves(&p, today(), owner, &lookup)));
    all.into_iter().map(|r| format!("{}|{}:{}|{}|{}", r.kind, r.path, r.line, r.owner, r.detail)).collect()
}

/// Several parents point at the same child promise: one child-side entry, with the count and the earliest need-by.
#[test]
fn askers_of_one_promise_are_one_entry() {
    // the promise (2027-01-01) was on time for each need-by, which has passed: all three are overdue
    let c = parse("dir/c.md", "# C\n\n- Owner: Cy\n- Commitment: 2027-01-01\n");
    let lookup = |t: &str| (t == "dir/c.md").then_some(Child::Read(&c));
    let ask = |title: &str, by: &str| {
        let p = parse("dir/a.md", &format!("# {}\n\n- [ ] [Part](c.md)  by: {}\n", title, by));
        child_moves(&p, today(), None, &lookup)
    };
    let mut all = ask("Late", "2027-01-04");
    all.extend(ask("API plan", "2027-01-02"));
    all.extend(ask("Ops", "2027-01-05"));
    let merged = merge_asks(all);
    assert_eq!(merged.len(), 1);
    let r = &merged[0];
    assert_eq!((r.kind, r.askers, r.due.unwrap().to_string()), ("deliver", 3, "2027-01-02".to_string()));
    assert_eq!(r.detail, "needed by 2027-01-02 (3 askers; earliest: API plan); promised 2027-01-01");
}

const P: &str = "# P\n\n- Owner: Pat\n\n- [ ] [Part](c.md)  by: 2027-02-01\n";

#[test]
fn disputed_link_is_a_decide_owned_by_the_link_line() {
    let c = "# C\n\n- Owner: Cy\n- Commitment: 2027-01-05 .. 2027-03-01\n";
    assert_eq!(queue(P, Some(c), None), ["decide|dir/a.md:5|Pat|promises 2027-03-01, asked by 2027-02-01"]);
    let p2 = "# P\n\n- Owner: Pat\n\n- [ ] Work  owner: Bob\n  - [ ] [Part](c.md)  by: 2027-02-01\n";
    assert!(queue(p2, Some(c), None).iter().any(|l| l.starts_with("decide|dir/a.md:6|Bob|")));
}

#[test]
fn unreadable_link_is_a_fix() {
    assert_eq!(queue(P, None, None), ["fix|dir/a.md:5|Pat|cannot read dir/c.md"]);
}

#[test]
fn requested_link_is_an_answer_in_the_child_file() {
    let c = "# C\n\n- Owner: Cy\n";
    assert_eq!(queue(P, Some(c), None), ["answer|dir/c.md:1|Cy|needed by 2027-02-01 (1 asker; earliest: P)"]);
    assert!(queue(P, Some(c), Some("pat")).is_empty());
    assert_eq!(queue(P, Some(c), Some("cy")).len(), 1);
}

#[test]
fn overdue_link_is_a_deliver_located_at_the_milestone() {
    let c = "# C\n\n- Owner: Cy\n- Commitment: 2027-01-04 .. 2027-01-05\n";
    let late = P.replace("2027-02-01", "2027-01-05");
    assert_eq!(queue(&late, Some(c), None), ["deliver|dir/c.md:1|Cy|needed by 2027-01-05 (1 asker; earliest: P); promised 2027-01-05"]);
    let pm = late.replace("c.md)", "c.md#m)");
    let cm = "# C\n\n- Owner: Cy\n\n- [ ] 2027-01-05 Part  ^m\n";
    assert_eq!(queue(&pm, Some(cm), None), ["deliver|dir/c.md:5|Cy|needed by 2027-01-05 (1 asker; earliest: P); promised 2027-01-05"]);
}
