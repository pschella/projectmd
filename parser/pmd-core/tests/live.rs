//! D50: contract states are computed live over one hop; across an access boundary a front file
//! carries the promise and the private plan stays unread by parents.
use pmd_core::*;

fn p(path: &str, src: &str) -> Project {
    parse(path, src)
}

fn states(parent: &Project, kids: &[&Project]) -> Vec<(u32, State)> {
    let lookup = |t: &str| kids.iter().find(|k| k.path == t).map(|k| Child::Read(*k));
    contracts_of(parent, Civil::new(2020,1,1), &lookup).into_iter().map(|c| (c.line, c.state)).collect()
}

const C: &str = "# C\n\n- Owner: @x\n- Commitment: 2027-06-30\n\n- [ ] 2027-06-30 Thing  ^m\n- [x] 2027-01-31 Early  ^e\n- [ ] 2027-03-31 Dropped  ^d\n";

#[test]
fn project_contract_states_are_live() {
    let c = p("c.md", C);
    let a = |by: &str| p("a.md", &format!("# A\n\n- [ ] [C](c.md)  by: {}\n", by));
    assert_eq!(states(&a("2027-09-30"), &[&c])[0].1, State::Agreed);
    assert_eq!(states(&a("2027-03-31"), &[&c])[0].1, State::Disputed);
    // the child edits its promise: the next read sees the new state, nothing was stored
    let moved = p("c.md", &C.replace("Commitment: 2027-06-30", "Commitment: 2027-02-28"));
    assert_eq!(states(&a("2027-03-31"), &[&moved])[0].1, State::Agreed);
    let none = p("c.md", "# C\n\n- Owner: @x\n- [ ] Todo\n");
    assert_eq!(states(&a("2027-03-31"), &[&none])[0].1, State::Requested);
    let done = p("c.md", &C.replace("- Commitment", "- Done: 2027-01-05\n- Commitment"));
    assert_eq!(states(&a("2027-03-31"), &[&done])[0].1, State::Delivered);
}

#[test]
fn deliverable_contract_states_are_live() {
    let c = p("c.md", C);
    let a = p("a.md", "# A\n\n- [ ] [M](c.md#m)  by: 2027-09-30\n- [ ] [M2](c.md#m)  by: 2027-03-31\n- [ ] [E](c.md#e)  by: 2027-03-31\n- [ ] [D](c.md#gone)  by: 2027-03-31\n");
    let s: Vec<State> = states(&a, &[&c]).into_iter().map(|x| x.1).collect();
    assert_eq!(s, [State::Agreed, State::Disputed, State::Delivered, State::Disputed]);
}

#[test]
fn a_link_without_by_has_no_contract() {
    let c = p("c.md", C);
    let a = p("a.md", "# A\n\n- [ ] [C](c.md)\n- [ ] [M](c.md#m)\n");
    assert!(states(&a, &[&c]).is_empty());
}

#[test]
fn unreadable_remote_link_is_unknown_and_warns_only_with_by() {
    let a = p("a.md", "# A\n\n- [ ] [R](https://x.example.org/r.md)  by: 2027-06-30\n- [ ] [S](https://x.example.org/s.md)\n");
    let lookup = |t: &str| if is_remote(t) { Some(Child::Unknown) } else { None };
    let d = contract_diags(&a, Civil::new(2020,1,1), &lookup);
    assert_eq!(d.len(), 1);
    assert_eq!((d[0].code, d[0].line), ("W-UNKNOWN", 3));
}

#[test]
fn unknown_child_is_scheduled_at_the_lines_own_by_or_estimate() {
    let mut a = p("a.md", "# A\n\n- [ ] [R](https://x.example.org/r.md)  by: 2027-06-30\n- [ ] [S](https://x.example.org/s.md)  2027-02-01 .. 2027-03-15\n");
    schedule(&mut a, Civil::new(2026, 11, 15), &Interfaces::new());
    assert_eq!(a.nodes[0].forecast.unwrap().end, Some(Civil::new(2027, 6, 30)));
    assert_eq!(a.nodes[1].forecast.unwrap().end.unwrap().to_string(), "2027-03-15".to_string());
}

// ---- front-file pattern (FORMAT §8.1) ---------------------------------------------------------

const FRONT: &str = "# Enclosures\n\n- Owner: M <m@example.com>\n- Commitment: 2028-03-15\n\n\
- [ ] 2027-06-30 Pilot delivered  after: ^pilot  ^pilot-ready\n\
- [ ] 2028-03-15 Series qualified  after: ^series  ^series-ready\n\n\
- [ ] [Pilot build](private/plan.md#pilot)  ^pilot\n\
- [ ] [Series validation](private/plan.md#series)  ^series\n";
const PLAN: &str = "# Enclosures plan\n\n- Owner: M <m@example.com>\n\n- [ ] Tooling  10w  ^tooling\n- [ ] 2027-06-20 Pilot  after: ^tooling  ^pilot\n- [ ] 2028-03-20 Series  ^series\n";

#[test]
fn parent_reads_only_the_front_and_sees_no_error_for_the_private_plan() {
    let front = p("teams/enc/enc.md", FRONT);
    let parent = p("units/hw.md", "# HW\n\n- Owner: R <r@example.com>\n\n- [ ] [Enclosures](../teams/enc/enc.md)  by: 2028-03-31\n");
    let mut g = Graph::new("units/hw.md");
    g.add(parent.clone());
    g.add(front.clone());
    // the plan was never read; the parent's graph has no reference to it
    let own: Vec<_> = g.check().into_iter().filter(|d| d.path == "units/hw.md").collect();
    assert!(own.is_empty(), "{:?}", own);
    assert_eq!(states(&parent, &[&front])[0].1, State::Agreed);
    let late = p("units/hw.md", &"# HW\n\n- [ ] [Enclosures](../teams/enc/enc.md)  by: 2028-02-28\n");
    assert_eq!(states(&late, &[&front])[0].1, State::Disputed);
}

#[test]
fn owner_with_access_sees_the_plan_contradict_the_front() {
    let mut front = p("enc.md", FRONT.replace("private/plan.md", "plan.md").as_str());
    let plan = p("plan.md", PLAN);
    let mut ifs = Interfaces::new();
    ifs.insert("plan.md".into(), Interface::from_project(&plan));
    schedule(&mut front, Civil::new(2026, 11, 15), &ifs);
    let conflicts: Vec<_> = front.diagnostics.iter().filter(|d| d.code == "E-CONFLICT").collect();
    // the plan's series milestone (2028-03-20) is after the front promise (2028-03-15)
    assert!(conflicts.iter().any(|d| d.message.contains("Series qualified")), "{:?}", front.diagnostics);
    assert!(!conflicts.iter().any(|d| d.message.contains("Pilot delivered")), "pilot is on time");
}

#[test]
fn r8a_need_by_passed_and_not_delivered_is_overdue() {
    let c = p("c.md", C);
    let a = p("a.md", "# A\n\n- [ ] [C](c.md)  by: 2027-09-30\n- [ ] [M](c.md#m)  by: 2027-03-31\n- [ ] [E](c.md#e)  by: 2027-03-31\n");
    let lookup = |t: &str| if t == "c.md" { Some(Child::Read(&c)) } else { None };
    let s: Vec<State> = contracts_of(&a, Civil::new(2028, 1, 1), &lookup).into_iter().map(|x| x.state).collect();
    assert_eq!(s, [State::Overdue, State::Overdue, State::Delivered]);
    let d = contract_diags(&a, Civil::new(2028, 1, 1), &lookup);
    assert_eq!(d[0].code, "W-OVERDUE");
    assert!(d[0].message.contains("is overdue: needed by 2027-09-30, not delivered"), "{}", d[0].message);
}
