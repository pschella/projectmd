use pmd_core::fmt::{format, format_with};
use pmd_core::*;
use serde_json::Value as J;
use std::path::{Path, PathBuf};

fn examples_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut es: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    es.sort();
    for p in es {
        if p.is_dir() {
            if p.file_name().is_some_and(|n| n == ".pmd") {
                continue;
            }
            walk(&p, out);
        } else if p.extension().is_some_and(|e| e == "md") {
            out.push(p);
        }
    }
}

/// Every markdown project file under examples/, with the car profile's options.
fn example_files() -> (Vec<(PathBuf, String)>, Options) {
    let root = examples_root();
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(!files.is_empty(), "no example files found");
    let prof = std::fs::read_to_string(root.join("car/.pmd/profile.md")).map(|s| parse_profile(&s)).unwrap_or_default();
    let texts = files.into_iter().map(|f| {
        let src = std::fs::read_to_string(&f).unwrap();
        (f, src)
    });
    (texts.collect(), prof)
}

/// Project JSON without line numbers; diagnostics are compared by severity and code only.
fn shape(p: &Project) -> (J, Vec<(String, &'static str)>) {
    fn strip(v: &mut J) {
        match v {
            J::Object(m) => {
                m.remove("line");
                m.remove("diagnostics");
                for x in m.values_mut() {
                    strip(x);
                }
            }
            J::Array(a) => a.iter_mut().for_each(strip),
            _ => {}
        }
    }
    let mut v = serde_json::to_value(p).unwrap();
    strip(&mut v);
    let codes = p.diagnostics.iter().map(|d| (format!("{:?}", d.severity), d.code)).collect();
    (v, codes)
}

fn all_ids(src: &str, opts: &Options) -> Vec<String> {
    let p = parse("t.md", src, opts);
    let mut ids = Vec::new();
    p.walk(|n| {
        if let Some(i) = &n.id {
            ids.push(i.clone());
        }
    });
    ids
}

#[test]
fn idempotent_on_examples() {
    let (files, opts) = example_files();
    for (path, src) in &files {
        let once = format_with(src, false, &opts);
        assert_eq!(format_with(&once, false, &opts), once, "not idempotent: {}", path.display());
        let with_ids = format_with(src, true, &opts);
        assert_eq!(format_with(&with_ids, true, &opts), with_ids, "ids not idempotent: {}", path.display());
    }
}

#[test]
fn meaning_preserved_on_examples() {
    let (files, opts) = example_files();
    for (path, src) in &files {
        let name = path.display().to_string();
        let before = parse(&name, src, &opts);
        let formatted = format_with(src, false, &opts);
        let after = parse(&name, &formatted, &opts);
        assert_eq!(shape(&before), shape(&after), "meaning changed: {}", name);
    }
}

#[test]
fn ids_unique_stable_and_complete_on_examples() {
    let (files, opts) = example_files();
    for (path, src) in &files {
        let name = path.display().to_string();
        let before_ids = all_ids(src, &opts);
        let out = format_with(src, true, &opts);
        let ids = all_ids(&out, &opts);
        let mut uniq = ids.clone();
        uniq.sort();
        uniq.dedup();
        assert_eq!(uniq.len(), ids.len(), "duplicate ids after --ids: {}", name);
        for id in &before_ids {
            assert!(ids.contains(id), "existing id {} changed: {}", id, name);
        }
        let p = parse(&name, &out, &opts);
        p.walk(|n| {
            if n.kind != NodeKind::Heading {
                assert!(n.id.is_some(), "item without id at line {} in {}", n.line, name);
            }
        });
    }
}

#[test]
fn ids_are_readable_unique_and_stable() {
    let src = "# P\n\n- [ ] Dyno testing  1w\n- [ ] Dyno testing  2w\n- [ ] Review the 2027-03-01 plan for [the seat](https://x.example/seat)\n- [ ] Keep me  ^keep\n- [ ] Keep me too  ^keep-2\n- [ ] Keep me again\n";
    let out = format(src, true);
    assert!(out.contains("  ^dyno-testing\n"), "{}", out);
    assert!(out.contains("  ^dyno-testing-2\n"), "{}", out);
    // Dates and link targets are dropped from the id; at most four words.
    assert!(out.contains("  ^review-the-plan-for\n"), "{}", out);
    assert!(out.contains("^keep\n"), "{}", out);
    assert!(out.contains("^keep-2\n"), "{}", out);
    assert!(out.contains("Keep me again  ^keep-me-again\n"), "{}", out);
    assert_eq!(format(&out, true), out);
}

#[test]
fn suffix_order_and_two_space_separators() {
    let src = "# P\n\n- [ ] Test 12m cable harness  owner: @jan  3d  #public  ^tc\n";
    assert_eq!(format(src, false), "# P\n\n- [ ] Test 12m cable harness  3d  #public  owner: @jan  ^tc\n");
}

#[test]
fn key_order_after_by_owner_approver_then_alphabetical() {
    let src = "# P\n\n- [ ] Gate  approver: @b  replaced-by: ^x  owner: @a  by: 2027-Q1  after: ^a\n";
    assert_eq!(
        format(src, false),
        "# P\n\n- [ ] Gate  after: ^a  by: 2027-Q1  owner: @a  approver: @b  replaced-by: ^x\n"
    );
}

#[test]
fn checkbox_and_bullet_spelling() {
    let src = "# P\n\n*   [x]   Done item   owner:  @jan   1w\n+ [-] Dropped\n";
    assert_eq!(format(src, false), "# P\n\n- [x] Done item  1w  owner: @jan\n- [-] Dropped\n");
}

#[test]
fn leading_date_and_range_spacing() {
    let src = "# P\n\n- 2027-03-01   Launch  #public\n- 2027-01..2027-02 Seat concept\n";
    assert_eq!(format(src, false), "# P\n\n- 2027-03-01 Launch  #public\n- 2027-01 .. 2027-02 Seat concept\n");
}

#[test]
fn indentation_is_two_spaces_per_level() {
    let src = "# P\n\n- [ ] A  1w\n    - [ ] B  2w\n        - [ ] C  3w\n  - [ ] D  1w\n";
    assert_eq!(
        format(src, false),
        "# P\n\n- [ ] A  1w\n  - [ ] B  2w\n    - [ ] C  3w\n  - [ ] D  1w\n"
    );
}

#[test]
fn prose_and_code_are_untouched() {
    let src = "# P\n\nSome   prose  stays.\n\n```\n   - [ ] inside   owner: @x\n```\n\n* a prose bullet   with   spaces\n";
    let out = format(src, false);
    assert!(out.contains("Some   prose  stays.\n"));
    assert!(out.contains("   - [ ] inside   owner: @x\n"));
    assert!(out.contains("- a prose bullet   with   spaces\n"));
}

#[test]
fn blank_lines_trailing_spaces_and_final_newline() {
    let src = "# P\n## H\n- [ ] A 1w   \r\n\n\n\n## Two\n- [ ] B 2w";
    assert_eq!(format(src, false), "# P\n\n## H\n- [ ] A  1w\n\n## Two\n- [ ] B  2w\n");
}

#[test]
fn already_canonical_is_unchanged() {
    let src = "# P\n\n- [ ] A  1w  owner: @jan\n  - [x] B  #public  ^b\n";
    assert_eq!(format(src, false), src);
}

