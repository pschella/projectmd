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
