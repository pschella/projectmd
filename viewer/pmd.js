// WASM wrapper for pmd-core. See docs/VIEWER-CONTRACT.md.
let inst = null;
const enc = new TextEncoder();
const dec = new TextDecoder();

export async function loadPmd(url = new URL('./pmd.wasm', import.meta.url)) {
  if (inst) return inst;
  const resp = await fetch(url, { cache: 'no-cache' });
  if (!resp.ok) throw new Error('pmd.wasm: HTTP ' + resp.status);
  let mod;
  try {
    mod = await WebAssembly.instantiateStreaming(resp.clone(), {});
  } catch (e) {
    mod = await WebAssembly.instantiate(await resp.arrayBuffer(), {});
  }
  inst = mod.instance.exports;
  return inst;
}

function call(fn, text) {
  const ex = inst;
  if (!ex) throw new Error('pmd.wasm not loaded');
  const bytes = enc.encode(text);
  const ptr = ex.pmd_alloc(bytes.length);
  new Uint8Array(ex.memory.buffer, ptr, bytes.length).set(bytes); // re-read memory.buffer after alloc
  const r = BigInt.asUintN(64, BigInt(ex[fn](ptr, bytes.length)));
  ex.pmd_free(ptr, bytes.length);
  const rptr = Number(r >> 32n), rlen = Number(r & 0xffffffffn);
  const out = dec.decode(new Uint8Array(ex.memory.buffer, rptr, rlen).slice());
  ex.pmd_free(rptr, rlen);
  return JSON.parse(out);
}

/** req: {path, src, today, profile?, interfaces?} -> Project JSON (or {error}) */
export function parseProject(req) { return call('pmd_parse', JSON.stringify(req)); }
/** raw pmd.lock text -> {entries:[...]} */
export function parseLock(text) { return call('pmd_parse_lock', text); }
