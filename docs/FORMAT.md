# projectmd format reference

> Status: draft. Items marked **(open)** are not decided yet.

## 1 Overview

A projectmd project is a plain markdown file. Its H1 title names the project, the property
list under the title gives it keys, and list items are its work and milestones. A link to
another markdown file makes that file a sub-project. Headings, prose and ordering are free
and ignored by planning tools.

```markdown
# Aurora EV programme

- Owner: Sanne Visser <sanne.visser@example.com>
- Commitment: 2028-11-30
- State: concept

## Phase 1 — Concept  2026-11 .. 2027-06

- [ ] 2027-06-30 Design freeze  #public  ^design-freeze
- [ ] Concept & styling  12w  owner: @sanne
- [ ] [Powertrain](powertrain/powertrain.md)  by: 2028-02-28
```

## 2 The node model

Everything planned is a **node**: it may have a declared start, a declared end, a duration,
children and predecessors (`after:`).

| You write | Node reading | Dates it carries | Children |
|---|---|---|---|
| Task `- [ ] Seat concept  6w` | node with a duration | optional declared end | sub-items, if any |
| Milestone `- [ ] 2027-06-30 Design freeze` | point node: declared end, no duration | the date | none |
| Sub-project `- [ ] [Powertrain](powertrain/powertrain.md)  by: 2028-02-28` | node whose children live in another file | `by:` = what the parent asks; the child's Commitment = what it promises | the child's items |
| File (project) | node whose children are its items; its header sets its keys | Commitment = declared end | its items |
| Heading `## Phase 1 — Concept  2026-11 .. 2027-06` | optional grouping node; ignored if no planning items sit under it | its range, if written | items under it |

- **Declared**: written by a person. Never moves by itself.
- **Forecast**: computed by tools. Never written by a person.

`by:` (what a container asks) and `Commitment:` (what a node promises) are the only named
dates. For an inline item they are the same, so one date is written.

## 3 Item lines

### 3.1 Shape and kinds
```
- [checkbox] [leading date] title  suffix
```

- **Planning item**: a list item with a checkbox or a leading date. Anything else is **prose**
  and is ignored, except property sub-bullets (3.4).
- **Checkbox**: `[ ]` open · `[x]` done · `[-]` cancelled (does not block successors, never
  overdue). On a gate, `[x]` = passed and `[-]` = rejected.
- **Leading date**: a milestone's date. It is the declared end, like a trailing date, but reads as a point.

### 3.2 The suffix is parsed right to left

The title is everything before the suffix, verbatim, and is never empty. From the end of the
line, the parser consumes **bare tokens** (`#tag`, `^id`, a duration `3d` `2w` `12m`, a date,
a range `a .. b`) and **known keys** (`key: value`), and stops at the first thing that is
neither. Unknown `word:` is text (3.3). Any order is accepted; the canonical order is bare
tokens, then keys, and a formatter will rewrite to it.

| Line | Title | Suffix |
|---|---|---|
| `- [ ] Fix issue: retry logic  2w` | `Fix issue: retry logic` | duration 2w; `issue:` is not a known key |
| `- [ ] Test 12m cable harness  3d  owner: @jan` | `Test 12m cable harness` | estimate 3d; owner `@jan` |
| `- [ ] Review #3 comments  1w  ^review` | `Review #3 comments` | duration 1w; id `^review` |

A `` `code span` `` is never a token, so `` `after: ^x` `` stays in the title.

### 3.3 Known keys

| Key | Value | Repeatable | Meaning |
|---|---|---|---|
| `owner:` | `@handle` or `Name <email>` | no | who does it |
| `after:` | reference (6) or project link | yes: all must finish | starts after that item, or after that project's end |
| `by:` | date, coarse date, `^id` or `target#id` | no | on a sub-project: the bound the parent asks; inline: the declared end |
| `approver:` | `@handle` or `Name <email>` | yes: all must sign | who signs off a gate |
| `replaced-by:` | `^id` | no | tombstone (8) |

Profiles add keys (10). A value is one atom: a word, a `Name <email>` person, a
`[text](target)` or `[text][ref]` link, or a date range.

### 3.4 Vertical properties

`key: value` sub-bullets directly under a planning item, with no checkbox and a known key, set
that item's keys. A nested bullet with a checkbox is a child item; any other is prose.

```markdown
- [ ] Powertrain validation  12w
  - after: ^dyno-testing
  - after: [Pack validated](battery-pack.md#validated)
  - owner: @jan
```

## 4 Project header

The property sub-bullets directly under the H1 title are the header:

```markdown
# Battery pack — 60 kWh platform pack

- Owner: Priya Nair <priya.nair@example.com>
- Stakeholders: Jan de Boer <jan.deboer@example.com>, Tom Peeters <tom.peeters@example.com>
- Commitment: 2026-09-01 .. 2027-11-30
- State: in development
- ID: cars.example.com/platform/battery-pack
```

| Key | Value | Notes |
|---|---|---|
| `Owner` | one person | missing is a warning (rule 7) |
| `Stakeholders` | people or handles, comma-separated | roles are not modelled |
| `Commitment` | date or range | what the project promises its parents; its end is the declared end. Optional: absent means it accepts the parent's `by:` |
| `State` | free text | vocabulary **(open)** |
| `ID` | readable namespaced string | stable identity, needed only for projects referenced from other repositories; not a UUID |

Keys are case-insensitive. Unknown keys are kept. YAML front-matter with the same keys is an
accepted alternative; whether the list or YAML is preferred is **(open)**.

## 5 Dates and durations

| Form | Example | Expands to (inclusive) |
|---|---|---|
| Exact day | `2027-06-30` | that day |
| Month | `2027-06` | `2027-06-01 .. 2027-06-30` |
| Quarter | `2027-Q3` | `2027-07-01 .. 2027-09-30` |
| Half-year | `2027-H2` | `2027-07-01 .. 2027-12-31` |
| ISO week | `2027-W14` | Mon `2027-04-05` .. Sun `2027-04-11` |
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

**Links are always real, clickable paths or URLs**; no custom schemes. Where a link points (disk, git, object store, database, web server) and who may follow it is **outside the format**: resolving, authenticating and enforcing access is the job of whoever provides or follows the link. An unreadable link is not an error in the file; tools show it as "unknown" (with the last known interface from the lock file, if any). A stable id rides in the
link title. The path is a hint and the id is the truth; tools repair the path when a file moves.

```markdown
[Battery pack](../powertrain/battery-pack.md "id:cars.example.com/platform/battery-pack")
```

For long or remote targets, use reference-style links defined at the end of the file, so all
external dependencies can be read without parsing the body:

```markdown
- [ ] On-board charger integration  6w  after: [Inverter A-samples][inv-a]

[inv-a]: https://supplier-portal.example.net/projects/inv-2207.md#a-samples
```

| Form | Meaning |
|---|---|
| `^cell-nomination` | item defined by `^cell-nomination` in this file |
| `battery-pack.md#validated` | item `^validated` in another file; must be `#public` |
| `https://…/inv-2207.md#a-samples` | the same, in a remote project |
| `powertrain/powertrain.md` (no fragment) | the whole project; in `after:` it means that project's end (its Commitment) |

`^id` names an item in this file, `file#id` one in another file; ids are unique within a file.

**Public-only:** a reference into another file must target a `#public` milestone, so internal
items can be renamed or re-planned freely. Local references are unrestricted.

E.g. `after: [Pack validated](battery-pack.md#validated)` is valid only if `battery-pack.md` contains `- [ ] 2027-11-30 Pack validated  #public  ^validated`.

## 7 Scheduling

### 7.1 Forecast algorithm

```
start = latest of: declared start, forecast end of every `after:` node
end   = start + duration       if the node has a duration
      = latest child end       else if it has children
      = declared end           otherwise
an unfinished node never ends before today
```

A cancelled node (`[-]`) is not waited for. A sub-project's children are in another file, so its
end is that file's Commitment (7.3). Tools given the same files must compute the same forecast.

### 7.2 Conflict rule

> A node is in **conflict** when its forecast end is later than a declared end that applies to
> it: its own date, its container's `by:`, or its Commitment.

This covers overdue items and milestones whose predecessors finish late.

### 7.3 Locality rule

> **Only declared dates cross a file boundary. Forecasts never do.**

- A parent uses its child's Commitment; a cross-file `after:` uses the target's declared date.
  A file's forecast therefore needs only that file and the interfaces it links to: one hop, at any depth.
- A child whose forecast slips either absorbs it, or changes its Commitment or public milestone
  date, which is a visible interface change (8). Health signals (ok, at-risk, conflict, unknown)
  may cross as colour, never as a date.
- A child with no Commitment is held to its parent's `by:`.

## 8 Interfaces and breaking changes

A project's **interface** is its Commitment plus its `#public` milestones (id and date).

- **Non-breaking**: a date moved earlier, or a public milestone added.
- **Breaking**: a date moved later, or a public milestone removed or renamed. Consumers must re-check.

A removed or renamed public milestone leaves a cancelled tombstone, `- [-] … #public ^tooling-release replaced-by: ^tooling-release-2` **(open: exact shape)**.

## 9 Lock file

`pmd.lock` is a sidecar written only by tools, one per repository root. For each linked project
it records location, id, hash, Commitment, public milestones with dates, and the date last seen.
Example, for the Aurora battery pack and the supplier's inverter:

```text
# pmd.lock — written by tooling; do not edit
powertrain/battery-pack.md  id=cars.example.com/platform/battery-pack  hash=4c1e9a  commitment=2027-11-30  seen=2026-11-14
    design-freeze   2027-06-30
    validated       2027-11-30
https://supplier-portal.example.net/projects/inv-2207.md  id=supplier.example.net/inverters/inv-2207  hash=b07a3f  commitment=2027-09-30  seen=2026-11-12
    a-samples       2027-06-30
```

- A top-level view renders from one file and its lock entries, with no fetches. An unreachable
  child shows its last known interface, marked "unknown as of <date>".
- A changed hash or interface is the change notification. The lock's git history is a history of
  every commitment the project depended on. Exact syntax **(open)**.

## 10 Profiles, gates and extensions

A **profile** is `.pmd/profile.md`, the nearest one up the folder tree (like `.editorconfig`). It
declares the state vocabulary **(open)**, extra known keys, a registry of `@handles`, and optional
modules (resources, calendars, health labels).

A **gate** is a milestone with an `approver:`. Its decision is `[x]` or `[-]`, its record is prose
sub-bullets, and who and when come from version history. A decision is stale when the plan under
it changed after it was ticked.

**Out of the core**: cost, budget, resource allocation and capacity (left to external tooling that reads the project graph), in-file derived blocks, working-day calendars, health labels, custom state
machines, dependency types other than finish-to-start, ticket import.

## 11 Conformance levels

| Level | Adds | A file at this level can be used to… |
|---|---|---|
| 0 | checkboxes | keep a todo list |
| 1 | dates, durations, `owner:` | see a Gantt chart of one file |
| 2 | `^id`, `after:` | see dependencies and conflicts |
| 3 | links to files, `by:`, `Commitment:` | nest projects |
| 4 | `#public`, cross-file references, lock file | scale across teams and repositories |

## 12 Validation rules

E = error, W = warning. The message is what a person sees.

| # | Sev | Rule | Message | Suggested fix |
|---|---|---|---|---|
| 1 | E | Syntax error: a key with no value, an unclosed `<…>` or `[text](target)`, a single-valued key repeated | "Line 14: `owner:` has no value" | Give the key a value or close the bracket |
| 2 | E | Dangling local reference | "Line 9: no item `^dyno-testin` in this file" | Correct the id, or add `^id` to the target item |
| 3 | E | Reference to a non-public item | "`battery-pack.md#validated` is not `#public`" | Ask the owner to add `#public`, or depend on the whole project. An unreachable target is a W |
| 4 | E | Include cycle | "`a.md` includes `b.md`, which includes `a.md`" | Remove one sub-project link |
| 5 | E | Conflict (7.2) | "`Dyno testing` is forecast to end 2027-11-02, after its declared end 2027-10-31" | Move the declared date, shorten the estimate, or ask the container to move its `by:` |
| 6 | W | Stale lock: file hash differs from `pmd.lock` | "`pmd.lock` is out of date for `battery-pack.md`" | Refresh the lock with the tool |
| 7 | W | Missing `Owner` | "`Owner` is not set" | Add `- Owner: Name <email>` under the title |
| 8 | W | Unknown `@handle` | "`@jan` is not in the people registry; kept as written" | Add it to `.pmd/profile.md`, or write `Name <email>` |

## 13 Informal grammar

```ebnf
item       = indent , "- " , [ checkbox , " " ] , [ date_expr , " " ] , title , suffix , EOL ;
prop       = indent , "- " , known_key , ":" , " " , value , EOL ;
checkbox   = "[ ]" | "[x]" | "[X]" | "[-]" ;
title      = char , { char } ;                 (* non-empty, verbatim: the text before the suffix *)
suffix     = { " " , ( bare | key_value ) } ;  (* read right to left; stops at the first non-token *)
bare       = tag | item_id | duration | date_range | date_expr ;
tag        = "#" , name ;
item_id    = "^" , name ;
duration   = digit , { digit } , ( "d" | "w" | "m" ) ;
key_value  = known_key , ":" , " " , value ;
known_key  = "owner" | "after" | "by" | "approver" | "replaced-by" | profile_key ;
value      = person | link | date_range | ref | word ;
person     = name , " <" , email , ">" ;
link       = "[" , text , "](" , target , [ ' "' , text , '"' ] , ")"
           | "[" , text , "][" , label , "]" ;
date_range = date_expr , [ " " ] , ".." , [ " " ] , date_expr ;
date_expr  = exact | coarse ;
exact      = YYYY , "-" , MM , "-" , DD ;
coarse     = YYYY , "-" , ( MM | "Q" , [1-4] | "H" , [1-2] | "W" , WW ) ;
ref        = item_id | target , "#" , name ;
target     = relative_path | absolute_URL ;
```
