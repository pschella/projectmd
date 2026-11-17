# projectmd format reference

## 0 Status and stability

This is **format 0.9 (beta)**. [GRAMMAR.md](GRAMMAR.md) decides how text is read; this file decides
what it means. Where they differ, GRAMMAR wins for syntax and this file wins for semantics.

1. Files carry no version marker. Every file is read under the current format.
2. A change that alters the meaning of an existing valid file ships only with a `pmd fmt`
   migration and a header key that announces the new major version. Until that key exists, every
   file is 0.x or 1.x.
3. The conformance suite (`testdata/conformance`) is the arbiter between this text and any tool.
4. Sections marked *informative* describe tool output and conventions. They do not bind the format.

## 1 Overview

A projectmd project is a plain markdown file. Its H1 title names the project, the property
list under the title gives it keys, and list items are its work and milestones. A link to
another markdown file makes that file a sub-project. Headings, prose and ordering are free
and ignored by planning tools.

```markdown
# Lunar landing programme

- Owner: Sanne Visser <sanne.visser@example.com>
- Commitment: 2028-11-30

## Phase 1

- [ ] 2027-06-30 Design freeze  ^design-freeze
- [ ] Concept & design  12w  owner: @sanne
- [ ] [Lunar module](lunar-module/lunar-module.md)  by: 2028-02-28
```

## 2 The node model

Everything planned is a **node**: it may have a declared start, a declared end, an estimate,
children and predecessors (`after:`).

| You write | Node reading | Dates it carries | Children |
|---|---|---|---|
| Task `- [ ] Rendezvous concept  6w` | node with an estimate | optional declared end | sub-items, if any |
| Milestone `- [ ] 2027-06-30 Design freeze` | point node | the declared end | none |
| Sub-project `- [ ] [Lunar module](lunar-module/lunar-module.md)  by: 2028-02-28` | node whose children live in another file | `by:` = what the parent asks; the child's Commitment = what it promises | the child's items |
| Deliverable `- [ ] [Descent stage B-samples](descent-stage.md#b-samples)  by: 2027-09-30  ^b-samples` | node whose date comes from one milestone of another project | `by:` = what the parent asks; the milestone's date = what is promised | none |
| File (project) | node whose children are its items | Commitment = declared end | its items |
| Heading `## Phase 1` | grouping node; carries no planning data (§3.5) | none | items under it |

- **Declared**: written by a person. Never moves by itself.
- **Forecast**: computed by tools. Never written by a person.

`by:` (what a container asks) and `Commitment:` (what a node promises) are the only named
dates. On an inline item `by:` is its own declared end. A phase with dates is an item with
children (`- [ ] Phase 1  2026-11 .. 2027-06`), never a heading.

## 3 Item lines

### 3.1 Shape, kinds and line recognition

```
- [ ] [leading date] title  suffix
```

**Line recognition.**

1. A bullet is `-`, `*` or `+`. `pmd fmt` writes `-`. Ordered lists (`1.`) are prose.
2. A **planning item** is a bullet with a checkbox: `[ ]` open, `[x]` or `[X]` done, `[-]` cancelled.
   `pmd fmt` writes `[x]`.
3. Anything else is **prose** and is ignored, except property bullets (§3.4) and descriptions (§3.6).
   `W-KEYLIKE` hints at a forgotten checkbox.
4. A planning item nested under a prose bullet is a child of the nearest enclosing planning item.
   It is never dropped.
5. Planning items before the H1 belong to the project.
6. Keys and `#private` are case-insensitive. `pmd fmt` writes them lowercase; header keys are capitalised.

**Checkbox.** `[-]` means will not happen. It does not hold up successors, is never overdue, and
cancels the item's sub-items. On a milestone or deliverable it means withdrawn or rejected.
Acceptance of a sub-project is the parent's own `[x]` on its line. A **gate** is a milestone whose
owner decides it: record the outcome in prose and tick it `[x]` or `[-]`.

**Leading date.** `- [ ] 2027-06-30 Title` is a milestone: a point with a declared end. It is exposed
to parents unless tagged `#private` (§8).

### 3.2 What an item may declare

One way to say each thing.

| Form | Meaning | Kind |
|---|---|---|
| `- [ ] 2027-06-30 Title` | declared end; a point; exposed unless `#private` | milestone |
| `- [ ] 2027-06-30 Title  26w` | work with a deadline: forecast from the estimate; declared end is the date; exposed like a milestone | milestone |
| `Title  12w` | estimate: forecast only | task |
| `Title  by: 2027-06-30` | declared end: a private deadline | task |
| `Title  2027-01 .. 2027-03` | declared start and end: a fixed window | task |

Rules:

1. A declared end is given at most once. These combinations are `E-SYNTAX` ("more than one declared
   end"): range with estimate, range with `by:`, leading date with `by:`, leading date with range.
2. An estimate with `by:` is allowed: work with a private deadline.
3. A date at the end of a title is title text and gets `W-KEYLIKE`: write `by: DATE`, or lead with the date.
4. A leading date on a link line needs a `#id` target (§6). Without a fragment it is `E-SYNTAX`.

### 3.3 The suffix is parsed right to left

The title is everything before the suffix, verbatim, and is never empty. From the end of the
line, the parser consumes **bare tokens** (`#tag`, `^id`, an estimate `3d` `2w` `12m`, a range
`a .. b`) and **known keys** (`key: value`), and stops at the first thing that is neither. Any order
is accepted; `pmd fmt` writes the canonical order (§13). Unknown `word:` is title text, except after the first suffix part (§3.4 rule 6).

| Line | Title | Suffix |
|---|---|---|
| `- [ ] Fix issue: retry logic  2w` | `Fix issue: retry logic` | estimate 2w; `issue:` is not a known key |
| `- [ ] Test 12m cable harness  3d  owner: @jan` | `Test 12m cable harness` | estimate 3d; owner `@jan` |
| `- [ ] Review #3 comments  1w  ^review` | `Review #3 comments` | estimate 1w; id `^review` |

A `` `code span` `` is never a token, so `` `after: ^x` `` stays in the title.

### 3.4 Known keys

| Key | Value | Repeatable | Meaning |
|---|---|---|---|
| `owner:` | free text up to the next suffix token or two-space separator: `Henrik Osei`, `Name <email>`, `@jan` | no | who does it; may differ from the file's `Owner:` |
| `after:` | local `^id`, or a date | yes: all must finish | starts the day after that item ends; a date is the earliest start, inclusive |
| `by:` | date, coarse date, or local `^id` of an item with a declared end | no | on a link line: what the parent asks of the child; inline: the declared end |
| `done:` | date or coarse date or range | no | when a `[x]` item was completed; written by whoever ticks it; a coarse date or range counts as its end |

Rules:

1. No key takes a link. `after:` and `by:` never name another file: a cross-file edge is a **link
   line** (§6) that other items point at by `^id`.
2. A key with no value, or with a value of the wrong type, stays in the title and gets `W-KEYLIKE`.
   This holds when the key is the last word of the title.
3. `done:` on an open item gets `W-KEYLIKE` ("tick the box").
4. A key repeated where it may not be is `E-SYNTAX`.
5. An inline `owner:` value ends at the next two spaces, so `owner: Bo  effort: 6d` gives owner `Bo`.
   A vertical `- owner:` bullet runs to the end of its line (§3.5).
6. Once a suffix part has been recognised, an unknown `word: value` is not title text. It and its
   value (up to the next two spaces) are skipped, its value is never read as an estimate, date, id or
   tag, and the line gets `W-KEYLIKE`. Before the first suffix part, `word:` stays title text.

### 3.5 Vertical properties

`key: value` sub-bullets directly under a planning item, with no checkbox and a known key, set
that item's keys. The value runs to the end of the line. A nested bullet with a checkbox is a
child item; any other nested bullet is part of the description (§3.6).

```markdown
- [ ] Lunar module validation  12w
  - after: ^thrust-stand-testing
  - after: ^descent-validated
  - owner: Henrik Osei
```

### 3.6 Headings carry no planning data

A heading is a title only: no dates, estimates, keys or ids. Suffix-like text in a heading is
title text; a known `key:` in a heading gets `W-KEYLIKE`. The H1 is the same: its facts are the
header keys (§4). A closing ` #…` preceded by a space is stripped; `## Use C#` keeps its `#`.
A heading's span is its children's span. It is never clamped to today and never a conflict source.
A heading with no planning items under it is ignored.

### 3.7 Item descriptions

Text directly under a planning item and indented deeper than it is the item's **description**:
paragraphs and non-planning bullets (no checkbox, not a property bullet). It ends at the next
planning item, or at text indented at or above the item.

- Use it for acceptance criteria ("done when ..."), the reason a date moved, and links to specs.
- Tools carry it (`description` in JSON). `pmd fmt` never rewrites its text.
- Prose anywhere else stays free text.

```markdown
- [ ] Descent engine design  40w  after: ^engine  owner: Kofi Mensah
  Done when the engine passes the 600 s hot-fire review.
  Slipped two weeks: nozzle supplier changed (see decision log).
```

## 4 Project header

The property sub-bullets directly under the H1 title are the header:

```markdown
# Descent stage — lunar module

- Owner: Priya Nair <priya.nair@example.com>
- Commitment: 2026-09-01 .. 2027-11-30
```

| Key | Value | Notes |
|---|---|---|
| `Owner` | one person | missing is `W-NO-OWNER` (§11) |
| `Commitment` | date or range | what the project promises its parents, for the whole file; its end is the declared end. `^id` (or `DATE .. ^id`) names a local item with a declared end, whose date is then the Commitment: one place to move a promise. Optional: absent means it accepts the parent's `by:` |
| `Done` | exact date | the project was delivered on this date |

- Keys are case-insensitive. Any other key (`Stakeholders`, `Start date`, ...) is kept as written
  and has no meaning in the core.
- A header key set twice is `E-SYNTAX`.
- Every list item in the header run is a header key, including a prose bullet with a colon
  (`- Note: x`). Separate a note list from the header with a heading or a paragraph.
- There is no YAML front matter.

## 5 Dates and estimates

| Form | Example | Expands to (inclusive) |
|---|---|---|
| Exact day | `2027-06-30` | that day |
| Month | `2027-06` | `2027-06-01 .. 2027-06-30` |
| Quarter | `2027-Q3` | `2027-07-01 .. 2027-09-30` |
| Range | `2027-01 .. 2027-06` | `2027-01-01 .. 2027-06-30` |

- **Coarse dates** (month, quarter): as a start (`after: 2027-Q3`, range start) they mean the first
  day of the span; as an end (`by:`, leading date, range end, `done:`) the last day.
- Impossible days (`2027-02-30`) and unknown coarse forms (`2027-Q5`) are `E-DATE`. A range whose
  end is before its start is `E-DATE`.
- An **estimate** is a duration token: `Nd` weekdays, `Nw` weeks, `Nm` calendar months. `Nd` counts
  Monday to Friday; `Nw` and `Nm` are calendar time. There are no holiday calendars, no story points and
  no resource levelling. The arithmetic is in §7.1.
- Estimates are forecasts. Milestone dates, `by:`, ranges and Commitments are declared and never
  move automatically.

## 6 Links and references

**Links are always real, clickable paths or URLs**; no custom schemes. Where a link points (disk,
git, object store, database, web server) and who may follow it is **outside the format**: resolving,
authenticating and enforcing access is the job of whoever provides or follows the link. A link that
cannot be read is shown as unknown or private; it is not an error in the file (`W-UNKNOWN` appears
only on a line with `by:`, §9).

For long or remote targets, use reference-style links defined at the end of the file, so all
external dependencies can be read without parsing the body.

Cross-file edges exist **only as link lines**. A task depends on another project by pointing at the
link line:

```markdown
- [ ] [Guidance computer flight unit][gc-flight]  ^gc-flight
- [ ] Landing radar integration  6w  after: ^gc-flight

[gc-flight]: https://supplier-portal.example.net/projects/gc-2207.md#a-samples
```

| Link line | Meaning |
|---|---|
| `[Lunar module](lunar-module/lunar-module.md)` | sub-project: the whole file; its end is the child's Commitment |
| `[Descent stage B-samples](descent-stage.md#b-samples)` | deliverable: one interface milestone of another project (remote URLs work the same) |
| `2027-06-30 [Descent stage B-samples](descent-stage.md#b-samples)` | dated deliverable: a milestone whose declared date is the promise and whose forecast is the linked milestone's forecast (same owner) or its declared date (another owner). Without its own `^id` it takes the link's fragment as id, if free in the file |

- `^id` names an item in this file. Ids are unique within a file (`E-DUP-ID`).
- A deliverable must target a dated milestone that is not `#private` (§8), else `E-REF-PUBLIC`.
- A missing file or missing id is exactly one diagnostic, on the link line (`E-REF-MISSING`).

### 6.1 Promotion

An item grows into a sub-project by replacing its title with a link to a new file. The line keeps
its checkbox, `^id`, `owner:`, `by:`, `after:` and estimate; nothing else changes, and items that
depend on `^rendezvous` keep doing so.

```markdown
- [ ] Rendezvous concept  6w  after: ^design  owner: @mia  by: 2027-09-30  ^rendezvous
- [ ] [Rendezvous concept](rendezvous.md)  6w  after: ^design  owner: @mia  by: 2027-09-30  ^rendezvous
```

Until `rendezvous.md` has a Commitment the parent schedules the line from its own estimate; once the
child promises, the promise is the line's end (§7.5).

## 7 Scheduling

Tools given the same files and the same today must compute the same forecast. Every date below is
a whole day; "today" is `--today D`, else `PMD_TODAY`, else the system date.

> **A declared date says what was promised; the forecast says what will happen. Tools never let the first stand in for the second.**

### 7.1 Vocabulary and arithmetic

| Term | Meaning |
|---|---|
| declared start / end | written dates: range, leading date, `by:`, Commitment |
| estimate | `Nd` `Nw` `Nm` |
| forecast start / end | computed by tools |
| today | the day tools run (or `--today`) |

**Days are inclusive.** An item occupies whole days `start .. end`, both included. There are two kinds
of estimate:

- `Nd` is N **weekdays**, Monday to Friday. The start counts as the first. A start on a Saturday or
  Sunday moves to the Monday after it. There are no holidays and no calendars: `d` is never calendar time.
- `Nw` and `Nm` are **calendar time**. `Nw` is 7N days: an estimate of `Nw` starting on S ends on
  `S + 7N − 1`. For `Nm`, add N calendar months to S (day clamped to the last day of the month), then
  subtract one day.

"Starts after X" means the day after X's end. Whoever needs calendar time uses `w`, `m`, a range
`A .. B` or dates.

| Estimate | Start | End | Why |
|---|---|---|---|
| `3d` | 2027-03-01 | 2027-03-03 | 3 weekdays: Mon, Tue, Wed |
| `3d` | 2027-03-04 | 2027-03-08 | 3 weekdays: Thu, Fri, then Mon (the weekend is skipped) |
| `5d` | 2027-03-01 | 2027-03-05 | 5 weekdays: Mon to Fri |
| `1d` | 2027-03-06 | 2027-03-08 | a Saturday start moves to Monday |
| `10d` | 2027-03-03 | 2027-03-16 | 10 weekdays, two weeks |
| `1w` | 2027-01-31 | 2027-02-06 | 7 calendar days |
| `1w` | 2027-03-06 | 2027-03-12 | 7 calendar days, Saturday to Friday |
| `1m` | 2027-01-31 | 2027-02-27 | 01-31 + 1 month = 02-28 (clamped), minus 1 day |
| `1m` | 2027-02-01 | 2027-02-28 | 03-01 minus 1 day |

A **point** (a milestone, a deliverable, or an open item with no estimate) may fall on the day its
predecessor ends. Work with an estimate starts the day after.

### 7.2 What each kind reads

| Kind | Declared start | Declared end | Estimate | `by:` | `after:` | Children |
|---|---|---|---|---|---|---|
| task | range start | range end | forecast | declared end | yes | sub-items |
| milestone | no | leading date (the promise) | no (point); forecast from its `after:` when it has any, else its date | not allowed (`E-SYNTAX`) | yes | sub-items |
| dated work (leading date + estimate) | no | leading date | forecast | not allowed | yes | sub-items |
| heading | no | no | no | no | no | its items; span only |
| sub-project line | no | no | used until the child promises | the contract (§9) | delays the start | no (child's file) |
| deliverable line | no | no | ignored | the contract (§9) | delays the start | no |
| dated deliverable | no | leading date (the promise) | ignored | not allowed | read, but never moves the declared date | no |
| project (file) | Commitment start | Commitment end, or `Done:` | no | no | no | its items |

### 7.3 The algorithm

Process nodes in dependency order. Rule 0 handles cycles; rules 1 to 4 run for each node, and
rule 3's table gives the end.
In the examples today is 2027-03-01.

**0. Cycles.** If `after:` edges form a cycle (including an item that follows its own ancestor or
descendant), every item in the cycle gets `E-CYCLE` ("remove one `after:`") and every `after:` edge
inside the cycle is ignored. The result does not depend on file order.

**1. Start.** `start = max(declared range start, each after:)`. An `^id` means that item's end plus
one day (the same day if this node is a point). A date means that date (a coarse date: its first day).

- `Order parts  2w  after: 2027-04-01` starts 2027-04-01.
- `Pack  1w  after: ^order` where `^order` ends 2027-04-14 starts 2027-04-15.
- `Packed  after: ^pack` (a point) where `^pack` ends 2027-04-21 is 2027-04-21.

**2. Not started yet.** If the node is unfinished, has an estimate, and has no `after: DATE` on or
before today (the record of an actual start), then `start = max(start, today)`.

- `Survey  2w` with nothing else starts today, 2027-03-01, and ends 2027-03-14.
- `Fit  1w  after: ^p` where `^p` is `[x] ... done: 2027-02-10` starts today, not 2027-02-11.
- `Survey  2w  after: 2027-02-15` keeps start 2027-02-15: a recorded start (end: see rule 4).

**3. End.** The first of these that applies:

| | Case | End |
|---|---|---|
| a | done with `done: D` | `D` (start moves back to `D` if later) |
| b | link node | §7.5 |
| c | it has live (not cancelled) sub-items that have an end | the latest sub-item end; `start = min(own start, earliest sub-item start)`. The node's own declared end is not part of the forecast: it gives slack and conflict only (§7.6, §7.7). An estimate on a parent is used only while no sub-item has an end, so promoting an item into sub-items needs no edit |
| d | estimate | the N-th weekday counting `start` as the first; a Saturday or Sunday `start` moves to Monday (§7.1) |
| e | the node has a start that comes from an `after:` | it is a point: `end = start`. For `after: ^id` the point falls on that item's end day |
| f | its declared end (leading date or `by:`), when nothing above applies | that date, but never before `start` |
| g | nothing to date it: no estimate, no sub-items, no declared end, no `after:` | a point today: `start = end = today` |

A range `A .. B` with no estimate and no sub-items is a fixed window, not a point: its end is `B`
even when an `after:` moves its start (case f). A declared end never drives the forecast of a node
that something else dates. Slack and conflict still compare against it (§7.6, §7.7).

- a: `[x] Survey  2w  done: 2027-02-20` ends 2027-02-20.
- c: `Phase  4w` with sub-items ending 2027-04-30 ends 2027-04-30; the `4w` is ignored. `Phase  4w  by: 2027-04-20` also ends 2027-04-30, with slack −10 and `E-CONFLICT`.
- d: `Test  3d  after: 2027-03-10` is 2027-03-10 .. 2027-03-12.
- e: `Packed  after: ^pack` (a point) where `^pack` ends 2027-04-21 is 2027-04-21.
  `- [ ] 2027-06-30 Design freeze  after: ^design` where `^design` ends 2027-05-14 forecasts
  2027-05-14, with slack +47d (§7.6). `Validation  8w  after: ^design-freeze` starts 2027-05-15 and
  ends 2027-07-09.
- f: `Report  by: 2027-06-30` ends 2027-06-30. A milestone with no `after:` ends on its own date.
- g: `Decide launch site  owner: Mara` is 2027-03-01. `Order  2w  after: ^decide` starts 2027-03-02.
  Tomorrow the decision is still open and everything behind it starts a day later. A decision
  someone owes delays what waits on it, a day per day.

**4. Never before today.** An unfinished node does not end before today: `end = max(end, today)`.
Headings and nodes in a file with `Done:` are exempt.

- `Review  by: 2027-02-01` ends 2027-03-01 and is in conflict (§7.7).
- `Survey  2w  after: 2027-02-15` is 2027-02-15 .. 2027-03-01.

**Successors follow the forecast.** A successor starts after what it follows is forecast to end, not
after a milestone's declared date. So work after a milestone may start before the milestone's date.
Work that must wait for a calendar date says `after: DATE`.

### 7.4 Done, cancelled and finished

| Mark | Effect |
|---|---|
| `[x]` with `done: D` | ends on `D`; successors follow the actual; slack shows an early finish |
| `[x]` without `done:` | keeps its computed dates and imposes no constraint on successors (finished, date unknown). Never invent a date |
| `[-]` | and its descendants get no dates, are not waited for, are not exposed, and raise no conflicts |
| header `Done: D` | the project is finished: its span ends at `D`; nothing is clamped to today; no conflict or overdue is reported |

When work has started, tick its predecessors and record the actual start with `after: DATE`.
Where `done:`, `[x]` and `Done:` disagree, the item's own `done:` wins for the item; `Done:` only
finishes the project as a whole.

### 7.5 Link nodes

| Link line | Start | End |
|---|---|---|
| sub-project | the child's Commitment start if it states one; else the start its `after:` gives (rule 1 of §7.3); else the end (a point) | the child's `Done:`, else its Commitment end, else own `by:`, else §7.3 g |
| deliverable, another owner | as an item | the milestone's `done:`, else its declared date, else own `by:` |
| deliverable, same owner | as an item | the linked item's forecast end |
| dated deliverable | as an item | declared end = its own date (the promise); forecast = the linked milestone's forecast if same owner (taken from that file scheduled one hop), else that milestone's declared date |

1. `after:` never moves the end of a line that has a promise.
2. **Same owner.** If both files' `Owner:` are equal (the same `<email>` when both have one, else the
   same text ignoring case and extra spaces), the link is transparent: the target may be any `^id` item, and the line takes that item's forecast end as its promise.
3. A link line already `[x]` or `[-]` in the parent has no contract.
4. A missing file, a missing id or a target that is not exposed is one diagnostic on the link line
   (`E-REF-MISSING` or `E-REF-PUBLIC`). The line then has no contract state.
5. **Another owner.** Links to another owner's file are contracts. A deliverable ends at the milestone's `done:`, else its declared date, else own `by:`. A dated deliverable forecasts at that milestone's declared date. A sub-project ends at the child's declared end (its Commitment, or `Done:` once set). Nobody's tools read another owner's forecast.

### 7.6 Slack

> **Slack** of every node with a declared end = declared end − forecast end, in days. Negative means late.

`Test  3w  by: 2027-04-30` starting 2027-04-15 ends 2027-05-05: slack −5.

Inputs that finish early leave float: the milestone `Design freeze` above has slack +47d. Inputs
that finish late give negative slack on it and `E-CONFLICT` (§7.7).

### 7.7 Conflict rule

> A node is in **conflict** (`E-CONFLICT`) when its forecast end is later than a declared end that
> applies to it: its own, its container's, or the project's Commitment.

1. An unfinished node whose declared end is before today is in conflict: "overdue: not marked done".
2. A conflict is reported where it originates, not on every node it passes through. A project whose forecast end is after its Commitment gets one `E-CONFLICT`, on the project's title line (the H1). Finished-late work is excluded.
3. **Link lines are contracts, not conflicts.** A promise later than `by:` is `W-DISPUTED` (§9) on that line, never `E-CONFLICT`.
4. A dated deliverable is a milestone of this project, so the general rule applies: it is in conflict when its forecast is later than its own date. Its forecast is the linked item's forecast (same owner) or the linked milestone's declared date (another owner, §7.5).

### 7.8 Locality rule

> **Between owners, only declared dates cross a link. Forecasts never do.** A file's tools read only
> the file and its direct links (one hop). Files with the same `Owner:` (§7.5 rule 2) are one plan split
> over files: their links are transparent, and their tools may read each other's forecasts.

A milestone's forecast may differ from its date; between owners only the date, the promise, crosses the link.

- A parent schedules a sub-project or deliverable of another owner from the child's own file: its Commitment or
  milestone date. Nothing deeper is read.
- A child whose forecast slips past its promise sees `E-CONFLICT` in its own check. Its owner
  either absorbs the slip or moves the Commitment or milestone date. The move is how a delay is
  communicated: the parent's contract is `disputed` the next time it reads the child. Nothing is
  written above the mover.
- A child with no Commitment is held to its parent's `by:`.

## 8 Interfaces and fronts

A project's **interface** is the part of its own file that other projects may rely on: the header
`Owner`, `Commitment`, `Done` and every milestone (a checkbox item with a leading date) not tagged
`#private`. Nothing else is: tasks, sub-project and deliverable lines are internal. To expose
something else, add a milestone that is `after:` it. Between two files with the same owner (§7.5, the owner-equality rule) links are transparent: a
deliverable may target any item with an `^id` and takes its forecast. Across different owners only the interface is referable, and only its declared dates cross.
The interface is derived by reading the file; no separate file is defined.

Changes: a date moved earlier or a milestone added is non-breaking. Moved later, removed, renamed or
made `#private` is breaking (consumers see `disputed`). A withdrawn milestone is marked `[-]`; the
replacement is named in its description.

### 8.1 Front file (across an access boundary)

When the plan itself must stay private, the owner keeps two **ordinary project files**: the
private plan, and a thin **front** the parent may read. The front's milestones are the promise;
its **dated deliverables** point into the plan: each is a milestone whose date is the promise and
whose link says how it is met. The private plan needs no Commitment of its own: the front holds the promise.

```markdown
# Enclosures

- Owner: Marit Solheim <marit.solheim@example.com>
- Commitment: 2028-03-15

- [ ] 2027-06-30 [Pilot enclosures delivered](https://store.internal.example.org/enclosures/plan.md#pilot)  ^pilot-enclosures
- [ ] 2028-03-15 [Series enclosures qualified](https://store.internal.example.org/enclosures/plan.md#series)  ^series-ready
```

- The declared dates are the promise. They change only when the owner moves a promise. The
  linked item is the forecast; front and plan have the same owner, so it may be any item with an `^id`, and the dated lines show slack against that forecast.
- The owner's tools can read the plan: a plan that forecasts past a front milestone is
  `E-CONFLICT` in the owner's own check, which keeps plan and promise honest.
- A parent's tools read the front only. Links into the plan are not followed, so they are not
  errors; they show as private. A team that plans in a spreadsheet keeps the front by hand.

## 9 Contracts

A large project is **distributed**: sub-projects live in different places and are kept by
different teams. What crosses each boundary is a **contract**: a need-by date (`by:`) on a link
line (the **ask**), matched by the child's promise (its Commitment, or a milestone date).

- a **project contract**: `by:` on a sub-project line, against the child's Commitment;
- a **deliverable contract**: `by:` on a deliverable line, against that milestone's date.

A link line without `by:` is a dependency, not a contract, and has no state; the line just shows the
promise. The **state** is computed live, never stored, from the parent's `by:` and the child's file as
it is now (one hop). The first row that holds wins. The last column says who moves next.

| # | State | Condition | Diagnostic | Whose move |
|---|---|---|---|---|
| 1 | `delivered` | the child has `Done:`, or the milestone is ticked | none | nobody |
| 2 | `unknown` | the child cannot be read | `W-UNKNOWN` (only with `by:`) | parent: fix the link or access |
| 3 | `disputed` | the target milestone is cancelled `[-]`, or the promise is later than `by:` (whether or not `by:` has passed) | `W-DISPUTED` | parent: accept, re-plan or talk |
| 4 | `overdue` | `by:` is before today, the target is not delivered, and no promise is later than `by:` (the promise was on or before `by:` and missed, or there is none) | `W-OVERDUE` | child: deliver, or move the promise |
| 5 | `requested` | `by:` set, no promise | `I-REQUESTED` | child: answer |
| 6 | `agreed` | promise on or before `by:` | none | child: deliver |

A disputed contract sits with the parent because the child's promise is a fact about what the child
can do, while the parent's `by:` is a wish. The child cannot make the wish come true by editing it,
and it may not edit the parent's file. The parent can accept the promise (move `by:` to it), re-plan
around it, or tell the child why the wish stands. Only the parent can change the ask.

A promiser who cannot deliver on time moves the promise; from that moment the contract is disputed and the next move is the asker's, even if the original need-by date has passed. Overdue means the promiser is silent.

Other rules:

1. A target that is done ends at its done date in the parent's schedule; the parent need not tick it.
   Acceptance is the parent's own `[x]` on its line.
2. A child that edits its promise changes the state on the next read of the parent.
3. Reading *A* costs 1 + *d* reads (*d* = direct links); nothing depends on the size of the graph.
   `pmd check --deep` is the opt-in walk of everything.
4. **Scaling works like Go compilation:** if A links B and B links C, A reads B only. B's owner
   decides whether C's move changes B's own promise. A slip climbs one level per owner decision.
5. What-if scenarios are copies or branches of the files, evaluated with `pmd schedule`.
6. Tools may cache what they read (HTTP validators, a local directory, an organisation indexer).
   That is a tool concern.

## 10 Out of the core

Cost, budget, resource allocation and capacity (left to external tooling that reads the project
graph), in-file derived sections, working-day calendars, health labels, custom state machines,
dependency types other than finish-to-start, ticket import.

## 11 Validation rules

E = error, W = warning, I = info. The message is what a person sees. Negotiation is not an error:
`pmd check --strict` turns warnings into errors, for CI. Codes are stable.

| Code | When | Example message | Fix |
|---|---|---|---|
| E-SYNTAX | more than one date, id, estimate, range or declared end; empty title; empty owner; a single-valued key or header key set twice; a leading date on a link without `#id` | "Line 14: more than one declared end (`2027-06-30` and `by: 2027-05-01`)" | keep one |
| E-DATE | invalid date or range | "Line 7: `2027-02-30` is not a date" | write a valid date, coarse date or range |
| E-DUP-ID | the same `^id` twice in a file | "`^audit` is defined twice" | rename one |
| E-REF-LOCAL | unknown `^id` in `after:`, `by:` or `Commitment:`; `by: ^id` to an item with no date | "Line 9: no item `^thrust-stand-testing` in this file" | correct the id, or add `^id` to the target |
| E-REF-PUBLIC | a deliverable targets an item that is not a public milestone (files with different owners) | "`descent-stage.md#validated` is not in its interface (exposed: ^a 2027-01-01, ...)" | the target must be a dated milestone not tagged `#private`: ask its owner, or depend on the whole project |
| E-REF-MISSING | a linked file cannot be read, or the `#id` is missing there (lists up to 8 exposed milestones); one diagnostic, on the link line | "Line 12: cannot read file `ascent/hatch.md`" | fix the path or restore the file. An unreachable remote target is shown as unknown only |
| E-CYCLE | an include cycle between files | "`a.md` includes `b.md`, which includes `a.md`" | remove one sub-project link |
| E-CYCLE | an `after:` cycle in one file | "`Build` and `Test` wait for each other" | remove one `after:` |
| E-CONFLICT | forecast end after a declared end that applies (§7.7); open item past its declared end; a project's forecast end after its Commitment, on its H1 line | "`Thrust-stand testing` is forecast to end 2027-11-02, after its declared end 2027-10-31" | move the declared date, shorten the estimate, or ask the container to move its `by:` |
| W-OVERDUE | contract `overdue` (§9) | "`Lunar module` is overdue: needed by 2028-02-28, not delivered" | child: deliver (tick, or `Done:`) or move the promise; parent: move `by:` if the need changed |
| W-DISPUTED | contract `disputed` (§9): promise later than `by:`, or the target is cancelled | "`Lunar module` promises 2028-03-31, after `by: 2028-02-28`" | parent: move `by:`, re-plan, or talk to the child |
| W-UNKNOWN | link not readable here, on a line with `by:` | "`Lunar module` could not be read; shown as unknown" | check the link and its access |
| I-REQUESTED | contract `requested` (info) | "`Lunar module` has not answered `by: 2028-02-28`" | child: state a Commitment or milestone date |
| W-KEYLIKE | a known key with no value or an invalid value (also as the last word of a title); an unknown `word:` skipped in the suffix (§3.4 rule 6); a known key in a heading; a date at the end of a title; a list item with a leading date, no checkbox, and a planning suffix (`^id`, a known key or an estimate) (a bare dated note is prose: no diagnostic); `done:` on an open item | "Line 5: `by: Friday` is not a valid `by:` value; kept in the title" | write a valid value, or reword; tick the box before `done:`; use `by: DATE`; use a `#tag` for data other tools read |
| W-NO-OWNER | missing `Owner` | "`Owner` is not set" | add `- Owner: Name <email>` under the title |

Unclosed brackets (`[x](y.md` or `<a@b`) are text, not a diagnostic.

## 12 Informal grammar

Informative summary; [GRAMMAR.md](GRAMMAR.md) is normative.

```ebnf
item       = indent , bullet , " " , checkbox , " " , [ date_expr , " " ] , title , suffix , EOL ;
prop       = indent , bullet , " " , known_key , ":" , " " , text , EOL ;
bullet     = "-" | "*" | "+" ;
checkbox   = "[ ]" | "[x]" | "[X]" | "[-]" ;
title      = char , { char } ;                 (* non-empty, verbatim: the text before the suffix *)
suffix     = { " " , ( bare | key_value ) } ;  (* read right to left; stops at the first non-token *)
bare       = tag | item_id | estimate | date_range ;
tag        = "#" , name ;
item_id    = "^" , name ;
estimate   = digit , { digit } , ( "d" | "w" | "m" ) ;
key_value  = known_key , ":" , " " , value ;
known_key  = "owner" | "after" | "by" | "done" ;
value      = person | text | date_range | date_expr | item_id ;   (* per key, §3.4 *)
person     = name , " <" , email , ">" ;
date_range = date_expr , [ " " ] , ".." , [ " " ] , date_expr ;
date_expr  = exact | coarse ;
exact      = YYYY , "-" , MM , "-" , DD ;
coarse     = YYYY , "-" , ( MM | "Q" , [1-4] ) ;
```

## 13 Working with tools and agents

Most plans will be written, checked and changed by AI agents as well as people. The format stays
plain markdown for both; these conventions make machine edits small, safe and traceable.

**Canonical form.** `pmd fmt FILE` rewrites a file into one spelling without changing its meaning
(parsing before and after gives the same result; prose and descriptions are untouched).

1. Bullets are `-`, indented two spaces per level. `[X]` becomes `[x]`; `*` and `+` become `-`.
2. One space after the checkbox. Order within a line: checkbox, leading date, title, then range or
   estimate, `#private`, keys `after` `by` `owner` `done`, then `^id` last.
3. Suffix parts are separated from the title and from each other by two spaces. Keys and tags are lowercase;
   header keys are capitalised.
4. An unknown `word:` stays in the title untouched.
5. No column alignment, no trailing spaces, one blank line before a heading, a final newline.

People may write freely. Tools and agents write canonical form, so every edit is a one-line diff
and concurrent edits rarely conflict.

**Diagnostics an agent can act on.** `pmd check` prints one line per finding,
`path:line: SEV CODE message`, followed by an indented `fix:` line from §11. Codes are stable.

| Exit code | Meaning |
|---|---|
| 0 | clean |
| 1 | findings that fail: errors, and warnings with `--strict` |
| 2 | usage or I/O error |

Only diagnostics and the exit code are normative. All other tool output (`tree`, `ready`, `why`,
`changes`, `replan`, `remote:` notes, slack) is informative. With `--json`, tools print the same
information as JSON; the shapes are given by the schemas in `docs/schema/`. Project files are always markdown.

**Conventions.**

- **An entry point.** A folder of plans may hold an `AGENTS.md` that points to the cheat sheet and
  states local rules. Agents read it first.
- **Write `done:` when ticking** `[ ]` to `[x]`, with the real completion date. If unknown, leave it out.
- **Edit only what you own.** To ask another project for a date, write `by:` on your own link to it;
  never change its Commitment or milestones. Never write forecasts.
- **Reasons travel with changes.** When a change moves a Commitment, a `by:` or a milestone, say why
  in the item's description or the commit message.
- **Claim work** by recording `after: <today>` in the same edit that starts it.

### 13.1 Tool views (informative)

Each view is derived from the files; none is stored. A view reads one file and its direct links
unless `--deep` is given.

| Command | Shows |
|---|---|
| `pmd schedule FILE... [--deep] [--rows] [--today D] [--json]` | the forecast schedule. `--rows`: one JSON line per item, for other tools (schema `schema/rows.schema.json`); each file is scheduled once. `--deep` follows links into child files |
| `pmd load FILE... [--deep] [--owner NAME] [--today D] [--json]` | per owner and ISO week, the weeks in which an owner has two or more open leaf items at once. Informative: the forecast never levels work (schema `schema/load.schema.json`) |
| `pmd ready FILE... [--deep] [--owner NAME] [--today D] [--json]` | the queue: one line per move. Kinds: `do` (open work whose predecessors are done), `decide` (a disputed contract on a link line you own), `fix` (an unknown link), `answer` (child side: the parent asked by a date and the child has not promised), `deliver` (child side: overdue). `answer` and `deliver` appear when the parent is among the files read |
| `pmd why FILE [^id] [--deep] [--json]` | the **driver** of an item's end: the one constraint that fixed it (its own date, an estimate from a start set by a predecessor or `after: DATE`, the latest child, the linked project's promise, its own `by:`). Prints the chain back to its root cause in this file. A chain that ends on a link line names the child and milestone; drill down there. The critical path of a large programme is found by following drivers one file at a time |
| `pmd changes OLD NEW [--json]` | events between two versions of one file: Commitment moved; milestone added, removed, moved, done; link added or removed; `by:` added, moved or removed; item done, cancelled, owner changed. One line each, with the day delta. With git: `pmd changes <(git show HEAD~1:plan.md) plan.md` |
| `pmd replan FILE [--write]` | a dry run by default: the smallest edits to the file's own declared dates that make it consistent (accept each disputed promise; move each declared end its forecast overruns), then lists the promises that would move because parents will see those. `--write` applies. Never edits other files, shortens estimates or touches done items |

`pmd board FILE...` (informative) shows one file as four columns: waiting (with what it waits on), ready,
doing (with its start) and done (last 14 days). The footer gives work in progress per owner, throughput,
median cycle time and a flow forecast for each open milestone that waits on open items.

The **track record** (informative) is computed from declared and `done:` dates. A promise record
(`record 3/5, late +6d`) shows on a link line: how many of the child's finished exposed milestones
kept their declared date. An estimate record shows how often finished estimates held. Each needs at
least 3 finished items. A record is shown next to a forecast and never feeds into one.

Tool views are informative. The JSON schemas in `docs/schema/*.json` and the conformance suite in
`testdata/conformance` (golden cases and executable laws) decide what the tools must output.

`pmd ready` lines are `path:line  KIND  title  owner: X  due: D  [started: D]  [detail]`. The owner is the
item's, else the nearest ancestor's, else the file's `Owner:`. `--owner` filters all kinds.
