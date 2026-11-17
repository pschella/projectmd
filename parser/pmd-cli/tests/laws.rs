//! The ownership law: `fmt` and `replan --write` change only the file they are given.
//! The plan generator mirrors pmd-core/tests/laws.rs.

use std::fs;
use std::path::Path;
use std::process::Command;

const PLANS: u64 = 300;
const TODAY: &str = "2027-03-01";

struct Rng(u64);

impl Rng {
    fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn range(&mut self, n: u64) -> u64 {
        self.next() % n
    }
}

struct Plan {
    child: String,
    parent: String,
}

fn plan(seed: u64) -> Plan {
    let mut r = Rng::new(seed + 1);
    let n_ms = 2 + r.range(3) as usize;
    let mut child = String::from("# Child\n\n- Owner: Bob\n");
    child.push_str(&format!("- Commitment: 2027-{:02}-28\n\n", 10 + r.range(2)));
    for i in 0..n_ms {
        let m = 2 + r.range(9);
        child.push_str(&format!("- [ ] 2027-{:02}-{:02} Gate {}  ^m{}\n", m, 1 + r.range(28), i, i));
        if r.range(2) == 0 {
            child.push_str(&format!("- [ ] Work {}  {}w  after: ^m{}\n", i, 1 + r.range(3), i));
        }
    }
    child.push_str("- [ ] Private task  2w\n");
    let target = r.range(n_ms as u64) as usize;
    let parent = format!(
        "# Parent\n\n- Owner: Ann\n\n- [ ] [Child](child.md)  by: 2027-12-31  ^c\n- [ ] [Child gate](child.md#m{})  by: 2027-11-30  ^d\n- [ ] Integrate  1w  after: ^d\n",
        target
    );
    Plan { child, parent }
}

fn run(bin: &Path, dir: &Path, args: &[&str]) {
    let st = Command::new(bin).current_dir(dir).args(args).output().expect("run pmd");
    assert!(st.status.code().is_some(), "pmd {:?} did not exit normally", args);
}

/// Bytes of every file in `dir` except `named`, to check that no other file changed.
fn others(dir: &Path, named: &str) -> Vec<(String, Vec<u8>)> {
    let mut v: Vec<(String, Vec<u8>)> = fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.file_name().to_string_lossy().to_string()))
        .filter(|n| n != named)
        .map(|n| (n.clone(), fs::read(dir.join(&n)).unwrap()))
        .collect();
    v.sort();
    v
}

#[test]
fn ownership_fmt_and_replan_write_only_their_file() {
    let bin = Path::new(env!("CARGO_BIN_EXE_pmd"));
    let root = std::env::temp_dir().join(format!("pmd-laws-{}", std::process::id()));
    for i in 0..PLANS {
        let p = plan(i);
        let dir = root.join(format!("p{}", i));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("parent.md"), &p.parent).unwrap();
        fs::write(dir.join("child.md"), &p.child).unwrap();
        fs::write(dir.join("other.md"), "# Other\n\n- Owner: Cy\n\n- [ ] Work  1w\n").unwrap();

        let before = others(&dir, "parent.md");
        run(bin, &dir, &["fmt", "parent.md"]);
        assert_eq!(others(&dir, "parent.md"), before, "plan {}: fmt parent.md changed another file", i);

        let before = others(&dir, "parent.md");
        run(bin, &dir, &["replan", "parent.md", "--write", "--today", TODAY]);
        assert_eq!(others(&dir, "parent.md"), before, "plan {}: replan --write changed another file", i);

        let before = others(&dir, "child.md");
        run(bin, &dir, &["fmt", "child.md"]);
        assert_eq!(others(&dir, "child.md"), before, "plan {}: fmt child.md changed another file", i);
        fs::remove_dir_all(&dir).ok();
    }
    fs::remove_dir_all(&root).ok();
}
