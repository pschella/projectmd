use pmd_core::*;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut es: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    es.sort();
    for p in es {
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().map(|e| e == "md").unwrap_or(false) && !p.components().any(|c| c.as_os_str() == ".pmd") {
            out.push(p);
        }
    }
}

#[test]
fn parse_all_car_examples() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/car");
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(!files.is_empty(), "no example files found");
    let prof = std::fs::read_to_string(root.join(".pmd/profile.md")).map(|s| parse_profile(&s)).unwrap_or_default();
    let mut total = 0;
    let mut g = Graph::new(&root.join("aurora.md").to_string_lossy());
    for f in &files {
        let src = std::fs::read_to_string(f).unwrap();
        let p = parse(&f.to_string_lossy(), &src, &prof);
        assert!(!p.title.is_empty(), "{} has no title", f.display());
        for d in &p.diagnostics {
            eprintln!("DIAG {}:{}: {:?} {} {}", f.strip_prefix(&root).unwrap().display(), d.line, d.severity, d.code, d.message);
            total += 1;
        }
        g.add(p);
    }
    for d in g.check() {
        eprintln!("GRAPH {}:{}: {} {}", d.path, d.line, d.code, d.message);
        total += 1;
    }
    eprintln!("examples: {} files, {} diagnostics", files.len(), total);
}

#[test]
fn aurora_shape() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/car");
    let src = std::fs::read_to_string(root.join("aurora.md")).unwrap();
    let p = parse("aurora.md", &src, &Options::default());
    assert_eq!(p.title, "Aurora — compact electric hatchback");
    assert_eq!(p.id.as_deref(), Some("cars.example.com/aurora"));
    assert_eq!(p.stakeholders.len(), 3);
    let subs = {
        let mut n = 0;
        p.walk(|x| n += (x.kind == NodeKind::Subproject) as usize);
        n
    };
    assert!(subs >= 6);
    assert!(p.find_id("sop").is_some());
}

/// `cargo test --release -- --ignored bench --nocapture`
#[test]
#[ignore]
fn bench_parse_100k() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/car");
    let src = std::fs::read_to_string(root.join("aurora.md")).unwrap();
    let o = Options::default();
    let t = std::time::Instant::now();
    let mut n = 0;
    for _ in 0..100_000 {
        n += parse("aurora.md", &src, &o).nodes.len();
    }
    eprintln!("100k parses of {} bytes: {:?} ({} nodes)", src.len(), t.elapsed(), n);
}
