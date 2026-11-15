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

A project that links other projects may have a **lock file**: a sidecar with the same name and
the extension `.lock` (`aurora.md` → `aurora.lock`), written only by tools. No project is special:
every file can have its own lock, so any sub-project can be moved elsewhere, lock and all, and is
then a top-level project in every respect.

A large project is usually **distributed**: groups of sub-projects live in different places
(repositories, servers, supplier portals), are kept by different teams with different tools, and
show the larger project only their interfaces. What crosses each boundary is a **contract**: the
parent's need-by date (`by:` on the link) and the child's promise (its Commitment and public
milestones). The lock records the contracts with each **directly** linked project and nothing
more: location, id, hash, `by`, Commitment, start, public milestones with dates, the date last
seen and, if the child cannot keep its promise, `late` (its forecast end). It never records
tasks, people or other internals. Example, for the Aurora battery pack and the supplier's
inverter:

```text
# aurora.lock — written by tooling; do not edit
powertrain/battery-pack.md  id=cars.example.com/platform/battery-pack  hash=4c1e9a  by=2027-12-15  commitment=2027-11-30  late=2028-01-21  seen=2026-11-14
    design-freeze   2027-06-30
    validated       2027-11-30
https://supplier-portal.example.net/projects/inv-2207.md  id=supplier.example.net/inverters/inv-2207  hash=b07a3f  commitment=2027-09-30  seen=2026-11-12
    a-samples       2027-06-30
```

A contract **holds** if the promise is no later than the need-by date, is **broken** if it is
later (an error, `E-BOUND`, §12), and is **at risk** if the child is `late` past the need-by date.

**Scaling works like Go compilation.** If A links B and B links C, then everything A needs from C
is already folded into B's contract with A. B's tools schedule B against B's contracts with its
children, treating a late child as finishing at its `late` date. If that pushes B past its own
Commitment, B is late. A's tools read only B, never C:

- **Refreshing** `A.lock` reads `A.md`, each directly linked file and that file's own lock, and
  nothing deeper. The cost depends on the number of direct links, not the size of the graph.
- **Reading** needs no fetches: any view of A draws from `A.md` and `A.lock` alone. Expanding a
  sub-project loads that file and its lock, one level at a time. An unreachable project shows
  its last known contract, marked "unknown as of <date>".
- **Risk propagates, internals do not.** When C slips, B becomes late, and the next refresh of
  `A.lock` shows A's contract with B at risk. A sees that, but not why. The reason lives in B and
  is B's to show.
- **Each owner decides what is visible.** Without a published `B.lock`, A still has B's promise,
  but whether B is late is unknown.
- **Checking** works the same way. `pmd check FILE` validates FILE against its own lock and its
  direct links, and warns when the lock is stale. `pmd check --deep FILE` validates the whole
  graph reachable from FILE, for when that is wanted and reachable.
- If a direct link cannot be read, its previous entry is kept as it was.
- A changed hash or contract is the change notification. The lock's history is a history of
  every contract the project depended on.

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

| Code | Rule | Message | Suggested fix |
|---|---|---|---|
| E-SYNTAX | Syntax error: a key with no value, an unclosed `<…>` or `[text](target)`, a single-valued key repeated | "Line 14: `owner:` has no value" | Give the key a value or close the bracket |
| E-DATE | Invalid date | "Line 7: `2027-02-30` is not a date" | Write a valid date, rough date or range |
| E-DUP-ID | The same `^id` twice in a file | "`^audit` is defined twice" | Rename one of them |
| E-REF-LOCAL | Dangling local reference | "Line 9: no item `^dyno-testin` in this file" | Correct the id, or add `^id` to the target item |
| E-REF-PUBLIC | Reference to a non-public item | "`battery-pack.md#validated` is not `#public`" | Ask the owner to add `#public`, or depend on the whole project |
| E-REF-MISSING | Linked local file cannot be read | "cannot read file: `body/doors.md`" | Fix the path, or restore the file. An unreachable remote target is only shown as unknown |
| E-CYCLE | Include cycle | "`a.md` includes `b.md`, which includes `a.md`" | Remove one sub-project link |
| E-CONFLICT | Conflict (7.2) | "`Dyno testing` is forecast to end 2027-11-02, after its declared end 2027-10-31" | Move the declared date, shorten the estimate, or ask the container to move its `by:` |
| E-BOUND | Broken contract: a child's promise is later than the parent's `by:` (§9) | "`Powertrain` commits to 2028-03-31, after the bound `by: 2028-02-28`" | Agree a new date: the parent moves its `by:` or the child's owner moves its Commitment |
| W-STALE-LOCK | Stale lock: file hash differs from the lock | "`aurora.lock` is out of date for `battery-pack.md`" | Refresh the lock with `pmd lock` |
| W-NO-OWNER | Missing `Owner` | "`Owner` is not set" | Add `- Owner: Name <email>` under the title |
| W-UNKNOWN-HANDLE | Unknown `@handle` | "`@jan` is not in the people registry; kept as written" | Add it to `.pmd/profile.md`, or write `Name <email>` |

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

## 14 Working with tools and agents

Most plans will be written, checked and changed by AI agents as well as people. The format
stays plain markdown for both; these conventions make machine edits small, safe and traceable.

- **Canonical form.** `pmd fmt FILE` rewrites a file into one canonical spelling without
  changing its meaning (parsing before and after gives the same result; prose is untouched):
  `- ` bullets indented two spaces per level; one space after the checkbox; suffix parts
  separated from the title and from each other by two spaces, in the order range or duration,
  tags, keys (`after`, `by`, `owner`, `approver`, then others alphabetically), `^id` last; no
  column alignment; no trailing spaces; one blank line before a heading; a final newline.
  People may write freely. Tools and agents write canonical form, so every edit is a
  one-line diff and concurrent edits rarely conflict.
- **Ids on demand.** `pmd fmt --ids` gives every item without an `^id` a readable one made
  from its title (lowercase, hyphenated, at most four words, unique in the file). Existing ids
  never change. Agents refer to items by id, not by title.
- **Safe concurrent edits.** A file's hash (as in the lock) is its version. A tool or agent
  reads a file and its hash, edits it, and writes only if the hash is still the same;
  otherwise it reads again. Writing commands take `--if-hash HASH`.
- **Diagnostics an agent can act on.** `pmd check` prints one line per finding,
  `path:line: SEV CODE message`, followed by an indented `fix:` line with the suggested fix
  from §12. Codes are stable. With `--json`, `pmd check` and `pmd schedule` print the same
  information as JSON. Tool output may be JSON; project files are always markdown.
- **An entry point for agents.** A folder of plans may hold an `AGENTS.md` that points to the
  cheat sheet and states local rules. Agents read it first.
- **Edit only what you own.** To ask another project for a date, write `by:` on your own link
  to it; never change its Commitment or milestones. Never write forecasts or lock files.
- **Reasons travel with changes.** When a change moves a Commitment, a `by:` or a public
  milestone, the commit message says why. The lock diff shows what changed; the commit says why.

