use pmd_core::*;

#[test]
fn fewer_than_three_finished_promises_has_no_record() {
    let p = parse("c.md", "# C\n\n- [x] 2027-01-10 A  done: 2027-01-10  ^a\n- [x] 2027-01-20 B  done: 2027-01-26  ^b\n- [ ] 2027-01-30 C  ^c\n");
    assert_eq!(promise_record(&p), None);
}

#[test]
fn promise_record_counts_kept_and_median_of_the_late() {
    // A kept on time, B late by 6, C kept (early), D late by 7 -> 2 of 4 kept, median late (6+7)/2
    let p = parse(
        "c.md",
        "# C\n\n- [x] 2027-01-10 A  done: 2027-01-10  ^a\n- [x] 2027-01-20 B  done: 2027-01-26  ^b\n- [x] 2027-01-30 C  done: 2027-01-28  ^c\n- [x] 2027-02-01 D  done: 2027-02-08  ^d\n- [ ] 2027-02-10 E  ^e\n",
    );
    let r = promise_record(&p).unwrap();
    assert_eq!((r.total, r.kept, r.median_late), (4, 2, Some(6)));
    assert_eq!(promise_text(&r), "record 2/4, late +6d");
}

#[test]
fn all_kept_has_no_lateness() {
    let p = parse("c.md", "# C\n\n- [x] 2027-01-10 A  done: 2027-01-09  ^a\n- [x] 2027-01-20 B  done: 2027-01-20  ^b\n- [x] 2027-01-30 C  done: 2027-01-29  ^c\n");
    let r = promise_record(&p).unwrap();
    assert_eq!((r.total, r.kept, r.median_late), (3, 3, None));
    assert_eq!(promise_text(&r), "record 3/3");
}

#[test]
fn estimate_record_is_the_median_ratio_of_finished_items() {
    // ratios in weekdays (D71): A 2/4 = 0.5, B 6/4 = 1.5, C 4/2 = 2.0 -> median 1.5
    let p = parse(
        "c.md",
        "# C\n\n- [x] Task A  4d  after: 2027-01-01  done: 2027-01-04  ^a\n- [x] Task B  4d  after: 2027-01-01  done: 2027-01-08  ^b\n- [x] Task C  2d  after: 2027-01-10  done: 2027-01-14  ^c\n- [ ] Task D  2d  after: 2027-01-10  ^d\n",
    );
    let r = estimate_record(&p).unwrap();
    assert_eq!(r.n, 3);
    assert!((r.median_ratio - 1.5).abs() < 1e-9);
}

#[test]
fn estimate_record_needs_three_items_with_a_start_and_a_done_date() {
    let p = parse("c.md", "# C\n\n- [x] Task A  4d  after: 2027-01-01  done: 2027-01-04  ^a\n- [x] Task B  4d  done: 2027-01-08\n- [x] Task C  2d  after: 2027-01-10  done: 2027-01-14  ^c\n");
    assert_eq!(estimate_record(&p), None);
}
