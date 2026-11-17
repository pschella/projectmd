use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn run(args: &[&str]) -> String {
    let out = Command::new(env!("CARGO_BIN_EXE_pmd")).args(args).env("PMD_TODAY", "2027-01-14").output().expect("run pmd");
    String::from_utf8_lossy(&out.stdout).into_owned()
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

const PARENT: &str = "# Parent\n\n- Owner: Pat\n- Commitment: 2027-03-01\n\n- [ ] [Impl](child.md)  by: 2027-01-20  ^impl\n- [ ] RC  after: ^impl  ^rc\n- [ ] Sign-off  3d  after: ^rc  ^so\n";
const CHILD: &str = "# Child\n\n- Owner: Cy\n- Commitment: 2027-02-01\n\n- [ ] Build  5d  after: 2027-01-20  ^build\n- [ ] Ready  after: ^build  ^ready\n";

#[test]
fn chain_through_after_items_ends_on_the_promise_of_a_link() {
    let f = files("why_after", &[("parent.md", PARENT), ("child.md", CHILD)]);
    let out = run(&["why", &f[0], "^so"]);
    let l: Vec<&str> = out.lines().collect();
    assert!(l[0].contains("Sign-off") && l[0].contains("forecast 2027-02-04") && l[0].contains("estimate, starts 2027-02-02"), "{out}");
    assert!(l[1].contains("<-") && l[1].contains("RC") && l[1].ends_with("after"), "{out}");
    assert!(l[2].contains("Impl") && l[2].contains("2027-02-01") && l[2].contains("promise of") && l[2].contains("(Commitment)"), "{out}");
    assert!(l[3].contains("drill down: pmd why") && l[3].contains("child.md"), "{out}");
    assert_eq!(l.len(), 4);
}

#[test]
fn deep_continues_in_the_child_file() {
    let f = files("why_deep", &[("parent.md", PARENT), ("child.md", CHILD)]);
    let out = run(&["why", "--deep", &f[0], "^so"]);
    assert!(out.contains("== ") && out.contains("child.md ==") && !out.contains("drill down"), "{out}");
    assert!(out.contains("Child") && out.contains("latest child") && out.contains("Ready"), "{out}");
    assert!(out.contains("Build") && out.contains("estimate, starts 2027-01-20 (after: date)"), "{out}");
    let j: serde_json::Value = serde_json::from_str(&run(&["why", "--deep", &f[0], "^so", "--json"])).unwrap();
    let hops = j[0].as_array().unwrap();
    assert!(hops.len() >= 5 && hops[2]["child"]["file"].as_str().unwrap().ends_with("child.md"), "{j}");
    assert_eq!(hops[0]["driver"]["kind"], "estimate");
}

#[test]
fn not_started_work_ends_on_today() {
    let f = files("why_today", &[("a.md", "# A\n\n- Owner: Ann\n\n- [ ] Task  5d  ^t\n")]);
    let out = run(&["why", &f[0], "^t"]);
    assert!(out.contains("forecast 2027-01-18") && out.contains("not started: earliest start is today"), "{out}");
}

#[test]
fn open_undated_decision_is_due_today() {
    let f = files("why_undated", &[("a.md", "# A\n\n- Owner: Ann\n\n- [ ] Decide  owner: Dana  ^d\n- [ ] Build  3d  after: ^d  ^b\n")]);
    let out = run(&["why", &f[0], "^b"]);
    assert!(out.contains("Build") && out.contains("<-") && out.contains("Decide"), "{out}");
    assert!(out.contains("open, undated: due today (owner: Dana)"), "{out}");
}

#[test]
fn default_explains_the_project_end_and_every_conflict() {
    let f = files("why_conf", &[("a.md", "# A\n\n- Owner: Ann\n- Commitment: 2027-01-20\n\n- [ ] Late  10d  by: 2027-01-18  ^l\n")]);
    let out = run(&["why", &f[0]]);
    assert!(out.contains("A  forecast 2027-01-23") && out.contains("(declared 2027-01-20, 3d late)"), "{out}");
    assert!(out.contains("Late") && out.contains("by: 2027-01-18") || out.matches("forecast").count() >= 2, "{out}");
}

#[test]
fn ready_queue_across_two_files() {
    let f = files("ready_q", &[("parent.md", "# P\n\n- Owner: Pat\n\n- [ ] [Impl](child.md)  by: 2027-01-20\n"), ("child.md", "# C\n\n- Owner: Cy\n")]);
    let single = run(&["ready", &f[0]]);
    assert!(!single.contains("ANSWER") && single.contains("0 ready"), "{single}");
    let out = run(&["ready", "--deep", &f[0]]);
    assert!(out.contains("child.md:1  ANSWER  Impl  owner: Cy  due: 2027-01-20  needed by 2027-01-20 (1 asker; earliest: P)"), "{out}");
    let j: serde_json::Value = serde_json::from_str(&run(&["ready", "--json", &f[0], &f[1]])).unwrap();
    assert_eq!(j[0]["kind"], "answer");
    assert_eq!(j[0]["detail"], "needed by 2027-01-20 (1 asker; earliest: P)");
    assert_eq!(j[0]["askers"], 1);
}

#[test]
fn two_askers_of_one_child_promise_are_one_ready_line() {
    let f = files("ready_two_askers", &[
        ("alpha.md", "# Alpha\n\n- Owner: Pat\n\n- [ ] [Impl](child.md)  by: 2027-01-12\n"),
        ("beta.md", "# Beta\n\n- Owner: Bo\n\n- [ ] [Impl](child.md)  by: 2027-01-13\n"),
        ("child.md", "# C\n\n- Owner: Cy\n- Commitment: 2027-01-10\n"),
    ]);
    let out = run(&["ready", "--deep", &f[0], &f[1]]);
    assert_eq!(out.matches("DELIVER").count(), 1, "{out}");
    assert!(out.contains("needed by 2027-01-12 (2 askers; earliest: Alpha); promised 2027-01-10"), "{out}");
    let j: serde_json::Value = serde_json::from_str(&run(&["ready", "--json", "--deep", &f[0], &f[1]])).unwrap();
    let asks: Vec<&serde_json::Value> = j.as_array().unwrap().iter().filter(|e| e["file"] == f[2].as_str()).collect();
    assert_eq!(asks.len(), 1);
    assert_eq!(asks[0]["askers"], 2);
}

#[test]
fn a_declared_end_continues_through_its_predecessor() {
    let f = files("why_declared", &[("w.md", "# T\n\n- Owner: A\n\n- [ ] A  1w  ^a\n- [ ] 2027-01-25 Gate  after: ^a  ^g\n")]);
    let out = run(&["why", &f[0], "^g"]);
    let l: Vec<&str> = out.lines().collect();
    // D68: the milestone follows its input; its declared date is the slack reference
    assert!(l[0].contains("forecast 2027-01-20  (declared 2027-01-25, slack +5d)"), "{out}");
    assert!(l[1].contains("<-") && l[1].contains("A"), "{out}");
}
