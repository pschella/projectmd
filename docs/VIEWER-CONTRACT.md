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

### `pmd_parse` request

`path`, `src` and `today` (`YYYY-MM-DD`) are required; `interfaces` is optional (default: none).

```json
{
  "path": "examples/lunar/hq/lunar-programme.md",
  "src": "<markdown text>",
  "today": "2026-11-14",
  "interfaces": {
    "<normalized path or URL of another project>": {
      "commitment": "2028-02-15",
      "milestones": { "validated": "2028-02-15" }
    }
  }
}
```

`interfaces` holds the **declared** dates of other projects (locality rule, FORMAT §7.3).
The viewer fills it by reading the opened file's direct links live (one hop) and from child
projects it has already loaded; a link it cannot read is simply absent (unknown).
The keys are paths normalised the same way the parser resolves links: relative to the
current file's folder, `.` and `..` segments collapsed, with URLs kept as they are.
A `commitment` or milestone value is any date expression (FORMAT §5); a milestone's end day is
used. The viewer omits `#private` and cancelled milestones, so only the public interface crosses a
file boundary. An interface holds no forecast: only declared dates do.

### `pmd_parse` result

This is the same `Project` JSON as `pmd parse FILE`, plus forecasts:

- every `Node` gains
  `"forecast": { "start": "YYYY-MM-DD"|null, "end": "YYYY-MM-DD"|null, "declared_end": "YYYY-MM-DD"|null, "conflict": bool }`;
- every `Node` with a link gains `"link_resolved": "<normalized path or URL>"`;
- `Project` gains `"forecast": { "start", "end", "declared_end", "conflict" }`, the
  project's span over its top-level nodes. `declared_end` is the end of its Commitment;
- `Project.diagnostics` also includes the scheduler's `E-CONFLICT` and `E-CYCLE`.

Contract states: an `interfaces` entry may also carry `"src"`, the child's source text. The parser
parses it and returns `Project.contracts`: `{ "<line>": "requested"|"agreed"|"disputed"|"overdue"|"delivered"|"unknown" }`
for each unfinished sub-project/deliverable line with a `by:` (same rules as `pmd tree`). Without `src`, no state.

Forecast rules follow FORMAT §7.1–7.3:
- A sub-project's end is its Commitment from `interfaces`; while the child has none, the line's own estimate (duration, range, `after:`) or its `by:`. Its start comes from its own `after:`/range, else the Commitment start.
- Dates are inclusive days: a duration of N days from S ends on S+N-1 (`Nm`: S + N months - 1 day),
  and a successor of an `after:` dependency starts the day after it ends (milestones too).
- A node with its own declared date/range and children spans min(starts) .. max(ends) of both.
- A deliverable line takes its date from the target milestone in `interfaces`; local items
  follow it with `after: ^id`. `after:`/`by:` never name another file.
- A node with nothing to place it in time gets `start`/`end` = null.
- Durations are calendar time; `Nm` means calendar months.
- An unfinished node never ends before `today`.
- A cancelled node is not waited for.

On a malformed request (invalid JSON, missing or ill-typed `path`/`src`/`today`) the result is `{ "error": "<message>" }`.
