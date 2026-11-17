mod common;
use common::parse_str;

fn desc(src: &str) -> Vec<Option<String>> {
    let p = parse_str(src);
    let mut v = Vec::new();
    p.walk(|n| v.push(n.description.clone()));
    v
}

#[test]
fn paragraph_and_bullets_with_breaks_and_fence() {
    let s = "# T\n\n- [ ] A  ^a\n  Done when:\n\n  - spec signed\n    - nested note\n  ```\n    code\n\n  ```\n\n- [ ] B\n";
    let d = desc(s);
    assert_eq!(d[0].as_deref(), Some("Done when:\n\n- spec signed\n  - nested note\n```\n  code\n\n```"));
    assert_eq!(d[1], None);
}

#[test]
fn ends_at_next_item_and_sub_items_and_keys_are_not_description() {
    let s = "# T\n\n- [ ] A\n  - owner: Ann\n  - a note\n  - [ ] Sub\n    sub text\n  after sub\n- [ ] B\n  by: later\n";
    let d = desc(s);
    assert_eq!(d[0].as_deref(), Some("- a note"));
    assert_eq!(d[1].as_deref(), Some("sub text"));
    assert_eq!(d[2].as_deref(), Some("by: later"));
}

#[test]
fn header_bullets_and_top_level_prose_are_not_description() {
    let s = "# T\n\n- Owner: Ann\n  - sub of header\n\nSome prose.\n\n- [ ] A\nprose at column 0\n\n## H\n  indented prose\n";
    assert_eq!(desc(s), [None]);
}

#[test]
fn json_omits_description_when_none() {
    let p = parse_str("# T\n\n- [ ] A\n  text\n- [ ] B\n");
    let j = serde_json::to_value(&p).unwrap();
    assert_eq!(j["nodes"][0]["description"], "text");
    assert!(j["nodes"][1].get("description").is_none());
}
