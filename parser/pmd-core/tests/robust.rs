//! Robustness: nothing panics on arbitrary input, `fmt` is idempotent and meaning-preserving.
//! Inputs are built deterministically (xorshift PRNG, fixed seed) from fragments of valid syntax.

use pmd_core::*;
use std::panic::{catch_unwind, AssertUnwindSafe};

struct Rng(u64);
impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn pick<'a>(&mut self, v: &[&'a str]) -> &'a str {
        v[self.below(v.len())]
    }
}

const BULLETS: &[&str] = &["- [ ] ", "- [x] ", "- [X] ", "- [-] ", "- ", "* ", "+ ", "1. ", "- [ ]", "-[ ] ", "- [?] ", "-"];
const WORDS: &[&str] = &[
    "Design", "Build the thing", "Ship", "Review", "[Sub](sub/sub.md)", "[Doc](../a.md#frag)", "[ref]", "[Ref][r]", "`code`", "`unterminated",
    "**bold**", "caf\u{e9}", "\u{65e5}\u{672c}\u{8a9e}", "\u{1f600}", "a\u{200b}b", "<!--", "-->", "<!-- c -->", "[x]", "\\", "|", "#", "@", "^", "  ",
];
const SUFFIX: &[&str] = &[
    "2027-01-07", "2027-01-07..2027-02-01", "2027-02", "2027-Q1", "2027", "2027-13-45", "2027-02-30", "0000-00-00", "~3d", "~2w", "~1m", "~0d", "~99999d",
    "~x", "^id", "^a-b_c", "^", "@ann", "@ann@x.org", "@", "#tag", "#", "by: 2027-03-01", "after: ^a", "after: 2027-01-01", "after: [S](s.md)",
    "owner: Ann <ann@x.org>", "owner: @bob", "done: 2027-01-01", "key: v1, v2", "key:", ": x", "est: 3d", "start: 2027-01-01", "by: ", "by:2027-01-01",
];
const HEADERS: &[&str] = &[
    "# Title", "# ", "#", "## Section", "### Deep", "###### Six", "####### Seven", "Title\n=====", "---", "- Owner: Ann", "- Commitment: 2027-06-30",
    "- Done: 2027-01-01", "Owner: Ann", "- Status: x", "- by: 2027-05-01", "[r]: ./x.md", "[r]: x.md \"Title\"", "<!-- comment", "-->", "```", "~~~",
    "    indented code", "> quote", "| a | b |", "",
];

fn line(r: &mut Rng) -> String {
    let mut s = String::new();
    match r.below(10) {
        0 | 1 => s.push_str(r.pick(HEADERS)),
        _ => {
            for _ in 0..r.below(4) {
                s.push_str(r.pick(&["  ", "    ", "\t", " ", "\t\t"]));
                if r.below(3) == 0 {
                    break;
                }
            }
            s.push_str(r.pick(BULLETS));
            for _ in 0..1 + r.below(4) {
                s.push_str(r.pick(WORDS));
                s.push(' ');
            }
            for _ in 0..r.below(5) {
                s.push_str(if r.below(3) == 0 { " " } else { "  " });
                s.push_str(r.pick(SUFFIX));
            }
        }
    }
    if r.below(40) == 0 {
        s = s.repeat(1 + r.below(300)); // very long line
    }
    s
}

fn mutate(r: &mut Rng, s: &mut String) {
    for _ in 0..r.below(4) {
        let chars: Vec<char> = s.chars().collect();
        if chars.is_empty() {
            return;
        }
        let i = r.below(chars.len());
        let mut c = chars;
        match r.below(3) {
            0 => {
                c.remove(i);
            }
            1 => {
                let ch = r.pick(&["[", "]", "(", ")", "`", " ", "\t", "\r", "-", ":", "@", "^", "~", "\u{e9}", "\u{0}"]).chars().next().unwrap();
                c.insert(i, ch)
            }
            _ => {
                let j = r.below(c.len());
                c.swap(i, j)
            }
        }
        *s = c.into_iter().collect();
    }
}

fn gen(r: &mut Rng) -> String {
    if r.below(50) == 0 {
        return String::new();
    }
    let eol = if r.below(5) == 0 { "\r\n" } else { "\n" };
    let mut out = String::new();
    if r.below(4) != 0 {
        out.push_str("# Project\n\n");
    }
    for _ in 0..r.below(14) {
        let mut l = line(r);
        if r.below(6) == 0 {
            mutate(r, &mut l);
        }
        out.push_str(&l);
        out.push_str(eol);
        if r.below(8) == 0 {
            out.push_str(eol);
        }
    }
    if r.below(10) == 0 {
        out.pop();
    }
    out
}

fn today() -> Civil {
    match parse_date("2027-01-07") {
        DateParse::Ok(c, _) => c,
        _ => unreachable!(),
    }
}

fn cases() -> Vec<String> {
    let mut r = Rng(0x9E37_79B9_7F4A_7C15);
    let mut v: Vec<String> = ["", "\n", "\r\n", "# ", "- [ ] ", "\t", "# T\n- [ ] a  2027-01-01..2027-01-02 ~3d ^a @b #c\n"].iter().map(|s| s.to_string()).collect();
    v.extend((0..2000).map(|_| gen(&mut r)));
    v
}

/// Meaning of a node, without line numbers or spans.
fn proj(n: &Node, out: &mut String, depth: usize) {
    let keys: Vec<_> = n.keys.iter().map(|(k, vs)| (k, vs.iter().map(|v| (format!("{:?}", v.kind), &v.text)).collect::<Vec<_>>())).collect();
    out.push_str(&format!(
        "{}{:?} {:?} {:?} date={:?} dur={:?} keys={:?} id={:?} tags={:?}\n",
        " ".repeat(depth),
        n.kind,
        n.checkbox,
        n.title,
        n.date.as_ref().map(|d| (&d.text, d.range)),
        n.duration.as_ref().map(|d| &d.text),
        keys,
        n.id,
        n.tags
    ));
    for c in &n.children {
        proj(c, out, depth + 1);
    }
}

fn meaning(src: &str) -> String {
    let p = parse("t.md", src);
    let mut s = format!("title={:?} owner={:?}\n", p.title, p.owner.as_ref().map(|o| &o.text));
    for n in &p.nodes {
        proj(n, &mut s, 0);
    }
    s
}

#[test]
fn nothing_panics() {
    for src in cases() {
        let r = catch_unwind(AssertUnwindSafe(|| {
            let mut p = parse("dir/t.md", &src);
            schedule(&mut p, today(), &Interfaces::new());
            let _ = ready(&p, today(), None, &|_| None);
            let _ = ready(&p, today(), Some("ann"), &|_| Some(Child::Unknown));
            let _ = fmt::format(&src);
        }));
        assert!(r.is_ok(), "panic on input {:?}", src);
    }
}

#[test]
fn fmt_is_idempotent() {
    for src in cases() {
        let once = fmt::format(&src);
        let twice = fmt::format(&once);
        assert_eq!(once, twice, "fmt not idempotent on {:?}", src);
    }
}

#[test]
fn fmt_preserves_meaning() {
    for src in cases() {
        let out = fmt::format(&src);
        assert_eq!(meaning(&src), meaning(&out), "fmt changed meaning of {:?}\nformatted: {:?}", src, out);
    }
}
