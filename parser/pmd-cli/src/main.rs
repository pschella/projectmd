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
                    if required && depth == 0 {
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

const USAGE: &str = "usage: pmd parse FILE | check FILE... [--deep] [--strict] [--today D] [--json] | schedule FILE... [--today D] [--json] | ready FILE... [--deep] [--owner NAME] [--today D] [--json] | board FILE... [--deep] [--owner NAME] [--today D] [--json] | why FILE [^id] [--deep] [--today D] [--json] | changes OLD NEW [--json] | replan FILE [--write] [--today D] [--json] | fmt FILE... [--check] | tree FILE... [--depth N] [--today D]";

const HELP: &str = "\
commands:
  parse FILE       print the parsed project as JSON
  check FILE...    report diagnostics (--deep: whole link graph, --strict: warnings fail, --json)
  schedule FILE... print the forecast schedule (--json)
  ready FILE...    the queue: do (work that can start today), decide (disputed link: accept, re-plan or
                   talk), fix (unreadable link); with --deep or several FILES also answer (a child has
                   not answered `by:`) and deliver (overdue), listed under the child's Owner
                   (--deep, --owner NAME, --json)
  board FILE...    the board, one file at a time (informative): waiting (with what it waits on), ready,
                   doing (with its start) and done (last 14 days), in file order; the footer gives work
                   in progress per owner, throughput, median cycle time and a flow forecast for each open
                   milestone that waits on open items. --owner filters the cards, --deep loads links
                   (--deep, --owner NAME, --json; several FILES give a JSON array)
  why FILE [^id]   what fixes the date of an item (default: the project end and every conflict):
                   one hop per line back to the root cause; --deep follows links into child files
  changes OLD NEW  the events between two versions of one file: promises (for parents), asks (for the
                   child named by a link) and own items done or cancelled (--json)
  replan FILE      propose the smallest edits to this file's own dates that make it consistent: accept
                   disputed promises (`by:`), move overrun ends, move the Commitment; a dry run unless
                   --write; lists the promises that would move (--write, --json)
  fmt FILE...      rewrite files in canonical form (--check: only report, exit 1 if any would change)
  tree FILE...     print the project tree (--depth N)

options:
  --today D        use date D (YYYY-MM-DD) instead of today (also: PMD_TODAY)
  --write          replan: apply the edits to FILE (only when there are some)
  -h, --help       print this help
  -V, --version    print the version

exit codes:
  0  success, nothing found
  1  findings: check errors (or warnings with --strict), fmt --check would change a file
  2  usage error, unreadable file, or other I/O error";

fn usage() -> ! {
    eprintln!("{}", USAGE);
    exit(2)
}

fn today() -> String {
    if let Ok(t) = std::env::var("PMD_TODAY") {
        if !t.is_empty() {
            return t;
        }
    }
    let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
    Civil::from_days((secs / 86400) as i64).to_string()
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("--version" | "-V") => {
            println!("pmd {}", env!("CARGO_PKG_VERSION"));
            return;
        }
        Some("--help" | "-h" | "help") => {
            println!("{}\n\n{}", USAGE, HELP);
            return;
        }
        _ => {}
    }
    if args.len() < 2 {
        usage();
    }
    // FILEs may come before or after the options; check, schedule and tree take several.
    let (mut files, mut opts) = (Vec::new(), Vec::new());
    let mut it = args[1..].iter();
    while let Some(a) = it.next() {
        if let Some((k, v)) = a.split_once('=').filter(|(k, _)| matches!(*k, "--today" | "--depth" | "--owner")) {
            opts.push(k.to_string());
            opts.push(v.to_string());
        } else if matches!(a.as_str(), "--today" | "--depth" | "--owner") {
            opts.push(a.clone());
            opts.extend(it.next().cloned());
        } else if matches!(a.as_str(), "--deep" | "--strict" | "--json" | "--check" | "--write") {
            opts.push(a.clone());
        } else if a.starts_with('-') && a.len() > 1 {
            usage();
        } else {
            files.push(a.clone());
        }
    }
    if files.is_empty() || (files.len() > 1 && !matches!(args[0].as_str(), "check" | "schedule" | "tree" | "fmt" | "ready" | "board" | "why" | "changes")) {
        usage();
    }
    if args[0] == "changes" && files.len() != 2 {
        usage();
    }
    let file = &files[0];
    let multi = |f: &dyn Fn(&str) -> i32| -> ! { exit(files.iter().map(|x| f(x)).max().unwrap_or(0)) };
    match args[0].as_str() {
        "parse" => cmd_parse(file),
        "check" => multi(&|f| cmd_check(f, today_arg(&opts), has_flag(&opts, "--json"), has_flag(&opts, "--deep"), has_flag(&opts, "--strict"))),
        "schedule" => multi(&|f| cmd_schedule(f, today_arg(&opts), has_flag(&opts, "--json"))),
        "changes" => exit(cmd_changes(&files[0], &files[1], has_flag(&opts, "--json"))),
        "replan" => exit(cmd_replan(file, today_arg(&opts), has_flag(&opts, "--write"), has_flag(&opts, "--json"))),
        "ready" => cmd_ready(&files, today_arg(&opts), opts.iter().position(|o| o == "--owner").and_then(|i| opts.get(i + 1)).map(|s| s.as_str()), has_flag(&opts, "--json"), has_flag(&opts, "--deep")),
        "board" => cmd_board(&files, today_arg(&opts), opts.iter().position(|o| o == "--owner").and_then(|i| opts.get(i + 1)).map(|s| s.as_str()), has_flag(&opts, "--json"), has_flag(&opts, "--deep")),
        "why" => {
            let id = files.get(1).map(|i| i.strip_prefix('^').unwrap_or_else(|| usage()));
            if files.len() > 2 {
                usage();
            }
            exit(cmd_why(file, id, today_arg(&opts), has_flag(&opts, "--json"), has_flag(&opts, "--deep")))
        }
        "fmt" => cmd_fmt(&args[1..]),
        "tree" => {
            let mut depth = 1usize;
            let mut i = 0;
            while i < opts.len() {
                if opts[i] == "--depth" && i + 1 < opts.len() {
                    depth = opts[i + 1].parse().unwrap_or_else(|_| usage());
                    i += 2;
                } else if opts[i] == "--today" {
                    i += 2;
                } else {
                    usage();
                }
            }
            multi(&|f| cmd_tree(f, depth, today_arg(&opts)))
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
        } else if matches!(rest[i].as_str(), "--depth" | "--owner") {
            i += 2;
        } else if matches!(rest[i].as_str(), "--json" | "--deep" | "--strict" | "--write") {
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
fn interfaces_for(g: &Graph, today: Civil) -> Interfaces {
    let mut ifs = Interfaces::new();
    for p in g.projects() {
        ifs.insert(normalize(&p.path), Interface::from_project(p));
    }
    let ps: Vec<(String, &Project)> = g.projects().iter().map(|p| (normalize(&p.path), p)).collect();
    add_forecasts(&mut ifs, &ps, today);
    ifs
}

/// What the graph knows about a link target: remote targets are not fetched (unknown), a local
/// file that was read is `Read`, a local file that is absent is `E-REF-MISSING`'s business.
fn lookup_in<'a>(g: &'a Graph) -> impl Fn(&str) -> Option<Child<'a>> + 'a {
    move |t: &str| if is_remote(t) { Some(Child::Unknown) } else { g.get(t).map(Child::Read) }
}

/// The graph with every file scheduled: contract targets that are plain items (same owner)
/// promise their forecast end.
fn scheduled(g: &Graph, ifs: &Interfaces, today: Civil) -> Graph {
    let mut sg = Graph::new(g.root());
    for p in g.projects() {
        let mut c = p.clone();
        schedule(&mut c, today, ifs);
        sg.add(c);
    }
    sg
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
        "fix": fix_for(d),
    })
}

/// Default: FILE and its direct links, read live (one level). `--deep`: the
/// whole graph reachable from FILE. `--strict`: warnings fail too (info never does).
fn cmd_check(root: &str, today: Civil, json: bool, deep: bool, strict: bool) -> i32 {
    let ld = if deep { Loader::load(root) } else { Loader::load_depth(root, 1) };
    let g = ld.graph.as_ref().unwrap();
    let rootn = normalize(root);
    let mut diags = all_diags(g);
    if !deep {
        diags.retain(|d| normalize(&d.path) == rootn);
    }
    let ifs = interfaces_for(g, today);
    let sg = scheduled(g, &ifs, today);
    for p in g.projects().iter().filter(|p| deep || normalize(&p.path) == rootn) {
        diags.extend(contract_diags(p, today, &lookup_in(&sg)));
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
            if let Some(f) = fix_for(d) {
                println!("  fix: {}", f);
            }
        }
        for r in g.remote_links() {
            println!("remote: {} (not checked)", r);
        }
    }
    failed as i32
}

fn span(f: &Forecast) -> String {
    let s = |d: Option<Civil>| d.map(|d| d.to_string()).unwrap_or_else(|| "?".into());
    format!("{} .. {}", s(f.start), s(f.end))
}

fn cmd_schedule(root: &str, today: Civil, json: bool) -> i32 {
    let ld = Loader::load_depth(root, 1);
    let g = ld.graph.as_ref().unwrap();
    let Some(p0) = g.get(root) else {
        eprintln!("pmd: cannot read {}", root);
        return 2;
    };
    let ifs = interfaces_for(g, today);
    let mut p = p0.clone();
    schedule(&mut p, today, &ifs);
    let est = estimate_record(&p);
    if json {
        // The same Project JSON as `pmd parse`, with the forecasts the scheduler filled in.
        let mut v = serde_json::to_value(&p).unwrap();
        if let Some(r) = est {
            v["estimate_record"] = json!(r);
        }
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        return 0;
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
            let late = match (n.checkbox, n.done_date(), f.declared_end) {
                (Checkbox::Done, Some(d), Some(e)) if d > e => Some(d.to_days() - e.to_days()),
                _ => None,
            };
            // slack is printed when something was forecast (estimate, after:, sub-items) or it is nonzero
            let sourced = n.duration.is_some() || n.keys.contains_key("after") || !n.children.is_empty();
            let decl = match f.slack_days {
                _ if late.is_some() => format!("{}  done {}d late", decl, late.unwrap()),
                Some(s) if f.declared_end.is_some() && (s != 0 || sourced) => format!("{}  slack {:+}d", decl, s),
                _ => decl,
            };
            println!("{}{}{}  {}{}{}", "  ".repeat(level), mark, title, span(&f), decl, if f.conflict { "  !CONFLICT" } else { "" });
            walk(&n.children, level + 1);
        }
    }
    walk(&p.nodes, 1);
    for d in p.diagnostics.iter().filter(|d| matches!(d.code, "E-CONFLICT" | "E-CYCLE")) {
        println!("{}:{}: {} {}", p.path, d.line, d.code, d.message);
    }
    if let Some(r) = est {
        println!("estimates: \u{d7}{:.1} over {} finished items", r.median_ratio, r.n);
    }
    0
}

/// `pmd ready FILE...`: the queue (D59): work that may start now plus the contract moves owed.
fn cmd_ready(files: &[String], today: Civil, owner: Option<&str>, json: bool, deep: bool) -> ! {
    let (mut all, mut asks, mut seen, mut code) = (Vec::new(), Vec::new(), HashSet::new(), 0);
    for f in files {
        let ld = if deep { Loader::load(f) } else { Loader::load_depth(f, 1) };
        let g = ld.graph.as_ref().unwrap();
        let Some(p0) = g.get(f) else {
            eprintln!("pmd: cannot read {}", f);
            code = 2;
            continue;
        };
        let ifs = interfaces_for(g, today);
        let sg = scheduled(g, &ifs, today);
        let todo: Vec<&Project> = if deep { std::iter::once(p0).chain(g.projects().iter().filter(|p| normalize(&p.path) != normalize(f))).collect() } else { vec![p0] };
        for p0 in todo {
            if !seen.insert(normalize(&p0.path)) {
                continue;
            }
            let mut p = p0.clone();
            schedule(&mut p, today, &ifs);
            all.extend(ready(&p, today, owner, &lookup_in(&sg)));
            if deep || files.len() > 1 {
                asks.extend(child_moves(p0, today, owner, &lookup_in(&sg)));
            }
        }
    }
    // the children's moves: one entry per promise, however many askers point at it
    all.extend(merge_asks(asks));
    if json {
        let v: Vec<_> = all
            .iter()
            .map(|r| {
                let mut v = json!({ "file": r.path, "line": r.line, "kind": r.kind, "title": r.title, "owner": r.owner, "node": if r.node == NodeKind::Milestone { "milestone" } else { "task" }, "detail": r.detail, "due": r.due.map(|d| d.to_string()), "declared": r.declared.map(|d| d.to_string()), "started": r.started.map(|d| d.to_string()), "askers": r.askers });
                if let Some(rec) = &r.record {
                    v["record"] = json!(rec);
                }
                v
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
    } else {
        for r in &all {
            let part = |k: &str, v: Option<String>| v.map(|v| format!("  {}{}", k, v)).unwrap_or_default();
            let owner = Some(r.owner.clone()).filter(|o| !o.is_empty());
            println!("{}:{}  {}  {}{}{}{}{}", r.path, r.line, r.kind.to_uppercase(), r.title, part("owner: ", owner), part("due: ", r.due.map(|d| d.to_string())), part("started: ", r.started.map(|d| d.to_string())), part("", Some(r.detail.clone()).filter(|d| !d.is_empty())));
        }
        if !all.is_empty() {
            println!();
        }
        println!("{} ready ({} file{} read)", all.len(), seen.len(), if seen.len() == 1 { "" } else { "s" });
    }
    exit(code)
}

/// `pmd board FILE... [--deep] [--owner NAME] [--json]`: one file as four columns (D66, informative).
fn cmd_board(files: &[String], today: Civil, owner: Option<&str>, json: bool, deep: bool) -> ! {
    let (mut boards, mut code) = (Vec::new(), 0);
    for f in files {
        let ld = if deep { Loader::load(f) } else { Loader::load_depth(f, 1) };
        let g = ld.graph.as_ref().unwrap();
        let Some(p0) = g.get(f) else {
            eprintln!("pmd: cannot read {}", f);
            code = 2;
            continue;
        };
        let ifs = interfaces_for(g, today);
        let sg = scheduled(g, &ifs, today);
        let mut p = p0.clone();
        schedule(&mut p, today, &ifs);
        boards.push(board(&p, today, owner, &lookup_in(&sg)));
    }
    if json {
        let mut v: Vec<_> = boards.iter().map(board_json).collect();
        let out = if files.len() == 1 && v.len() == 1 { v.remove(0) } else { serde_json::Value::Array(v) };
        println!("{}", serde_json::to_string_pretty(&out).unwrap());
    } else {
        for (i, b) in boards.iter().enumerate() {
            if i > 0 {
                println!();
            }
            print_board(b);
        }
    }
    exit(code)
}

/// `3 weeks`, `1 week`, or `19 days` when the window is not a whole number of weeks.
fn window_label(days: i64) -> String {
    match days {
        d if d % 7 == 0 && d / 7 == 1 => "1 week".to_string(),
        d if d % 7 == 0 => format!("{} weeks", d / 7),
        d => format!("{} days", d),
    }
}


fn print_board(b: &Board) {
    println!("{}", b.path);
    let card = |c: &Card, detail: Option<String>| {
        let owner = Some(c.owner.as_str()).filter(|o| !o.is_empty()).map(|o| format!("  owner: {}", o)).unwrap_or_default();
        let detail = detail.map(|d| format!("  {}", d)).unwrap_or_default();
        println!("  {}:{}  {}{}{}", b.path, c.line, c.title, owner, detail);
    };
    println!("WAITING ({})", b.waiting.len());
    for c in &b.waiting {
        card(c, Some(format!("waits on {}", c.waits_on.join(", "))));
    }
    println!("READY ({})", b.ready.len());
    for c in &b.ready {
        card(c, None);
    }
    println!("DOING ({})", b.doing.len());
    for c in &b.doing {
        card(c, c.since.map(|s| format!("since {}", s)));
    }
    println!("DONE ({})", b.done.len());
    for c in &b.done {
        card(c, c.done.map(|d| format!("done {}", d)));
    }
    if b.older_done > 0 {
        println!("  {} older done, not shown", b.older_done);
    }
    println!();
    let wip: Vec<String> = b.wip.iter().map(|(o, n)| format!("{} {}", owner_label(o), n)).collect();
    println!("work in progress: {}", if wip.is_empty() { "none".to_string() } else { wip.join(", ") });
    if let Some(tp) = b.throughput {
        println!("throughput: {:.1} items/week (last {})", tp, window_label(b.throughput_days));
    }
    if let Some(c) = b.cycle_time {
        println!("cycle time: median {} days over {} items", c.median_days, c.n);
    }
    if let Some(tp) = b.throughput {
        for f in &b.flow {
            let id = f.id.as_ref().map(|i| format!("^{}", i)).unwrap_or_else(|| format!("line {}", f.line));
            let declared = f.declared.map(|d| format!(" (declared {})", d)).unwrap_or_default();
            println!("flow forecast: {} {}: {} items left at {:.1}/week \u{2192} about {}{}", id, f.title, f.items_left, tp, f.about, declared);
        }
    }
}

/// `pmd changes OLD NEW [--json]`: the events from one file's old version to its new one (D61).
fn cmd_changes(old: &str, new: &str, json: bool) -> i32 {
    let (a, b) = (parse(old, &read_or_die(old)), parse(new, &read_or_die(new)));
    let evs = changes(&a, &b);
    if json {
        println!("{}", serde_json::to_string_pretty(&evs).unwrap());
        return 0;
    }
    for c in &evs {
        // a `by:` change shows its label once: `by: OLD → NEW`
        let by = c.old.as_deref().or(c.new.as_deref()).is_some_and(|v| v.starts_with("by: "));
        let label = if by { "by: " } else { "" };
        let date = |v: &String| v.strip_prefix("by: ").unwrap_or(v).to_string();
        let vals = match (&c.old, &c.new, c.delta_days) {
            (Some(o), Some(n), Some(d)) => format!("{}{} \u{2192} {} ({:+}d)", label, date(o), date(n), d),
            (Some(o), Some(n), None) => format!("{}{} \u{2192} {}", label, date(o), date(n)),
            (Some(v), None, _) | (None, Some(v), _) => format!("{}{}", label, date(v)),
            (None, None, _) => String::new(),
        };
        let parts = [c.kind.to_string(), c.subject.clone(), vals, format!("[{}]", c.audience)];
        println!("{}", parts.iter().filter(|x| !x.is_empty()).cloned().collect::<Vec<_>>().join("  "));
    }
    println!("{}", if evs.is_empty() { "no changes".to_string() } else { format!("{} changes", evs.len()) });
    0
}

/// `pmd replan FILE [--write] [--today D] [--json]`: the edits that make FILE consistent (D62).
fn cmd_replan(file: &str, today: Civil, write: bool, json: bool) -> i32 {
    let src = read_or_die(file);
    let path = normalize(file);
    let ld = Loader::load_depth(file, 1);
    let g = ld.graph.as_ref().unwrap();
    // each pass: this text in place of the file on disk, scheduled with its direct links as `check` does
    let sched = |text: &str| {
        let mut h = Graph::new(&path);
        for p in g.projects().iter().filter(|p| normalize(&p.path) != path) {
            h.add(p.clone());
        }
        h.add(parse(&path, text));
        let ifs = interfaces_for(&h, today);
        let mut mine = parse(&path, text);
        schedule(&mut mine, today, &ifs);
        let sg = scheduled(&h, &ifs, today);
        let cs = contracts_of(&mine, today, &lookup_in(&sg));
        (mine, cs)
    };
    let (text, edits) = replan(&src, &sched);
    let wrote = write && !edits.is_empty();
    if wrote {
        if let Err(e) = fs::write(file, &text) {
            eprintln!("pmd: {}: {}", file, e);
            return 2;
        }
    }
    if json {
        println!("{}", json!({ "edits": edits, "written": wrote }));
        return 0;
    }
    if edits.is_empty() {
        println!("consistent: nothing to replan");
        return 0;
    }
    let show = |e: &Edit| format!("{}:{}  {}  {} \u{2192} {} ({:+}d)", file, e.line, e.what, e.old, e.new, e.delta_days);
    for e in &edits {
        println!("{}", show(e));
    }
    let promises: Vec<&Edit> = edits.iter().filter(|e| e.promise).collect();
    if !promises.is_empty() {
        println!("\npromises that move (parents will see these):");
        for e in promises {
            println!("  {}", show(e));
        }
    }
    if wrote {
        println!("{} edits written", edits.len());
    } else {
        println!("{} edits (dry run; --write to apply)", edits.len());
    }
    0
}

/// What a hop says about why its item ends when it does.
fn reason(h: &Hop, more: bool) -> String {
    let when = |d: Option<Civil>| d.map(|d| d.to_string()).unwrap_or_default();
    match h.driver.kind {
        "declared" => match (h.declared, h.slack, h.end) {
            (Some(d), Some(s), _) if more && h.line != 1 && s != 0 => format!("declared {}; reached {} (slack {:+}d) via", d, Civil::from_days(d.to_days() - s), s),
            _ => "declared date".into(),
        },
        "estimate" => match h.start_driver.as_ref().map(|s| s.kind) {
            Some("today") => "estimate; not started: earliest start is today".into(),
            Some("after") => format!("estimate, starts {} after the next item", when(h.start)),
            Some("after-date") => format!("estimate, starts {} (after: date)", when(h.start)),
            Some("declared") => format!("estimate from declared start {}", when(h.start)),
            _ => "estimate".into(),
        },
        "child" => "latest child".into(),
        "promise" => {
            let base = match &h.child {
                Some((f, Some(id))) => format!("promise of {} ^{}", f, id),
                Some((f, None)) => format!("promise of {} (Commitment)", f),
                None => "promise".into(),
            };
            match &h.record {
                Some(r) => format!("{}, {}", base, promise_text(r)),
                None => base,
            }
        }
        "by" => match (h.declared, h.slack) {
            (Some(d), Some(s)) if more && s != 0 => format!("own by: {}; reached {} (slack {:+}d) via", d, Civil::from_days(d.to_days() - s), s),
            _ => "own by:".into(),
        },
        "today" => "unfinished: clamped to today".into(),
        "done" => "done".into(),
        "after" => "after".into(),
        "after-date" => "after: date".into(),
        "undated" => format!("open, undated: due today (owner: {})", h.owner),
        _ => "no date".into(),
    }
}

fn hop_json(h: &Hop) -> serde_json::Value {
    let mut v = json!({ "file": h.path, "line": h.line, "title": h.title, "owner": h.owner, "start": h.start.map(|d| d.to_string()), "end": h.end.map(|d| d.to_string()), "declared": h.declared.map(|d| d.to_string()), "driver": h.driver, "start_driver": h.start_driver, "child": h.child.as_ref().map(|(f, id)| json!({ "file": f, "id": id })) });
    if let Some(r) = &h.record {
        v["record"] = json!(r);
    }
    v
}

/// `pmd why FILE [^id]`: the chain of drivers from an item (default: the project end and every
/// conflict) to its root cause; `--deep` continues inside linked files while they are readable.
fn cmd_why(root: &str, id: Option<&str>, today: Civil, json: bool, deep: bool) -> i32 {
    let ld = if deep { Loader::load(root) } else { Loader::load_depth(root, 1) };
    let g = ld.graph.as_ref().unwrap();
    if g.get(root).is_none() {
        eprintln!("pmd: cannot read {}", root);
        return 2;
    }
    let ifs = interfaces_for(g, today);
    let sched = |path: &str| {
        g.get(path).map(|p0| {
            let mut p = p0.clone();
            schedule(&mut p, today, &ifs);
            p
        })
    };
    let p = sched(root).unwrap();
    let lines: Vec<Option<u32>> = match id {
        Some(i) => match p.find_id(i) {
            Some(n) => vec![Some(n.line)],
            None => {
                eprintln!("pmd: no item ^{} in {}", i, root);
                return 2;
            }
        },
        None => std::iter::once(None).chain(conflict_lines(&p).into_iter().map(Some)).collect(),
    };
    let mut chains = Vec::new();
    for l in lines {
        let mut hops = why(&p, l);
        while deep && hops.len() < 50 {
            let Some((target, frag)) = hops.last().and_then(|h| h.child.clone()) else { break };
            let Some(c) = sched(&target) else { break };
            let at = frag.as_deref().map(|f| c.find_id(f).map(|n| n.line));
            let more = match at {
                Some(None) => break,
                Some(Some(n)) => why(&c, Some(n)),
                None => why(&c, None),
            };
            if more.is_empty() {
                break;
            }
            hops.extend(more);
        }
        for h in hops.iter_mut() {
            h.record = h.child.as_ref().and_then(|(f, _)| g.get(f)).and_then(promise_record);
        }
        chains.push(hops);
    }
    if json {
        let v: Vec<_> = chains.iter().map(|c| c.iter().map(hop_json).collect::<Vec<_>>()).collect();
        println!("{}", serde_json::to_string_pretty(&v).unwrap());
        return 0;
    }
    for (n, hops) in chains.iter().enumerate() {
        if n > 0 {
            println!();
        }
        for (i, h) in hops.iter().enumerate() {
            let end = h.end.map(|d| d.to_string()).unwrap_or_else(|| "?".into());
            if i > 0 && h.path != hops[i - 1].path {
                println!("  == {} ==", h.path);
            }
            if i == 0 {
                let late = match (h.end, h.declared) {
                    (Some(e), Some(d)) if e > d => format!(", {}d late", e.to_days() - d.to_days()),
                    _ => h.slack.map(|s| format!(", slack {:+}d", s)).unwrap_or_default(),
                };
                let decl = h.declared.map(|d| format!("  (declared {}{})", d, late)).unwrap_or_default();
                println!("{}:{}  {}  forecast {}{}  {}", h.path, h.line, h.title, end, decl, reason(h, hops.get(i + 1).is_some_and(|x| x.path == h.path)));
            } else {
                println!("  <- {}:{}  {}  {}  {}", h.path, h.line, h.title, end, reason(h, hops.get(i + 1).is_some_and(|x| x.path == h.path)));
            }
            if let (Some((f, id)), true) = (&h.child, hops.get(i + 1).is_none_or(|x| x.path == h.path)) {
                println!("     drill down: pmd why {}{}", f, id.as_ref().map(|i| format!(" ^{}", i)).unwrap_or_default());
            }
        }
    }
    0
}

/// `pmd fmt FILE... [--check]`: rewrite each FILE in canonical form (FORMAT §13); prints the
/// path of every file that changed (or, with `--check`, would change).
fn cmd_fmt(rest: &[String]) {
    let check = rest.iter().any(|a| a == "--check");
    let files: Vec<&String> = rest.iter().filter(|a| !a.starts_with("--")).collect();
    if files.is_empty() || rest.iter().any(|a| a.starts_with("--") && a != "--check") {
        usage();
    }
    let mut changed = false;
    for file in files {
        let src = read_or_die(file);
        let out = fmt::format(&src);
        if out != src {
            changed = true;
            println!("{}", file);
            if !check {
                if let Err(e) = fs::write(file, out) {
                    eprintln!("pmd: {}: {}", file, e);
                    exit(2);
                }
            }
        }
    }
    if check && changed {
        exit(1);
    }
}

struct TreeCtx {
    /// Direct links read once, live (`None`: not readable here).
    kids: HashMap<String, Option<Project>>,
    today: Civil,
}

fn cmd_tree(root: &str, depth: usize, today: Civil) -> i32 {
    let mut ctx = TreeCtx { kids: HashMap::new(), today };
    let path = normalize(root);
    let src = read_or_die(&path);
    let proj = parse(&path, &src);
    tree_project(&mut ctx, &proj, depth, 0, &mut vec![path.clone()]);
    0
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
    let mut c = p.commitment.as_ref().map(|c| format!("  commitment={}", c.text)).unwrap_or_default();
    if let Some(d) = &p.done {
        c.push_str(&format!("  done={}", d.text));
    }
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
        contracts_of(p, ctx.today, &lookup).into_iter().map(|c| (c.line, c.state)).collect()
    };
    tree_nodes(ctx, p, &p.nodes, &states, depth, level + 1, stack);
}

fn contract_summary(ctx: &TreeCtx, states: &HashMap<u32, State>, n: &Node, target: &str) -> String {
    let mut s = String::new();
    let frag = n.link.as_ref().and_then(|l| split_fragment(&l.target).1);
    if let (Some(f), Some(Some(c))) = (frag, ctx.kids.get(target)) {
        if let Some(i) = interface_items(c).into_iter().find(|i| i.id.as_deref() == Some(f)) {
            s.push_str(&format!(" {}", i.date));
            if i.checkbox == 'x' {
                if let Some(d) = &i.done {
                    s.push_str(&format!(" done {}", d));
                    if let DateParse::Ok(dd, _) = parse_date(d) {
                        if dd > i.end {
                            s.push_str(&format!(" {}d late", dd.to_days() - i.end.to_days()));
                        }
                    }
                }
            }
        }
    }
    if let Some(st) = states.get(&n.line) {
        match st.actor() {
            Some(a) => s.push_str(&format!(" {} \u{2192} {}", st.as_str(), a)),
            _ => s.push_str(&format!(" {}", st.as_str())),
        }
    }
    if let Some(Some(c)) = ctx.kids.get(target) {
        if let Some(r) = promise_record(c) {
            s.push_str(&format!("  {}", promise_text(&r)));
        }
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
            NodeKind::Milestone if n.link.is_some() => {
                let link = n.link.as_ref().unwrap();
                let target = resolve(&p.path, &link.target);
                println!("{}* <> {}{}  -> {}{}", ind, link.text, extra, link.target, contract_summary(ctx, states, n, &target));
            }
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
