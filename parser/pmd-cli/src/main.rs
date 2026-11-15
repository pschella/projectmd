//! `pmd` command line: filesystem access lives here; parsing lives in pmd-core.

use pmd_core::*;
use serde_json::json;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fs;
use std::process::exit;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Default)]
struct Loader {
    graph: Option<Graph>,
    unreadable: Vec<(String, String)>,
}

impl Loader {
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
            let bytes = match fs::read(&path) {
                Ok(b) => b,
                Err(e) => {
                    if required {
                        ld.unreadable.push((path, e.to_string()));
                    }
                    continue;
                }
            };
            let src = String::from_utf8_lossy(&bytes);
            let proj = parse(&path, &src);
            for l in project_links(&proj) {
                if depth >= max || l.remote || seen.contains(&l.path) {
                    continue;
                }
                seen.insert(l.path.clone());
                q.push_back((l.path, l.kind == LinkKind::Subproject, depth + 1));
            }
            ld.graph.as_mut().unwrap().add(proj);
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
        "usage: pmd parse FILE | check FILE [--deep] [--strict] [--today D] [--json] | schedule FILE [--today D] [--json] | fmt FILE [--check] | tree FILE [--depth N]"
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
        if matches!(a.as_str(), "--today" | "--depth") {
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
        "check" => cmd_check(file, today_arg(&opts), has_flag(&opts, "--json"), has_flag(&opts, "--deep"), has_flag(&opts, "--strict")),
        "schedule" => cmd_schedule(file, today_arg(&opts), has_flag(&opts, "--json")),
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
    let proj = parse(file, &src);
    println!("{}", serde_json::to_string_pretty(&proj).unwrap());
}

fn has_flag(rest: &[String], flag: &str) -> bool {
    rest.iter().any(|a| a == flag)
}

fn today_arg(rest: &[String]) -> Civil {
    let mut t = today();
    let mut i = 0;
    while i < rest.len() {
        if rest[i] == "--today" && i + 1 < rest.len() {
            t = rest[i + 1].clone();
            i += 2;
        } else if matches!(rest[i].as_str(), "--json" | "--deep" | "--strict") {
            i += 1;
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

/// Declared dates of every project in the loaded graph (read live; nothing is stored).
fn interfaces_for(g: &Graph) -> Interfaces {
    let mut ifs = Interfaces::new();
    for p in g.projects() {
        ifs.insert(normalize(&p.path), Interface::from_project(p));
    }
    ifs
}

/// What the graph knows about a link target: remote targets are not fetched (unknown), a local
/// file that was read is `Read`, a local file that is absent is `E-REF-MISSING`'s business.
fn lookup_in<'a>(g: &'a Graph) -> impl Fn(&str) -> Option<Child<'a>> + 'a {
    move |t: &str| if is_remote(t) { Some(Child::Unknown) } else { g.get(t).map(Child::Read) }
}

fn sev_str(d: &Diagnostic) -> &'static str {
    match d.severity {
        Severity::E => "E",
        Severity::W => "W",
        Severity::I => "I",
    }
}

fn diag_json(d: &Diagnostic) -> serde_json::Value {
    json!({
        "path": d.path,
        "line": d.line,
        "severity": sev_str(d),
        "code": d.code,
        "message": d.message,
        "fix": fix_hint(d.code),
    })
}

/// Default: FILE and its direct links, read live (one level). `--deep`: the
/// whole graph reachable from FILE. `--strict`: warnings fail too (info never does).
fn cmd_check(root: &str, today: Civil, json: bool, deep: bool, strict: bool) {
    let ld = if deep { Loader::load(root) } else { Loader::load_depth(root, 1) };
    let g = ld.graph.as_ref().unwrap();
    let rootn = normalize(root);
    let mut diags = all_diags(g);
    if !deep {
        diags.retain(|d| normalize(&d.path) == rootn);
    }
    let ifs = interfaces_for(g);
    for p in g.projects().iter().filter(|p| deep || normalize(&p.path) == rootn) {
        diags.extend(contract_diags(p, &lookup_in(g)));
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
    let failed = !ld.unreadable.is_empty() || diags.iter().any(|d| d.is_error() || (strict && d.severity == Severity::W));
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
    let ld = Loader::load_depth(root, 1);
    let g = ld.graph.as_ref().unwrap();
    let Some(p0) = g.get(root) else {
        eprintln!("pmd: cannot read {}", root);
        exit(2);
    };
    let ifs = interfaces_for(g);
    let mut p = p0.clone();
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
    for d in p.diagnostics.iter().filter(|d| matches!(d.code, "E-CONFLICT" | "E-CYCLE")) {
        println!("{}:{}: {} {}", p.path, d.line, d.code, d.message);
    }
}

/// `pmd fmt FILE [--check]`: rewrite FILE in canonical form (FORMAT §13).
fn cmd_fmt(file: &str, rest: &[String]) {
    let mut check = false;
    let mut i = 0;
    while i < rest.len() {
        match rest[i].as_str() {
            "--check" => check = true,
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
    let out = fmt::format(&src);
    if check {
        if out != src {
            println!("{}", file);
            exit(1);
        }
        return;
    }
    if out != src {
        if let Err(e) = fs::write(file, out) {
            eprintln!("pmd: {}: {}", file, e);
            exit(2);
        }
    }
}

struct TreeCtx {
    /// Direct links read once, live (`None`: not readable here).
    kids: HashMap<String, Option<Project>>,
}

fn cmd_tree(root: &str, depth: usize) {
    let mut ctx = TreeCtx { kids: HashMap::new() };
    let path = normalize(root);
    let src = read_or_die(&path);
    let proj = parse(&path, &src);
    tree_project(&mut ctx, &proj, depth, 0, &mut vec![path.clone()]);
}

fn read_child(ctx: &mut TreeCtx, target: &str) {
    if ctx.kids.contains_key(target) {
        return;
    }
    let child = if is_remote(target) {
        None
    } else {
        fs::read(target).ok().map(|b| {
            parse(target, &String::from_utf8_lossy(&b))
        })
    };
    ctx.kids.insert(target.to_string(), child);
}

fn tree_project(ctx: &mut TreeCtx, p: &Project, depth: usize, level: usize, stack: &mut Vec<String>) {
    let ind = "  ".repeat(level);
    let c = p.commitment.as_ref().map(|c| format!("  commitment={}", c.text)).unwrap_or_default();
    println!("{}{}{}  ({})", ind, p.title, c, p.path);
    let mut targets = Vec::new();
    p.walk(|n| {
        if let (true, Some(l)) = (n.is_link_node(), &n.link) {
            targets.push(resolve(&p.path, &l.target));
        }
    });
    for t in &targets {
        read_child(ctx, t);
    }
    let states: HashMap<u32, State> = {
        let kids = &ctx.kids;
        let lookup = |t: &str| -> Option<Child<'_>> { if is_remote(t) { Some(Child::Unknown) } else { kids.get(t).and_then(|k| k.as_ref()).map(Child::Read) } };
        contracts_of(p, &lookup).into_iter().map(|c| (c.line, c.state)).collect()
    };
    tree_nodes(ctx, p, &p.nodes, &states, depth, level + 1, stack);
}

fn contract_summary(ctx: &TreeCtx, states: &HashMap<u32, State>, n: &Node, target: &str) -> String {
    let mut s = String::new();
    if let Some(Some(c)) = ctx.kids.get(target) {
        if let Some(cm) = &c.commitment {
            s.push_str(&format!(" commitment={}", cm.text));
        }
    }
    if let Some(st) = states.get(&n.line) {
        s.push_str(&format!(" {}", st.as_str()));
    }
    s
}

fn tree_nodes(ctx: &mut TreeCtx, p: &Project, nodes: &[Node], states: &HashMap<u32, State>, depth: usize, level: usize, stack: &mut Vec<String>) {
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
            NodeKind::Deliverable => {
                let link = n.link.as_ref().unwrap();
                let target = resolve(&p.path, &link.target);
                println!("{}<> {}{}  -> {}{}", ind, link.text, extra, link.target, contract_summary(ctx, states, n, &target));
            }
            NodeKind::Subproject => {
                let link = n.link.as_ref().unwrap();
                let target = resolve(&p.path, &link.target);
                println!("{}=> {}{}  -> {}{}", ind, link.text, extra, link.target, contract_summary(ctx, states, n, &target));
                if depth > 1 && !is_remote(&target) && !stack.contains(&target) {
                    if let Some(Some(child)) = ctx.kids.get(&target).cloned() {
                        stack.push(target.clone());
                        tree_project(ctx, &child, depth - 1, level + 1, stack);
                        stack.pop();
                    }
                }
            }
        }
        tree_nodes(ctx, p, &n.children, states, depth, level + 1, stack);
    }
}
