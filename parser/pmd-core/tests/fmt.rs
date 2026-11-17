use pmd_core::fmt::format;
use pmd_core::*;
use serde_json::Value as J;
use std::path::{Path, PathBuf};

fn examples_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/org")
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut es: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    es.sort();
    for p in es {
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().is_some_and(|e| e == "md") {
            out.push(p);
        }
    }
}

/// Every markdown project file under examples/org.
fn example_files() -> Vec<(PathBuf, String)> {
    let root = examples_root();
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(!files.is_empty(), "no example files found");
    let texts = files.into_iter().map(|f| {
        let src = std::fs::read_to_string(&f).unwrap();
        (f, src)
    });
    texts.collect()
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

#[test]
fn idempotent_on_examples() {
    let files = example_files();
    for (path, src) in &files {
        let once = format(src);
        assert_eq!(format(&once), once, "not idempotent: {}", path.display());
    }
}

#[test]
fn meaning_preserved_on_examples() {
    let files = example_files();
    for (path, src) in &files {
        let name = path.display().to_string();
        let before = parse(&name, src);
        let formatted = format(src);
        let after = parse(&name, &formatted);
        assert_eq!(shape(&before), shape(&after), "meaning changed: {}", name);
    }
}

#[test]
fn suffix_order_and_two_space_separators() {
    let src = "# P\n\n- [ ] Test 12m cable harness  owner: @jan  3d  #public  ^tc\n";
    assert_eq!(format(src), "# P\n\n- [ ] Test 12m cable harness  3d  #public  owner: @jan  ^tc\n");
}

#[test]
fn key_order_after_by_owner_done() {
    let src = "# P\n\n- [ ] Gate  done: 2027-01-01  owner: @a  by: 2027-Q1  after: ^a\n";
    assert_eq!(
        format(src),
        "# P\n\n- [ ] Gate  after: ^a  by: 2027-Q1  owner: @a  done: 2027-01-01\n"
    );
}

#[test]
fn checkbox_and_bullet_spelling() {
    let src = "# P\n\n*   [x]   Done item   owner:  @jan   1w\n+ [-] Dropped\n";
    assert_eq!(format(src), "# P\n\n- [x] Done item  1w  owner: @jan\n- [-] Dropped\n");
}

#[test]
fn leading_date_and_range_spacing() {
    let src = "# P\n\n- [ ] 2027-03-01   Launch  #public\n- [ ] 2027-01..2027-02 Seat concept\n";
    // 0.9: a leading range is not a form; the line stays as written (title text)
    assert_eq!(format(src), "# P\n\n- [ ] 2027-03-01 Launch  #public\n- [ ] 2027-01..2027-02 Seat concept\n");
}

#[test]
fn indentation_is_two_spaces_per_level() {
    let src = "# P\n\n- [ ] A  1w\n    - [ ] B  2w\n        - [ ] C  3w\n  - [ ] D  1w\n";
    assert_eq!(
        format(src),
        "# P\n\n- [ ] A  1w\n  - [ ] B  2w\n    - [ ] C  3w\n  - [ ] D  1w\n"
    );
}

#[test]
fn prose_and_code_are_untouched() {
    let src = "# P\n\nSome   prose  stays.\n\n```\n   - [ ] inside   owner: @x\n```\n\n* a prose bullet   with   spaces\n";
    let out = format(src);
    assert!(out.contains("Some   prose  stays.\n"));
    assert!(out.contains("   - [ ] inside   owner: @x\n"));
    assert!(out.contains("- a prose bullet   with   spaces\n"));
}

#[test]
fn blank_lines_trailing_spaces_and_final_newline() {
    let src = "# P\n## H\n- [ ] A 1w   \r\n\n\n\n## Two\n- [ ] B 2w";
    assert_eq!(format(src), "# P\n\n## H\n- [ ] A  1w\n\n## Two\n- [ ] B  2w\n");
}

#[test]
fn already_canonical_is_unchanged() {
    let src = "# P\n\n- [ ] A  1w  owner: @jan\n  - [x] B  #public  ^b\n";
    assert_eq!(format(src), src);
}

