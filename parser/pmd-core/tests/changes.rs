use pmd_core::*;

/// Events as `kind|subject|old|new|delta|audience`, from two versions of `parent.md`.
fn ev(old: &str, new: &str) -> Vec<String> {
    let (a, b) = (parse("parent.md", old), parse("parent.md", new));
    changes(&a, &b)
        .into_iter()
        .map(|c| {
            let d = c.delta_days.map(|d| format!("{:+}", d)).unwrap_or_default();
            format!("{}|{}|{}|{}|{}|{}", c.kind, c.subject, c.old.unwrap_or_default(), c.new.unwrap_or_default(), d, c.audience)
        })
        .collect()
}

const OLD: &str = "# Parent\n\n- Owner: Pat\n- Commitment: 2027-03-01\n\n- [ ] [Impl](child.md)  by: 2027-01-20  ^impl\n- [ ] [QA](qa.md#ready)  by: 2027-01-26  ^qa\n- [ ] 2027-02-10 Ready for launch  ^ready\n- [ ] Tidy up  ^tidy\n";

#[test]
fn identical_versions_have_no_events() {
    assert!(ev(OLD, OLD).is_empty());
}

#[test]
fn every_kind_in_new_line_order() {
    let new = "# Parent\n\n- Owner: Sam\n- Commitment: 2027-03-08\n\n- [ ] [Impl](child.md)  by: 2027-01-27  ^impl\n- [ ] [QA](qa.md#ready)  by: 2027-02-02  ^qa\n- [ ] 2027-02-17 Ready for launch  ^ready\n- [x] Tidy up  ^tidy  done: 2027-01-10\n- [ ] [Docs](docs.md)  by: 2027-02-01  ^docs\n";
    assert_eq!(
        ev(OLD, new),
        [
            "promise moved|Commitment|2027-03-01|2027-03-08|+7|parents",
            "owner changed|Owner|Pat|Sam||self",
            "ask moved|child.md|by: 2027-01-20|by: 2027-01-27|+7|child child.md",
            "ask moved|qa.md#ready|by: 2027-01-26|by: 2027-02-02|+7|child qa.md",
            "promise moved|^ready Ready for launch|2027-02-10|2027-02-17|+7|parents",
            "own done|^tidy Tidy up||2027-01-10||self",
            "ask added|docs.md||by: 2027-02-01||child docs.md",
        ]
    );
}

#[test]
fn promise_added_removed_done_cancelled() {
    let new = "# Parent\n\n- Commitment: 2027-03-01\n\n- [ ] [Impl](child.md)  by: 2027-01-20  ^impl\n- [ ] 2027-01-30 Ship  ^rc\n- [-] 2027-02-10 Ready for launch  ^ready\n";
    let old = "# Parent\n\n- Commitment: 2027-03-01\n\n- [ ] [Impl](child.md)  by: 2027-01-20  ^impl\n- [ ] 2027-02-10 Ready for launch  ^ready\n- [ ] 2027-01-20 Gone  ^gone\n";
    assert_eq!(
        ev(old, new),
        [
            "promise added|^rc Ship||2027-01-30||parents",
            "promise cancelled|^ready Ready for launch|2027-02-10|||parents",
            "promise removed|^gone Gone|2027-01-20|||parents",
        ]
    );
}

#[test]
fn milestone_ticked_done_with_its_done_date() {
    let old = "# P\n\n- [ ] 2027-02-10 Ready  ^ready\n";
    let new = "# P\n\n- [x] 2027-02-10 Ready  ^ready  done: 2027-02-08\n";
    assert_eq!(ev(old, new), ["promise done|^ready Ready||2027-02-08||parents"]);
}

#[test]
fn link_added_and_removed_and_by_added() {
    let old = "# P\n\n- [ ] [Impl](child.md#a)  ^impl\n- [ ] [Old](gone.md)  by: 2027-01-05  ^old\n";
    let new = "# P\n\n- [ ] [Impl](child.md#a)  by: 2027-01-20  ^impl\n- [ ] [New](new.md#b)  by: 2027-02-01  ^new\n";
    assert_eq!(
        ev(old, new),
        [
            "ask added|child.md#a||by: 2027-01-20||child child.md",
            "ask added|new.md#b||by: 2027-02-01||child new.md",
            "ask removed|gone.md|by: 2027-01-05|||child gone.md",
        ]
    );
}

#[test]
fn own_item_cancelled_and_owner_changed() {
    let old = "# P\n\n- Owner: Pat\n\n- [ ] Draft  ^draft\n- [ ] Review\n";
    let new = "# P\n\n- Owner: Ann\n\n- [-] Draft  ^draft\n- [x] Review\n";
    assert_eq!(ev(old, new), ["owner changed|Owner|Pat|Ann||self", "own cancelled|^draft Draft||||self", "own done|Review||||self"]);
}

#[test]
fn own_items_match_by_title_without_id() {
    let old = "# P\n\n- [ ] Write spec\n";
    let new = "# P\n\n- [x] Write spec  done: 2027-01-09\n";
    assert_eq!(ev(old, new), ["own done|Write spec||2027-01-09||self"]);
}
