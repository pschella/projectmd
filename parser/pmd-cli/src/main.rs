//! `pmd` command line: filesystem access lives here; parsing lives in pmd-core.

use pmd_core::*;
use serde_json::json;
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
        Loader::load_depth(root, usize::MAX)
    }

    /// Like `load`, but stop `max` links away from root (1 = root and its direct links).
    fn load_depth(root: &str, max: usize) -> Loader {
        let mut ld = Loader { graph: Some(Graph::new(root)), ..Default::default() };
        let root = normalize(root);
        let mut seen: HashSet<String> = HashSet::new();
        let mut q: VecDeque<(String, bool, usize)> = VecDeque::new();
        q.push_back((root.clone(), true, 0));
        seen.insert(root);
        while let Some((path, required, depth)) = q.pop_front() {
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
                if depth >= max || l.remote || seen.contains(&l.path) {
                    continue;
                }
                seen.insert(l.path.clone());
                q.push_back((l.path, l.kind == LinkKind::Subproject, depth + 1));
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
    eprintln!(
        "usage: pmd parse FILE | check FILE [--deep] [--today D] [--json] | schedule FILE [--today D] [--json] | lock FILE [--if-hash HASH] | fmt FILE [--ids] [--check] [--if-hash HASH] | tree FILE [--depth N] | stats FILE"
    );
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
    // FILE may come before or after the options.
    let (mut file, mut opts) = (None, Vec::new());
    let mut it = args[1..].iter();
    while let Some(a) = it.next() {
        if matches!(a.as_str(), "--today" | "--depth" | "--if-hash") {
            opts.push(a.clone());
            opts.extend(it.next().cloned());
        } else if a.starts_with("--") || file.is_some() {
            opts.push(a.clone());
        } else {
            file = Some(a.clone());
        }
    }
    let Some(file) = file else { usage() };
    let file = &file;
    match args[0].as_str() {
        "parse" => cmd_parse(file),
        "check" => cmd_check(file, today_arg(&opts), has_flag(&opts, "--json"), has_flag(&opts, "--deep")),
        "schedule" => cmd_schedule(file, today_arg(&opts), has_flag(&opts, "--json")),
        "lock" => cmd_lock(file, if_hash_arg(&opts)),
        "fmt" => cmd_fmt(file, &opts),
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

fn has_flag(rest: &[String], flag: &str) -> bool {
    rest.iter().any(|a| a == flag)
}

/// Value of `--if-hash HASH` (exits with usage when the value is missing or empty).
fn if_hash_arg(rest: &[String]) -> Option<String> {
    let mut out = None;
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--if-hash" && i + 1 < rest.len() && !rest[i + 1].is_empty() {
            out = Some(rest[i + 1].to_ascii_lowercase());
            i += 2;
        } else {
            i += 1;
        }
    }
    out
}

/// Exit 3 unless the hash of `bytes` starts with `want` (a hash or a prefix of one).
fn check_hash(path: &str, bytes: &[u8], want: &str) {
    let have = hash_prefix(bytes);
    if !have.starts_with(want) {
        eprintln!("pmd: {} changed (hash {}), re-read", path, have);
        exit(3)
    }
}

fn today_arg(rest: &[String]) -> Civil {
    let mut t = today();
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--today" && i + 1 < rest.len() {
            t = rest[i + 1].clone();
            i += 2;
        } else if rest[i] == "--json" || rest[i] == "--deep" {
            i += 1;
        } else if rest[i] == "--if-hash" && i + 1 < rest.len() {
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

/// Key of a lock entry's location, as the scheduler looks it up (remote URLs stay as they are).
fn iface_key(dir: &str, loc: &str) -> String {
    if is_remote(loc) {
        loc.to_string()
    } else if dir.is_empty() {
        normalize(loc)
    } else {
        normalize(&format!("{}/{}", dir, loc))
    }
}

/// Declared dates of every project known: FILE's own lock entries (their `late` included),
/// overridden by the loaded projects, which keep the `late` of their lock entry.
fn interfaces_for(root: &str, g: &Graph) -> Interfaces {
    let mut ifs = Interfaces::new();
    let own = own_lock(root);
    if let Some((dir, lock)) = &own {
        for e in &lock.entries {
            ifs.insert(iface_key(dir, &e.location), Interface::from_lock_entry(e));
        }
    }
    for p in g.projects() {
        let key = normalize(&p.path);
        let mut i = Interface::from_project(p);
        if let Some((dir, lock)) = &own {
            if let Some(e) = lock.entries.iter().find(|e| iface_key(dir, &e.location) == key) {
                i.late = Interface::from_lock_entry(e).late;
            }
        }
        ifs.insert(key, i);
    }
    ifs
}

fn diag_json(d: &Diagnostic) -> serde_json::Value {
    json!({
        "path": d.path,
        "line": d.line,
        "severity": if d.is_error() { "E" } else { "W" },
        "code": d.code,
        "message": d.message,
        "fix": fix_hint(d.code),
    })
}

/// W-STALE-LOCK for each direct local link whose file differs from FILE's lock (or is missing
/// from it). Only checked when FILE has a lock.
fn stale_lock_diags(root: &str, g: &Graph) -> Vec<Diagnostic> {
    let mut out = Vec::new();
    let (Some((dir, lock)), Some(p)) = (own_lock(root), g.get(&normalize(root))) else { return out };
    let lockname = lock_path(root);
    let lockname = Path::new(&lockname).file_name().map(|f| f.to_string_lossy().into_owned()).unwrap_or_default();
    let mut seen = HashSet::new();
    for l in project_links(p) {
        if l.remote || !seen.insert(l.path.clone()) {
            continue;
        }
        let Some(h) = g.hash(&l.path) else { continue };
        let e = lock.entries.iter().find(|e| iface_key(&dir, &e.location) == l.path);
        if e.and_then(|e| e.field("hash")) != Some(h) {
            let mut d = Diagnostic::new(Severity::W, "W-STALE-LOCK", l.line, format!("`{}` is out of date for `{}`", lockname, l.path));
            d.path = p.path.clone();
            out.push(d);
        }
    }
    out
}

/// Default: FILE, its own lock and its direct links (one level, like `lock`). `--deep`: the
/// whole graph reachable from FILE.
fn cmd_check(root: &str, today: Civil, json: bool, deep: bool) {
    let ld = if deep { Loader::load(root) } else { Loader::load_depth(root, 1) };
    let g = ld.graph.as_ref().unwrap();
    let rootn = normalize(root);
    let mut diags = all_diags(g);
    if !deep {
        diags.retain(|d| normalize(&d.path) == rootn);
    }
    diags.extend(stale_lock_diags(root, g));
    let ifs = interfaces_for(root, g);
    for p in g.projects().iter().filter(|p| deep || normalize(&p.path) == rootn) {
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
    let failed = !ld.unreadable.is_empty() || diags.iter().any(|d| d.is_error());
    if json {
        let mut list: Vec<serde_json::Value> = ld
            .unreadable
            .iter()
            .map(|(p, e)| json!({ "path": p, "line": 0, "severity": "E", "code": "E-REF-MISSING", "message": format!("cannot read file: {}", e), "fix": null }))
            .collect();
        list.extend(diags.iter().map(diag_json));
        println!("{}", json!({ "diagnostics": list, "remote": g.remote_links(), "ok": !failed }));
    } else {
        for (p, e) in &ld.unreadable {
            println!("{}:0: E E-REF-MISSING cannot read file: {}", p, e);
        }
        for d in &diags {
            println!("{}", d);
            if let Some(f) = fix_hint(d.code) {
                println!("  fix: {}", f);
            }
        }
        for r in g.remote_links() {
            println!("remote: {} (not checked)", r);
        }
    }
    if failed {
        exit(1);
    }
}

fn span(f: &Forecast) -> String {
    let s = |d: Option<Civil>| d.map(|d| d.to_string()).unwrap_or_else(|| "?".into());
    format!("{} .. {}", s(f.start), s(f.end))
}

fn cmd_schedule(root: &str, today: Civil, json: bool) {
    let ld = Loader::load(root);
    if !ld.unreadable.is_empty() {
        eprintln!("pmd: cannot read {}", root);
        exit(2);
    }
    let g = ld.graph.as_ref().unwrap();
    let ifs = interfaces_for(root, g);
    let mut p = g.get(root).unwrap().clone();
    schedule(&mut p, today, &ifs);
    if json {
        // The same Project JSON as `pmd parse`, with the forecasts the scheduler filled in.
        println!("{}", serde_json::to_string_pretty(&p).unwrap());
        return;
    }
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

/// Path of the lock of a project file: `NAME.md` -> `NAME.lock`.
fn lock_path(md: &str) -> String {
    Path::new(md).with_extension("lock").to_string_lossy().into_owned()
}

/// Build and write FILE's lock (`NAME.lock` beside it). Reads FILE, each direct local link and
/// that link's own lock; nothing deeper.
fn cmd_lock(file: &str, if_hash: Option<String>) {
    let path = normalize(file);
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("pmd: cannot read {}: {}", file, e);
            exit(2)
        }
    };
    let mut ld = Loader { graph: Some(Graph::new(&path)), ..Default::default() };
    let opts = ld.options_for(&path);
    let proj = parse(&path, &String::from_utf8_lossy(&bytes), &opts);
    let mut direct: Vec<String> = project_links(&proj).into_iter().filter(|l| !l.remote).map(|l| l.path).collect();
    direct.sort();
    direct.dedup();
    ld.graph.as_mut().unwrap().add_hashed(proj, hash_prefix(&bytes));

    let mut child_locks: HashMap<String, Lock> = HashMap::new();
    for t in direct.iter().filter(|t| **t != path) {
        match fs::read(t) {
            Ok(b) => {
                let opts = ld.options_for(t);
                let child = parse(t, &String::from_utf8_lossy(&b), &opts);
                ld.graph.as_mut().unwrap().add_hashed(child, hash_prefix(&b));
            }
            Err(e) => eprintln!("pmd: warning: cannot read {} ({}); keeping its previous entry", t, e),
        }
        if let Ok(s) = fs::read_to_string(lock_path(t)) {
            child_locks.insert(t.clone(), Lock::parse(&s));
        }
    }

    let out = lock_path(&path);
    let prev = fs::read_to_string(&out).ok().map(|s| Lock::parse(&s));
    let lock = build_lock(&path, ld.graph.as_ref().unwrap(), &child_locks, prev.as_ref(), today_arg(&[]));
    if let Some(h) = &if_hash {
        // The lock being overwritten is the version this write is based on (missing = empty).
        check_hash(&out, &fs::read(&out).unwrap_or_default(), h);
    }
    if let Err(e) = fs::write(&out, lock.to_string()) {
        eprintln!("pmd: {}: {}", out, e);
        exit(2);
    }
    println!("wrote {} ({} entries)", out, lock.entries.len());
}

/// `pmd fmt FILE [--ids] [--check] [--if-hash HASH]`: rewrite FILE in canonical form (FORMAT §14).
fn cmd_fmt(file: &str, rest: &[String]) {
    let (mut ids, mut check) = (false, false);
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--ids" => ids = true,
            "--check" => check = true,
            "--if-hash" if i + 1 < rest.len() => i += 1,
            _ => usage(),
        }
        i += 1;
    }
    let bytes = match fs::read(file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("pmd: {}: {}", file, e);
            exit(2)
        }
    };
    let src = String::from_utf8_lossy(&bytes).into_owned();
    let mut ld = Loader::default();
    let opts = ld.options_for(&normalize(file));
    let out = fmt::format_with(&src, ids, &opts);
    if check {
        if out != src {
            println!("{}", file);
            exit(1);
        }
        return;
    }
    if let Some(h) = if_hash_arg(rest) {
        check_hash(file, &bytes, &h);
    }
    if out != src {
        if let Err(e) = fs::write(file, out) {
            eprintln!("pmd: {}: {}", file, e);
            exit(2);
        }
    }
}

/// FILE's own lock (`NAME.lock` beside FILE) with the folder its locations are relative to.
fn own_lock(file: &str) -> Option<(String, Lock)> {
    let n = normalize(file);
    let dir = match n.rfind('/') {
        Some(0) => "/".to_string(),
        Some(i) => n[..i].to_string(),
        None => String::new(),
    };
    let s = fs::read_to_string(lock_path(&n)).ok()?;
    Some((dir, Lock::parse(&s)))
}

struct TreeCtx {
    lock: Option<(String, Lock)>,
    ld: Loader,
}

fn cmd_tree(root: &str, depth: usize) {
    let mut ctx = TreeCtx { lock: own_lock(root), ld: Loader::default() };
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
