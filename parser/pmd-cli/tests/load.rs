use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Run pmd with today fixed at 2027-03-10 (a Wednesday, ISO week 2027-W10).
fn pmd(args: &[&str]) -> (String, i32) {
    let out = Command::new(env!("CARGO_BIN_EXE_pmd")).args(args).env("PMD_TODAY", "2027-03-10").output().expect("run pmd");
    (String::from_utf8_lossy(&out.stdout).into_owned(), out.status.code().unwrap_or(-1))
}

/// Write one file into a fresh directory and return its path.
fn file(name: &str, src: &str) -> String {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    let p = dir.join("team.md");
    fs::write(&p, src).unwrap();
    p.to_str().unwrap().to_string()
}

/// The bakery example: a root file with two deliverable links into the files next to it.
fn bakery() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/bakery/bakery.md").to_str().unwrap().to_string()
}

const TREE: &str = "# Team

- Owner: Ana

## Work
- [ ] Parent  2w  #core  ^parent
  - [ ] Child  3d  owner: Ben  #api  ^child
- [x] Old  owner: Ben  done: 2027-03-02
- [-] Dropped
";

#[test]
fn rows_inherit_owner_carry_tags_and_nest() {
    let f = file("load_rows_tree", TREE);
    let (out, code) = pmd(&["schedule", &f, "--rows"]);
    assert_eq!(code, 0);
    let rows: Vec<serde_json::Value> = out.lines().map(|l| serde_json::from_str(l).expect("each line is one JSON object")).collect();
    assert_eq!(rows.len(), 4, "one row per planning item, no heading: {}", out);
    let keys: Vec<&str> = rows[0].as_object().unwrap().keys().map(String::as_str).collect();
    assert_eq!(
        keys,
        ["file", "line", "id", "title", "kind", "state", "owner", "owner_inherited", "start", "end", "declared_end", "slack_days", "estimate_days", "tags", "parent_line", "leaf"]
    );
    let (parent, child, old, dropped) = (&rows[0], &rows[1], &rows[2], &rows[3]);
    assert_eq!(parent["file"], f.as_str());
    assert_eq!(parent["line"], 6);
    assert_eq!(parent["id"], "parent");
    assert_eq!(parent["kind"], "task");
    assert_eq!(parent["state"], "open");
    // the file's Owner: text, inherited by the item
    assert_eq!(parent["owner"], "Ana");
    assert_eq!(parent["owner_inherited"], true);
    assert_eq!(parent["tags"], serde_json::json!(["core"]));
    assert!(parent["parent_line"].is_null());
    // an open sub-item makes the parent not a leaf
    assert_eq!(parent["leaf"], false);
    assert_eq!(parent["estimate_days"], 14);
    // the child's own owner, its tags without `#`, and the enclosing item's line
    assert_eq!(child["line"], 7);
    assert_eq!(child["owner"], "Ben");
    assert_eq!(child["owner_inherited"], false);
    assert_eq!(child["tags"], serde_json::json!(["api"]));
    assert_eq!(child["parent_line"], 6);
    assert_eq!(child["leaf"], true);
    assert_eq!(child["estimate_days"], 3);
    assert_eq!(old["state"], "done");
    assert_eq!(old["owner_inherited"], false);
    assert_eq!(old["end"], "2027-03-02");
    assert_eq!(dropped["state"], "cancelled");
    assert_eq!(dropped["owner"], "Ana");
    assert_eq!(dropped["owner_inherited"], true);
}

#[test]
fn rows_without_deep_are_the_named_file_and_deep_adds_the_linked_files() {
    let root = bakery();
    let (out, code) = pmd(&["schedule", &root, "--rows", "--today", "2027-03-01"]);
    assert_eq!(code, 0);
    let own: Vec<serde_json::Value> = out.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
    assert!(!own.is_empty());
    assert!(own.iter().all(|r| r["file"].as_str().unwrap().ends_with("bakery.md")), "{}", out);

    let (out, code) = pmd(&["schedule", &root, "--rows", "--deep", "--today", "2027-03-01"]);
    assert_eq!(code, 0);
    let all: Vec<serde_json::Value> = out.lines().map(|l| serde_json::from_str(l).expect("each line is one JSON object")).collect();
    assert!(all.len() > own.len());
    assert!(all.iter().any(|r| r["file"].as_str().unwrap().ends_with("fitout/fitout.md")), "{}", out);
    assert!(all.iter().any(|r| r["file"].as_str().unwrap().ends_with("licences.md")), "{}", out);
    // each file is listed once, in load order: the root first
    assert!(all[0]["file"].as_str().unwrap().ends_with("bakery.md"));
}

/// Two overlapping two-week tasks of Ben (and of Cy), a sequential pair of Seq, and a point of Ben.
const TEAM: &str = "# Team

- Owner: Ana

## Work
- [ ] Alpha  2w  owner: Ben  ^a
- [ ] Beta  2w  owner: Ben  ^b
- [ ] Seq one  1w  after: 2027-03-15  owner: Seq  ^s1
- [ ] Seq two  1w  after: ^s1  owner: Seq  ^s2
- [ ] Point  owner: Ben  ^p
- [ ] Solo  2w  owner: Cy  ^s
- [ ] Other  2w  owner: Cy  ^o
";

#[test]
fn load_reports_overlaps_per_owner_and_week_but_not_sequence_or_points() {
    let f = file("load_text", TEAM);
    let (out, code) = pmd(&["load", &f]);
    assert_eq!(code, 0);
    let p = &f;
    let expected = format!(
        "Ben  2027-W10  2 at once: {p}:6 Alpha; {p}:7 Beta\n\
         Ben  2027-W11  2 at once: {p}:6 Alpha; {p}:7 Beta\n\
         Ben  2027-W12  2 at once: {p}:6 Alpha; {p}:7 Beta\n\
         Cy  2027-W10  2 at once: {p}:11 Solo; {p}:12 Other\n\
         Cy  2027-W11  2 at once: {p}:11 Solo; {p}:12 Other\n\
         Cy  2027-W12  2 at once: {p}:11 Solo; {p}:12 Other\n\
         6 owner-weeks with more than one item (1 file read)\n",
        p = p
    );
    assert_eq!(out, expected);
}

#[test]
fn load_owner_filter_uses_ready_matching() {
    let f = file("load_owner", TEAM);
    let (out, code) = pmd(&["load", &f, "--owner", "CY"]);
    assert_eq!(code, 0);
    assert!(out.starts_with("Cy  2027-W10"), "{}", out);
    assert!(!out.contains("Ben") && !out.contains("Seq"), "{}", out);
    assert!(out.ends_with("3 owner-weeks with more than one item (1 file read)\n"), "{}", out);
    let (out, _) = pmd(&["load", &f, "--owner", "nobody"]);
    assert_eq!(out, "no owner has more than one item at once (1 file read)\n");
}

#[test]
fn load_groups_unowned_items_and_says_so_when_nothing_overlaps() {
    let f = file("load_no_owner", "# P\n\n- [ ] X  2w  ^x\n- [ ] Y  2w  ^y\n");
    let (out, code) = pmd(&["load", &f]);
    assert_eq!(code, 0);
    assert!(out.starts_with("(no owner)  2027-W10  2 at once: "), "{}", out);
    assert!(out.ends_with("3 owner-weeks with more than one item (1 file read)\n"), "{}", out);

    let g = file("load_none", "# P\n\n- Owner: Ana\n\n- [ ] X  1w  after: 2027-03-08  ^x\n- [ ] Y  1w  after: ^x  ^y\n");
    let (out, code) = pmd(&["load", &g]);
    assert_eq!(code, 0);
    assert_eq!(out, "no owner has more than one item at once (1 file read)\n");
}

#[test]
fn load_json_has_the_shape_and_every_week_with_an_item() {
    let f = file("load_json", TEAM);
    let (out, code) = pmd(&["load", &f, "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("load --json prints valid JSON");
    assert_eq!(v["today"], "2027-03-10");
    let owners = v["owners"].as_array().unwrap();
    let names: Vec<&str> = owners.iter().map(|o| o["owner"].as_str().unwrap()).collect();
    assert_eq!(names, ["Ben", "Cy", "Seq"]);
    let ben = &owners[0]["weeks"];
    let weeks: Vec<&str> = ben.as_array().unwrap().iter().map(|w| w["week"].as_str().unwrap()).collect();
    assert_eq!(weeks, ["2027-W10", "2027-W11", "2027-W12"]);
    assert_eq!(ben[0]["count"], 2);
    assert_eq!(ben[0]["items"][0]["title"], "Alpha");
    assert_eq!(ben[0]["items"][0]["line"], 6);
    assert_eq!(ben[0]["items"][1]["title"], "Beta");
    // weeks with one item count too in JSON
    let seq = &owners[2]["weeks"];
    assert_eq!(seq.as_array().unwrap().len(), 2);
    assert_eq!(seq[0]["count"], 1);
    assert_eq!(seq[1]["week"], "2027-W12");
}

#[test]
fn unreadable_file_is_exit_2() {
    let (_, code) = pmd(&["load", "/nonexistent/projectmd/nope.md"]);
    assert_eq!(code, 2);
    let (_, code) = pmd(&["schedule", "/nonexistent/projectmd/nope.md", "--rows"]);
    assert_eq!(code, 2);
}
