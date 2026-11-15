# projectmd — design overview

This document explains **how projectmd works and why**, for a developer or reviewer who is new
to it. It covers the requirements, the concepts that meet them, what the files look like, how
they link, where they can live, and what tools can check. It is the place to start before the
reference documents, and the place to aim feedback at.

| For the exact rules, see | |
|---|---|
| [FORMAT.md](FORMAT.md) | format reference (normative semantics) |
| [GRAMMAR.md](GRAMMAR.md) | precise grammar of project, lock and interface files |
| [CHEATSHEET.md](CHEATSHEET.md) | the whole format on one page |
| [../examples/org/](../examples/org/README.md) | the scenario this design is tested against |

---

## 1 The problem

Large undertakings consist of many interdependent projects that must come together on strict
dates, with milestones, decision gates and sign-offs. They are owned by different people in
different parts of an organisation (or in different organisations), who use different tools.

- **Ticket trackers** (JIRA-like) are good at *what* but poor at *when*: timelines and
  cross-team dependencies are bolted on.
- **Scheduling tools** (MS-Project-like) are good at *when* but heavy, centralised and siloed:
  one file, one owner, one tool, everyone else gets a PDF.
- **Text todo lists** are what people actually write, but they say nothing about time.

projectmd keeps the todo list and adds just enough structure to answer *when?*, *what depends on
what?* and *who promised what to whom?*, in a way that scales from one person's list to a
programme spread over many teams, stores and companies.

## 2 Requirements

| # | Requirement |
|---|---|
| R1 | **Plain text people and AI agents both write.** Readable without tools; editable by any editor or agent; diffs stay small. |
| R2 | **Every file is a project.** A todo list is already a valid plan; there is no separate "project definition". |
| R3 | **Time at every level.** Dates, rough dates, durations and dependencies, with real calendar time (no story points). |
| R4 | **Mixed scheduling.** Some dates are promises; most are estimates. Tools forecast the rest and flag conflicts. |
| R5 | **Delegation with accountability.** A level asks for deliverables by dates; the level below promises dates and is accountable for keeping them, or for saying it cannot. Negotiation is normal. |
| R6 | **Any storage, any tool.** Plans may live on disk, in git, on a shared drive, in an object store, a database or a supplier's portal; each level may use its own tools. |
| R7 | **Access control is external; exposure is the only control in the format.** Who can read what is decided by the store. Within the format, a project only decides *what it shows the level above*. |
| R8 | **Scale by locality.** By default a level only needs to know whether the levels *directly* beneath it are on time. Reading, refreshing and checking never depend on the size of the graph. |
| R9 | **Drill down lazily.** Expand one level at a time, as far as access allows. |
| R10 | **Grow gradually, top-down or bottom-up.** A programme lead may sketch the top and link empty placeholders; a team may start with a todo list and be linked in later. Every intermediate state is valid. |
| R11 | **No imposed methodology.** No workflows, ticket types, roles or required fields. |
| R12 | **The format is the product.** Tools are replaceable reference implementations; behaviour that matters is specified independently of them. |

Explicitly **out of scope** (left to other tools that read the graph): cost, budget, resource
allocation and capacity; working-day calendars; permissions; ticket workflows; dependency types
other than finish-to-start.

## 3 The idea in one paragraph

A plan is a markdown file. Its list items are tasks and milestones; a link to another file
makes that file a **sub-project**, wherever it lives. Between a parent and a child the only
things that cross are a **contract** — the parent's need-by date (`by:`) against the child's
**promise** (its `Commitment:` and milestones) — and, when the child cannot keep its promise, a
dated **lateness signal**. The child's promise is its **interface**: a small projectmd file it
publishes; everything else in its plan is private. Each project keeps a tool-written **lock
file** caching the interfaces of the projects it links directly, with the state of each
contract. A slip deep down therefore surfaces one level at a time, and any view of any project
needs only that project's file and its lock.

```
              reads only                    reads only
  Sales ─────────────────────► Hardware ─────────────────────► Displays ───► teams …
  vellmar.md                   hardware.public.md              displays.public.md
  vellmar.lock  (cache of       (interface, published by        (interface, published by
   hardware's & software's       hardware from hardware.md       displays from displays.md
   interfaces + contract state)  + hardware.lock)                + displays.lock)
```

## 4 Concepts

### 4.1 One node model

Everything in a plan is a **node**: a task, a milestone, a sub-project, a deliverable, a heading,
and the file itself. All nodes have the same properties (dates, owner, dependencies, children),
so "same shape at every level" is literally true. Dates come in two kinds only:

- **declared**: written by a person — a milestone's date, a task's date or range, a `by:`, a
  `Commitment:`;
- **forecast**: computed by tools from durations, dependencies and children.

There is **one conflict rule**: a node is in conflict when its forecast end is later than a
declared end that applies to it (its own date, its container's `by:`, or the Commitment).

### 4.2 Contracts, promises and interfaces

| Concept | Written by | Where | Meaning |
|---|---|---|---|
| **need-by** | parent | `by:` on its link to the child | "I need this by …" |
| **promise** | child | `Commitment:` in its header; its milestones' dates | "I will deliver by …" |
| **contract** | — | the pair of the two | holds, is disputed, at risk, delivered … (§7.3) |
| **interface** | child | its published `NAME.public.md` | everything the child exposes: promise, milestones, lateness, freshness |
| **lateness** | child (or its tools) | `Late:` in the interface | "I will not make my promise; expect …" |
| **freshness** | child's tools | `As-of:` in the interface | "this information is current as of …" |

Each side **edits only its own file.** A parent never changes a child's promise; a child never
changes a parent's need-by. Agreement is reached outside the format (a conversation, a pull
request, an agent); the files record the outcome.

### 4.3 What is private and what is exposed

- **Milestones** (items with a leading date) are **exposed by default**: they are what a parent
  wants to see. `#private` keeps an internal checkpoint out.
- **The breakdown** — tasks, sub-project lines, deliverable lines — is **private by default**.
  `#public` exposes one, as a milestone at its declared end.
- Header fields `Owner`, `Commitment`, `State`, `ID`, `Done`, `Late` are exposed; free text,
  risks, people other than the owner, and links are not.

A parent may reference only exposed items of another project. Internal items can therefore be
renamed, re-estimated or restructured without anyone else noticing.

## 5 What the files look like

### 5.1 A project file

```markdown
# Displays                                         ← H1: the project's name

- Owner: Greta Lindqvist <greta.lindqvist@example.com>   ← header: a property list
- Parent: [Hardware](../hardware.md)                     ←   (YAML front matter also allowed)
- Commitment: 2028-02-29
- State: in-development
- ID: pis.example.com/units/hardware/displays

Station display units: panel, enclosure, controller board.   ← prose is just prose

## Work                                            ← headings group nodes (optional)

- [x] 2026-10-30 Display architecture agreed  #private  done: 2026-10-30  ^arch
- [ ] [Display panels](../../../teams/display-panels/display-panels.public.md)  by: 2027-12-31
- [ ] [Enclosures](../../../teams/enclosures/enclosures.public.md)  by: 2027-12-31
- [ ] Pilot batch assembly and test  20w  after: ^arch  ^pilot-assembly
- [ ] 2027-07-15 Pilot batch ready  ^pilot-batch
- [ ] Series assembly and test  3w  after: [Panels][panels]  after: [Enclosures][enclosures]
- [ ] 2028-02-29 Series displays ready  ^series-ready

[panels]: ../../../teams/display-panels/display-panels.public.md
[enclosures]: ../../../teams/enclosures/enclosures.public.md
```

**Header.** Consecutive `Key: value` list items right after the H1. Core keys: `Owner`,
`Stakeholders`, `Commitment`, `State`, `ID`; for interfaces and delivery: `Done`, `Late`,
`As-of`, `Plan`; informational: `Parent`. Any other key is allowed and kept. Profiles
(`.pmd/profile.md`) may declare vocabularies and extra keys.

**Item line.** `- [checkbox] [leading date] title  suffix`.

- A list item is a planning item only if it has a checkbox or a leading date; otherwise it is
  prose. Checkbox: `[ ]` open, `[x]` done, `[-]` cancelled (or gate rejected, or withdrawn).
- A **leading date** makes the item a **milestone**.
- The **suffix** is read right to left and stops at the first word that is not a planning
  token, so ordinary titles ("Fix issue: login", "12m cable") are safe. Tokens: a duration
  (`3w`, `10d`, `2m`) or range (`2027-03 .. 2027-06`), `#tags`, known keys (`after:`, `by:`,
  `owner:`, `approver:`, `done:`, `replaced-by:` + profile keys), and `^id` last. Code spans
  escape anything.
- Sub-bullets without a checkbox are notes; with a checkbox they are child nodes.

**Node kinds**, decided by shape alone:

| Shape | Kind |
|---|---|
| `- [ ] 2027-07-15 Pilot batch ready` | milestone |
| `- [ ] Pilot batch assembly  20w` | task |
| `- [ ] [Enclosures](…/enclosures.public.md)  by: …` — whole title is a link to a file | sub-project |
| `- [ ] [Pack B-samples](…/battery-pack.md#b-samples)  by: …` — link to a milestone | deliverable |
| `## Phase 1  2026-Q4 .. 2027-Q4` | heading (range = estimate) |

**Dates.** Exact days, months (`2027-06`), quarters (`2027-Q3`), halves (`2027-H2`), ISO weeks
(`2027-W14`) and ranges (`a .. b`). Coarse dates are exact aliases for inclusive ranges; a
deadline uses the end of its range. Durations are real calendar time (`1w` = 7 days, `1m` = one
calendar month).

### 5.2 An interface file

Generated by `pmd export displays.md`, or written by hand, or emitted by any other tool:

```markdown
<!-- projectmd interface 0.2 -->
# Displays

- Owner: Greta Lindqvist <greta.lindqvist@example.com>
- Commitment: 2028-02-29
- State: in-development
- ID: pis.example.com/units/hardware/displays
- Late: 2028-04-05
- As-of: 2026-11-12
- Plan: [Displays plan](displays.md)

- [ ] 2027-07-15 Pilot batch ready  ^pilot-batch
- [ ] 2028-02-29 Series displays ready  ^series-ready
```

It is an ordinary projectmd file — **there is no second format** — restricted to the H1, the
header and the exposed milestones. `Plan:` lets readers who have access drill into the full
plan. Exporting an interface gives the same interface. A team with no tooling can keep such a
file by hand; a team using a database or spreadsheet can generate it.

### 5.3 A lock file

`displays.lock`, next to `displays.md`, written only by tools:

```text
# projectmd lock 0.2
../../../teams/display-panels/display-panels.public.md  id=pis.example.com/teams/display-panels  hash=d5d329  by=2027-12-31  commitment=2027-11-30  start=2027-05-31  status=agreed  as-of=2026-11-12  seen=2026-11-15
    pilot-panels  2027-05-31
    series-panels  2027-11-30
../../../teams/enclosures/enclosures.public.md  id=pis.example.com/teams/enclosures  hash=3bfa72  by=2027-12-31  commitment=2027-12-15  start=2027-06-30  late=2028-03-15  status=at-risk  as-of=2026-11-13  seen=2026-11-15
    pilot-enclosures  2027-06-30
    series-ready  2027-12-15
```

One entry per **directly** linked project: where it is, its id, the **interface hash**, the
contract fields (`by`, `commitment`, `late`, `status`), freshness (`as-of`, `seen`), `done`, and
its exposed milestones. A deliverable contract gets its own entry, `location#id  by=…
status=…`. Never tasks, people or other internals. The lock's history (in whatever store keeps
history) is the history of every contract the project depended on.

## 6 How files link and where they live

### 6.1 Links

- **Links are real paths or URLs**, clickable in any markdown tool. No custom schemes.
- A link item whose whole title is a link to a file is a **sub-project**; to `file#id`, a
  **deliverable**. `after: target` is a dependency on a local `^id`, another file's exposed
  milestone (`file#id`), or a whole project (its end).
- A stable **id** may ride in the link title (`"id:pis.example.com/teams/enclosures"`). The path
  is a hint and the id is the truth; tools can repair a path after a move.
- Long or remote targets use **reference-style links** at the end of the file, so all external
  dependencies of a plan can be read at a glance.
- A project may have several parents (a DAG), each with its own need-by. `Parent:` in a child's
  header is optional and informational (navigation only).
- A milestone deeper than a direct link is reached by the level in between **re-exporting** it:
  a milestone of its own `after:` the deeper one. Nobody references two levels down.

### 6.2 Where things can live

The format only holds links; resolving them, authenticating and enforcing permissions is the job
of whoever provides or follows the link (the store, a server, the viewer). Typical patterns:

| Pattern | The parent links | The parent can see | Use when |
|---|---|---|---|
| **Same store, open** | the child's full plan `child.md` | everything; tools derive the interface | one team, one repo, small programme |
| **Separate stores** | the child's `child.public.md` | the interface only; `Plan:` opens more if access allows | different units, tools or companies |
| **Thin file** | a hand-kept interface | the interface only | a party with no projectmd tooling |
| **Generated** | an interface emitted by another tool or a database view | the interface only | a team that plans elsewhere |

The sidecar convention (`NAME.lock` and `NAME.public.md` next to `NAME.md`) needs only that a
store can hold a second object next to the first; object stores and databases can. History is
**not assumed**: delivery dates are recorded in the files (`done:`, `Done:`), not taken from
version control.

**No file is special.** There is no root, registry or central lock. Every project is the top of
its own view; any sub-project can be moved elsewhere, lock and all, and stands on its own.

### 6.3 Growing a plan

Each step is optional and keeps everything valid:

1. A todo list: `- [ ] Order cells`.
2. Add time where it helps: a date, a duration, an `after:`. Tools now forecast and flag conflicts.
3. Split an item that grew into its own file and link it: it is now a sub-project.
4. Promise and expose: add a `Commitment:`; milestones are exposed by default.
5. Distribute: move the sub-project to its own store and let the parent link its interface.

Top-down, a lead writes the skeleton and links placeholder files; a link with `by:` to a child
with no promise yet is simply a **requested** contract. Bottom-up, an existing team plan is
linked into a new parent. Both meet in the same graph (examples/org shows both).

## 7 How it computes

### 7.1 Forecast

```
start = latest of: declared start, forecast end of every `after:` node,
        start of the enclosing heading's range
end   = start + duration       if the node has a duration
      = latest child end       else if it has children
      = declared end           otherwise
an unfinished node never ends before today; a milestone never ends before its start
```

A sub-project or deliverable ends at the child's promised date — or its `late` date, if the
child reported one. A cancelled node is not waited for. A heading's range is an **estimate**: it
holds back the work under it, but never causes a conflict or extends a forecast. Tools given the
same files must compute the same forecast.

### 7.2 The locality rule

> **Promises cross boundaries as contracts; lateness crosses as a dated signal (`late`,
> `as-of`); neither ever changes another file's declared dates. A file's tools read only the
> file, its lock and the interfaces of its direct links.**

This is what makes R8 a mechanical property rather than a hope.

### 7.3 Contract states

Derived per contract, written into the lock, tested in this order:

| State | When | Severity in the parent's check |
|---|---|---|
| `accepted` | the parent ticked its own link line `[x]` | — |
| `delivered` | the child has `Done:`, or the milestone is `[x]` | — |
| `unknown` | the latest refresh could not read the child (last known values kept) | W-UNKNOWN |
| `requested` | the parent has a `by:`, the child has no promise yet | I-REQUESTED |
| `disputed` | the promise is later than the need-by, or the milestone was withdrawn | W-DISPUTED |
| `at-risk` | the promise holds, but the child reports `late` past the need-by | W-AT-RISK |
| `agreed` | the promise is no later than the need-by | — |

A link without `by:` is a dependency, not a contract. Negotiation (`requested`, `disputed`) is
a normal state, not an error; `pmd check --strict` makes warnings fail, for CI where a level
wants every contract agreed.

### 7.4 How a slip travels (hoisting)

Like export data in a Go build: everything a parent needs from a grandchild is already folded
into the child's interface. From examples/org, where the enclosures team's mould supplier slips:

| Refresh | Who runs it | What changes | What does not |
|---|---|---|---|
| 0 | enclosures | its plan forecasts 2028-03-15 > Commitment 2027-12-15: `E-CONFLICT` in its own check; owner re-plans or publishes `Late: 2028-03-15` | — |
| 1 | displays | `displays.lock`: enclosures `late=2028-03-15 status=at-risk`; displays' own forecast passes its Commitment → `displays.public.md` gets `Late: 2028-04-05` | hardware, sales |
| 2 | hardware | `hardware.lock`: displays at-risk; `hardware.public.md` gets `Late: 2028-04-26` | sales |
| 3 | sales | `vellmar.lock`: hardware `status=at-risk`; sales' own forecast (rollout ends 2028-06-25) still meets 2028-06-30, so the slip is **absorbed** | the customer promise |

Two properties follow and are the acceptance test of the design:

- **A slip that stays within a team's Commitment changes no file above the team.** Its
  interface is unchanged, so its interface hash is unchanged, so no lock above it changes.
- **A slip past a Commitment changes exactly one lock per level per refresh,** and each level
  sees only *that* the level below is late and by how much — never why. The why is in the
  child's plan, for those allowed to read it.

**Freshness.** Because lateness climbs one refresh at a time, the top could look on time while a
team several levels down is already late. So freshness is hoisted too: a project's `As-of` is
never later than the oldest `as-of` among its unfinished direct links. A view can say "on time,
as of 3 days ago" and know that this covers everything beneath.

## 8 What tools can check, and how

Checks are grouped by **what they need to read**. Everything above the last row is bounded by
the number of direct links, never by graph size.

| Scope (reads) | Checks | Codes |
|---|---|---|
| **One file** | syntax, dates, duplicate ids, local references, missing owner, unknown `@handle`, near-miss keys (`by: Friday`), forecast and conflicts against its own dates and Commitment | E-SYNTAX, E-DATE, E-DUP-ID, E-REF-LOCAL, W-NO-OWNER, W-UNKNOWN-HANDLE, W-KEYLIKE, E-CONFLICT |
| **File + its lock** | contract states of every direct link, from cached interfaces (no fetches) | W-DISPUTED, W-AT-RISK, W-UNKNOWN, I-REQUESTED |
| **File + interfaces of direct links** | lock freshness (interface hash changed), references target exposed items, linked local files exist | W-STALE-LOCK, E-REF-PUBLIC, E-REF-MISSING |
| **Whole reachable graph** (`--deep`, opt-in) | include cycles; every file's checks | E-CYCLE + all of the above |

Each finding is one line, `path:line: SEV CODE message`, followed by a `fix:` hint; codes are
stable; `--json` gives the same as JSON. Severity: E fails, W fails only with `--strict`, I
never fails.

### 8.1 Reference tools

| Command | Does | Reads |
|---|---|---|
| `pmd check FILE [--deep] [--strict] [--json]` | validate | §8 |
| `pmd schedule FILE` | forecast dates and conflicts | file + lock |
| `pmd export FILE` | write the interface (`NAME.public.md`) | file + lock |
| `pmd lock FILE` | refresh `NAME.lock` | file + interface of each direct link |
| `pmd fmt FILE [--ids] [--check]` | canonical form; generate readable `^id`s | file |
| `pmd tree`, `pmd stats` | outline, counts | file (+ lock) |

A typical **refresh cycle** for one project: `pmd lock` (pull the children's interfaces),
`pmd check` (see contract states and own conflicts), re-plan if needed, `pmd export` (publish
the interface for the parent). Run bottom-up by each owner, on their own schedule, in their own
CI; nobody orchestrates the whole graph.

### 8.2 How tools built on the format scale

Views, dashboards, reports and bots (the repository's expandable Gantt viewer is one example)
all inherit their scaling from the composition rules, not from clever implementation. Let *d*
be the number of direct links of a project and *n* the size of the whole graph.

| Operation | Reads | Writes | Depends on *n*? |
|---|---|---|---|
| Show a project with the state of everything directly below it | its file + its lock (2) | — | no |
| Expand one child | the child's file or interface + its lock (2) | — | no |
| Drill *k* levels along one path | 2*k* | — | no |
| Refresh a project | its file + *d* interfaces | its lock, its interface | no |
| Check a project | its file + its lock (+ *d* interfaces for staleness) | — | no |
| Propagate a slip *h* levels up | one refresh per level, on the path only | ≤ 2 files per level; 0 if absorbed or inside the Commitment | no |
| Deep check, or draw everything at once | all reachable files | — | yes — opt-in only |

Why this holds:

- **Bounded fan-in.** Everything a level needs about the levels below is folded into the
  interfaces of its direct links. A view never has to "load the programme" to say whether it is
  on time; the lock already says it for each direct child.
- **Changes travel only when promises change.** Locks compare interface hashes, so internal edits
  stop at the team. A real slip moves up one level per refresh, along one path; siblings and
  unrelated branches are never read or written.
- **No shared mutable state.** Every file has one owner and is written only by that owner (or the
  owner's tools). There is no central index, database or global lock, so refreshes in different
  subtrees run in parallel, on each owner's own schedule and infrastructure, without coordination.
- **Static, cacheable files.** Plans, interfaces and locks are plain files that any store, web
  server or CDN can serve and cache; the interface hash is a natural cache key. A tool that has
  seen a hash before needs nothing new.
- **Graceful at access boundaries.** If a child cannot be read, the parent still shows the last
  known contract from its lock, marked `unknown` with its age. Drilling down stops where access
  stops; nothing above breaks.
- **Any project is a root.** Opening a team plan costs the same as opening the top of the
  programme, so tools can start anywhere (a team's own dashboard, a business unit's report,
  re-rooting a view on a sub-project).
- **Aggregation is linear in projects, not tasks.** A portfolio tool that wants "every late
  contract in the organisation" crawls locks (one small file per project) instead of parsing
  every plan, and can stop at any depth.

What deliberately does *not* scale is asking for everything at once (a deep check, a fully
expanded chart of every task); those are opt-in and linear in *n*. Measurements with synthetic
graphs of up to ~111,000 files are in [BENCHMARKS.md](BENCHMARKS.md): showing the top level stays
under 10 ms regardless of graph size, while a full load grows linearly.

### 8.3 Working with AI agents

Agents are expected to make most edits. The conventions keep those edits small and safe:
canonical form (`pmd fmt`, so every edit is a one-line diff), readable `^id`s on demand, writes
guarded by the file hash (`--if-hash`), diagnostics with fix hints and JSON output, an
`AGENTS.md` per folder pointing at the cheat sheet, write `done:` when ticking, and **edit only
what you own** (to ask another project for a date, change your own `by:`, never its file).

### 8.4 Interoperability

Different tools at different levels must agree. Normative today: the grammar (with a
language-neutral conformance corpus in `testdata/`) and the semantics in FORMAT. Planned:
golden tests (input files → expected JSON) for scheduling, contract states, export, lock and
fmt, and a JSON schema for parser output. Generated files carry a format version
(`projectmd interface 0.2`, `projectmd lock 0.2`).

## 9 Requirements → concepts

| Req | Met by |
|---|---|
| R1 plain text, human + agent | markdown lists; suffix parsed right to left with known keys only; canonical form; one-line diffs |
| R2 every file a project | one node model; a checkbox list is a valid plan (conformance level 0) |
| R3 time at every level | dates, coarse dates, ranges, calendar durations, `after:` on any node |
| R4 mixed scheduling | declared vs forecast; one forecast algorithm; one conflict rule |
| R5 delegation, accountability | `by:` vs `Commitment:`; deliverable contracts; contract states; own-Commitment conflict; `Late:`; `done:` |
| R6 any storage, any tool | real links; interface is a plain projectmd file any tool can emit; sidecars only; no reliance on history |
| R7 exposure is the only control | interface = milestones (unless `#private`) + header; breakdown private unless `#public`; separate stores link only interfaces |
| R8 scale by locality | locality rule; per-project locks of direct links; hoisted lateness and freshness; interface hash |
| R9 lazy drill-down | lock gives the first level free; `Plan:` and links expand one level at a time |
| R10 grow either way | conformance levels 0–4; `requested` placeholders; any file can be linked later; no special root |
| R11 no methodology | no workflows or roles; open `State`; profiles for local vocabularies |
| R12 format is the product | normative FORMAT/GRAMMAR; conformance corpus; planned golden tests; versioned generated files |

## 10 Open questions — feedback welcome

1. **Tombstones.** The exact shape for a removed or renamed exposed milestone
   (`[-] … replaced-by: ^new`).
2. **State vocabulary.** `State` is free text; should the core suggest a minimal set, or leave it
   entirely to profiles?
3. **Header style.** Property list vs YAML front matter as the documented default.
4. **Freshness drag.** One team that never refreshes keeps `As-of` old for everything above it.
   That is honest, but should views also show *which* link is stale?
5. **`#private` noise.** Exposing milestones by default means internal checkpoints need a tag.
   Is that the right default for most teams?
6. **How asks reach a child.** Currently outside the format (notifications, PRs, agents), with
   an optional `Parent:` link. Is a child-side view of "what is asked of me" needed in the core?
7. **Dependency types.** Only finish-to-start. Enough?
8. **Gaps from a JIRA / MS Project perspective.** A list of 25 questions is pending review
   (e.g. baselines, recurring work, partial completion).

## 11 Status

The format is an **early draft (0.2)**. The reference parser, CLI and viewer are being brought
up to 0.2 (export, interface hash, contract states, `#private`); until then some behaviour
described here exists only in the specification and in the hand-written `examples/org/`.
