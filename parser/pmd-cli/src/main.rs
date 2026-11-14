//! `pmd` command line: filesystem access lives here; parsing lives in pmd-core.

use pmd_core::*;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::path::Path;
use std::process::exit;
use std::rc::Rc;
use std::time::{Duration as StdDuration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Default)]
struct Loader {
    graph: Option<Graph>,
    profiles: HashMap<String, Option<Rc<Options>>>,
    read: StdDuration,
    parse: StdDuration,
    files: usize,
    bytes: usize,
    lines: usize,
    unreadable: Vec<(String, String)>,
}

impl Loader {
    fn options_for(&mut self, path: &str) -> Rc<Options> {
        for cand in profile_candidates(path) {
            let e = self.profiles.entry(cand.clone()).or_insert_with(|| {
                fs::read_to_string(&cand).ok().map(|s| Rc::new(parse_profile(&s)))
            });
            if let Some(o) = e {
                return o.clone();
            }
        }
        Rc::new(Options::default())
    }

    /// Load root and every file reachable through sub-project links (and existing local refs).
    fn load(root: &str) -> Loader {
        let mut ld = Loader { graph: Some(Graph::new(root)), ..Default::default() };
        let root = normalize(root);
        let mut seen: HashSet<String> = HashSet::new();
        let mut q: VecDeque<(String, bool)> = VecDeque::new();
        q.push_back((root.clone(), true));
        seen.insert(root);
        while let Some((path, required)) = q.pop_front() {
            let t = Instant::now();
            let bytes = match fs::read(&path) {
                Ok(b) => b,
                Err(e) => {
                    ld.read += t.elapsed();
                    if required {
                        ld.unreadable.push((path, e.to_string()));
                    }
                    continue;
                }
            };
            let src = String::from_utf8_lossy(&bytes);
            ld.read += t.elapsed();
            let opts = ld.options_for(&path);
            let t = Instant::now();
            let proj = parse(&path, &src, &opts);
            ld.parse += t.elapsed();
            ld.files += 1;
            ld.bytes += bytes.len();
            ld.lines += src.bytes().filter(|b| *b == b'\n').count() + usize::from(!src.ends_with('\n') && !src.is_empty());
            for l in project_links(&proj) {
                if l.remote || seen.contains(&l.path) {
                    continue;
                }
                seen.insert(l.path.clone());
                q.push_back((l.path, l.kind == LinkKind::Subproject));
            }
            ld.graph.as_mut().unwrap().add_hashed(proj, hash_prefix(&bytes));
        }
        ld
    }
}

fn all_diags(g: &Graph) -> Vec<Diagnostic> {
    let mut v = Vec::new();
    for p in g.projects() {
        for d in &p.diagnostics {
            let mut d = d.clone();
            d.path = p.path.clone();
            v.push(d);
        }
    }
    v.extend(g.check());
    v.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
    v
}

fn usage() -> ! {
    eprintln!("usage: pmd parse FILE | check ROOT.md [--today D] | schedule FILE [--today D] | lock ROOT.md | tree ROOT.md [--depth N] | stats ROOT.md");
    exit(2)
}

fn today() -> String {
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    Civil::from_days((secs / 86400) as i64).to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() < 2 {
        usage();
    }
    let file = &args[1];
    match args[0].as_str() {
        "parse" => cmd_parse(file),
        "check" => cmd_check(file, today_arg(&args[2..])),
        "schedule" => cmd_schedule(file, today_arg(&args[2..])),
        "lock" => cmd_lock(file),
        "tree" => {
            let mut depth = 1usize;
            let mut i = 2;
            while i < args.len() {
                if args[i] == "--depth" && i + 1 < args.len() {
                    depth = args[i + 1].parse().unwrap_or_else(|_| usage());
                    i += 2;
                } else {
                    usage();
                }
            }
            cmd_tree(file, depth)
        }
        "stats" => cmd_stats(file),
        _ => usage(),
    }
}

fn read_or_die(path: &str) -> String {
    match fs::read(path) {
        Ok(b) => String::from_utf8_lossy(&b).into_owned(),
        Err(e) => {
            eprintln!("pmd: {}: {}", path, e);
            exit(2)
        }
    }
}

fn cmd_parse(file: &str) {
    let src = read_or_die(file);
    let mut ld = Loader::default();
    let opts = ld.options_for(&normalize(file));
    let proj = parse(file, &src, &opts);
    println!("{}", serde_json::to_string_pretty(&proj).unwrap());
}

fn today_arg(rest: &[String]) -> Civil {
    let mut t = today();
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--today" && i + 1 < rest.len() {
            t = rest[i + 1].clone();
            i += 2;
        } else {
            usage();
        }
    }
    match parse_date(&t) {
        DateParse::Ok(c, _) if t.len() == 10 => c,
        _ => {
            eprintln!("pmd: bad date: {}", t);
            exit(2)
        }
    }
}

/// Declared dates of every project known: lock entries first, loaded projects override.
fn interfaces_for(root: &str, g: &Graph) -> Interfaces {
    let mut ifs = Interfaces::new();
    if let Some((dir, lock)) = find_lock(root) {
        for e in &lock.entries {
            let key = if is_remote(&e.location) || dir.is_empty() { normalize(&e.location) } else { normalize(&format!("{}/{}", dir, e.location)) };
            ifs.insert(key, Interface::from_lock_entry(e));
        }
    }
    for p in g.projects() {
        ifs.insert(normalize(&p.path), Interface::from_project(p));
    }
    ifs
}

fn cmd_check(root: &str, today: Civil) {
    let ld = Loader::load(root);
    for (p, e) in &ld.unreadable {
        println!("{}:0: E E-REF-MISSING cannot read file: {}", p, e);
    }
    let g = ld.graph.as_ref().unwrap();
    let mut diags = all_diags(g);
    let ifs = interfaces_for(root, g);
    for p in g.projects() {
        let mut c = p.clone();
        let before = c.diagnostics.len();
        schedule(&mut c, today, &ifs);
        for d in &c.diagnostics[before..] {
            let mut d = d.clone();
            d.path = p.path.clone();
            diags.push(d);
        }
    }
    diags.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
    for d in &diags {
        println!("{}", d);
    }
    for r in g.remote_links() {
        println!("remote: {} (not checked)", r);
    }
    if !ld.unreadable.is_empty() || diags.iter().any(|d| d.is_error()) {
        exit(1);
    }
}

fn span(f: &Forecast) -> String {
    let s = |d: Option<Civil>| d.map(|d| d.to_string()).unwrap_or_else(|| "?".into());
    format!("{} .. {}", s(f.start), s(f.end))
}

fn cmd_schedule(root: &str, today: Civil) {
    let ld = Loader::load(root);
    if !ld.unreadable.is_empty() {
        eprintln!("pmd: cannot read {}", root);
        exit(2);
    }
    let g = ld.graph.as_ref().unwrap();
    let ifs = interfaces_for(root, g);
    let mut p = g.get(root).unwrap().clone();
    schedule(&mut p, today, &ifs);
    let pf = p.forecast.unwrap();
    println!("{}  {}{}", p.title, span(&pf), if pf.conflict { "  !CONFLICT" } else { "" });
    fn walk(nodes: &[Node], level: usize) {
        for n in nodes {
            let f = n.forecast.unwrap();
            let title = n.link.as_ref().map(|l| l.text.clone()).unwrap_or_else(|| n.title.clone());
            let mark = match n.checkbox {
                Checkbox::Done => "[x] ",
                Checkbox::Cancelled => "[-] ",
                _ => "",
            };
            let decl = f.declared_end.map(|d| format!("  (declared {})", d)).unwrap_or_default();
            println!("{}{}{}  {}{}{}", "  ".repeat(level), mark, title, span(&f), decl, if f.conflict { "  !CONFLICT" } else { "" });
            walk(&n.children, level + 1);
        }
    }
    walk(&p.nodes, 1);
    for d in p.diagnostics.iter().filter(|d| matches!(d.code, "E-CONFLICT" | "E-BOUND" | "E-CYCLE")) {
        println!("{}:{}: {} {}", p.path, d.line, d.code, d.message);
    }
}

fn cmd_lock(root: &str) {
    let ld = Loader::load(root);
    if !ld.unreadable.is_empty() {
        eprintln!("pmd: cannot read {}", root);
        exit(2);
    }
    let g = ld.graph.as_ref().unwrap();
    let mut lock = build_lock(root, g);
    lock.set_seen(&today());
    let dir = Path::new(root).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let out = dir.join("pmd.lock");
    if let Err(e) = fs::write(&out, lock.to_string()) {
        eprintln!("pmd: {}: {}", out.display(), e);
        exit(2);
    }
    println!("wrote {} ({} entries)", out.display(), lock.entries.len());
}

fn find_lock(root: &str) -> Option<(String, Lock)> {
    let mut dir = match normalize(root).rfind('/') {
        Some(i) => normalize(root)[..i].to_string(),
        None => String::new(),
    };
    loop {
        let p = if dir.is_empty() { "pmd.lock".to_string() } else { format!("{}/pmd.lock", dir) };
        if let Ok(s) = fs::read_to_string(&p) {
            return Some((dir, Lock::parse(&s)));
        }
        if dir.is_empty() {
            return None;
        }
        dir = match dir.rfind('/') {
            Some(0) => "/".to_string(),
            Some(i) => dir[..i].to_string(),
            None => String::new(),
        };
        if dir == "/" {
            return None;
        }
    }
}

struct TreeCtx {
    lock: Option<(String, Lock)>,
    ld: Loader,
}

fn cmd_tree(root: &str, depth: usize) {
    let mut ctx = TreeCtx { lock: find_lock(root), ld: Loader::default() };
    let path = normalize(root);
    let src = read_or_die(&path);
    let opts = ctx.ld.options_for(&path);
    let proj = parse(&path, &src, &opts);
    tree_project(&mut ctx, &proj, depth, 0, &mut vec![path.clone()]);
}

fn lock_summary(ctx: &TreeCtx, path: &str) -> String {
    if let Some((dir, lock)) = &ctx.lock {
        let rel = if dir.is_empty() { path } else { path.strip_prefix(&format!("{}/", dir)).unwrap_or(path) };
        if let Some(e) = lock.entry(rel) {
            let mut s = String::new();
            if let Some(c) = e.field("commitment") {
                s.push_str(&format!(" commitment={}", c));
            }
            s.push_str(&format!(" public={}", e.milestones.len()));
            return s;
        }
    }
    String::new()
}

fn tree_project(ctx: &mut TreeCtx, p: &Project, depth: usize, level: usize, stack: &mut Vec<String>) {
    let ind = "  ".repeat(level);
    let c = p.commitment.as_ref().map(|c| format!("  commitment={}", c.text)).unwrap_or_default();
    println!("{}{}{}  ({})", ind, p.title, c, p.path);
    tree_nodes(ctx, p, &p.nodes, depth, level + 1, stack);
}

fn tree_nodes(ctx: &mut TreeCtx, p: &Project, nodes: &[Node], depth: usize, level: usize, stack: &mut Vec<String>) {
    let ind = "  ".repeat(level);
    for n in nodes {
        let mut extra = String::new();
        if let Some(d) = &n.date {
            extra.push_str(&format!("  {}", d.text));
        }
        if let Some(r) = &n.range {
            extra.push_str(&format!("  {}", r.text));
        }
        if let Some(d) = &n.duration {
            extra.push_str(&format!("  {}", d.text));
        }
        match n.kind {
            NodeKind::Heading => println!("{}## {}{}", ind, n.title, extra),
            NodeKind::Milestone => println!("{}* {}{}", ind, n.title, extra),
            NodeKind::Task => {
                let m = match n.checkbox {
                    Checkbox::Done => "[x]",
                    Checkbox::Cancelled => "[-]",
                    _ => "[ ]",
                };
                println!("{}{} {}{}", ind, m, n.title, extra)
            }
            NodeKind::Subproject => {
                let link = n.link.as_ref().unwrap();
                let target = resolve(&p.path, &link.target);
                let remote = is_remote(&target);
                println!("{}=> {}{}  -> {}{}", ind, link.text, extra, link.target, lock_summary(ctx, &target));
                if depth > 1 && !remote && !stack.contains(&target) {
                    if let Ok(b) = fs::read(&target) {
                        let src = String::from_utf8_lossy(&b);
                        let opts = ctx.ld.options_for(&target);
                        let child = parse(&target, &src, &opts);
                        stack.push(target.clone());
                        tree_project(ctx, &child, depth - 1, level + 1, stack);
                        stack.pop();
                    }
                }
            }
        }
        tree_nodes(ctx, p, &n.children, depth, level + 1, stack);
    }
}

#[cfg(unix)]
fn peak_rss_kb() -> Option<u64> {
    extern "C" {
        fn getrusage(who: i32, usage: *mut [i64; 18]) -> i32;
    }
    let mut u = [0i64; 18];
    // SAFETY: rusage is 2 timevals + 14 longs on 64-bit unix; the buffer is large enough.
    let r = unsafe { getrusage(0, &mut u) };
    if r != 0 {
        return None;
    }
    let v = u[4].max(0) as u64;
    Some(if cfg!(target_os = "macos") { v / 1024 } else { v })
}
#[cfg(not(unix))]
fn peak_rss_kb() -> Option<u64> {
    None
}

fn cmd_stats(root: &str) {
    let ld = Loader::load(root);
    let g = ld.graph.as_ref().unwrap();
    let t = Instant::now();
    let check = g.check();
    let check_t = t.elapsed();
    let nodes: usize = g.projects().iter().map(|p| p.count_nodes()).sum();
    let pd: usize = g.projects().iter().map(|p| p.diagnostics.len()).sum();
    println!("files:       {}", ld.files);
    println!("lines:       {}", ld.lines);
    println!("bytes:       {}", ld.bytes);
    println!("nodes:       {}", nodes);
    println!("diagnostics: {} ({} parse, {} graph)", pd + check.len(), pd, check.len());
    println!("read:        {:?}", ld.read);
    println!("parse:       {:?}", ld.parse);
    println!("check:       {:?}", check_t);
    if let Some(k) = peak_rss_kb() {
        println!("peak rss:    {} KiB", k);
    }
}
