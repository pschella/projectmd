# projectmd cheat sheet (format 0.9)

A projectmd file is markdown. Planning meaning comes only from the shapes below; everything
else is free text.

```markdown
# Lunar module — descent stage

- Owner: Elena Brandt <elena.brandt@lm-co.example.net>
- Commitment: 1963-01-07 .. 1968-11-30

## Work
- [x] 1963-06-28 Engine concept selected  done: 1963-07-12  ^engine
- [ ] Descent engine design  40w  after: ^engine  owner: Kofi Mensah
  Done when the engine passes the 600 s hot-fire review.
- [ ] 1965-03-31 Design freeze  ^design-freeze
- [ ] [Guidance computer][agc]  ^agc
- [ ] Integrated descent test  12w  after: ^design-freeze  after: ^agc
- [ ] [Landing radar](radar/radar.md)  by: 1965-06-30
- [ ] [LM test article](lm.md#ta-int)  by: 1965-11-30  ^ta-int
- [ ] [First flight-ready LM](lm.md#flight-ready)  by: 1968-11-30  ^flight-ready
- Throttle range still under study.        <- prose: no checkbox, ignored

[agc]: https://guidance-lab.example.edu/plans/agc.md#flight-unit
```

| Token | Meaning |
|---|---|
| `- [ ]` `[x]` `[-]` | open · done · will not happen (cancels sub-items too); a gate is a milestone its owner decides |
| `done: 2027-05-28` | completion date, written when ticking `[x]`; without it `[x]` = finished, date unknown (never invent one) |
| `- [ ] 2027-06-30 …` | leading date: a milestone, exposed to parents; with `26w`: work with that deadline; `Commitment: ^id` makes it the project's promise. A milestone is forecast from its `after:` when it has one, so to make work wait for a date, write `after: DATE` |
| `#private` | keeps a milestone out of its parents' view |
| `2027-Q3` `2027-06` | coarse date: first day as a start, last day as an end |
| `a .. b` | range: with no estimate and no sub-items, a fixed window (start = later of a and `after:`, end = b); with sub-items it ends at the latest sub-item, and b gives only slack and conflict |
| `12w` `3d` `2m` | estimate, days inclusive: `d` counts weekdays Mon to Fri (no holidays), `w` and `m` are calendar time (1w = 7 days, 1m = 1 calendar month) |
| `^id` | defines an id for local references |
| `owner: Name` | who does it; text up to the next token (`Henrik Osei`, `Name <email>`, `@h`). A decision you need from Mara: `- [ ] Decide X  owner: Mara` in your file; undated, it is due today, so what waits on it slips daily |
| `after: ^id` | starts the day after that local item ends; repeatable; `after: DATE` = earliest start (also records an actual start) |
| `by: date` / `by: ^id` | the declared end; on a link line, the need-by asked of the child |
| `[Title](path.md)` | sub-project: a link line; the child's Commitment is its promise |
| `[Title](path.md#id)` | deliverable: a dated, non-`#private` milestone of another owner's project; `by:` is the ask; one line per need |
| `[Title](path.md#id)` with the same `Owner:` | same owner, one plan split over files: any `^id` is a target, and the line takes its forecast end |
| `2027-09-30 [Title](path.md#id)` | dated deliverable: a milestone whose date is your promise; id defaults to the fragment |
| indented text | text indented under an item is its description: put "done when …" there |

Rules:
1. Read each line from the end: tokens and keys form the suffix; the rest is the title. Known keys: `owner`, `after`, `by`, `done`.
   One declared end per item: leading date, `a .. b` or `by:` (estimate + `by:` is fine). A date at the end of a title is text (`W-KEYLIKE`): write `by: DATE`.
2. Sub-bullets `key: value` under an item set its keys (the value runs to the end of the line). The H1 sub-bullets are the header (`Owner`, `Commitment`, `Done`).
3. Headings carry no planning data. `after:` and `by:` take local `^id`s or dates only: to depend on another file, add a link line with an `^id` and point at it (`after:` on that line).
4. To grow an item into a sub-project, replace its title with a link; nothing else changes.
5. Write only declared dates, never a forecast. Hide a plan behind a thin front file. When work has started, tick its predecessors and add `after: DATE`.
5a. A promise is not a prediction: a declared date never drives the forecast when something else dates the node. A milestone with inputs falls when its inputs finish; slack = declared − forecast; successors follow the forecast. To wait for a calendar date, write `after: DATE`.
5b. Between owners, only declared dates cross a link. Same-owner files are one plan split over files: forecasts flow (the front-file pattern).

For agents (FORMAT §13):
6. Read `AGENTS.md` if the folder has one. Refer to items by `^id`; give every promise others use an explicit `^id`. Keep titles stable: why a date moved goes in the description or commit message.
7. Edit only files you own. Ask for a date with `by:` on your own link line; never edit a Commitment elsewhere. Link, never copy: if another file has the item, link its milestone (`[Title](path.md#id)`) instead of copying it into your file.
8. Write canonical form (`pmd fmt`) and finish with `pmd check` clean (exit 0; `--strict` in CI; `PMD_TODAY=YYYY-MM-DD` replaces `--today`).
9. `pmd ready FILE... [--deep] [--owner NAME]` is your queue. Kinds: `DO` (work that can start), `DECIDE` (a disputed promise on your link line), `ANSWER` (a parent asked you for a date), `DELIVER` (a promise you owe is overdue), `FIX` (unreadable link). Claim work with `after: <today>` in the same edit.
10. `pmd why FILE ^id` names what drives an item's end (follow the chain down); `pmd changes OLD NEW` lists what moved between two versions; `pmd replan FILE` proposes edits that make your dates consistent (`--write` applies); `pmd board FILE` shows the work as a board (informative).
11. `pmd schedule FILE... --rows` prints one JSON line per item for other tools; `pmd load FILE...` shows where an owner has two or more items at once (informative).
