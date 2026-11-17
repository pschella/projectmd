# projectmd — design overview

This document explains **how projectmd works and why**, for a developer or reviewer who is new
to it. It states the requirements, the concepts that meet them and where each rule is defined.
The rules themselves live in the reference documents, which this one links rather than repeats.

projectmd is meant to be used by **humans and AI agents together**: the same plain files are read and
written by people in any editor and by agents through the tools, and either can own a project,
make a promise or owe a decision (§8.3).

| For the exact rules, see | |
|---|---|
| [FORMAT.md](FORMAT.md) | semantics: node model, scheduling, interfaces, contracts, diagnostics |
| [GRAMMAR.md](GRAMMAR.md) | how text is parsed |
| [CHEATSHEET.md](CHEATSHEET.md) | the whole format on one page |
| [../examples/org/](../examples/org/README.md) | the scenario this design is tested against |

---

## 1 The problem

Large undertakings consist of many interdependent projects that must come together on strict
dates, with milestones, decision gates and sign-offs. They are owned by different people in
different parts of an organisation (or in different organisations), who use different tools.

- **Ticket trackers** are good at *what* but poor at *when*: timelines and cross-team
  dependencies are bolted on.
- **Scheduling tools** are good at *when* but heavy, centralised and siloed: one file, one
  owner, and everyone else gets a PDF.
- **Text todo lists** are what people actually write, but they say nothing about time.

projectmd keeps the todo list and adds just enough structure to answer *when?*, *what depends on
what?* and *who promised what to whom?*, from one person's list to a programme spread over many
teams, stores and companies.

## 2 Requirements

| # | Requirement |
|---|---|
| R1 | **Plain text people and AI agents both write.** Readable without tools; editable by any editor or agent; diffs stay small. |
| R2 | **Every file is a project.** A todo list is already a valid plan; there is no separate "project definition". |
| R3 | **Time at every level.** Dates, rough dates, durations and dependencies, in calendar time, or weekdays for `d` (no story points). |
| R4 | **Mixed scheduling.** Some dates are promises; most are estimates. Tools forecast the rest and flag conflicts. |
| R5 | **Delegation with accountability.** A level asks for deliverables by dates; the level below promises dates and is accountable for keeping them, or for saying it cannot. Negotiation is normal. |
| R6 | **Any storage, any tool.** Plans may live on disk, in git, on a shared drive, in an object store, a database or a supplier's portal; each level may use its own tools. |
| R7 | **Access control is external; exposure is the only control in the format.** Who can read what is decided by the store. Within the format, a project only decides *what it shows the level above*. |
| R8 | **Scale by locality.** A level only needs to know whether the levels *directly* beneath it are on time. Reading and checking never depend on the size of the graph. |
| R9 | **Drill down lazily.** Expand one level at a time, as far as access allows. |
| R10 | **Grow gradually, top-down or bottom-up.** Every intermediate state is valid. |
| R11 | **No imposed methodology.** No workflows, ticket types, roles or required fields. |
| R12 | **The format is the product.** Tools are replaceable reference implementations; behaviour that matters is specified independently of them. |

Explicitly **out of scope** (left to tools that read the graph): cost, budget, resource allocation
and capacity; working-day calendars; permissions; ticket workflows; dependency types other than
finish-to-start.

## 3 The idea in one paragraph

A plan is a markdown file. Its list items are tasks and milestones; a link to another file makes
that file a **sub-project**, wherever it lives. Between a parent and a child the only things that
cross are a **contract**: the parent's need-by date against the child's **promise**. Forecasts never
cross; a child that cannot keep its promise moves it, and that move is the signal. The contract
state is computed live from the two files, one hop at a time, so a slip deep down surfaces one
level at a time, and any view of any project needs only that project's file and its direct links.
Only project files exist; nothing is generated.

```
              reads only                    reads only
  Sales ─────────────────────► Hardware ─────────────────────► Displays ───► teams …
  vellmar.md                   hardware.md (front)             displays.md
  + hardware.md, software.md   + its direct links              + its direct links
```

## 4 Concepts

### 4.1 One node model

Everything in a plan is a **node**: a task, a milestone, a sub-project, a deliverable, a heading,
and the file itself. They share the same properties (dates, owner, dependencies, children), so
"same shape at every level" is literally true. Dates are either *declared* (written by a person)
or *forecast* (computed by tools). The node kinds are in [FORMAT §2](FORMAT.md#2-the-node-model).

### 4.2 The three laws

Three laws shape every other rule. Nothing in the format contradicts them.

| Law | Statement | What it buys |
|---|---|---|
| **Ownership** | Every file has one owner. You write only what you own. | no merge conflicts between owners; a clear author for every promise |
| **Locality** | Between owners, only declared dates cross a link. Forecasts never do. | privacy between owners (a parent sees promises, not plans); scale (a view reads a file and its direct links); one owner's files are one plan, so they link transparently |
| **Derivation** | Nothing is stored that can be computed: schedules, contract states, whose move it is, queues, drivers and change events. | no stored state to go stale or to synchronise; any tool, given the files, gets the same answer |

Ownership means a parent cannot edit a child's promise and a child cannot edit a parent's ask.
Locality is defined in [FORMAT §7.8](FORMAT.md#78-locality-rule).

### 4.3 The promise protocol

Between a parent and a child there is one exchange. The parent **asks** with `by:` on its link
line. The child **promises** with a Commitment or a dated milestone. The pair is a **contract**;
its state is derived, and every state names who moves next
([FORMAT §9](FORMAT.md#9-contracts)).

| State | Meaning | Whose move |
|---|---|---|
| `delivered` | child has `Done:`, or the milestone is ticked | nobody |
| `unknown` | child cannot be read | parent: fix the link or access |
| `disputed` | promise later than `by:` (whether or not `by:` has passed), or milestone cancelled | parent: accept, re-plan or talk |
| `overdue` | `by:` passed, not delivered, and no promise later than `by:` | child: deliver, or move the promise |
| `requested` | `by:` set, no promise | child: answer |
| `agreed` | promise on or before `by:` | child: deliver |

Each side edits only its own file. Agreement is reached outside the format (a conversation, a pull
request, an agent); the files record the outcome. A disputed contract sits with the parent because
the child's promise is a fact about what it can do, and the parent's `by:` is a wish.

A promiser who cannot deliver on time moves the promise; from that moment the contract is disputed and the next move is the asker's, even if the original need-by date has passed. Overdue means the promiser is silent.

A slip is renegotiated one hop at a time. Here the descent stage slips; each owner decides what
it means for their own promise:

```
  Descent stage            Lunar module (LM)              Programme HQ
  -------------            -----------------              ------------
  forecast passes its
  Commitment (own check:
  E-CONFLICT)
  moves Commitment
  2027-11-30 -> 12-31 ----> reads it: contract with
                            descent stage is `disputed`
                            LM owner decides:
                            (a) accept: by: -> 12-31;
                                LM forecast still inside      sees nothing change
                                its own Commitment  ------->  (contract still agreed)
                            (b) LM forecast now passes
                                its own Commitment:
                                LM moves its Commitment ----> reads it: `disputed`
                                                              HQ owner decides ...
```

A slip that stays inside a team's Commitment changes nothing above the team. A slip past a
Commitment changes exactly one file per level, the file whose owner moves the promise. Each level
sees that the level below moved it, and to when, never why; the why stays in the child's plan,
for those allowed to read it. A worked example is in
[examples/org/README.md](../examples/org/README.md#how-a-slip-travels).

### 4.4 Prior art

The protocol follows Promise Theory: an agent can only promise its own behaviour, and a request
is not a promise. A request is an imposition on someone else; a promise is that agent's own answer
to it. Here `by:` is the request and the Commitment or milestone is the promise. Nobody
promises on another's behalf, and an unanswered request stays visibly `requested`.

### 4.5 Interfaces and exposure

| Concept | Written by | Where |
|---|---|---|
| **ask** (need-by) | parent | `by:` on its link to the child |
| **promise** | child | `Commitment:` in its header; its milestones' dates |
| **interface** | child | derived from its own file ([FORMAT §8](FORMAT.md#8-interfaces-and-fronts)) |

Milestones are exposed by default, because they are what a parent wants to see; `#private` keeps
an internal checkpoint out. The breakdown (tasks, sub-project and deliverable lines) is always
private: to expose one piece of it, add a milestone that is `after:` it. When the whole plan must
stay private, the owner keeps a thin **front** file ([FORMAT §8.1](FORMAT.md#81-front-file-across-an-access-boundary)). Front and plan share one owner, so the front's dated lines show slack against the plan's forecast, and a parent sees only the dates.

## 5 Everything else is derived

Each of these is computed from the files when asked. None is stored, so none can go stale.

- **Schedule.** Forecast start and end of every node, slack, conflicts
  ([FORMAT §7](FORMAT.md#7-scheduling)). A declared date says what was promised; the forecast says
  what will happen. Tools never let the first stand in for the second. A decision someone owes delays
  what waits on it a day per day.
- **Contract state.** From the ask and the promise, one hop ([FORMAT §9](FORMAT.md#9-contracts)).
- **The queue.** `pmd ready`: work to do, and the moves owed (decide, fix, answer, deliver), for one
  owner, human or agent ([FORMAT §13.1](FORMAT.md#131-tool-views-informative)).
- **Drivers and the critical path.** `pmd why` names the one constraint that fixed an item's end.
  The critical path of a large programme is found by following drivers one file at a time, never by
  a global computation.
- **Change events.** `pmd changes OLD NEW` reads two versions of one file and lists what moved:
  the notification feed for parents and children.
- **Replan proposals.** `pmd replan` proposes the smallest edits to a file's own dates that make it
  consistent, and lists the promises that would move.
- **The board.** Four kanban-style columns and a flow forecast, from the same files
  ([BOARDS.md](BOARDS.md)).

## 6 What the files look like

A project file is a markdown file: an H1 title, a header of `Key: value` list items, then list
items for the work and milestones. Shape decides meaning; headings only group. The item syntax,
suffix rules and node kinds are defined in [FORMAT §1-§5](FORMAT.md) and [GRAMMAR.md](GRAMMAR.md).
Text indented under an item is its description: acceptance criteria, reasons, links to specs.
Real files: [examples/lunar/hq/lunar-programme.md](../examples/lunar/hq/lunar-programme.md) (the programme) and
[examples/org/teams/enclosures/](../examples/org/teams/enclosures/) (a team's front and private plan).

## 7 How files link and where they live

### 7.1 Links

- Links are real paths or URLs, clickable in any markdown tool. There are no custom schemes.
- A link line is the only cross-file edge: a sub-project links a file, a deliverable links a
  milestone in a file. `after:` and `by:` take local `^id`s or dates, so a task depends on
  another file by pointing at its link line ([FORMAT §6](FORMAT.md#6-links-and-references)).
- A link that cannot be read is shown as unknown or private, not as an error.
- A project may have several parents (a DAG), each with its own need-by.
- An item grows into a sub-project by **promotion**: replace its title with a link to a new file
  ([FORMAT §6.1](FORMAT.md#61-promotion)).
- A milestone deeper than a direct link is reached by the level in between **re-exposing** it,
  as a milestone of its own `after:` the deeper one. Nobody references two levels down.

### 7.2 Where things can live

The format only holds links. Resolving them, authenticating and enforcing permissions is the job
of whoever provides or follows the link (the store, a server, the viewer). Typical patterns:

| Pattern | The parent links | The parent can see | Use when |
|---|---|---|---|
| **Same store, open** | the child's plan `child.md` | everything; the interface is derived | one team, one repo, small programme |
| **Separate stores** | the child's front `child.md` (plan elsewhere) | the front only | different units, tools or companies |
| **Hand-kept or generated front** | a front written by hand or emitted by another tool | the front only | a team that plans in a spreadsheet or database |

Tools may cache what they read. History is **not assumed**: delivery dates are recorded in the
files (`done:`, `Done:`). **No file is special.** There is no root or central registry: every
project is the top of its own view.

### 7.3 Growing a plan

Each step is optional and keeps every file valid; the steps are listed in the
[README](../README.md#grow-it-gradually-top-down-or-bottom-up). A link with `by:` to a child with
no promise yet is simply a **requested** contract, which is what makes top-down planning work.
Bottom-up, an existing team plan is linked into a new parent. Both meet in the same graph
([examples/org/](../examples/org/README.md) shows both).

## 8 What tools can check and show

Checks are grouped by **what they need to read**. Everything except the last row is bounded by the
number of direct links, never by the size of the graph. Codes and messages are in
[FORMAT §11](FORMAT.md#11-validation-rules).

| Scope (reads) | Checks |
|---|---|
| **One file** | syntax, dates, duplicate ids, local references, missing owner, near-miss keys (`by: Friday`), dependency cycles, forecast conflicts against its own dates and Commitment |
| **File + its direct links** (1 + *d* reads) | contract states of every direct link, deliverable targets are exposed milestones, linked local files exist |
| **Whole reachable graph** (`--deep`, opt-in) | include cycles, plus everything above for every file |

Each finding is one line, `path:line: SEV CODE message`, followed by a `fix:` hint. Severity: E
fails, W fails only with `--strict`, I never fails. `--json` gives the same as JSON.

### 8.1 Reference tools

| Command | Does | Reads |
|---|---|---|
| `pmd parse FILE` | the parsed structure, as JSON | file |
| `pmd check FILE [--deep] [--strict] [--today D] [--json]` | validate | §8 above |
| `pmd schedule FILE [--today D] [--json]` | forecast dates and conflicts | file + direct links |
| `pmd tree FILE [--depth N]` | outline with contract states | file + direct links |
| `pmd ready FILE... [--deep] [--owner NAME] [--today D] [--json]` | the queue: do, decide, fix, answer, deliver | file + direct links (`--deep`: all linked files) |
| `pmd why FILE [^id] [--deep] [--json]` | what drives an item's end | file + direct links |
| `pmd changes OLD NEW [--json]` | events between two versions of one file | the two versions |
| `pmd replan FILE [--write]` | edits that make a file consistent (dry run by default) | file + direct links |
| `pmd fmt FILE [--check]` | canonical form (rewrites in place unless `--check`) | file |

A typical cycle for one project: `pmd check` (contract states and own conflicts), `pmd replan` if
needed, and edit the front if a promise moves. Each owner works on their own schedule; nobody
orchestrates the whole graph. Only diagnostics are normative; other output is informative.

### 8.2 Why it scales

Views, dashboards and bots (the repository's Gantt viewer is one example) inherit their scaling
from the three laws, not from clever implementation. The cost of each operation is in
[FORMAT §9](FORMAT.md#9-contracts). The reasons it holds:

- **Bounded fan-in.** Everything a level needs about the levels below is already in the promise of
  its direct links. A view never has to load the programme to say whether it is on time.
- **Changes travel only when promises change.** Internal edits stop at the team. A real slip moves
  up one level each time an owner moves a promise; unrelated branches are never read or written.
- **No shared mutable state.** Every file has one owner and is written only by that owner (or the
  owner's tools). There is no central index, so work in different subtrees runs in parallel.
- **Plain files.** Plans and fronts can be served and cached by any store, web server or CDN.
- **Graceful at access boundaries.** If a child cannot be read, its line is shown as unknown and
  scheduled at the parent's own `by:` or estimate. Drilling down stops where access stops.
- **Any project is a root.** Opening a team plan costs the same as opening the top of the programme.

Where it strains: very wide nodes over slow links (500 children on 50 servers means 500 fetches;
use HTTP caching or a tool-local cache), and a promise that moves but stays inside `by:` is
invisible to the parent, which is intended. Asking for everything at once (a deep check, a fully
expanded chart) is opt-in and linear in the graph size; measurements are in
[BENCHMARKS.md](BENCHMARKS.md).

### 8.3 Humans and agents

Plans mix humans and agents. Agents make most edits; the conventions that keep their edits small
and safe are in the [CHEATSHEET](CHEATSHEET.md) and
[FORMAT §13](FORMAT.md#13-working-with-tools-and-agents): canonical form, a short `AGENTS.md` per
folder, diagnostics with fix hints and JSON output, `done:` written when ticking, and **edit only
what you own**.

The queue is the same for both. `pmd ready --deep ROOT --owner NAME` lists a person's or an agent's
work and owed moves. A decision a human owes is an open item owned by that human, so its delay shows in the forecast. Agents claim work by recording
`after: <today>`. This was tested in a simulated mixed team: agents planning a library release
with two humans.

### 8.4 Interoperability

Different tools at different levels must agree. Normative today: the grammar and FORMAT, with a
language-neutral conformance corpus in `testdata/` as the arbiter. Planned: golden schedule
tests (input files to expected JSON) and JSON schemas for tool output.

## 9 Requirements → concepts

| Req | Met by |
|---|---|
| R1 plain text, human + agent | markdown lists; suffix parsed right to left with known keys only; canonical form; item descriptions |
| R2 every file a project | one node model; a checkbox list is a valid plan |
| R3 time at every level | dates, coarse dates, ranges, calendar durations, `after:` on any node |
| R4 mixed scheduling | declared vs forecast; one forecast algorithm; one conflict rule; derivation law |
| R5 delegation, accountability | the promise protocol: `by:` vs `Commitment:`; states with whose move; moving a promise is the delay notice; `done:` |
| R6 any storage, any tool | real links; one file type, any tool can emit it; no reliance on history |
| R7 exposure is the only control | interface = milestones (unless `#private`) + header; breakdown always private; separate stores link only fronts |
| R8 scale by locality | locality law; 1 + *d* reads; only promises cross |
| R9 lazy drill-down | links expand one level at a time |
| R10 grow either way | every intermediate state is valid; `requested` placeholders; any file can be linked later; no special root |
| R11 no methodology | no workflows or roles; extra header keys are kept as written |
| R12 format is the product | normative FORMAT and GRAMMAR; stability policy; conformance corpus; golden tests planned |

## 10 Open questions

1. **`#private` noise.** Exposing milestones by default means internal checkpoints need a tag.
   Is that the right default for most teams?
2. **Dependency types.** Only finish-to-start. Enough?
3. **Recurring needs.** A need that repeats (one launch vehicle per flight) costs a few lines per
   instance; a sub-project per instance is the structural answer today. Is a series form ever worth it?
4. **The "why" without version history.** Reasons for a moved promise go in item descriptions and
   commit messages. A description records the latest reason; stores without history lose the earlier ones.
5. **Working time vs calendar time.** Decided (D71): `d` counts weekdays, Monday to Friday, with no
   holidays or calendars; `w` and `m` stay calendar time. Who needs calendar time uses weeks, a range or dates.
6. **Conditional plans.** A plan that depends on a decision (if the gate passes, scope A; else
   scope B) is written today as two items, one of which is cancelled `[-]` once the gate decides.
   Is a conditional form worth its complexity?

## 11 Status

The format is **0.9 (beta)**. The stability policy is in [FORMAT §0](FORMAT.md#0-status-and-stability).
The reference parser, CLI and viewer implement FORMAT.md; the open questions in §10 are still open.
