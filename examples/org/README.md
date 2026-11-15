# Worked example: a company, three stores

A passenger information system (station displays and apps) sold to the fictional Vellmar
Transit Authority, planned across sales, business units and teams. It shows projectmd at
scale: separate stores with separate access, each level seeing only the interface of the level
below. Today is 2026-11-15; every lock, `*.public.md` and date was written by hand exactly as
the spec says a tool would (see the last section).

**The rule.** Each folder under `org/` stands for a separate store (sales: shared drive; units:
git; teams: object store). A level links only the `NAME.public.md` of the level below, never its
full plan, except where noted below. Each public file has `Plan:` back to its full plan, for
readers who may open it. Full plans may carry an informational `Parent:` for "go up".

```
org/
├── sales/vellmar/       vellmar.md + .lock                       store 1: sales
├── units/               store 2: business units
│   ├── hardware/        hardware.md + .lock + .public.md
│   │   └── displays/    displays.md + .lock + .public.md         sub-unit, same store
│   └── software/        software.md + .lock + .public.md
└── teams/               store 3: teams (no links, so no locks)
    ├── display-panels/  enclosures/  backend/  mobile-app/  data-feeds/   *.md + *.public.md
    └── accessibility/   accessibility.md   placeholder, no public file yet
```

`hardware.md` could read `displays.md` directly (same store), but links `displays.public.md`
anyway: when displays moves to another store, nothing changes.

Milestones (dated items) are in the interface by default; internal checkpoints such as
Enclosures' `Series moulds delivered by tooling supplier` carry `#private`.

## Contracts (all locks)

| Lock | Link | `by:` | Promise | State |
|---|---|---|---|---|
| vellmar | Hardware | 2028-03-31 | 2028-03-31, late 2028-04-26 | at-risk |
| vellmar | Hardware `#pilot-displays` | 2027-07-31 | 2027-07-31 | agreed |
| vellmar | Software | 2028-04-30 | 2028-04-30 | agreed |
| vellmar | Software `#pilot-ready` | 2027-08-31 | 2027-08-31 | agreed |
| hardware | Displays | 2028-02-29 | 2028-02-29, late 2028-04-05 | at-risk |
| displays | Display panels | 2027-12-31 | 2027-11-30 | agreed |
| displays | Enclosures | 2027-12-31 | 2027-12-15, late 2028-03-15 | at-risk |
| software | Backend | 2028-01-31 | 2028-01-31 | agreed |
| software | Mobile app | 2027-12-31 | 2028-02-29 | disputed |
| software | Data feeds `#timetable-adapter` | 2026-12-15 | `[x]` done 2026-11-06 | delivered |
| software | Accessibility | 2027-06-30 | none yet | requested |

The slip travels up as lateness: Enclosures (late 2028-03-15) pushes Displays (series assembly
3w, 2028-03-16 .. 2028-04-05), which pushes Hardware (integration 3w, 2028-04-06 ..
2028-04-26). Sales absorbs it: rollout installation (8w) starts 2028-05-01 after Software's
Commitment and ends 2028-06-25, inside 2028-06-30. So Sales has no conflict; it sees Hardware
`at-risk`. Freshness: each As-of is at most the oldest unfinished link's (Sales sees 2026-11-10
via Mobile app); the finished Data feeds and the not yet published Accessibility do not count.

**Disputed is a warning, not an error.** Software asks Mobile app for 2027-12-31, the team
promises 2028-02-29: `W-DISPUTED`. Negotiation is normal, so `pmd check` passes and
`pmd check --strict` (CI) fails. Software's own plan is consistent: it schedules against the
promise (system test 2028-03-01 .. 2028-04-25, inside its 2028-04-30).

**Accessibility** was created top-down by Software and has no public file. Its lock entry stays
minimal (`by`, `status=requested`); a tool would derive an interface from a full plan, and
there is nothing to derive, no `hash`.

## The twelve steps, as they work now

1. **Sales records the customer terms.** `vellmar.md` has the Commitment 2028-06-30 and public
   milestones (pilot live 2027-09-30, rollout complete 2028-06-30). `pmd export vellmar.md`.
2. **Work is split top-down** before the child has a plan: `by:` on the link, and a missing
   promise is `requested`, an unreadable target `unknown`, not an error. Accessibility shows it;
   its `Parent:` tells the team who asks.
3. **A unit accepts.** It writes its own Commitment. `pmd lock software.md` records
   `status=agreed` (Backend). A later Commitment (Mobile app) is `disputed`, a warning.
4. **The parent asks for specific milestones.** `- [ ] [Pilot displays](...#pilot-displays)  by:
   2027-07-31` is a deliverable contract; `pmd lock vellmar.md` writes its own lock entry and
   milestone line (agreed).
5. **Units split further, teams choose their tools.** Hardware links Displays, Displays links two
   teams, each level reading only the interface below. Teams may emit `*.public.md` from
   any tool, or `pmd export` it.
6. **A team slips beyond its Commitment** (Enclosures: tooling supplier, moulds 2028-01-19).
   `pmd export enclosures.md` writes `Late: 2028-03-15`. Then, one refresh per level:
   `pmd lock displays.md`: enclosures line `hash=...->3bfa72`, `+late=2028-03-15`,
   `status=agreed->at-risk`. Displays forecasts 2028-04-05, `pmd export` writes `Late:`
   (`hash=...->97dbd1`). `pmd lock hardware.md`: displays line `+late=2028-04-05`,
   `status=agreed->at-risk`; hardware exports `Late: 2028-04-26`. `pmd lock vellmar.md`:
   hardware line `+late=2028-04-26`, `status=agreed->at-risk`; Sales forecast 2028-06-25, no export.
   A team slip inside its Commitment (Backend loses a week: forecast 2028-01-24 becomes
   2028-01-31) changes only `backend.md`; `pmd export` gives the same public file except
   `As-of`, so the hash is unchanged and **no file above the team changes**.
7. **The owner formally re-plans.** Enclosures moves its Commitment to 2028-03-15 and the
   `series-ready` date. The hash changes; the next `pmd lock displays.md` shows
   `commitment=2027-12-15->2028-03-15`, `-late=2028-03-15`, `status=at-risk->disputed`
   (a breaking change). Displays then moves its `by:` or pushes back; the commit says why.
8. **"Are the layers below me on time?"** Read `NAME.lock`: one state per contract, no fetches.
   `pmd check --strict software.md` reports Mobile app (W-DISPUTED, fails strict) and Accessibility (I-REQUESTED, info only).
9. **Expand lazily.** A viewer draws Sales from `vellmar.md` + `vellmar.lock`; opening Hardware
   loads `hardware.public.md` + `hardware.lock`; whoever may read more follows `Plan:`.
10. **Bottom-up.** Data feeds started as a todo list; see below. Any file can be linked later.
11. **Different stores.** Links are plain paths or URLs; the lock sits beside the plan, and
    the interface is the only thing that has to cross: no history, no store features needed.
12. **Agents edit.** They read `docs/CHEATSHEET.md`, edit only their own files, write canonical
    form, finish with `pmd check` (`--strict` in CI) and never touch locks.

## Data feeds, bottom-up in five snippets

```markdown
- [ ] Timetable adapter                          1. a todo list (level 0)
- [ ] Realtime vehicle feed
```
```markdown
- [ ] Timetable adapter  2026-09-01 .. 2026-11-06     2. dates and durations (1-2)
- [ ] Realtime vehicle feed  20w
```
```markdown
- [ ] Timetable adapter  ^timetable-adapter           3. split with ids and after: (2)
- [ ] Realtime vehicle feed  20w  after: ^timetable-adapter
```
```markdown
- Commitment: 2027-06-30                              4. Commitment, milestones public by default (3-4)
- [x] 2026-11-06 Timetable adapter live  done: 2026-11-06  ^timetable-adapter
```
```markdown
- [ ] [Timetable adapter](../../teams/data-feeds/data-feeds.public.md#timetable-adapter)  by: 2026-12-15
```
5. Software links it with `by:` (in `software.md`); the lock says `delivered`.

## Not implemented yet

`parser/` and `pmd` do not implement this spec yet, so `pmd lock`, `pmd export` and the check
codes are as specified, not as run. This folder is meant as the acceptance test and golden data
for the tools: `hash=` values are real (`shasum -a 256`, 6 chars, `As-of` line removed).
The format is in [../../docs/FORMAT.md](../../docs/FORMAT.md).
