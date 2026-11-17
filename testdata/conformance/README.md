# Conformance suite

This directory is the arbiter between FORMAT.md and any tool (FORMAT §0). A second implementation
proves itself by producing the same projection for every case below, without reading the Rust code.

## Layout

- `cases/<nn-name>/` — one case. Files:
  - `today` — one line, `YYYY-MM-DD`: the day the tools run.
  - `main.md` — the root project. Children it links to sit next to it at the relative paths used in the links.
  - `expected.json` — the normative projection of `main.md` (below).
  - `README.md` — which rule the case pins (FORMAT section) and the hand derivation.
- `DISAGREEMENTS.md` — cases where the hand derivation from FORMAT.md and the reference tool differ.
  Each entry names the sentence that decides it. A human decides whether the spec or the tool is wrong.
  The skip list in `parser/pmd-core/tests/golden.rs` is empty: every case is active.
- `arithmetic.txt`, `dates.txt`, `items.txt` — earlier, prose-level corpora; not part of the golden set.

Cases are numbered in the order of the list in dev/explorations/conformance.md §1. Pairs that test
the same property (`33`/`34`) carry the same items in a different order.

## Projection (`expected.json`)

```json
{
  "nodes":       [ { "line": 5, "kind": "task", "checkbox": "open", "title": "Test",
                     "id": "t", "start": "2027-03-01", "end": "2027-03-03", "slack": -2 } ],
  "project":     { "start": "...", "end": "...", "commitment_end": "..." },
  "contracts":   [ { "line": 5, "state": "agreed", "actor": "child" } ],
  "diagnostics": [ { "line": 5, "code": "E-CONFLICT" } ]
}
```

- `kind`: `task`, `milestone`, `subproject`, `deliverable`, `heading`. `checkbox`: `none`, `open`,
  `done`, `cancelled`. Dates are `YYYY-MM-DD` strings. `slack` is declared end minus forecast end, in days.
- `nodes` are the items to assert, matched to the computed nodes by `line`. Every field written is compared.
  `null` means the field must be absent. A field left out is not asserted.
- `project` is compared the same way: `start`/`end` are the forecast span of the file, `commitment_end` is its declared Commitment end.
- `contracts` (§9) and `diagnostics` (§11) are compared exactly, as lists. Diagnostics are sorted by line, then code.
- `actor` is the actor of the contract state (FORMAT §9); `null` for `delivered`.
- Only normative values are compared: forecast dates, slack, contract states, diagnostic codes and lines.
  Messages, titles of diagnostics and tool-view output are not.

A line that a case rejects (for example E-SYNTAX) may have no node: then no node is listed for it.

## Running it against another implementation

1. For each case directory, read `today`, load `main.md` and its direct links from the directory.
2. Compute the forecasts with the rules of FORMAT §7 (one hop: children are read, grandchildren are not),
   the contract states of §9, and the diagnostics of §11 for `main.md`.
3. Produce the projection above for `main.md`.
4. Compare with `expected.json` using the rules above. Report each mismatch as case, field, expected, got.

The reference implementation does this in `parser/pmd-core/tests/golden.rs`
(`cargo test -p pmd-core --test golden`). Its laws (`laws.rs`) run the properties of FORMAT §8 and §7.8
on generated plans; the ownership law also needs the `pmd` binary (`cargo build --release` in `parser/`).

## Adding a case

Write the input and the derivation first, with the rule number in `README.md`. Only then run the tool.
If the tool disagrees, do not copy its value into `expected.json`: note the case and the sentence in
`DISAGREEMENTS.md` and decide it with a human (the skip list is for a temporary hold only). A bless (`PMD_BLESS=1`) is only for formatting a
file whose values were derived by hand.
