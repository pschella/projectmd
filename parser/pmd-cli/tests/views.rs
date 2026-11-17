use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn pmd(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_pmd")).args(args).env("PMD_TODAY", "2027-01-11").output().expect("run pmd");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code().unwrap_or(-1))
}

/// Write files into a fresh directory and return their paths.
fn files(name: &str, fs_: &[(&str, &str)]) -> Vec<String> {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    fs_.iter()
        .map(|(n, s)| {
            let p = dir.join(n);
            fs::write(&p, s).unwrap();
            p.to_str().unwrap().to_string()
        })
        .collect()
}

#[test]
fn changes_prints_one_event_per_line_and_a_count() {
    let old = "# P\n\n- Commitment: 2027-03-01\n\n- [ ] 2027-02-10 Ready  ^ready\n- [ ] [QA](qa.md#ok)  by: 2027-01-26  ^qa\n";
    let new = "# P\n\n- Commitment: 2027-03-01\n\n- [ ] 2027-02-17 Ready  ^ready\n- [ ] [QA](qa.md#ok)  by: 2027-02-02  ^qa\n";
    let f = files("views_changes", &[("old.md", old), ("new.md", new)]);
    let (out, code) = pmd(&["changes", &f[0], &f[1]]);
    assert_eq!(code, 0);
    assert_eq!(
        out,
        "promise moved  ^ready Ready  2027-02-10 \u{2192} 2027-02-17 (+7d)  [parents]\nask moved  qa.md#ok  by: 2027-01-26 \u{2192} 2027-02-02 (+7d)  [child qa.md]\n2 changes\n"
    );
}

#[test]
fn changes_without_events_says_so() {
    let f = files("views_changes_none", &[("a.md", "# P\n\n- [ ] X\n"), ("b.md", "# P\n\n- [ ] X\n")]);
    let (out, code) = pmd(&["changes", &f[0], &f[1]]);
    assert_eq!((out.as_str(), code), ("no changes\n", 0));
}

#[test]
fn changes_needs_exactly_two_files() {
    let f = files("views_changes_args", &[("a.md", "# P\n")]);
    let (_, code) = pmd(&["changes", &f[0]]);
    assert_eq!(code, 2);
}

#[test]
fn replan_dry_run_leaves_the_file_and_write_applies_it() {
    let parent = "# Parent\n\n- Owner: Pat\n- Commitment: 2027-03-01\n\n- [ ] [Impl](child.md)  by: 2027-01-20  ^impl\n";
    let child = "# Child\n\n- Owner: Cy\n- Commitment: 2027-02-01\n\n- [ ] Build  5d  after: 2027-01-20  ^build\n";
    let f = files("views_replan", &[("parent.md", parent), ("child.md", child)]);
    let (out, code) = pmd(&["replan", &f[0]]);
    assert_eq!(code, 0);
    assert!(out.contains("accept dispute  2027-01-20 \u{2192} 2027-02-01 (+12d)"), "{}", out);
    assert!(out.ends_with("1 edits (dry run; --write to apply)\n"), "{}", out);
    assert_eq!(fs::read_to_string(&f[0]).unwrap(), parent);

    let (out, _) = pmd(&["replan", &f[0], "--write"]);
    assert!(out.ends_with("1 edits written\n"), "{}", out);
    assert_eq!(fs::read_to_string(&f[0]).unwrap(), parent.replace("by: 2027-01-20", "by: 2027-02-01"));
    assert_eq!(pmd(&["replan", &f[0]]).0, "consistent: nothing to replan\n");
}
