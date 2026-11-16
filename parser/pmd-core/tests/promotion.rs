//! Promotion rule (D47, examples/README.md): replacing an item's title with a link to a new
//! file keeps its checkbox, `^id`, `owner:`, `by:` and estimate; nothing else changes.
use pmd_core::*;

const HEAD: &str = "# Interior\n\n- Owner: Mia Berg <mia@example.com>\n\n- [ ] 2027-01-15 Styling frozen  ^styling\n";
const BEFORE: &str = "- [ ] Seat concept  6w  after: ^styling  owner: @mia  by: 2027-09-30  ^seats\n- [ ] Seat validation  2w  after: ^seats\n";
const AFTER: &str = "- [ ] [Seat concept](seats.md)  6w  after: ^styling  owner: @mia  by: 2027-09-30  ^seats\n- [ ] Seat validation  2w  after: ^seats\n";
const CHILD_NO_COMMITMENT: &str = "# Seats\n\n- Owner: Mia Berg <mia@example.com>\n\n- [ ] Frame design  3w\n";
const CHILD_PROMISED: &str = "# Seats\n\n- Owner: Mia Berg <mia@example.com>\n- Commitment: 2027-09-30\n\n- [ ] Frame design  3w\n";

fn plan(body: &str) -> Project {
    parse("interior.md", &format!("{}{}", HEAD, body))
}

#[test]
fn both_forms_parse_and_keep_checkbox_id_owner_by_estimate() {
    let (b, a) = (plan(BEFORE), plan(AFTER));
    let (nb, na) = (b.find_id("seats").unwrap(), a.find_id("seats").unwrap());
    assert_eq!(nb.kind, NodeKind::Task);
    assert_eq!(na.kind, NodeKind::Subproject);
    assert_eq!(na.title, "Seat concept");
    for (x, y) in [(nb, na)] {
        assert_eq!(x.checkbox, y.checkbox);
        assert_eq!(x.duration, y.duration);
        assert_eq!(x.keys["owner"][0].text, y.keys["owner"][0].text);
        assert_eq!(x.keys["by"][0].text, y.keys["by"][0].text);
        assert_eq!(x.keys["after"][0].text, y.keys["after"][0].text);
    }
    assert!(a.diagnostics.is_empty(), "{:?}", a.diagnostics);
}

#[test]
fn local_after_still_resolves_and_the_estimate_schedules_until_the_child_promises() {
    let mut ids = Interfaces::new();
    let mut before = plan(BEFORE);
    let want = {
        schedule(&mut before, Civil::new(2026, 11, 15), &ids);
        let f = before.nodes[1].forecast.unwrap();
        (f.start, f.end, before.nodes[2].forecast.unwrap().start)
    };
    // after promotion, with a child that has no Commitment: the same dates
    ids.insert("seats.md".into(), Interface::default());
    let mut after = plan(AFTER);
    schedule(&mut after, Civil::new(2026, 11, 15), &ids);
    let f = after.nodes[1].forecast.unwrap();
    assert_eq!((f.start, f.end, after.nodes[2].forecast.unwrap().start), want);
    assert_eq!(f.start, Some(Civil::new(2027, 1, 16)), "after: ^styling, the day after");
    assert!(after.diagnostics.iter().all(|d| d.code != "E-REF-LOCAL"));
    // once the child promises, the promise is the end; the dependent follows it
    ids.insert("seats.md".into(), Interface::from_texts(Some("2027-10-31"), []));
    let mut promised = plan(AFTER);
    schedule(&mut promised, Civil::new(2026, 11, 15), &ids);
    assert_eq!(promised.nodes[1].forecast.unwrap().end, Some(Civil::new(2027, 10, 31)));
    assert_eq!(promised.nodes[2].forecast.unwrap().start, Some(Civil::new(2027, 11, 1)));
}

#[test]
fn the_new_line_has_no_status_without_by_and_requested_with_one() {
    let st = |parent: &str, child: &str| {
        let pr = parse("interior.md", &format!("{}{}", HEAD, parent));
        let ch = parse("seats.md", child);
        let lookup = |t: &str| if t == "seats.md" { Some(Child::Read(&ch)) } else { None };
        contracts_of(&pr, Civil::new(2020,1,1), &lookup).into_iter().map(|c| c.state).collect::<Vec<_>>()
    };
    // promoted line with `by:`: requested until the child promises, then agreed
    assert_eq!(st(AFTER, CHILD_NO_COMMITMENT), [State::Requested]);
    assert_eq!(st(AFTER, CHILD_PROMISED), [State::Agreed]);
    // the same line without `by:` has no contract and gets no status
    let no_by = AFTER.replace("  by: 2027-09-30", "");
    assert!(st(&no_by, CHILD_NO_COMMITMENT).is_empty());
    assert!(st(&no_by, CHILD_PROMISED).is_empty());
}
