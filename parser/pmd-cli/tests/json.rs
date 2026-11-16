use std::path::PathBuf;
use std::process::Command;

/// examples/org is the stable CLI fixture: a root with two disputes and one conflict below it.
fn org(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/org").join(rel)
}

fn top() -> PathBuf {
    org("sales/vellmar/vellmar.md")
}

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_pmd")).args(args).output().expect("run pmd");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn check_json_has_diagnostics_remote_and_ok() {
    let file = top();
    let (code, out) = run(&["check", file.to_str().unwrap(), "--json", "--today", "2026-11-15"]);
    let v: serde_json::Value = serde_json::from_str(&out).expect("check --json prints valid JSON");
    let diags = v["diagnostics"].as_array().expect("diagnostics is an array");
    assert!(v["remote"].is_array(), "remote is an array");
    let ok = v["ok"].as_bool().expect("ok is a bool");
    // Same exit code rule as text mode: 0 when ok, 1 otherwise.
    assert_eq!(code == 0, ok, "exit code {} must match ok={}", code, ok);
    for d in diags {
        assert!(d["path"].is_string());
        assert!(d["line"].is_number());
        assert!(matches!(d["severity"].as_str(), Some("E") | Some("W") | Some("I")));
        assert!(d["code"].is_string());
        assert!(d["message"].is_string());
        assert!(d["fix"].is_string() || d["fix"].is_null());
    }
}

#[test]
fn schedule_json_is_project_with_forecasts() {
    let file = top();
    let (code, out) = run(&["schedule", file.to_str().unwrap(), "--json", "--today", "2026-11-15"]);
    assert_eq!(code, 0, "schedule --json exits 0 on a readable project");
    let v: serde_json::Value = serde_json::from_str(&out).expect("schedule --json prints valid JSON");
    assert!(v["title"].is_string(), "project title");
    assert!(v["nodes"].is_array(), "nodes");
    assert!(v["forecast"].is_object(), "project forecast");
    assert!(v["forecast"]["conflict"].is_boolean(), "forecast.conflict");
    let first = v["nodes"].as_array().unwrap().first().expect("at least one node");
    assert!(first["forecast"].is_object(), "node forecast");
}

#[test]
fn check_is_one_level_unless_deep() {
    let file = top();
    let f = file.to_str().unwrap();
    let (c1, shallow) = run(&["check", f, "--today", "2026-11-15"]);
    let (c2, deep) = run(&["check", f, "--deep", "--today", "2026-11-15"]);
    // the dispute and conflict below the front files show only with --deep
    assert_eq!((c1, shallow.trim()), (0, ""), "{}", shallow);
    assert_eq!(c2, 1);
    assert!(deep.contains("displays.md:12: W W-DISPUTED") && deep.contains("E-CONFLICT"), "{}", deep);
}

#[test]
fn usage_and_unknown_flags_exit_2() {
    let f = top();
    assert_eq!(run(&["frobnicate", f.to_str().unwrap()]).0, 2);
    assert_eq!(run(&["fmt", f.to_str().unwrap(), "--if-hash", "x"]).0, 2);
}

#[test]
fn org_states_are_live_and_fronts_hide_plans() {
    let chk = |f: &str| run(&["check", org(f).to_str().unwrap(), "--today", "2026-11-15"]);
    let (c, out) = chk("sales/vellmar/vellmar.md");
    assert_eq!((c, out.trim()), (0, ""), "{}", out);
    let (c, out) = chk("units/hardware/displays/displays.md");
    assert_eq!(c, 1);
    assert!(out.contains("W-DISPUTED") && out.contains("E-CONFLICT"), "{}", out);
    let (c, out) = chk("units/software/private/software-plan.md");
    assert_eq!(c, 0);
    assert!(out.contains("W-DISPUTED") && out.contains("I-REQUESTED") && !out.contains("Timetable"), "{}", out);
    let (c, out) = chk("teams/enclosures/enclosures.md");
    assert_eq!((c, out.trim()), (0, ""), "{}", out);
}

#[test]
fn fmt_prints_changed_files_only() {
    let dir = std::env::temp_dir().join(format!("pmd-fmt-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let (a, b) = (dir.join("a.md"), dir.join("b.md"));
    std::fs::write(&a, "# T\n\n* [ ]   Task  2w\n").unwrap();
    std::fs::write(&b, "# T\n\n- [ ] Task  2w\n").unwrap();
    let (an, bn) = (a.to_str().unwrap(), b.to_str().unwrap());
    let (code, out) = run(&["fmt", an, bn]);
    assert_eq!((code, out.trim()), (0, an));
    let (code, out) = run(&["fmt", an, bn]);
    assert_eq!((code, out.as_str()), (0, ""));
    std::fs::remove_dir_all(&dir).ok();
}

#[test]
fn tree_depth_forms_and_file_position() {
    let f = top();
    let f = f.to_str().unwrap();
    let (c1, a) = run(&["tree", f, "--depth", "2", "--today", "2026-11-15"]);
    let (c2, b) = run(&["tree", f, "--depth=2", "--today=2026-11-15"]);
    let (c3, d) = run(&["tree", "--depth", "2", f, "--today", "2026-11-15"]);
    assert!(c1 < 2 && c2 < 2 && c3 < 2, "usage error");
    assert!(!a.is_empty());
    assert_eq!(a, b);
    assert_eq!(a, d);
    let (_, shallow) = run(&["tree", f, "--today", "2026-11-15"]);
    assert!(a.lines().count() >= shallow.lines().count());
}

#[test]
fn check_takes_several_files() {
    let (a, b) = (top(), org("teams/backend/backend.md"));
    let (c, out) = run(&["check", a.to_str().unwrap(), b.to_str().unwrap(), "--today", "2026-11-15"]);
    let (c1, _) = run(&["check", a.to_str().unwrap(), "--today", "2026-11-15"]);
    let (c2, _) = run(&["check", b.to_str().unwrap(), "--today", "2026-11-15"]);
    assert_eq!(c, c1.max(c2));
    assert!(out.is_empty() || out.contains(':'));
}
