# Changelog

All notable changes are listed here. The format follows [Keep a Changelog](https://keepachangelog.com/).
Format versions and tool versions move together while the format is beta.

## 0.9.0 - 2026-11-17 (beta)

### Format
- Project files only: every file is a project; no generated or side files.
- An inline `owner:` value ends at the two-space separator. An unknown `key: value` after the suffix has
  started is ignored (`W-KEYLIKE`); `#tags` are the extension point for other tools.
- Promise protocol: `by:` asks, `Commitment:` and milestones promise; every contract state (requested,
  agreed, disputed, overdue, delivered, unknown) names whose move is next. A dispute is the parent's move.
- 0.9 consolidation: one way to say each thing. Removed the bare trailing date and the leading range
  (a trailing date is title text, `W-KEYLIKE`); headings carry no planning data; days are inclusive;
  a single scheduling algorithm; cycles drop every edge in the cycle.
- Item descriptions: indented text under an item belongs to it ("done when ...", why a date moved).
- Undated decisions owed by an owner are due today, so what waits on them slips.
- Estimates in days are weekdays (`d` counts Monday to Friday, no holidays; a Saturday or Sunday start moves to Monday); `w` and `m` stay calendar time (D71).

- Promise protocol (D68, D69): a promise is not a prediction. A declared date never drives the forecast
  when something else dates the node; a milestone with inputs falls when its inputs finish; slack =
  declared − forecast; to wait for a calendar date write `after: DATE`. Between owners, only declared
  dates cross a link; same-owner files are one plan split over files.
- Range rule: `A .. B` without sub-items is a fixed window; with sub-items it ends at the latest sub-item.

### Tools
- `pmd ready`: the queue, with kinds DO, DECIDE, ANSWER, DELIVER, FIX; `--owner`, `--deep`, `--json`.
- `pmd why`: what fixes an item's date, one hop per line, drilling down across files.
- `pmd changes OLD NEW`: events between two versions of one file (promises, asks, own items done or cancelled).
- `pmd replan FILE`: dry run by default; proposes the smallest edits to a file's own dates; `--write` applies.
- `pmd schedule FILE... --rows`: one JSON line per item, for other tools such as resource planning (schema `rows.schema.json`).
- `pmd load FILE...`: per owner and ISO week, where an owner has two or more items at once. Informative; the forecast never levels work (schema `load.schema.json`).
- `pmd check --deep` schedules each file once, in place (timings in [BENCHMARKS](docs/BENCHMARKS.md)).
- `pmd board FILE`: waiting, ready, doing and done columns, with work in progress, throughput, cycle time and a flow forecast.
- Track record (informative): a promise record on link lines and an estimate record, from declared and `done:` dates; shown only with at least 3 finished items.
- Item descriptions: indented text under an item (for example "done when ...") is shown with it.
- `pmd fmt` canonical form; `pmd check --deep --strict`.
- GitHub Action (`action.yml`): checks plans in CI, fails on findings, and writes `pmd changes` to the job summary on pull requests.
- Release and CI workflows; GitHub Pages website.

### Conformance and schemas
- Conformance suite (`testdata/conformance`): 81 golden cases and executable laws that any implementation must pass.
- JSON schemas for tool output in `docs/schema/`.

### Viewer
- Time slider; detail panel with a why-chain; attention bar; filters; board tab.
- Weekend shading when zoomed to days.
- "Critical path" toggle: highlights the chain of items that fixes the project's end.
- "Overlap" attention chip: an owner with more than one item at once.
- Legend.
- Hardened against untrusted files: text rendered escaped, Content-Security-Policy, bounded fetches.

### Docs and examples
- Cheat sheet, tutorial, adopting guide, boards guide, AGENTS.md template.
- Examples: lunar, org and bakery.
- Website.

## Earlier drafts (0.1-0.2)

Initial format, parser, per-project lock files and the first viewer. Superseded by 0.9.
