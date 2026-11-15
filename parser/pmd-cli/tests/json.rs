use std::path::PathBuf;
use std::process::Command;

fn aurora() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/car/aurora.md")
}

fn run(args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_pmd")).args(args).output().expect("run pmd");
    (out.status.code().unwrap_or(-1), String::from_utf8_lossy(&out.stdout).into_owned())
}

#[test]
fn check_json_has_diagnostics_remote_and_ok() {
    let file = aurora();
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
        assert!(matches!(d["severity"].as_str(), Some("E") | Some("W")));
        assert!(d["code"].is_string());
        assert!(d["message"].is_string());
        assert!(d["fix"].is_string() || d["fix"].is_null());
    }
}

#[test]
fn schedule_json_is_project_with_forecasts() {
    let file = aurora();
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
    let file = aurora();
    let f = file.to_str().unwrap();
    let (_, shallow) = run(&["check", f, "--today", "2026-11-15"]);
    let (_, deep) = run(&["check", f, "--deep", "--today", "2026-11-15"]);
    // aurora.md's own contract error is found either way ...
    assert!(shallow.contains("aurora.md:32: E E-BOUND"), "{}", shallow);
    // ... but the error two levels down (software.md) only with --deep.
    assert!(!shallow.contains("software.md"), "{}", shallow);
    assert!(deep.contains("software.md:23: E E-BOUND"), "{}", deep);
}
