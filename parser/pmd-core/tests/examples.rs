use pmd_core::*;
use std::path::{Path, PathBuf};

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = std::fs::read_dir(dir) else { return };
    let mut es: Vec<_> = rd.flatten().map(|e| e.path()).collect();
    es.sort();
    for p in es {
        if p.is_dir() {
            walk(&p, out);
        } else if p.extension().map(|e| e == "md").unwrap_or(false) {
            out.push(p);
        }
    }
}

#[test]
fn parse_all_org_examples() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/org");
    let mut files = Vec::new();
    walk(&root, &mut files);
    assert!(!files.is_empty(), "no example files found");
    let mut total = 0;
    let mut g = Graph::new(&root.join("sales/vellmar/vellmar.md").to_string_lossy());
    for f in &files {
        let src = std::fs::read_to_string(f).unwrap();
        let p = parse(&f.to_string_lossy(), &src);
        assert!(!p.title.is_empty(), "{} has no title", f.display());
        for d in p.diagnostics.iter().filter(|d| d.is_error()) {
            eprintln!("DIAG {}:{}: {} {}", f.strip_prefix(&root).unwrap().display(), d.line, d.code, d.message);
            total += 1;
        }
        g.add(p);
    }
    for d in g.check() {
        eprintln!("GRAPH {}:{}: {} {}", d.path, d.line, d.code, d.message);
        total += 1;
    }
    assert_eq!(total, 0, "examples/org must have no parse or graph errors");
}

/// Parse time per file over a generated tree (`go run ./cmd/pmdgen -out DIR ...`):
/// `PMD_BENCH_DIR=DIR cargo test --release -p pmd-core --test examples -- --ignored --nocapture`
#[test]
#[ignore]
fn bench_parse_per_file() {
    let Ok(dir) = std::env::var("PMD_BENCH_DIR") else { return };
    let mut files = Vec::new();
    walk(Path::new(&dir), &mut files);
    let srcs: Vec<String> = files.iter().map(|f| std::fs::read_to_string(f).unwrap()).collect();
    let t = std::time::Instant::now();
    let nodes: usize = srcs.iter().map(|s| parse("f.md", s).nodes.len()).sum();
    let el = t.elapsed();
    eprintln!("parsed {} files ({} top-level nodes) in {:?}: {:?} per file", srcs.len(), nodes, el, el / srcs.len() as u32);
}
