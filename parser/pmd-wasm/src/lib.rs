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

#[no_mangle]
pub extern "C" fn pmd_parse_lock(in_ptr: u32, in_len: u32) -> u64 {
    let bytes = mem::read(in_ptr, in_len);
    ret(lock_json(&String::from_utf8_lossy(&bytes)))
}

// ---- implementation (plain Rust, also used by tests) ----

fn parse_request(req: &str) -> String {
    let r: J = match serde_json::from_str(req) {
        Ok(v) => v,
        Err(e) => return err(format!("bad request: {}", e)),
    };
    let path = match r.get("path").and_then(J::as_str) {
        Some(p) => p,
        None => return err("request needs a string `path`"),
    };
    let src = match r.get("src").and_then(J::as_str) {
        Some(p) => p,
        None => return err("request needs a string `src`"),
    };
    let today = match r.get("today").and_then(J::as_str).map(|t| (t.len() == 10, parse_date(t))) {
        Some((true, DateParse::Ok(c, _))) => c,
        _ => return err("request needs `today` as YYYY-MM-DD"),
    };
    let opts = match r.get("profile").and_then(J::as_str) {
        Some(p) => parse_profile(p),
        None => Options::default(),
    };
    let mut ifs = Interfaces::new();
    if let Some(m) = r.get("interfaces").and_then(J::as_object) {
        for (k, v) in m {
            let ms = v.get("milestones").and_then(J::as_object);
            let i = Interface::from_texts(
                v.get("commitment").and_then(J::as_str),
                v.get("start").and_then(J::as_str),
                ms.into_iter().flatten().filter_map(|(id, d)| d.as_str().map(|d| (id.as_str(), d))),
            );
            ifs.insert(k.clone(), i.clone());
            ifs.insert(normalize(k), i);
        }
    }
    let mut p = parse(path, src, &opts);
    schedule(&mut p, today, &ifs);
    match serde_json::to_string(&p) {
        Ok(s) => s,
        Err(e) => err(e),
    }
}

fn date_expr(text: &str) -> J {
    match span_of_text(text) {
        Some((s, e)) => json!({ "text": text, "start": s, "end": e, "range": text.contains("..") }),
        None => J::Null,
    }
}

fn lock_json(src: &str) -> String {
    let lock = Lock::parse(src);
    let entries: Vec<J> = lock
        .entries
        .iter()
        .map(|e| {
            let mut ms = serde_json::Map::new();
            for (id, d) in &e.milestones {
                if let Some(r) = d.strip_prefix("->") {
                    ms.insert(id.clone(), json!({ "replaced_by": r.trim() }));
                } else {
                    let x = date_expr(d);
                    if !x.is_null() {
                        ms.insert(id.clone(), x);
                    }
                }
            }
            json!({
                "location": e.location,
                "id": e.field("id"),
                "hash": e.field("hash"),
                "commitment": e.field("commitment").map(date_expr).unwrap_or(J::Null),
                "start": e.field("start").and_then(span_of_text).map(|s| s.0.to_string()),
                "seen": e.field("seen"),
                "milestones": ms,
            })
        })
        .collect();
    json!({ "entries": entries }).to_string()
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

    fn root() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/car")
    }

    #[test]
    fn parse_aurora_end_to_end() {
        let src = std::fs::read_to_string(root().join("aurora.md")).unwrap();
        let lock = std::fs::read_to_string(root().join("pmd.lock")).unwrap();
        let l = call(pmd_parse_lock, &lock);
        let mut ifs = serde_json::Map::new();
        for e in l["entries"].as_array().unwrap() {
            let ms: serde_json::Map<String, J> = e["milestones"]
                .as_object()
                .unwrap()
                .iter()
                .filter_map(|(k, v)| v["text"].as_str().map(|t| (k.clone(), json!(t))))
                .collect();
            ifs.insert(e["location"].as_str().unwrap().to_string(), json!({ "commitment": e["commitment"]["text"], "milestones": ms }));
        }
        let req = json!({ "path": "aurora.md", "src": src, "today": "2026-11-14", "interfaces": ifs });
        let p = call(pmd_parse, &req.to_string());
        assert!(p.get("error").is_none(), "{}", p);
        assert_eq!(p["forecast"]["declared_end"], "2028-11-30");
        assert_eq!(p["forecast"]["conflict"], false);
        let mut forecasts = 0;
        let mut linked = 0;
        fn walk(n: &J, f: &mut usize, l: &mut usize) {
            if n["forecast"]["end"].is_string() {
                *f += 1;
            }
            if n["link_resolved"].is_string() {
                *l += 1;
            }
            for c in n["children"].as_array().unwrap() {
                walk(c, f, l);
            }
        }
        for n in p["nodes"].as_array().unwrap() {
            walk(n, &mut forecasts, &mut linked);
        }
        assert!(forecasts > 10, "{}", forecasts);
        assert_eq!(linked, 6);
        // Powertrain sub-project end comes from the lock's Commitment
        let pt = p["nodes"][1]["children"].as_array().unwrap().iter().find(|n| n["link_resolved"] == "powertrain/powertrain.md").unwrap();
        assert_eq!(pt["forecast"]["end"], "2028-02-15");
        assert_eq!(pt["forecast"]["declared_end"], "2028-02-28");
    }

    #[test]
    fn errors_and_lock() {
        assert!(call(pmd_parse, "{").get("error").is_some());
        assert!(call(pmd_parse, r##"{"path":"a.md","src":"# T"}"##).get("error").is_some());
        let l = call(pmd_parse_lock, "a.md  id=x  hash=h  commitment=2027-Q3  seen=2026-11-14\n    m1  2027-06-30\n    m2  -> m3\n");
        assert_eq!(l["entries"][0]["commitment"]["end"]["m"], 9);
        assert_eq!(l["entries"][0]["milestones"]["m2"]["replaced_by"], "m3");
    }
}
