# projectmd cheat sheet

A projectmd file is markdown. Planning meaning comes only from the shapes below; everything
else is free text.

```markdown
# Lunar module — descent stage

- Owner: Elena Brandt <elena.brandt@lm-co.example.net>
- Commitment: 1963-01-07 .. 1968-11-30

## Work
- [x] 1963-06-28 Engine concept selected  done: 1963-07-12  ^engine
- [ ] Descent engine design  40w  after: ^engine  owner: Kofi Mensah
- [ ] 1965-03-31 Design freeze  ^design-freeze
- [ ] [Guidance computer][agc]  ^agc
- [ ] Integrated descent test  12w  after: ^design-freeze  after: ^agc
- [ ] [Landing radar](radar/radar.md)  by: 1965-06-30
- [ ] [LM test article](lm.md#ta-int)  by: 1965-11-30  ^ta-int
- [ ] [First flight-ready LM](lm.md#flight-ready)  by: 1968-11-30  ^flight-ready
- Throttle range still under study.        <- prose: no checkbox, ignored (so is `- 1965-03-31 note`)

[agc]: https://guidance-lab.example.edu/plans/agc.md#flight-unit
```

| Token | Meaning |
|---|---|
| `- [ ]` `[x]` `[-]` | open · done · will not happen (cancels its sub-items too); a gate is a milestone its owner decides |
| `done: 2027-05-28` | completion date (a month, quarter or range also works; its end counts), written when ticking `[x]`; without it `[x]` = finished, date unknown: no constraint on successors (never invent one) |
| `- [ ] 2027-06-30 …` | leading date: milestone; its date is the declared end (with `26w`: work with that deadline); `Commitment: ^id` (or `DATE .. ^id`) makes it the project's promise |
| `#private` | keeps a milestone out of its parents' view (milestones are exposed by default) |
| `2027-Q3` `2027-06` | coarse date; deadlines use the end of the span |
| `a .. b` | range: declared start a, declared end b |
| `12w` `3d` `2m` | estimate in real calendar time (1w = 7 days, 1m = 1 calendar month) |
| `^id` | defines an id for local references |
| `owner: Name` | who does it (may differ from the file's Owner: a decision you need from Mara is `- [ ] Decide X  owner: Mara` in your file; undated, it is due today, so what waits on it slips daily); any text up to the next token (`Henrik Osei`, `Name <email>`, `@h`) |
| `after: ^id` | starts the day after that local item ends; repeatable; `after: DATE` = earliest start, inclusive (also records an actual start) |
| `by: date` / `by: ^id` | the declared end; on a link line, the need-by asked of the child |
| `[Title](path.md)` | sub-project (a link line); the child's Commitment is its promise |
| `[Title](path.md#id)` | deliverable: a *dated, non-`#private` milestone* of another project; `by:` is the contract (same `Owner:` in both files: any `^id` item, its end is its forecast end); one line per need (two needs from one child = two lines, two `by:`) |
| `2027-09-30 [Title](path.md#id)` | dated deliverable: a milestone whose date is your promise; id defaults to the fragment |
| `` `code` `` | never parsed as a token |

Rules:
1. Read the line from the end: keys and tokens form the suffix; the rest is the title. Known keys: `owner`, `after`, `by`, `done`.
2. Sub-bullets `key: value` under an item set its keys. The H1 sub-bullets are the header
   (`Owner`, `Commitment`, `Done`).
3. `after:` and `by:` take local `^id`s or dates only. To depend on another file, add a link
   line with an `^id` and point at it. A dependency on a sibling or parent milestone (e.g. the parent's release candidate) is a deliverable line to that milestone plus a local `after:` on it. To wait for another project's milestone, link it and put `after:` on that line, not its date.
4. To grow an item into a sub-project, replace its title with a link; nothing else changes.
5. Write only declared dates, never a forecast. To hide a plan from parents, keep a thin front
   file (header, dated deliverables into the plan). A need inside a task: split the task.
   When work has started, tick its predecessors and add `after: DATE` for the actual start (keep the `after: ^id`).

Rules for agents (FORMAT §13):
6. Read `AGENTS.md` if the folder has one. Refer to items by `^id`; give every promise others use an explicit `^id`.
   Keep titles stable: history ("was …") goes in prose or the commit message, not the title.
7. Edit only files you own. Ask for a date with `by:` on your own link line; never edit a Commitment elsewhere.
8. Write canonical form (`pmd fmt`) and finish with `pmd check` clean (silent and exit 0 when clean; `--strict` in CI; `PMD_TODAY=YYYY-MM-DD` replaces `--today`).
   Say why in the commit message when a Commitment, `by:` or milestone moves.
9. Find work with `pmd ready FILE... [--owner NAME]` (open items whose predecessors are done). When you
   start one, record `after: <today>` in the same edit; its `owner:` says whose it is.
