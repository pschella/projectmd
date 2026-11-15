# Worked example: a company, three stores

A passenger information system (station displays and apps) sold to the fictional Vellmar Transit
Authority, planned across sales, business units and teams. Today is 2026-11-15. Every file is an
ordinary project file written by people; nothing here is generated, and contract states are
computed live when a tool reads a file and its direct links.

**The rule.** Each folder under `org/` stands for a separate store with its own access (sales:
shared drive; units: git; teams: object store). A level links only the **front file** of the level
below. A front file is a thin project (header, milestones, deliverable lines into the private plan)
that the owner keeps by hand; the breakdown stays in `private/NAME-plan.md`, readable inside the
owner's store only. Same-store links point at plans directly (Hardware to Displays).

```
org/
├── sales/vellmar/vellmar.md          store 1: sales (links fronts of the units)
├── units/                            store 2: business units
│   ├── hardware/hardware.md          front, readable by sales
│   │   ├── private/hardware-plan.md  plan (links displays.md, same store)
│   │   └── displays/displays.md      sub-unit plan, same store (links team fronts)
│   └── software/software.md          front;  private/software-plan.md  plan
└── teams/                            store 3: teams
    ├── display-panels/ enclosures/ backend/ mobile-app/ data-feeds/
    │       NAME.md (front)   private/NAME-plan.md (plan)
    └── accessibility/accessibility.md   placeholder plan, no front yet
```

Fronts promise with declared dates: each front milestone is `after:` a deliverable line that
points at the same milestone in the plan. The owner's own
`pmd check NAME.md` reads front and plan, so a plan that drifts past its promise shows `E-CONFLICT`
there. A parent reads the front only; the deliverable lines into `private/` are never followed,
so nothing is an error and nothing about the breakdown crosses the boundary.

## Contract states (computed live, `pmd check` / `pmd tree`)

| File | Link | `by:` | Promise (child's file) | State |
|---|---|---|---|---|
| vellmar | Hardware | 2028-03-31 | 2028-03-31 | agreed |
| vellmar | Hardware `#pilot-displays` | 2027-07-31 | 2027-07-31 | agreed |
| vellmar | Software | 2028-04-30 | 2028-04-30 | agreed |
| vellmar | Software `#pilot-ready` | 2027-08-31 | 2027-08-31 | agreed |
| hardware-plan | Displays | 2028-02-29 | 2028-02-29 | agreed |
| displays | Display panels | 2027-12-31 | 2027-11-30 | agreed |
| displays | Enclosures | 2027-12-31 | 2028-03-15 | disputed |
| software-plan | Backend | 2028-01-31 | 2028-01-31 | agreed |
| software-plan | Mobile app | 2027-12-31 | 2028-02-29 | disputed |
| software-plan | Data feeds `#timetable-adapter` | 2026-12-15 | `[x]` done 2026-11-06 | delivered |
| software-plan | Accessibility | 2027-06-30 | none yet | requested |

`pmd check` results (`--today 2026-11-15`): `vellmar.md`, both fronts and all team fronts and
plans are clean; `displays.md` reports `W-DISPUTED` (Enclosures) and `E-CONFLICT` (its series
assembly is forecast to end 2028-04-05, after its Commitment 2028-02-29); `software-plan.md`
reports `W-DISPUTED` (Mobile app) and `I-REQUESTED` (Accessibility). Disputed is a warning, not
an error: `pmd check --strict` (CI) fails on it, plain `pmd check` passes.

## How a slip travels

Enclosures' tooling supplier slips (moulds 2028-01-18). The owner moves the series milestone and
the Commitment to 2028-03-15 in the plan **and** the front; the owner's check keeps the two
honest (a front promise earlier than the plan is `E-CONFLICT`). No other file is written.

- Next time anyone reads Displays, the Enclosures contract is `disputed` (promise 2028-03-15,
  `by: 2027-12-31`), and Displays' own check shows `E-CONFLICT`. Hardware and Sales still see
  Displays at 2028-02-29: `agreed`. Forecasts never cross a file boundary, only declared dates do.
- Displays' owner decides: absorb, or move the Commitment and `series-ready` to 2028-04-05. That
  one edit is the whole propagation step. On its next read Hardware shows Displays `disputed` and
  its own `E-CONFLICT` (integration test ends 2028-04-26, after 2028-03-31).
- Hardware moves its Commitment and `series-displays` to 2028-04-26 (plan and front). Sales reads
  the front: Hardware `disputed`. Sales absorbs it: rollout installation (8w) still ends
  2028-06-25, inside 2028-06-30. Nothing else changes.

A slip inside a team's Commitment (Backend loses a week) changes only its private plan; no front
changes, so nobody above sees anything.

## Other things shown

- **Top-down request.** Software created Accessibility with `by: 2027-06-30` before it had a plan:
  `requested`, scheduled at the line's own `by:` until it promises.
- **Delivered.** Data feeds' front has `[x] Timetable adapter live`; Software's deliverable line
  turns `delivered`.
- **Bottom-up.** Data feeds started as a todo list and grew (snippets below); any file can be linked later.
- **Unreadable links.** A parent that cannot read a link shows it as unknown (private); `W-UNKNOWN`
  appears only on lines with `by:`, and the parent schedules the line at its own `by:` or estimate.
- **Agents** read `docs/CHEATSHEET.md`, edit only their own files, write canonical form and finish
  with `pmd check` (`--strict` in CI).

## Data feeds, bottom-up in five snippets

```markdown
- [ ] Timetable adapter                          1. a todo list
- [ ] Realtime vehicle feed
```
```markdown
- [ ] Timetable adapter  2026-09-01 .. 2026-11-06     2. dates and durations
- [ ] Realtime vehicle feed  20w
```
```markdown
- [ ] Timetable adapter  ^timetable-adapter           3. split with ids and after:
- [ ] Realtime vehicle feed  20w  after: ^timetable-adapter
```
```markdown
- Commitment: 2027-06-30                              4. Commitment, milestones
- [x] 2026-11-06 Timetable adapter live  done: 2026-11-06  ^timetable-adapter
```
```markdown
- [ ] [Timetable adapter](../../../teams/data-feeds/data-feeds.md#timetable-adapter)  by: 2026-12-15
```
5. Software links the front's milestone with `by:`; the state reads `delivered`.

The format is in [../../docs/FORMAT.md](../../docs/FORMAT.md).
