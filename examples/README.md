# Format examples — designing a new car

A worked example of the format (see [../docs/FORMAT.md](../docs/FORMAT.md)): the plan for a fictional
electric car, 10 project files up to five levels deep. Open it in the viewer with
`?root=/examples/car/aurora.md`.

## Tree

```
examples/
├── README.md
└── car/
    ├── AGENTS.md                        Rules for agents editing this plan
    ├── aurora.md                        Aurora — compact EV (programme, top level)
    ├── body/
    │   └── body.md                      Body & chassis
    ├── homologation/
    │   └── homologation.md              Homologation / type approval
    ├── powertrain/
    │   ├── powertrain.md                Powertrain (includes a supplier inverter by URL)
    │   └── battery-pack/
    │       ├── battery-pack.md          Battery pack, also included by Transit-E
    │       ├── cell-module.md           Cell module (detailed; level 4)
    │       └── cell-module/
    │           └── supplier-qualification.md   Cell supplier qualification (level 5)
    ├── software/
    │   ├── software.md                  Vehicle software (coarse, one conflicting bound)
    │   └── adas.md                      Driver assistance (exact dates, conflicting commitment)
    └── van/
        └── transit-e.md                 Transit-E van, a second parent of the battery pack
```

Project files: 10 under `car/` (plus `AGENTS.md`), deepest chain five levels: Aurora → Powertrain →
Battery pack → Cell module → Cell supplier qualification.

## What each file demonstrates

| File | Shows |
|------|-------|
| [aurora.md](car/aurora.md) | Top-level programme: header, milestones with checkboxes, sub-projects with checkboxes and **bounds** (`by:`), a gate (a milestone its owner decides) with **prose sub-bullets** for its decision criteria, a nested inline task, and one **deliverable line** (reference-style link) that a local `after: ^id` points at. Plain-prose `## Risks`. |
| [powertrain.md](car/powertrain/powertrain.md) | Mid-level project whose dated phases are **items with children** (headings carry no dates). A local and a **remote** sub-project (supplier URL, reference-style link), a task with **two dependencies** (repeated `after:` keys), and a remote dependency. |
| [battery-pack.md](car/powertrain/battery-pack/battery-pack.md) | **Two parents.** A **past, achieved** milestone (`[x]`), a **withdrawn** milestone (`[-]`, the replacement named in prose), a sub-project with a bound, a task whose title contains a colon (`Fix issue: …`), and a `## Risks` section that records the open conflict. |
| [cell-module.md](car/powertrain/battery-pack/cell-module.md) | **Level 4** of the chain. Detailed plan: exact dates, owners on tasks, a gate with prose criteria, and a dependency on a **level-5** milestone. |
| [cell-module/supplier-qualification.md](car/powertrain/battery-pack/cell-module/supplier-qualification.md) | **Level 5**, the most detailed file: exact dates for every gate, a **cancelled** task and a remote link in reference style. |
| [body.md](car/body/body.md) | A child depending on its parent's milestone through a deliverable line (`../aurora.md#design-freeze`), and a local tooling milestone that the parent's `by:` can point at. |
| [homologation.md](car/homologation/homologation.md) | Gates decided by an external authority; **two dependencies on other projects' milestones** through deliverable lines (`../powertrain/powertrain.md#validated`, `../software/software.md#feature-complete`). |
| [software.md](car/software/software.md) | **Coarse dates** throughout (`2027-Q2`, `2028-Q2`), a coarse commitment, a sub-project, and **one conflicting bound** (the ADAS line). |
| [adas.md](car/software/adas.md) | Exact dates for a safety-relevant function; a **remote** dependency (reference-style link); the **second intentional conflict** against its parent's bound. |
| [transit-e.md](car/van/transit-e.md) | Second parent of the battery pack. The **first intentional disagreement** is recorded here. |
| [AGENTS.md](car/AGENTS.md) | Rules for agents working in the folder: read the cheat sheet first, change only the files you were asked to change, run `pmd fmt` and `pmd check` after editing. |

## Promotion: a task becomes a sub-project

An item grows into a sub-project by replacing its title with a link to a new file; nothing else
on the line changes (checkbox, `^id`, `owner:`, `by:`, estimate, `after:`). Other items keep
depending on `^seats` unchanged. Before:

```markdown
- [ ] Seat concept  6w  after: ^styling  owner: @mia  by: 2027-09-30  ^seats
- [ ] Seat validation  2w  after: ^seats
```

After (`seats.md` exists but has no Commitment yet):

```markdown
- [ ] [Seat concept](seats.md)  6w  after: ^styling  owner: @mia  by: 2027-09-30  ^seats
- [ ] Seat validation  2w  after: ^seats
```

Until `seats.md` has a Commitment, a line with `by:` is `requested` (a line without `by:` has no contract) and is scheduled
from its own estimate (6w after `^styling`), so `Seat validation` is placed exactly as before. Once
`seats.md` promises a date, that date is the line's end. Covered by
`parser/pmd-core/tests/promotion.rs`.

## Intentional disagreements

Two child promises are later than their parent's `by:` on purpose, so that tooling has something
to flag. Negotiation is not an error: both are warnings (`pmd check --strict` fails on them).

1. **Transit-E asks the battery pack for `by: 2027-10-31`** ([transit-e.md](car/van/transit-e.md)),
   but the pack commits to 2027-11-30 ([battery-pack.md](car/powertrain/battery-pack/battery-pack.md)):
   `pmd check van/transit-e.md` reports `W-DISPUTED` (transit-e.md:17).
2. **Vehicle software asks ADAS for `by: 2028-05-31`** ([software.md](car/software/software.md)),
   but ADAS commits to 2028-08-31 ([adas.md](car/software/adas.md)): `pmd check --deep aurora.md`
   reports `W-DISPUTED` (software.md:19).

`pmd check --deep aurora.md` also reports `W-UNKNOWN` for the supplier's inverter plan
(powertrain.md:18): remote links are not fetched by the command line tool, so their entries are
shown as unknown (`W-UNKNOWN` only on a line with `by:`). Every other child commitment is at or before its parent's `by:`.

## Conventions used

Meaning comes from the **shape of a list item**, not from the heading it sits under.
Headings are free-form and only group things for humans (`## Milestones`, `## Work`,
`## Phase 1 …`, `## Risks` are all equally valid).

| Notation | Meaning |
|----------|---------|
| `- [ ] 2027-06-30 Design freeze` | **Milestone**: a list item with a leading date (exact or coarse). `[x]` = achieved; only past dates may be `[x]`. |
| `- [ ] Task` / `- [x] Task` / `- [-] Task` | **Task**: a list item with a checkbox and no leading date. `[x]` = done; `[-]` = cancelled (does not block successors, never overdue). |
| `- [ ] [Title](path.md)  by: …` | **Sub-project**: a checkbox, then a link to a project file (no `#fragment`) or a remote project. Its promise is its own `Commitment:`. |
| `- [ ] [Title](path.md#id)  ^dep` | **Deliverable**: a link to another project's milestone. Local items depend on it with `after: ^dep`; `after:` and `by:` never name another file. |
| `- [ ] Title  2w` | Duration estimate (calendar time). |
| Any other list item or paragraph | **Plain prose**, ignored by planning tools. Example: `## Risks` bullets. |
| `  - Decision criteria: …` under a gate | **Prose sub-bullet**, ignored by tools. A gate is a milestone its owner decides; the outcome is `[x]` passed or `[-]` rejected, who and when come from version history. |
| `^design-freeze` | Item id. Only needed when something references the item. |
| `#private` | Every milestone is in the interface unless `#private`; nothing else is. |
| `by: <date or ^id>` on a link line | The parent's **need-by** for that sub-project or deliverable. A `^id` uses that local milestone's date. |
| `Commitment:` in a header | The project's own promise to all its parents: a date (`2028-11-30`) or a range (`2027-01-04 .. 2028-11-30`). Absent means it accepts the parent's `by:`. |
| `after: <^id or date>` | Dependency on a local item (or a date). Repeat the key for several dependencies. A task `after:` X starts the day after X ends; a milestone may fall on that day. |
| `2w`, `12w`, `3d` | Duration estimate. |
| `2027-Q2`, `2027-12`, `2027-06` | Coarse date, alias for an explicit date range. |
| `a .. b` | Date range. |
| `@sanne` | Plain text: a person is `Name <email>` or any text. |
| `[text][ref]` with `[ref]: https://…` at the end | Reference-style link, used for remote targets and for item lines longer than about 110 characters. |

## Distributed example

[org/](org/README.md): a company in three stores (sales, units, teams). Each team and unit exposes a **front file** (its promise) to the level above and keeps its breakdown in a private plan; contract states are computed live, nothing is generated. Shows a team that moves its Commitment, disputed and requested contracts, a delivered deliverable and a bottom-up team.
