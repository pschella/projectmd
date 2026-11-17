//! C-ABI wrapper around pmd-core for the browser viewer (see docs/VIEWER-CONTRACT.md).
//! No wasm-bindgen: JSON strings cross the boundary through `pmd_alloc` buffers.

use pmd_core::*;
use serde_json::{json, Value as J};

// ---- memory: raw pointers on wasm32, a handle table on native targets (tests) ----

#[cfg(target_arch = "wasm32")]
mod mem {
    pub fn alloc(len: u32) -> u32 {
        Box::into_raw(vec![0u8; len as usize].into_boxed_slice()) as *mut u8 as u32
    }
    pub fn free(ptr: u32, len: u32) {
        if ptr == 0 {
            return;
        }
        // SAFETY: `ptr`/`len` came from `alloc` or `give` (boxed slices of exactly `len` bytes).
        unsafe { drop(Box::from_raw(std::slice::from_raw_parts_mut(ptr as *mut u8, len as usize))) }
    }
    pub fn read(ptr: u32, len: u32) -> Vec<u8> {
        // SAFETY: the caller promises a live buffer of `len` bytes.
        unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize).to_vec() }
    }
    pub fn give(v: Vec<u8>) -> (u32, u32) {
        let len = v.len() as u32;
        (Box::into_raw(v.into_boxed_slice()) as *mut u8 as u32, len)
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod mem {
    use std::cell::RefCell;
    use std::collections::HashMap;
    thread_local! {
        static BUFS: RefCell<(u32, HashMap<u32, Vec<u8>>)> = RefCell::new((0, HashMap::new()));
    }
    pub fn alloc(len: u32) -> u32 {
        give_vec(vec![0u8; len as usize]).0
    }
    fn give_vec(v: Vec<u8>) -> (u32, u32) {
        BUFS.with(|b| {
            let mut b = b.borrow_mut();
            b.0 += 1;
            let h = b.0;
            let len = v.len() as u32;
            b.1.insert(h, v);
            (h, len)
        })
    }
    pub fn free(ptr: u32, _len: u32) {
        BUFS.with(|b| {
            b.borrow_mut().1.remove(&ptr);
        });
    }
    pub fn read(ptr: u32, _len: u32) -> Vec<u8> {
        BUFS.with(|b| b.borrow().1.get(&ptr).cloned().unwrap_or_default())
    }
    pub fn give(v: Vec<u8>) -> (u32, u32) {
        give_vec(v)
    }
    #[allow(dead_code)]
    pub fn write(ptr: u32, data: &[u8]) {
        BUFS.with(|b| b.borrow_mut().1.get_mut(&ptr).unwrap().copy_from_slice(data));
    }
}

fn ret(s: String) -> u64 {
    let (p, l) = mem::give(s.into_bytes());
    ((p as u64) << 32) | l as u64
}

fn err(msg: impl std::fmt::Display) -> String {
    json!({ "error": msg.to_string() }).to_string()
}

#[no_mangle]
pub extern "C" fn pmd_alloc(len: u32) -> u32 {
    mem::alloc(len)
}

#[no_mangle]
pub extern "C" fn pmd_free(ptr: u32, len: u32) {
    mem::free(ptr, len)
}

#[no_mangle]
pub extern "C" fn pmd_parse(in_ptr: u32, in_len: u32) -> u64 {
    let bytes = mem::read(in_ptr, in_len);
    ret(match std::str::from_utf8(&bytes) {
        Ok(s) => parse_request(s),
        Err(e) => err(format!("request is not UTF-8: {}", e)),
    })
}

/// `pmd_board`: the board of one file (the JSON of `pmd board --json`). Same request as `pmd_parse`,
/// plus an optional `owner` string that filters the cards as `--owner` does.
#[no_mangle]
pub extern "C" fn pmd_board(in_ptr: u32, in_len: u32) -> u64 {
    let bytes = mem::read(in_ptr, in_len);
    ret(match std::str::from_utf8(&bytes) {
        Ok(s) => board_request(s),
        Err(e) => err(format!("request is not UTF-8: {}", e)),
    })
}

// ---- implementation (plain Rust, also used by tests) ----

/// A request read and scheduled: the file, today, and the sub-files the request supplied as `src`.
struct Scheduled {
    p: Project,
    today: Civil,
    kids: std::collections::HashMap<String, Project>,
}

/// Read `path`, `src`, `today` and the optional `interfaces`, and schedule the file. Errors are the JSON error text.
fn scheduled(r: &J) -> Result<Scheduled, String> {
    let path = match r.get("path").and_then(J::as_str) {
        Some(p) => p,
        None => return Err(err("request needs a string `path`")),
    };
    let src = match r.get("src").and_then(J::as_str) {
        Some(p) => p,
        None => return Err(err("request needs a string `src`")),
    };
    let today = match r.get("today").and_then(J::as_str).map(|t| (t.len() == 10, parse_date(t))) {
        Some((true, DateParse::Ok(c, _))) => c,
        _ => return Err(err("request needs `today` as YYYY-MM-DD")),
    };
    let mut ifs = Interfaces::new();
    let mut kids: std::collections::HashMap<String, Project> = std::collections::HashMap::new();
    if let Some(m) = r.get("interfaces").and_then(J::as_object) {
        for (k, v) in m {
            let ms = v.get("milestones").and_then(J::as_object);
            let i = Interface::from_texts(
                v.get("commitment").and_then(J::as_str),
                ms.into_iter().flatten().filter_map(|(id, d)| d.as_str().map(|d| (id.as_str(), d))),
            );
            if let Some(src) = v.get("src").and_then(J::as_str) {
                kids.insert(normalize(k), parse(k, src));
            }
            ifs.insert(k.clone(), i.clone());
            ifs.insert(normalize(k), i);
        }
    }
    let mut p = parse(path, src);
    schedule(&mut p, today, &ifs);
    Ok(Scheduled { p, today, kids })
}

fn parse_request(req: &str) -> String {
    let r: J = match serde_json::from_str(req) {
        Ok(v) => v,
        Err(e) => return err(format!("bad request: {}", e)),
    };
    let s = match scheduled(&r) {
        Ok(s) => s,
        Err(e) => return e,
    };
    // Live one-hop contract states, by line, for links with `by:` whose child `src` was supplied.
    let lookup = |t: &str| -> Option<Child<'_>> { s.kids.get(t).map(Child::Read) };
    let cs = contracts_of(&s.p, s.today, &lookup);
    let contracts: serde_json::Map<String, J> = cs.iter().map(|c| (c.line.to_string(), json!(c.state.as_str()))).collect();
    // whose move it is (`actor`: parent or child) and the move itself, by line
    let moves: serde_json::Map<String, J> = cs.iter().map(|c| (c.line.to_string(), json!({ "actor": c.state.actor(), "move": c.state.move_name() }))).collect();
    match serde_json::to_value(&s.p) {
        Ok(mut v) => {
            v["contracts"] = J::Object(contracts);
            v["contract_moves"] = J::Object(moves);
            v.to_string()
        }
        Err(e) => err(e),
    }
}

fn board_request(req: &str) -> String {
    let r: J = match serde_json::from_str(req) {
        Ok(v) => v,
        Err(e) => return err(format!("bad request: {}", e)),
    };
    let s = match scheduled(&r) {
        Ok(s) => s,
        Err(e) => return e,
    };
    let owner = r.get("owner").and_then(J::as_str);
    let lookup = |t: &str| -> Option<Child<'_>> { s.kids.get(t).map(Child::Read) };
    board_json(&board(&s.p, s.today, owner, &lookup)).to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Call an export the way the browser does: alloc, write, call, read, free.
    fn call(f: extern "C" fn(u32, u32) -> u64, input: &str) -> J {
        let b = input.as_bytes();
        let p = pmd_alloc(b.len() as u32);
        mem::write(p, b);
        let r = f(p, b.len() as u32);
        pmd_free(p, b.len() as u32);
        let (rp, rl) = ((r >> 32) as u32, (r & 0xffff_ffff) as u32);
        let out = mem::read(rp, rl);
        assert_eq!(out.len() as u32, rl);
        pmd_free(rp, rl);
        serde_json::from_slice(&out).unwrap()
    }

    #[test]
    fn parse_with_interfaces() {
        let src = "# Top\n\n- Owner: A <a@x.org>\n\n- [ ] [Child](c.md)  by: 2027-06-30\n";
        let ifs = json!({ "c.md": { "commitment": "2027-09-30", "milestones": {} } });
        let req = json!({ "path": "top.md", "src": src, "today": "2026-11-14", "interfaces": ifs });
        let p = call(pmd_parse, &req.to_string());
        assert!(p.get("error").is_none(), "{}", p);
        assert_eq!(p["nodes"][0]["forecast"]["end"], "2027-09-30");
        assert_eq!(p["nodes"][0]["forecast"]["declared_end"], "2027-06-30");
    }

    #[test]
    fn board_has_the_cli_shape() {
        let src = "# T\n\n- Owner: A\n\n- [ ] Write  1w  ^w\n- [ ] Ship  1w  after: ^w\n";
        let req = json!({ "path": "t.md", "src": src, "today": "2027-03-01" });
        let b = call(pmd_board, &req.to_string());
        assert!(b.get("error").is_none(), "{}", b);
        assert_eq!(b["file"], "t.md");
        assert!(b["columns"]["waiting"].is_array() && b["columns"]["ready"].is_array());
        assert!(call(pmd_board, "{").get("error").is_some());
    }

    #[test]
    fn errors() {
        assert!(call(pmd_parse, "{").get("error").is_some());
        assert!(call(pmd_parse, r##"{"path":"a.md","src":"# T"}"##).get("error").is_some());
    }
}
