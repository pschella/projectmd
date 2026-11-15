mod common;
use common::*;
use pmd_core::*;

fn codes(p: &Project) -> Vec<&'static str> {
    p.diagnostics.iter().map(|d| d.code).collect()
}

// ---- atomisation + suffix: every example row of GRAMMAR §4.3 --------------------------------

#[test]
fn suffix_examples_section_4_3() {
    // (text, title, suffix set)
    let rows: &[(&str, &str, &[&str])] = &[
        ("Fix issue: retry logic  2w", "Fix issue: retry logic", &["duration=2w"]),
        ("Test 12m cable harness  3d  owner: @jan", "Test 12m cable harness", &["duration=3d", "owner=@jan"]),
        ("Review #3 comments  1w  ^review", "Review #3 comments", &["duration=1w", "id=review"]),
        (
            "Tooling release  #public ^tr  owner: Marco Rossi <m@example.com>",
            "Tooling release",
            &["tag=public", "id=tr", "owner=Marco Rossi <m@example.com>"],
        ),
        ("Seat concept  2027-Q1 .. 2027-Q2", "Seat concept", &["range=2027-Q1..2027-Q2"]),
        ("2w", "2w", &[]),
        ("Ship it  after: ^a  after: ^b", "Ship it", &["after=^a", "after=^b"]),
        ("Explain `after: ^x` syntax", "Explain `after: ^x` syntax", &[]),
    ];
    for (text, title, sfx) in rows {
        let n = item(&format!("- [ ] {}", text)).unwrap();
        assert_eq!(n.title, *title, "title of {:?}", text);
        let mut want: Vec<String> = sfx.iter().map(|s| s.to_string()).collect();
        want.sort();
        assert_eq!(suffix_set(&n), want, "suffix of {:?}", text);
    }
}

#[test]
fn repeated_keys_keep_order() {
    let n = item("- [ ] Ship it  after: ^a  after: ^b  after: ^c").unwrap();
    let v: Vec<&str> = n.keys["after"].iter().map(|v| v.text.as_str()).collect();
    assert_eq!(v, ["^a", "^b", "^c"]);
}

#[test]
fn key_cannot_be_first_atom() {
    let n = item("- [ ] after: ^a").unwrap();
    assert_eq!(n.title, "after:");
    assert_eq!(n.id.as_deref(), Some("a"));
    assert!(n.keys.is_empty());
    let n = item("- [ ] after: @jan").unwrap();
    assert_eq!(n.title, "after: @jan");
}

#[test]
fn unknown_key_stays_in_title() {
    let n = item("- [ ] Do thing  ticket: PACK-1").unwrap();
    assert_eq!(n.title, "Do thing  ticket: PACK-1");
}

#[test]
fn key_values_are_type_checked() {
    let n = item("- [ ] Plan  by: Friday").unwrap();
    assert_eq!(n.title, "Plan  by: Friday");
    let n = item("- [ ] Plan  owner: Jan Doe").unwrap();
    assert_eq!(n.title, "Plan  owner: Jan Doe");
    let n = item("- [ ] Plan  after: pack.md#x").unwrap();
    assert_eq!(n.title, "Plan  after: pack.md#x");
    let n = item("- [ ] Plan  after: 2w").unwrap();
    assert_eq!(n.title, "Plan  after: 2w".replace("  after: 2w", "  after:")); // 2w is a duration, after: stays
    let n = item("- [ ] Plan  owner: @jan  by: 2027-Q2  after: ^n").unwrap();
    assert_eq!(n.title, "Plan");
    assert_eq!(n.keys.len(), 3);
}

#[test]
fn values_person_link_range_word() {
    let n = item("- [ ] X  owner: Priya Nair <p@example.com>").unwrap();
    let v = &n.keys["owner"][0];
    assert_eq!(v.kind, ValueKind::Person);
    assert_eq!(v.name.as_deref(), Some("Priya Nair"));
    assert_eq!(v.email.as_deref(), Some("p@example.com"));
    let n = item("- [ ] X  by: 2027-Q1 .. 2027-Q2").unwrap();
    assert_eq!(n.keys["by"][0].kind, ValueKind::Range);
    let n = item("- [ ] X  by: 2027-Q1..2027-Q2").unwrap();
    let v = &n.keys["by"][0];
    assert_eq!(v.kind, ValueKind::Range);
    assert_eq!(v.start.unwrap().to_string(), "2027-01-01");
    assert_eq!(v.end.unwrap().to_string(), "2027-06-30");
}

#[test]
fn atom_forms() {
    // inline link with title, spaces in text
    let p = parse_str("# T\n\n- [ ] [My plan](a/b.md \"hello there\")  by: 2027-01\n");
    let n = &p.nodes[0];
    assert_eq!(n.kind, NodeKind::Subproject);
    let l = n.link.as_ref().unwrap();
    assert_eq!((l.text.as_str(), l.target.as_str(), l.title.as_deref()), ("My plan", "a/b.md", Some("hello there")));
    // a word containing [ and ( without link
    let n = item("- [ ] Handle [x (y) z  2w").unwrap();
    assert_eq!(n.title, "Handle [x (y) z");
    // unclosed code swallows the rest
    let n = item("- [ ] Use `after: ^x  2w").unwrap();
    assert_eq!(n.title, "Use `after: ^x  2w");
    // email atom
    let n = item("- [ ] Mail <a@b.c> now  3d").unwrap();
    assert_eq!(n.title, "Mail <a@b.c> now");
    // '<' not an email
    let n = item("- [ ] a <b> c  3d").unwrap();
    assert_eq!(n.title, "a <b> c");
}

#[test]
fn reference_style_links() {
    let src = "# T\n\n- [ ] [Inverter][inv]  by: 2027-01\n- [ ] X  after: [Pack][]\n- [ ] Y  after: [Nope][zz]\n\n[inv]: https://x.example/inv.md\n[Pack]: pack.md#v \"The pack\"\n";
    let p = parse_str(src);
    assert_eq!(p.refdefs["inv"].target, "https://x.example/inv.md");
    assert_eq!(p.refdefs["pack"].title.as_deref(), Some("The pack"));
    assert_eq!(p.nodes[0].kind, NodeKind::Subproject);
    assert_eq!(p.nodes[0].link.as_ref().unwrap().target, "https://x.example/inv.md");
    // D47: a link is no `after:` value; it stays in the title
    assert_eq!(p.nodes[1].title, "X  after: [Pack][]");
    // unresolved reference is not a link either
    assert_eq!(p.nodes[2].title, "Y  after: [Nope][zz]");
    assert_eq!(p.nodes[2].kind, NodeKind::Task);
}

// ---- checkbox / leading date / kind -------------------------------------------------------

#[test]
fn checkbox_classification() {
    let cases = [
        ("- [ ] a", Checkbox::Open),
        ("- [x] a", Checkbox::Done),
        ("- [X] a", Checkbox::Done),
        ("- [-] a", Checkbox::Cancelled),
        ("* [ ] a", Checkbox::Open),
        ("+ [ ] a", Checkbox::Open),
    ];
    for (l, c) in cases {
        assert_eq!(item(l).unwrap().checkbox, c, "{}", l);
    }
    assert!(item("- [y] a").is_none());
    assert!(item("- [ ]a").is_none());
    assert!(item("- plain prose").is_none());
}

#[test]
fn leading_date_and_kinds() {
    let n = item("- [ ] 2027-01-29 Concept approval  ^ca").unwrap();
    assert_eq!(n.kind, NodeKind::Milestone);
    assert_eq!(n.date.as_ref().unwrap().text, "2027-01-29");
    assert_eq!(n.title, "Concept approval");
    let n = item("- 2027-06-30 Pack sign-off").unwrap();
    assert_eq!((n.kind, n.checkbox), (NodeKind::Milestone, Checkbox::None));
    // date alone is not a leading date
    let n = item("- [ ] 2027-06-30").unwrap();
    assert_eq!((n.kind, n.title.as_str()), (NodeKind::Task, "2027-06-30"));
    assert!(item("- 2027-06-30").is_none());
    // suffix date: task
    let n = item("- [ ] Release 2027-06").unwrap();
    assert_eq!(n.kind, NodeKind::Task);
    assert_eq!(n.date.as_ref().unwrap().text, "2027-06");
    // forms
    for d in ["2027-06", "2027-Q3"] {
        let n = item(&format!("- [ ] {} Beta", d)).unwrap();
        assert_eq!(n.kind, NodeKind::Milestone, "{}", d);
    }
}

#[test]
fn leading_range_makes_task_with_range() {
    for l in ["- [ ] 2026-01 .. 2026-03 Kickoff", "- [ ] 2026-01..2026-03 Kickoff", "- 2026-01..2026-03 Kickoff"] {
        let n = item(l).unwrap();
        assert_eq!(n.kind, NodeKind::Task, "{}", l);
        assert_eq!(n.title, "Kickoff");
        let r = n.range.as_ref().unwrap();
        assert_eq!(r.start.to_string(), "2026-01-01");
        assert_eq!(r.end.to_string(), "2026-03-31");
        assert!(n.date.is_none());
    }
}

#[test]
fn subproject_detection() {
    let k = |l: &str| item(l).unwrap().kind;
    assert_eq!(k("- [ ] [Pwr](powertrain/p.md)"), NodeKind::Subproject);
    assert_eq!(k("- [ ] [Pwr](powertrain/P.MD)  by: 2027-01"), NodeKind::Subproject);
    assert_eq!(k("- [ ] [Pwr](https://x.example/p.md?v=2)"), NodeKind::Subproject);
    assert_eq!(k("- [ ] [Pwr](p.md#frag)"), NodeKind::Deliverable);
    assert_eq!(k("- [ ] [Pwr](https://x.example/p.md?v=2#frag)"), NodeKind::Deliverable);
    assert_eq!(k("- [ ] [Pwr](#frag)"), NodeKind::Task);
    assert_eq!(k("- [ ] [Pwr](p.txt#frag)"), NodeKind::Task);
    assert_eq!(k("- [ ] [Pwr](p.txt)"), NodeKind::Task);
    assert_eq!(k("- [ ] See [Pwr](p.md)"), NodeKind::Task);
    assert_eq!(k("- 2027-01 [Pwr](p.md)"), NodeKind::Milestone);
}

// ---- dates §5.1 ----------------------------------------------------------------------------

fn d(s: &str) -> String {
    match parse_date(s) {
        DateParse::Ok(a, b) => format!("{}..{}", a, b),
        DateParse::Invalid => "error".into(),
        DateParse::NotDate => "not".into(),
    }
}

#[test]
fn dates_all_forms() {
    assert_eq!(d("2027-06-30"), "2027-06-30..2027-06-30");
    assert_eq!(d("2028-02-29"), "2028-02-29..2028-02-29");
    assert_eq!(d("2000-02-29"), "2000-02-29..2000-02-29");
    for bad in ["2027-02-29", "2100-02-29", "2027-02-30", "2027-04-31", "2027-06-00", "2027-00-10", "2027-13-01", "2027-00", "2027-13"] {
        assert_eq!(d(bad), "error", "{}", bad);
    }
    assert_eq!(d("2028-02"), "2028-02-01..2028-02-29");
    assert_eq!(d("2027-02"), "2027-02-01..2027-02-28");
    assert_eq!(d("2027-Q1"), "2027-01-01..2027-03-31");
    assert_eq!(d("2027-Q2"), "2027-04-01..2027-06-30");
    assert_eq!(d("2027-Q3"), "2027-07-01..2027-09-30");
    assert_eq!(d("2027-Q4"), "2027-10-01..2027-12-31");
    for not in ["2027-Q5", "2027-Q0", "2027-6-30", "27-06-30", "2027/06/30", "abc", "2027-06-3", "2027-0630"] {
        assert_eq!(d(not), "not", "{}", not);
    }
}

#[test]
fn civil_days_roundtrip() {
    for z in [-800000i64, -1, 0, 1, 11016, 19000, 2932896] {
        assert_eq!(Civil::from_days(z).to_days(), z);
    }
    assert_eq!(Civil::new(1970, 1, 1).to_days(), 0);
}

#[test]
fn invalid_dates_produce_e_date() {
    let p = parse_str("# T\n\n- [ ] Foo  2027-02-30\n- [ ] 2027-13 Bar\n- [ ] Baz  2027-13 .. 2027-14\n- [ ] Q  2027-Q2 .. 2027-Q1\n- [ ] R  by: 2027-02-30\n");
    let n = p.diagnostics.iter().filter(|d| d.code == "E-DATE").count();
    assert_eq!(n, 5, "{:?}", p.diagnostics);
    assert_eq!(p.nodes[0].title, "Foo");
    assert!(p.nodes[0].date.is_none());
    assert_eq!(p.nodes[1].kind, NodeKind::Milestone);
    assert!(p.nodes[1].date.is_none());
}

#[test]
fn range_expands_start_of_a_end_of_b() {
    let n = item("- [ ] Seat  2027-Q1 .. 2027-05").unwrap();
    let r = n.range.unwrap();
    assert_eq!(r.start.to_string(), "2027-01-01");
    assert_eq!(r.end.to_string(), "2027-05-31");
    assert!(r.range);
}

#[test]
fn durations() {
    let n = item("- [ ] A  2w").unwrap_or_else(|| panic!());
    assert_eq!(n.title, "A");
    assert_eq!(n.duration.as_ref().unwrap().days, 14);
    assert_eq!(item("- [ ] A  1m").unwrap().duration.unwrap().days, 30);
    // 0w, 02w, w are not durations
    assert_eq!(item("- [ ] A  0w").unwrap().title, "A  0w");
    assert_eq!(item("- [ ] A  02w").unwrap().title, "A  02w");
    assert_eq!(item("- [ ] A  w").unwrap().title, "A  w");
}

#[test]
fn tags_and_ids() {
    let n = item("- [ ] A #3 #x_y-z ^9a-b").unwrap();
    assert_eq!(n.title, "A #3");
    assert_eq!(n.tags, ["x_y-z"]);
    assert_eq!(n.id.as_deref(), Some("9a-b"));
    let n = item("- [ ] A ^-bad").unwrap();
    assert_eq!(n.title, "A ^-bad");
}

// ---- property bullets, header, front matter ---------------------------------------------------

#[test]
fn property_bullets() {
    let src = "# T\n\n## Work\n\n- [ ] Task\n  - owner: @jan\n  - done: 2027-01-01\n  - Notes: not a key\n  - after: ^other\n- [ ] Other  ^other\n";
    let p = parse_str(src);
    let t = &p.nodes[0].children[0];
    assert_eq!(t.keys["owner"][0].text, "@jan");
    assert_eq!(t.keys["done"].len(), 1);
    assert_eq!(t.keys["after"][0].text, "^other");
    assert!(t.children.is_empty());
    assert!(!codes(&p).contains(&"E-REF-LOCAL"));
}

#[test]
fn property_bullet_on_heading_and_prose_children_ignored() {
    let src = "# T\n\n## Phase\n- owner: @jan\n- note\n  - [ ] hidden task\n- [ ] Real\n";
    let p = parse_str(src);
    let h = &p.nodes[0];
    assert_eq!(h.kind, NodeKind::Heading);
    assert_eq!(h.keys["owner"][0].text, "@jan");
    assert_eq!(h.children.len(), 1);
    assert_eq!(h.children[0].title, "Real");
}

#[test]
fn repeated_non_repeatable_key_is_error() {
    let p = parse_str("# T\n\n- [ ] A  owner: @a  owner: @b\n- [ ] B  owner: @a\n  - owner: @c\n");
    assert_eq!(p.diagnostics.iter().filter(|d| d.code == "E-SYNTAX").count(), 2);
    assert_eq!(p.nodes[0].keys["owner"].len(), 1);
    assert_eq!(p.nodes[1].keys["owner"][0].text, "@c");
}

#[test]
fn header_property_list() {
    let src = "# Aurora  #big\n\n- Owner: Sanne Visser <s@example.org>\n- Stakeholders: Marco <m@example.org>, @board\n- Commitment: 2028-11-30\n- Phase: concept\n- Start date: 2026-01\n\nProse.\n\n- [ ] Task\n";
    let p = parse_str(src);
    assert_eq!(p.title, "Aurora");
    assert_eq!(p.suffix.tags, ["big"]);
    assert_eq!(p.owner.as_ref().unwrap().name.as_deref(), Some("Sanne Visser"));
    assert_eq!(p.header["stakeholders"], "Marco <m@example.org>, @board");
    assert_eq!(p.commitment.as_ref().unwrap().text, "2028-11-30");
    assert_eq!(p.header["phase"], "concept");
    assert_eq!(p.header["start date"], "2026-01");
    assert_eq!(p.nodes.len(), 1);
    assert!(p.diagnostics.is_empty(), "{:?}", p.diagnostics);
}

#[test]
fn header_followed_directly_by_todo_list() {
    let src = "# T\n\n- Owner: @jan\n- Phase: planned\n- [ ] First task\n- [ ] Second\n";
    let p = parse_str(src);
    assert_eq!(p.header.len(), 2);
    assert_eq!(p.nodes.len(), 2);
    assert_eq!(p.nodes[0].title, "First task");
}

#[test]
fn header_ends_at_first_non_matching_item() {
    let src = "# T\n\n- Owner: @jan\n- 2027-01-01 A milestone\n- Phase: late\n";
    let p = parse_str(src);
    assert_eq!(p.header.len(), 1);
    assert_eq!(p.nodes.len(), 1);
}

#[test]
fn no_header_when_first_item_does_not_match() {
    let p = parse_str("# T\n\n- [ ] task\n- Phase: x\n");
    assert!(p.header.is_empty());
    assert!(p.owner.is_none());
    assert!(codes(&p).contains(&"W-NO-OWNER"));
    assert_eq!(p.nodes.len(), 1);
}

#[test]
fn no_header_after_prose() {
    let p = parse_str("# T\n\nSome prose.\n\n- Phase: x\n");
    assert!(p.header.is_empty());
}

#[test]
fn header_not_applied_to_later_headings() {
    let p = parse_str("# T\n\n## Sec\n\n- Phase: x\n- [ ] A\n");
    assert!(p.header.is_empty());
}

#[test]
fn front_matter_is_not_supported() {
    let p = parse_str("---\nowner: Sanne <s@example.org>\n---\n\n# Aurora\n\n- [ ] 2027-01-01 M\n");
    assert!(p.owner.is_none());
    assert_eq!(p.title, "Aurora");
}

// ---- fences, comments, headings -------------------------------------------------------------

#[test]
fn fences_and_comments_ignored() {
    let src = "# T\n\n```\n- [ ] in fence\n```\n~~~~\n- [ ] in tilde\n~~~\n- [ ] still in\n~~~~\n<!-- - [ ] in comment\n- [ ] still comment -->\n<!-- one line - [ ] x -->\n- [ ] real\n````\n```\n- [ ] longer fence\n````\n- [ ] real2\n";
    let p = parse_str(src);
    let t: Vec<&str> = p.nodes.iter().map(|n| n.title.as_str()).collect();
    assert_eq!(t, ["real", "real2"]);
}

#[test]
fn headings_nesting_and_dropping() {
    let src = "# T\n\n## A  2026-11 .. 2027-06\n\n### A1\n\n- [ ] x\n\n## Risks\n\n- prose\n\n## B\n\n- [ ] y\n\n# Second h1\n\n- [ ] z\n";
    let p = parse_str(src);
    let top: Vec<&str> = p.nodes.iter().map(|n| n.title.as_str()).collect();
    assert_eq!(top, ["A", "B", "Second h1"]);
    assert!(p.nodes[0].range.is_none(), "headings carry no dates");
    assert_eq!(p.nodes[0].children[0].title, "A1");
    assert_eq!(p.nodes[0].children[0].children[0].title, "x");
}

#[test]
fn heading_syntax() {
    let p = parse_str("# T #\n\n## Foo ##\n- [ ] a\n####### not heading\n#tag not heading\n");
    assert_eq!(p.title, "T");
    assert_eq!(p.nodes[0].title, "Foo");
}

#[test]
fn nesting_by_indent_and_tabs() {
    let src = "# T\n\n- [ ] a\n  - [ ] b\n    - [ ] c\n\t- [ ] tab child of a? (4 cols)\n- [ ] d\n";
    let p = parse_str(src);
    assert_eq!(p.nodes.len(), 2);
    assert_eq!(p.nodes[0].children.len(), 1);
    let b = &p.nodes[0].children[0];
    assert_eq!(b.children[0].title, "c");
    assert_eq!(b.children[1].title, "tab child of a? (4 cols)");
}

#[test]
fn heading_resets_list_parent() {
    let p = parse_str("# T\n\n- [ ] a\n## H\n  - [ ] b\n");
    assert_eq!(p.nodes[0].title, "a");
    assert!(p.nodes[0].children.is_empty());
    assert_eq!(p.nodes[1].children[0].title, "b");
}

#[test]
fn crlf_lines() {
    let p = parse_str("# T\r\n\r\n- [ ] a  2w\r\n");
    assert_eq!(p.nodes[0].title, "a");
}

// ---- diagnostics ------------------------------------------------------------------------------

#[test]
fn duplicate_ids() {
    let p = parse_str("# T\n\n- [ ] a  ^x\n- [ ] b  ^x\n## H  ^x\n- [ ] c\n");
    let dups: Vec<u32> = p.diagnostics.iter().filter(|d| d.code == "E-DUP-ID").map(|d| d.line).collect();
    assert_eq!(dups, [4, 5]);
}

#[test]
fn dangling_local_refs() {
    let p = parse_str("# T\n\n- [ ] a  ^x\n- [ ] b  after: ^x  after: ^nope\n  - by: ^nada\n");
    let l: Vec<(u32, String)> = p.diagnostics.iter().filter(|d| d.code == "E-REF-LOCAL").map(|d| (d.line, d.message.clone())).collect();
    assert_eq!(l.len(), 2, "{:?}", l);
    assert!(l[0].1.contains("nope") && l[0].0 == 4);
    assert!(l[1].1.contains("nada") && l[1].0 == 5);
}

#[test]
fn no_owner_warning_only_when_missing() {
    let p = parse_str("# T\n\n- Owner: @a\n");
    assert!(p.diagnostics.is_empty());
    let p = parse_str("# T\n");
    assert_eq!(codes(&p), ["W-NO-OWNER"]);
}

#[test]
fn empty_and_weird_input_never_panics() {
    for s in ["", "\n", "-", "- ", "- [", "- [ ]", "- [ ] ", "[", "`", "- [ ] `", "- [ ] [a](", "- [ ] [a][", "- [ ] <", "---", "---\n", "# ", "#", "- [x] 2027-01-01 ..", "- [ ] .. ..", "- [ ] a: b: c: d:", "- [ ] é ü 日本語 2w", "[a]: ", "[a]: b \"c"] {
        let _ = parse_str(s);
        let _ = parse_str(&format!("# T\n\n{}\n", s));
    }
}

// ---- paths and graph ------------------------------------------------------------------------

#[test]
fn path_resolution() {
    assert_eq!(resolve("car/aurora.md", "powertrain/p.md"), "car/powertrain/p.md");
    assert_eq!(resolve("car/powertrain/p.md", "../body/b.md#x"), "car/body/b.md");
    assert_eq!(resolve("a.md", "./b/../c.md"), "c.md");
    assert_eq!(resolve("a/b.md", "../../../x.md"), "../../x.md");
    assert_eq!(resolve("a/b.md", "/abs/x.md"), "/abs/x.md");
    assert_eq!(resolve("a/b.md", "https://h.example/p.md#f"), "https://h.example/p.md");
    assert!(is_remote("HTTPS://x"));
    assert!(!is_remote("x.md"));
}

fn graph_of(files: &[(&str, &str)]) -> Graph {
    let mut g = Graph::new(files[0].0);
    for (p, s) in files {
        g.add(parse(p, s));
    }
    g
}

#[test]
fn graph_check_missing_public_and_ok() {
    let g = graph_of(&[
        (
            "r/root.md",
            "# Root\n\n- Owner: @a\n\n- [ ] [Sub](sub/sub.md)\n- [ ] [Gone](gone/gone.md)\n- [ ] [ok](sub/sub.md#pub)\n- [ ] [np](sub/sub.md#priv)\n- [ ] [nx](sub/sub.md#nothing)\n- [ ] [r](https://h.example/x.md#a)\n",
        ),
        ("r/sub/sub.md", "# Sub\n\n- Owner: @a\n\n- [ ] 2027-01-01 P  ^pub\n- [ ] 2027-01-01 Q  #private  ^priv\n"),
    ]);
    let d = g.check();
    let got: Vec<(&str, u32, &str)> = d.iter().map(|x| (x.code, x.line, x.path.as_str())).collect();
    assert!(got.contains(&("E-REF-MISSING", 6, "r/root.md")), "{:?}", d);
    assert!(got.contains(&("E-REF-PUBLIC", 8, "r/root.md")), "{:?}", d);
    assert!(got.contains(&("E-REF-MISSING", 9, "r/root.md")), "{:?}", d);
    assert_eq!(d.len(), 3, "{:?}", d);
    assert_eq!(g.remote_links(), ["https://h.example/x.md"]);
}

#[test]
fn graph_check_cycle() {
    let g = graph_of(&[
        ("a.md", "# A\n\n- Owner: @a\n\n- [ ] [B](b.md)\n"),
        ("b.md", "# B\n\n- Owner: @a\n\n- [ ] [C](c.md)\n"),
        ("c.md", "# C\n\n- Owner: @a\n\n- [ ] [A](a.md)\n- [ ] [Self](c.md)\n"),
        ("d.md", "# D\n\n- Owner: @a\n\n- [ ] [A](a.md)\n"),
    ]);
    let d = g.check();
    let cyc: Vec<&Diagnostic> = d.iter().filter(|x| x.code == "E-CYCLE").collect();
    assert_eq!(cyc.len(), 2, "{:?}", d);
    assert!(cyc.iter().any(|x| x.path == "c.md" && x.line == 5 && x.message.contains("a.md -> b.md -> c.md -> a.md")));
    assert!(cyc.iter().any(|x| x.path == "c.md" && x.line == 6));
}

#[test]
fn graph_no_false_cycle_on_diamond() {
    let g = graph_of(&[
        ("a.md", "# A\n\n- [ ] [B](b.md)\n- [ ] [C](c.md)\n"),
        ("b.md", "# B\n\n- [ ] [D](d.md)\n"),
        ("c.md", "# C\n\n- [ ] [D](d.md)\n"),
        ("d.md", "# D\n"),
    ]);
    assert!(g.check().iter().all(|x| x.code != "E-CYCLE"));
}

#[test]
fn json_is_lowercase() {
    let p = parse_str("# T\n\n- [ ] 2027-01-01 M  ^m\n");
    let j = serde_json::to_string(&p).unwrap();
    assert!(j.contains("\"nodes\"") && j.contains("\"kind\":\"milestone\"") && j.contains("\"checkbox\":\"open\""));
}
