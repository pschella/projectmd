# projectmd format reference

> Status: draft. Items marked **(open)** are not decided yet.

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

Everything planned is a **node**: it may have a declared start, a declared end, a duration,
children and predecessors (`after:`).

| You write | Node reading | Dates it carries | Children |
|---|---|---|---|
| Task `- [ ] Rendezvous concept  6w` | node with a duration | optional declared end | sub-items, if any |
| Milestone `- [ ] 2027-06-30 Design freeze` | point node: declared end, no duration | the date | none |
| Sub-project `- [ ] [Lunar module](lunar-module/lunar-module.md)  by: 2028-02-28` | node whose children live in another file | `by:` = what the parent asks; the child's Commitment = what it promises | the child's items |
| Deliverable `- [ ] [Descent stage B-samples](descent-stage.md#b-samples)  by: 2027-09-30  ^b-samples` | node whose date comes from one milestone in another project's interface | `by:` = the contract; the milestone's date = the promise | none |
| File (project) | node whose children are its items; its header sets its keys | Commitment = declared end | its items |
| Heading `## Phase 1` | optional grouping node, no dates; ignored if no planning items sit under it | none | items under it |

- **Declared**: written by a person. Never moves by itself.
- **Forecast**: computed by tools. Never written by a person.

`by:` (what a container asks) and `Commitment:` (what a node promises) are the only named
dates. For an inline item they are the same, so one date is written. A phase with dates is an
item with children (`- [ ] Phase 1  2026-11 .. 2027-06`), never a heading.

## 3 Item lines

### 3.1 Shape and kinds
```
- [ ] [leading date] title  suffix
```

- **Planning item**: a list item with a checkbox. Anything else is **prose** (a dated note stays a
  note) and is ignored, except property sub-bullets (3.4). `W-KEYLIKE` hints at a forgotten `- [ ]`.
- **Checkbox**: `[ ]` open · `[x]` done · `[-]` cancelled (does not block successors, never
  overdue; on a milestone or deliverable: withdrawn or rejected). Acceptance of a
  sub-project is the parent's own `[x]` on its line. A **gate** is just a milestone whose owner
  decides it: record the outcome in prose and `[x]` or `[-]`.
- **Leading date**: a milestone's date. It is the declared end, like a trailing date, but reads as a point.

### 3.2 The suffix is parsed right to left

The title is everything before the suffix, verbatim, and is never empty. From the end of the
line, the parser consumes **bare tokens** (`#tag`, `^id`, a duration `3d` `2w` `12m`, a date,
a range `a .. b`) and **known keys** (`key: value`), and stops at the first thing that is
neither. Unknown `word:` is text (3.3). Any order is accepted; `pmd fmt` writes duration, tags, keys, then `^id` last (§13).

| Line | Title | Suffix |
|---|---|---|
| `- [ ] Fix issue: retry logic  2w` | `Fix issue: retry logic` | duration 2w; `issue:` is not a known key |
| `- [ ] Test 12m cable harness  3d  owner: @jan` | `Test 12m cable harness` | estimate 3d; owner `@jan` |
| `- [ ] Review #3 comments  1w  ^review` | `Review #3 comments` | duration 1w; id `^review` |

A `` `code span` `` is never a token, so `` `after: ^x` `` stays in the title.

### 3.3 Known keys

| Key | Value | Repeatable | Meaning |
|---|---|---|---|
| `owner:` | any text: `Henrik Osei`, `Name <email>`, `@jan` (runs to the next suffix token) | no | who does it; may differ from the file's `Owner:`, which keeps the record (§13) |
| `after:` | local `^id` or date | yes: all must finish | starts the day after that item ends; a date is the earliest start, inclusive |
| `by:` | date, coarse date or local `^id` | no | on a link line: the need-by the parent asks; inline: the declared end |
| `done:` | date expression | no | when a `[x]` item was completed; written by whoever ticks it; a month, quarter or range counts as its end |

`after:` and `by:` never name another file: a cross-file edge is a **link line** (a sub-project or a
deliverable, §6) that other items point at by `^id`. Any other `word:` is text. A value is one atom: a word, a `Name <email>` person, a
`[text](target)` or `[text][ref]` link, or a date range.

### 3.4 Vertical properties

`key: value` sub-bullets directly under a planning item, with no checkbox and a known key, set
that item's keys. A nested bullet with a checkbox is a child item; any other is prose.

```markdown
- [ ] Lunar module validation  12w
  - after: ^thrust-stand-testing
  - after: ^descent-validated
  - owner: @jan
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
| `Owner` | one person | missing is W-NO-OWNER (§11) |
| `Commitment` | date or range | what the project promises its parents, for the whole file (later phases: a separate project); its end is the declared end. `^id` (or `DATE .. ^id`) names a local item with a declared end, whose date is then the Commitment: one place to move a promise. Optional: absent means it accepts the parent's `by:` |
| `Done` | exact date | the project was delivered on this date |

Keys are case-insensitive. Any other key (`Stakeholders`, `Start date`, ...) is kept
as written and has no meaning in the core. An owner is `Name <email>` or any text; `@name` is plain text. The header is a property list; there is no
YAML front matter.

## 5 Dates and durations

| Form | Example | Expands to (inclusive) |
|---|---|---|
| Exact day | `2027-06-30` | that day |
| Month | `2027-06` | `2027-06-01 .. 2027-06-30` |
| Quarter | `2027-Q3` | `2027-07-01 .. 2027-09-30` |
| Range | `2027-01 .. 2027-06` | `2027-01-01 .. 2027-06-30` |

- A range's **start** is used for starts; a **deadline** (Commitment, `by:`, a milestone date)
  uses the **end**: `by: 2028-Q3` means 2028-09-30. Impossible days (`2027-02-30`) and unknown
  coarse forms (`2027-Q5`) are invalid.
- Durations are real calendar time: `Nd` days, `Nw` weeks (1w = 7 days), `Nm` calendar months
  (2027-01-31 + 1m = 2027-02-28). No working-day calendars, no story points or other abstract
  effort units, no resource levelling: plans are expressed in real dates.
- A duration or range on a task is an estimate. Milestones, `by:` and Commitments are declared
  dates and never move automatically.

## 6 Links and references

**Links are always real, clickable paths or URLs**; no custom schemes. Where a link points (disk, git, object store, database, web server) and who may follow it is **outside the format**: resolving, authenticating and enforcing access is the job of whoever provides or follows the link. A link that cannot be read is shown as unknown or private; it is not an error in the file (`W-UNKNOWN` appears only on a line with `by:`, §9). The parent schedules such a line at its own `by:` or estimate.

For long or remote targets, use reference-style links defined at the end of the file, so all external dependencies can be read without parsing the body.

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
| `2027-06-30 [Descent stage B-samples](descent-stage.md#b-samples)` | dated deliverable: a milestone (exposed unless `#private`) whose declared date is the promise and whose forecast end is the linked milestone's date; `E-CONFLICT` if that is later. Without its own `^id` it takes the link's fragment as id (if free in the file) |

`^id` names an item in this file; ids are unique within a file. A deliverable must target a dated milestone that is not `#private` (§8), else `E-REF-PUBLIC`.

### 6.1 Promotion

An item grows into a sub-project by replacing its title with a link to a new file. The line keeps
its checkbox, `^id`, `owner:`, `by:`, `after:` and estimate; nothing else changes, and items that
depend on `^rendezvous` keep doing so.

```markdown
- [ ] Rendezvous concept  6w  after: ^design  owner: @mia  by: 2027-09-30  ^rendezvous
- [ ] [Rendezvous concept](rendezvous.md)  6w  after: ^design  owner: @mia  by: 2027-09-30  ^rendezvous
```

Until `rendezvous.md` has a Commitment the parent schedules the line from its own
estimate (duration, range, `after:`); once the child promises, the promise is the line's end.

## 7 Scheduling

### 7.1 Forecast algorithm

```
start = latest of: declared start, day after the forecast end of every `after:` node, every `after:` date
end   = start + duration       if the node has a duration
      = latest child end       else if it has children
      = declared end           otherwise
an unfinished node never ends before today
an unfinished item with nothing to date it is a point today
a milestone never ends before its start
```

A project with `Done:` is finished: its span ends at that date. Work with an estimate that follows another item starts the day after it ends; a point (a
milestone, a deliverable, or a task with no estimate and no sub-items) may fall on the day its
predecessor ends. An open item with no date, estimate, predecessor or sub-items (typically a
decision someone owes: `- [ ] Decide conflict policy  owner: Mara`) is due now: it is a point today,
so its successors wait for it and slip a day for every day it stays open. A cancelled node
(`[-]`) is not waited for, and cancels its descendants (no forecast, no conflicts, not exposed). A sub-project's children are in another file, so its end is that
file's Commitment, or, while the child has none, the line's own estimate. A deliverable's end is the
milestone's date; so is a dated deliverable's forecast end (its declared date stays the promise).
`after:` on a link line delays the earliest start of the child in the parent's view. A done item (`[x]`, milestones and dated items with estimates too) with `done: DATE` ends on that date, so successors follow the actual and slack shows an early finish; without `done:` it keeps its own forecast but imposes no constraint on successors (finished, date unknown). When work has started, tick its predecessors; record the actual start with `after: DATE`. An unfinished item whose declared end is before today reports `E-CONFLICT` "is overdue: not marked done by DATE". In a file whose header has `Done:` nothing is clamped to today and no conflict or overdue is reported. A dated deliverable (leading date, whole-title link to `file#id`) forecasts from the linked milestone's own forecast and earliest end (that file scheduled one hop, no deeper), and reports `E-CONFLICT` when that is later than its date.
A dated item with an estimate (`2027-06-30 Qualify  26w  after: ^x`) is work with a deadline: its forecast end is start + estimate, its declared end the date (exposed as a milestone; `E-CONFLICT` if later).
`pmd schedule` shows `slack +45d` for items with a declared end (declared end minus the earliest possible end from predecessors, estimate and children, not clamped to the declared date; negative in conflict); with `--json` it is `slack_days` in each `forecast`. Tools given the same files must compute the same forecast.

### 7.2 Conflict rule

> A node is in **conflict** when its forecast end is later than a declared end that applies to
> it: its own date, its container's `by:`, or its Commitment.

This covers overdue items and milestones whose predecessors finish late. Declared ends are
milestones, item dates, `by:` and Commitments. A conflict is reported where it originates, and
once against the file's own Commitment (a file forecast past its Commitment), not on every node
it passes through.

### 7.3 Locality rule

> **Only declared dates cross a file boundary. Forecasts never do.** A file's tools read only
> the file and its direct links (one hop).

- A parent schedules a sub-project or deliverable from the child's own file: its Commitment or
  milestone date. Nothing deeper is read.
- A child whose forecast slips past its promise sees `E-CONFLICT` in its own check. Its owner
  either absorbs the slip or moves the Commitment or milestone date. The move is how a delay is
  communicated: the parent's contract is `disputed` the next time it reads the child, and the two
  owners agree a new date outside the tools. Nothing is written above the mover.
- A child with no Commitment is held to its parent's `by:`.

## 8 Interfaces and fronts

A project's **interface** is the part of its own file that other projects may rely on: the header
`Owner`, `Commitment`, `Done` and every milestone (a checkbox item with a leading date) not tagged
`#private`. Nothing else is: tasks, sub-project and deliverable lines are internal. To expose
something else, add a milestone that is `after:` it. Between two files with the same `Owner:`
(exact text, trimmed) a deliverable or dated deliverable may target any item with an `^id`; its end is that
item's forecast (earliest) end. Across different owners only the interface is referable. The interface is derived by reading the file;
no separate file is defined. Changes: a date moved earlier or a milestone added is non-breaking;
moved later, removed, renamed or made `#private` is breaking (consumers see `disputed`). A
withdrawn milestone is marked `[-]`; the replacement is named in prose.

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
  linked item is the forecast: the front and the plan have the same `Owner:`, so it may be any
  item with an `^id` (not only an exposed milestone); its end is its forecast end.
- The owner's tools can read the plan: a plan that forecasts past a front milestone is
  `E-CONFLICT` in the owner's own check, which keeps plan and promise honest.
- A parent's tools read the front only. Links into the plan are not followed, so they are not
  errors; they show as private. A team that plans in a spreadsheet keeps the front by hand.

## 9 Contracts

A large project is **distributed**: sub-projects live in different places and are kept by
different teams. What crosses each boundary is a **contract**: a need-by date (`by:`) on a link
line, matched by the child's promise.

- a **project contract**: `by:` on a sub-project line, against the child's Commitment;
- a **deliverable contract**: `by:` on a deliverable line, against that milestone's date.

A link line without `by:` is a dependency, not a contract, and has no status. The **state** is
computed live, never stored, from the parent's `by:` and the child's file as it is now (one hop),
tested in this order (`delivered`, `unknown`, `requested`, `overdue`, `disputed`, `agreed`; the first that holds wins):

| State | Meaning |
|---|---|
| `delivered` | the child has `Done:`, or the milestone is `[x]` |
| `requested` | the parent asks with `by:`, the child has made no promise yet |
| `overdue` | the `by:` is before today and the target is not delivered (wins over `disputed` and `agreed`) |
| `disputed` | the promise is later than the `by:`, or the target was withdrawn |
| `agreed` | the promise is no later than the `by:` |
| `unknown` | the child cannot be read here: no state; the parent schedules the line at its own `by:` or estimate |

A sub-project or deliverable line whose target is done ends at the target's done date (its declared date if no `done:`) in the parent's schedule; the parent need not tick it. Acceptance is the parent's own `[x]` on its line. What-if scenarios are copies or branches of the files, evaluated with `pmd schedule`. A child that edits its promise changes the state
on the next read of the parent. Reading *A* costs 1 + *d* reads (*d* = direct links); nothing
depends on the size of the graph, and `pmd check --deep` is the opt-in walk of everything.
**Scaling works like Go compilation:** if A links B and B links C, A reads B only; B's owner
decides whether C's move changes B's own promise. A slip climbs one level per owner decision.

Tools may cache what they read (HTTP validators, a local directory, an organisation indexer).
That is a tool concern. A shared cache format may later become an optional companion spec.

## 10 Out of the core

Cost, budget, resource allocation and capacity (left to external tooling that reads the project
graph), in-file derived blocks, working-day calendars, health labels, custom state machines,
dependency types other than finish-to-start, ticket import.

## 11 Validation rules

E = error, W = warning, I = info. The message is what a person sees. Negotiation is not an error:
`pmd check --strict` turns warnings into errors, for CI. `--today D` sets the date used for "today"; the `PMD_TODAY=YYYY-MM-DD` environment variable is its default.

| Code | Rule | Message | Suggested fix |
|---|---|---|---|
| E-SYNTAX | Syntax error: a key with no value, an unclosed `<…>` or `[text](target)`, a single-valued key repeated | "Line 14: `owner:` has no value" | Give the key a value or close the bracket |
| E-DATE | Invalid date | "Line 7: `2027-02-30` is not a date" | Write a valid date, rough date or range |
| E-DUP-ID | The same `^id` twice in a file | "`^audit` is defined twice" | Rename one of them |
| E-REF-LOCAL | Dangling local reference | "Line 9: no item `^thrust-stand-testing` in this file" | Correct the id, or add `^id` to the target item |
| E-REF-PUBLIC | Deliverable targets an item that is not a public milestone (and the files have different owners) | "`descent-stage.md#validated` is not in its interface (exposed: ^a 2027-01-01, ...)" | The target must be a dated milestone not marked #private: ask its owner to add one, or depend on the whole project |
| E-REF-MISSING | Linked local file cannot be read; a missing `#id` lists the target's exposed milestones (at most 8) | "cannot read file: `ascent/hatch.md`" | Fix the path, or restore the file. An unreachable remote target is only shown as unknown |
| E-CYCLE | Include cycle | "`a.md` includes `b.md`, which includes `a.md`" | Remove one sub-project link |
| E-CONFLICT | Conflict (7.2) | "`Thrust-stand testing` is forecast to end 2027-11-02, after its declared end 2027-10-31" | Move the declared date, shorten the estimate, or ask the container to move its `by:` |
| W-OVERDUE | Contract `overdue` (§9): `by:` passed, target not delivered | "`Lunar module` is overdue: needed by 2028-02-28, not delivered" | Ask the child's owner for the delivery (if delivered, the owner ticks the milestone or sets `Done:`), or move the `by:` if the need changed |
| W-DISPUTED | Contract `disputed` (§9): promise later than `by:`, or target withdrawn | "`Lunar module` promises 2028-03-31, after `by: 2028-02-28`" | Agree a new date: the parent moves its `by:` or the child moves its Commitment |
| W-UNKNOWN | Link not readable here, on a line with `by:` | "`Lunar module` could not be read; shown as unknown" | Check the link and its access |
| I-REQUESTED | Contract `requested`: `by:` set, no promise yet (info) | "`Lunar module` has not answered `by: 2028-02-28`" | Wait, or ask the child's owner |
| W-KEYLIKE | Known key with a bad value, left in the title; or a list item with a known key, or a leading date plus `^id`, key or duration, but no `- [ ]` | "Line 5: `by: Friday` is not a valid `by:` value; kept in the title" | Write a valid value, or reword |
| W-NO-OWNER | Missing `Owner` | "`Owner` is not set" | Add `- Owner: Name <email>` under the title |

## 12 Informal grammar

```ebnf
item       = indent , "- " , checkbox , " " , [ date_expr , " " ] , title , suffix , EOL ;
prop       = indent , "- " , known_key , ":" , " " , value , EOL ;
checkbox   = "[ ]" | "[x]" | "[X]" | "[-]" ;
title      = char , { char } ;                 (* non-empty, verbatim: the text before the suffix *)
suffix     = { " " , ( bare | key_value ) } ;  (* read right to left; stops at the first non-token *)
bare       = tag | item_id | duration | date_range | date_expr ;
tag        = "#" , name ;
item_id    = "^" , name ;
duration   = digit , { digit } , ( "d" | "w" | "m" ) ;
key_value  = known_key , ":" , " " , value ;
known_key  = "owner" | "after" | "by" | "done" ;
value      = person | link | date_range | item_id | word ;
person     = name , " <" , email , ">" ;
link       = "[" , text , "](" , target , [ ' "' , text , '"' ] , ")"
           | "[" , text , "][" , label , "]" ;
date_range = date_expr , [ " " ] , ".." , [ " " ] , date_expr ;
date_expr  = exact | coarse ;
exact      = YYYY , "-" , MM , "-" , DD ;
coarse     = YYYY , "-" , ( MM | "Q" , [1-4] ) ;
```

## 13 Working with tools and agents

Most plans will be written, checked and changed by AI agents as well as people. The format
stays plain markdown for both; these conventions make machine edits small, safe and traceable.

- **Canonical form.** `pmd fmt FILE` rewrites a file into one canonical spelling without
  changing its meaning (parsing before and after gives the same result; prose is untouched):
  `- ` bullets indented two spaces per level; one space after the checkbox; suffix parts
  separated from the title and from each other by two spaces, in the order range or duration,
  tags, keys (`after`, `by`, `owner`, `done`), `^id` last; no
  column alignment; no trailing spaces; one blank line before a heading; a final newline.
  People may write freely. Tools and agents write canonical form, so every edit is a
  one-line diff and concurrent edits rarely conflict.
- **Diagnostics an agent can act on.** `pmd check` prints one line per finding,
  `path:line: SEV CODE message`, followed by an indented `fix:` line with the suggested fix
  from §11. Codes are stable. With `--json`, `pmd check` and `pmd schedule` print the same
  information as JSON. A clean `pmd check` prints nothing and exits 0. Tool output may be JSON; project files are always markdown.
- **An entry point for agents.** A folder of plans may hold an `AGENTS.md` that points to the
  cheat sheet and states local rules. Agents read it first.
- **Write `done:` when ticking** `[ ]` to `[x]`, with the real completion date; if unknown, leave it out. Never invent a date.
- **Edit only what you own.** To ask another project for a date, write `by:` on your own link
  to it; never change its Commitment or milestones. Never write forecasts.
- **Reasons travel with changes.** When a change moves a Commitment, a `by:` or a milestone,
  the commit message says why.
- **Finding work.** `pmd ready FILE... [--deep] [--owner NAME] [--today D] [--json]` lists open
  leaf items whose `after:`s are met, with their owner (item, else nearest ancestor, else file
  `Owner:`), `due:` (forecast end) and `started:` date. A dated milestone with no `after:` is ready
  only on its date. `--deep` follows links all the way down. This is a tool view; the format is
  unchanged. To claim an item, record `after: <today>` in the same edit.

