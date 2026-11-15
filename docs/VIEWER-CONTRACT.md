# Viewer ↔ parser contract

The browser viewer (`viewer/`) uses `pmd-core` compiled to WebAssembly (`viewer/pmd.wasm`).
There's no wasm-bindgen and no npm: the module exports plain C-ABI functions, and JSON
strings cross the boundary.

## WASM exports (crate `parser/pmd-wasm`, built by `parser/build-wasm.sh`)

| Export | Signature | Meaning |
|---|---|---|
| `memory` | — | linear memory |
| `pmd_alloc` | `(len: u32) -> u32` | allocate `len` bytes; returns a pointer |
| `pmd_free` | `(ptr: u32, len: u32)` | free a buffer from `pmd_alloc` or from a result |
| `pmd_parse` | `(in_ptr: u32, in_len: u32) -> u64` | input: UTF-8 JSON request (below). Returns `(ptr << 32) \| len` of a UTF-8 JSON result, which the caller frees |
| `pmd_parse_lock` | `(in_ptr: u32, in_len: u32) -> u64` | input: raw lock file text (`NAME.lock`). Returns JSON `{ "entries": [ { "location", "id", "hash", "commitment": DateExpr\|null, "start": "YYYY-MM-DD"\|null, "by": string\|null, "late": string\|null, "seen", "milestones": { "<id>": DateExpr \| {"replaced_by": "<id>"} } } ] }` |

### `pmd_parse` request

```json
{
  "path": "examples/car/aurora.md",
  "src": "<markdown text>",
  "today": "2026-11-14",
  "profile": "<text of .pmd/profile.md, optional>",
  "interfaces": {
    "<normalized path or URL of another project>": {
      "commitment": "2028-02-15",
      "start": "2026-10-01",
      "milestones": { "validated": "2028-02-15" }
    }
  }
}
```

`interfaces` holds the **declared** dates of other projects (locality rule, FORMAT §7.3).
The viewer fills it from the opened file's lock (`NAME.lock`: one entry per direct link) and from the lock of each loaded sub-project and from child projects it has already loaded.
The keys are paths normalised the same way the parser resolves links: relative to the
current file's folder, `.` and `..` segments collapsed, with URLs kept as they are.
An interface may include `"start"` (the project's earliest declared date, lock `start=`);
it is the forecast start of a sub-project that has no other start. It may include `"late"`
(lock `late=`): the child then counts as finishing at that date instead of its Commitment end.

### `pmd_parse` result

This is the same `Project` JSON as `pmd parse FILE`, plus forecasts:

- every `Node` gains
  `"forecast": { "start": "YYYY-MM-DD"|null, "end": "YYYY-MM-DD"|null, "declared_end": "YYYY-MM-DD"|null, "conflict": bool }`;
- every `Node` with a link gains `"link_resolved": "<normalized path or URL>"`;
- `Project` gains `"forecast": { "start", "end", "declared_end", "conflict" }`, the
  project's span over its top-level nodes. `declared_end` is the end of its Commitment;
- `Project.diagnostics` also includes `E-CONFLICT`.

Forecast rules follow FORMAT §7.1–7.3:
- A sub-project's end is its Commitment from `interfaces`, or failing that its own `by:`;
  its start is the interface `start` (or the Commitment start if that is a range).
- Dates are inclusive days: a duration of N days from S ends on S+N-1 (`Nm`: S + N months - 1 day),
  and a successor of an `after:` dependency starts the day after it ends.
- A node with its own declared date/range and children spans min(starts) .. max(ends) of both.
- A cross-file `after:` uses the target's declared date from `interfaces`; an unknown
  target is ignored.
- A node with nothing to place it in time gets `start`/`end` = null.
- Durations are calendar time; `Nm` means calendar months.
- An unfinished node never ends before `today`.
- A cancelled node is not waited for.

On malformed input the result is `{ "error": "<message>" }`.
