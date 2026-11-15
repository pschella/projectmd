//! Lock construction (FORMAT §9, GRAMMAR §8): one entry per direct link, `by` and `late`.
use pmd_core::*;
use std::collections::HashMap;

fn today() -> Civil {
    Civil::new(2026, 11, 15)
}

/// Graph holding the given files (path, source), each with its content hash. The first file is
/// the one whose lock is built; files left out of the graph are unreadable.
fn graph_of(files: &[(&str, &str)]) -> Graph {
    let mut g = Graph::new(files[0].0);
    for (path, src) in files {
        let p = parse(path, src, &Options::default());
        g.add_hashed(p, hash_prefix(src.as_bytes()));
    }
    g
}

fn locs(l: &Lock) -> Vec<&str> {
    l.entries.iter().map(|e| e.location.as_str()).collect()
}

fn one(path: &str, lock: Lock) -> HashMap<String, Lock> {
    HashMap::from([(path.to_string(), lock)])
}

#[test]
fn lock_lists_direct_links_only() {
    // d.md is linked by b.md, not by a.md: it must not appear, and must not be needed.
    let g = graph_of(&[
        ("r/a.md", "# A\n\n- [ ] [B](b.md)\n- [ ] [C](c.md)\n"),
        ("r/b.md", "# B\n\n- ID: ex/b\n- [ ] [D](d.md)\n"),
        ("r/c.md", "# C\n\n- ID: ex/c\n"),
    ]);
    let child = one("r/b.md", Lock::parse("d.md  id=ex/d  hash=dddd  seen=2026-01-01\n"));
    let l = build_lock("r/a.md", &g, &child, None, today());
    assert_eq!(locs(&l), ["b.md", "c.md"]);
    assert_eq!(l.entry("b.md").unwrap().field("id"), Some("ex/b"));
    assert_eq!(l.entry("b.md").unwrap().field("seen"), Some("2026-11-15"));
    assert_eq!(l.comments, ["# a.lock — tool-generated snapshot; rebuild with `pmd lock a.md`."]);
}

#[test]
fn by_is_captured_from_the_link_item() {
    let g = graph_of(&[
        ("a.md", "# A\n\n- [ ] 2027-05-01 Gate  ^gate\n- [ ] [B](b.md)  by: 2027-09-30\n- [ ] [C](c.md)  by: ^gate\n- [ ] [D](d.md)\n"),
        ("b.md", "# B\n"),
        ("c.md", "# C\n"),
        ("d.md", "# D\n"),
    ]);
    let l = build_lock("a.md", &g, &HashMap::new(), None, today());
    assert_eq!(l.entry("b.md").unwrap().field("by"), Some("2027-09-30"));
    assert_eq!(l.entry("c.md").unwrap().field("by"), Some("2027-05-01"), "^id resolves to the milestone's date");
    assert_eq!(l.entry("d.md").unwrap().field("by"), None);
}

#[test]
fn lateness_propagates_bottom_up_only_through_locks() {
    // C forecasts 2027-10-31 against Commitment 2027-06-30: late.
    // B's Commitment 2027-09-30 is then missed through C: B is late.
    // A's need-by for B is met on paper, but B's lateness is visible in A through B.lock only.
    let c = "# C\n\n- Commitment: 2027-06-30\n\n- [ ] Big task  2027-10-31\n";
    let b = "# B\n\n- Commitment: 2027-09-30\n\n- [ ] [C](c.md)  by: 2027-06-30\n";
    let a = "# A\n\n- Commitment: 2028-03-31\n\n- [ ] [B](b.md)  by: 2027-12-31\n";
    let gc = graph_of(&[("r/c.md", c)]);
    let c_lock = build_lock("r/c.md", &gc, &HashMap::new(), None, today());
    assert_eq!(c_lock.entries.len(), 0, "C has no links");

    // B refreshed with C's lock: C itself is scheduled against C.lock, so C's own date counts.
    let gb = graph_of(&[("r/b.md", b), ("r/c.md", c)]);
    let b_lock = build_lock("r/b.md", &gb, &one("r/c.md", c_lock.clone()), None, today());
    let ce = b_lock.entry("c.md").unwrap();
    assert_eq!(ce.field("late"), Some("2027-10-31"));
    assert_eq!(ce.field("by"), Some("2027-06-30"));
    assert_eq!(ce.field("commitment"), Some("2027-06-30"));

    // A refreshed with B's fresh lock: B is late, A sees it.
    let ga = graph_of(&[("r/a.md", a), ("r/b.md", b)]);
    let al = build_lock("r/a.md", &ga, &one("r/b.md", b_lock.clone()), None, today());
    assert_eq!(al.entry("b.md").unwrap().field("late"), Some("2027-10-31"));
    assert_eq!(al.entry("b.md").unwrap().field("by"), Some("2027-12-31"));

    // A refreshed against a stale B.lock (written before C slipped): no late.
    let stale = Lock::parse("c.md  id=ex/c  commitment=2027-06-30  seen=2026-11-01\n");
    let al = build_lock("r/a.md", &ga, &one("r/b.md", stale), None, today());
    assert_eq!(al.entry("b.md").unwrap().field("late"), None);
}

#[test]
fn grandchild_md_is_never_read() {
    // b.md links c.md, which is absent from the graph: only b.md's lock may describe it.
    let g = graph_of(&[("a.md", "# A\n\n- [ ] [B](b.md)\n"), ("b.md", "# B\n\n- [ ] [C](c.md)\n")]);
    let b_lock = Lock::parse("c.md  id=ex/c  commitment=2027-06-30  late=2027-10-31  seen=2026-11-01\n");
    let l = build_lock("a.md", &g, &one("b.md", b_lock), None, today());
    assert_eq!(locs(&l), ["b.md"]);
}

#[test]
fn remote_links_get_location_by_and_seen_only() {
    let g = graph_of(&[("root.md", "# Root\n\n- [ ] [R](https://h.example/r.md)  by: 2027-09-30\n")]);
    let l = build_lock("root.md", &g, &HashMap::new(), None, today());
    assert_eq!(locs(&l), ["https://h.example/r.md"]);
    assert_eq!(
        l.entries[0].fields,
        [("by".to_string(), "2027-09-30".to_string()), ("seen".to_string(), "2026-11-15".to_string())]
    );
}

#[test]
fn unreadable_direct_link_keeps_its_previous_entry() {
    let g = graph_of(&[("root.md", "# Root\n\n- [ ] [G](gone.md)\n- [ ] [B](b.md)\n"), ("b.md", "# B\n\n- ID: ex/b\n")]);
    let prev = Lock::parse("gone.md  id=ex/gone  hash=old  seen=2026-01-01\nb.md  id=ex/b  hash=zzz  seen=2026-03-01\n");
    let l = build_lock("root.md", &g, &HashMap::new(), Some(&prev), today());
    assert_eq!(locs(&l), ["b.md", "gone.md"]);
    assert_eq!(l.entry("gone.md").unwrap().field("hash"), Some("old"));
    assert_eq!(l.entry("gone.md").unwrap().field("seen"), Some("2026-01-01"));
    assert_ne!(l.entry("b.md").unwrap().field("hash"), Some("zzz"), "a readable link is read fresh");
}

#[test]
fn late_is_honoured_by_scheduling_from_the_file_own_lock() {
    // A.md's own lock says C ends late: B (Commitment 2027-09-30, by 2027-06-30 on C) is forecast late.
    let b = "# B\n\n- Commitment: 2027-09-30\n\n- [ ] [C](c.md)  by: 2027-06-30\n";
    let own = Interfaces::from([
        ("c.md".to_string(), Interface::from_lock_entry(&Lock::parse("c.md  commitment=2027-06-30  late=2027-10-31\n").entries[0])),
    ]);
    let mut p = parse("b.md", b, &Options::default());
    schedule(&mut p, today(), &own);
    assert_eq!(p.forecast.as_ref().unwrap().end, Some(Civil::new(2027, 10, 31)));
    assert!(p.forecast.as_ref().unwrap().conflict);
}
