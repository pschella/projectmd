use std::fs;
use std::path::PathBuf;
use std::process::Command;

/// Run pmd with today fixed at 2027-03-10.
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

/// One file with an item in every column, a waiting milestone, a dropped item and an older done item.
const COLUMNS: &str = "# Team

- Owner: Ana

## Work
- [x] Old thing  done: 2027-01-02
- [x] Refund API  owner: Ana  done: 2027-03-05
- [ ] Endpoint  3d  after: 2027-03-08  owner: Ben  ^api
  Done when the endpoint passes the contract tests.
- [ ] Receipts  2d  after: 2027-03-11  owner: Chen  ^rec
- [ ] Refund email  2d  owner: Ben
- [ ] Review endpoint  after: ^api  owner: Ana  ^review
- [ ] 2027-03-26 Refunds live  after: ^review  after: ^rec  ^release
- [-] Dropped idea
- [ ] 2027-04-01 Launch  ^launch
";

/// Four done items, the earliest on 2027-02-20, so the window is the 19 days up to today: throughput 4 per
/// 19 days (1.5/week), cycle times 3, 4 and 1 days (median 3), and an open milestone that waits on three items.
const FLOW: &str = "# Flow

- Owner: Ana

- [x] D1  owner: Ana  after: 2027-03-01  done: 2027-03-03
- [x] D2  owner: Ana  after: 2027-03-02  done: 2027-03-05
- [x] D3  owner: Ana  after: 2027-03-05  done: 2027-03-05
- [x] D4  owner: Ana  done: 2027-02-20
- [ ] Open task A  after: ^c  ^a
- [ ] Open task B  ^b
- [ ] Open task C  ^c
- [ ] 2027-04-30 Launch  after: ^a  after: ^b  ^launch
";

#[test]
fn board_prints_every_column_with_what_items_wait_on() {
    let f = file("board_columns", COLUMNS);
    let (out, code) = pmd(&["board", &f]);
    assert_eq!(code, 0);
    let expected = format!(
        "{p}\nWAITING (4)\n  {p}:10  Receipts  owner: Chen  waits on 2027-03-11\n  {p}:12  Review endpoint  owner: Ana  waits on Endpoint\n  {p}:13  Refunds live  owner: Ana  waits on Review endpoint, Receipts\n  {p}:15  Launch  owner: Ana  waits on 2027-04-01\nREADY (1)\n  {p}:11  Refund email  owner: Ben\nDOING (1)\n  {p}:8  Endpoint  owner: Ben  since 2027-03-08\nDONE (1)\n  {p}:7  Refund API  owner: Ana  done 2027-03-05\n  1 older done, not shown\n\nwork in progress: Ben 1\n",
        p = f
    );
    assert_eq!(out, expected);
}

#[test]
fn owner_filter_keeps_only_that_persons_cards_and_wip() {
    let f = file("board_owner", COLUMNS);
    let (out, code) = pmd(&["board", &f, "--owner", "BEN"]);
    assert_eq!(code, 0);
    let expected = format!(
        "{p}\nWAITING (0)\nREADY (1)\n  {p}:11  Refund email  owner: Ben\nDOING (1)\n  {p}:8  Endpoint  owner: Ben  since 2027-03-08\nDONE (0)\n\nwork in progress: Ben 1\n",
        p = f
    );
    assert_eq!(out, expected);
}

#[test]
fn throughput_cycle_time_and_flow_forecast_arithmetic() {
    let f = file("board_flow", FLOW);
    let (out, code) = pmd(&["board", &f]);
    assert_eq!(code, 0);
    let expected = format!(
        "{p}\nWAITING (2)\n  {p}:9  Open task A  owner: Ana  waits on Open task C\n  {p}:12  Launch  owner: Ana  waits on Open task A, Open task B\nREADY (2)\n  {p}:10  Open task B  owner: Ana\n  {p}:11  Open task C  owner: Ana\nDOING (0)\nDONE (3)\n  {p}:5  D1  owner: Ana  done 2027-03-03\n  {p}:6  D2  owner: Ana  done 2027-03-05\n  {p}:7  D3  owner: Ana  done 2027-03-05\n  1 older done, not shown\n\nwork in progress: none\nthroughput: 1.5 items/week (last 19 days)\ncycle time: median 3 days over 3 items\nflow forecast: ^launch Launch: 3 items left at 1.5/week \u{2192} about 2027-03-25 (declared 2027-04-30)\n",
        p = f
    );
    assert_eq!(out, expected);
}

#[test]
fn no_done_items_means_no_throughput_cycle_or_flow_lines() {
    let src = "# T\n\n- Owner: Ann\n\n- [ ] A  owner: Ann  ^a\n- [ ] B  after: ^a  ^b\n- [ ] 2027-04-01 M  after: ^b  ^m\n";
    let f = file("board_no_done", src);
    let (out, code) = pmd(&["board", &f]);
    assert_eq!(code, 0);
    assert!(out.contains("DONE (0)\n"), "{}", out);
    assert!(out.contains("work in progress: none\n"), "{}", out);
    assert!(!out.contains("throughput"), "{}", out);
    assert!(!out.contains("cycle time"), "{}", out);
    assert!(!out.contains("flow forecast"), "{}", out);
}

#[test]
fn json_has_the_columns_and_null_metrics_without_done_items() {
    let f = file("board_json", COLUMNS);
    let (out, code) = pmd(&["board", &f, "--json"]);
    assert_eq!(code, 0);
    let v: serde_json::Value = serde_json::from_str(&out).expect("board --json prints valid JSON");
    assert_eq!(v["file"], f.as_str());
    assert_eq!(v["columns"]["waiting"].as_array().unwrap().len(), 4);
    assert_eq!(v["columns"]["waiting"][1]["waits_on"], serde_json::json!(["Endpoint"]));
    assert_eq!(v["columns"]["ready"][0]["title"], "Refund email");
    assert_eq!(v["columns"]["doing"][0]["since"], "2027-03-08");
    assert_eq!(v["columns"]["done"][0]["done"], "2027-03-05");
    assert_eq!(v["older_done"], 1);
    assert_eq!(v["wip"]["Ben"], 1);
    // one finished item in the window: below the threshold, so no throughput and no flow forecast
    assert!(v["throughput_per_week"].is_null());
    assert_eq!(v["flow_forecast"], serde_json::json!([]));
    assert!(v["cycle_time"].is_null());

    let none = file("board_json_none", "# T\n\n- [ ] A  ^a\n");
    let (out, _) = pmd(&["board", &none, "--json"]);
    let v: serde_json::Value = serde_json::from_str(&out).unwrap();
    assert!(v["throughput_per_week"].is_null());
    assert_eq!(v["flow_forecast"], serde_json::json!([]));
}

#[test]
fn throughput_window_is_the_time_since_the_earliest_record_and_needs_three_items() {
    // earliest record 2027-02-18: the window is the 21 days up to today, three items finished in it
    let three = file("board_window", "# W\n\n- Owner: Ana\n\n- [x] A  done: 2027-02-18\n- [x] B  done: 2027-02-25\n- [x] C  done: 2027-03-04\n- [ ] Open  ^o\n");
    let (out, code) = pmd(&["board", &three]);
    assert_eq!(code, 0);
    assert!(out.contains("throughput: 1.0 items/week (last 3 weeks)\n"), "{}", out);
    // two finished items are below the threshold: no throughput and no flow forecast
    let two = file("board_two", "# W\n\n- Owner: Ana\n\n- [x] A  done: 2027-03-01\n- [x] B  done: 2027-03-04\n- [ ] 2027-04-01 M  after: ^o  ^m\n- [ ] Open  ^o\n");
    let (out, _) = pmd(&["board", &two]);
    assert!(!out.contains("throughput") && !out.contains("flow forecast"), "{}", out);
}
