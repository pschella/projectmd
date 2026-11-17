# pmd-wasm

- `pmd_board(ptr, len) -> u64` (next to `pmd_parse`) takes the same request JSON (`path`, `src`, `today`, optional `interfaces`) plus an optional `owner` string, and returns the board of that one file.
- The response is the JSON of `pmd board FILE --json`: `file`, `columns` (`waiting`, `ready`, `doing`, `done`, each a list of cards with `line`, `title`, `owner`, and optional `id`, `waits_on`, `since`, `done`), `older_done`, `wip`, `throughput_per_week`, `throughput_window_days`, `cycle_time`, `flow_forecast`.
- `owner` filters the cards the way `pmd board --owner` does; `today` is read exactly as in `pmd_parse`.
- Only the file in the request and the `interfaces` it supplies are read (one hop, FORMAT §7.8).
- Errors come back as `{"error": "..."}`, as for `pmd_parse`.
