# Format examples — designing a new car

A worked example of the format (see [../docs/FORMAT.md](../docs/FORMAT.md)): the plan for a fictional
electric car, 20 project files up to five levels deep. Open it in the viewer with
`?root=/examples/car/aurora.md`.

## Tree

```
examples/
├── README.md
└── car/
    ├── .pmd/profile.md                  profile: states, extra key, people registry
    ├── aurora.md                        Aurora — compact EV (programme, top level)
    ├── aurora.lock                      tool-written contracts with aurora.md's direct links (`pmd lock aurora.md`)
    ├── body/
    │   ├── body.md                      Body & chassis
    │   ├── closures.md                  Doors, tailgate, bonnet (detailed)
    │   └── paint-shop.md                Paint shop (coarse, quarters only)
    ├── homologation/
    │   ├── homologation.md              Homologation / type approval
    │   └── crash-tests.md               Crash test programme (sub-project of homologation)
    ├── interior/
    │   ├── interior.md                  Interior package
    │   └── seats.md                     Seats, supplier (near-plain todo list)
    ├── manufacturing/
    │   ├── plant-readiness.md           Plant readiness (top-level branch of Aurora)
    │   └── supplier-ppap.md             Supplier PPAP (under plant readiness)
    ├── powertrain/
    │   ├── powertrain.md                Powertrain (includes a supplier inverter by URL)
    │   └── battery-pack/
    │       ├── battery-pack.md          Battery pack, also included by Transit-E
    │       ├── cell-module.md           Cell module (detailed; level 4)
    │       ├── bms.md                   Battery management, owned by another team
    │       └── cell-module/
    │           └── supplier-qualification.md   Cell supplier qualification (level 5)
    ├── software/
    │   ├── software.md                  Vehicle software (coarse, one conflicting bound)
    │   ├── infotainment.md              Infotainment (vendor-led, mixed precision)
    │   ├── adas.md                      Driver assistance (exact dates, conflicting commitment)
    │   └── ota-backend.md               Over-the-air backend (quarters only)
    └── van/
        └── transit-e.md                 Transit-E van, a second parent of the battery pack
```

Project files: 20 under `car/`, deepest chain five levels: Aurora → Powertrain → Battery pack →
Cell module → Cell supplier qualification.

## What each file demonstrates

| File | Shows |
|------|-------|
| [aurora.md](car/aurora.md) | Top-level programme: header with a readable `ID`, milestones with checkboxes, sub-projects with checkboxes and **bounds** (`by:`), a gate with an `approver:` sub-bullet and **prose sub-bullets** for its decision criteria, a nested inline task, and dependencies on grandchildren's public milestones (one through a reference-style link). Plain-prose `## Risks`. |
| [powertrain.md](car/powertrain/powertrain.md) | Mid-level project with **free headings that carry date ranges**. A local and a **remote** sub-project (supplier URL, reference-style link), a task with **two dependencies** (repeated `after:` keys), and a remote dependency. |
| [battery-pack.md](car/powertrain/battery-pack/battery-pack.md) | **Two parents.** A **past, achieved** milestone (`[x]`), a **tombstone** for a renamed public milestone (`[-]` with `replaced-by:`), sub-projects with bounds, a task whose title contains a colon (`Fix issue: …`), and a `## Risks` section that records the open conflict. |
| [cell-module.md](car/powertrain/battery-pack/cell-module.md) | **Level 4** of the chain. Detailed plan: exact dates, owners on tasks, a gate with an approver and prose criteria, and a dependency on a **level-5** public milestone. |
| [cell-module/supplier-qualification.md](car/powertrain/battery-pack/cell-module/supplier-qualification.md) | **Level 5**, the most detailed file: exact dates for every gate, a `ticket:` from the profile, a **cancelled** task, a remote link in reference style, and a `qualification` state. |
| [bms.md](car/powertrain/battery-pack/bms.md) | A sub-project **owned by another team** (electronics). Only the handover points are tracked; a `#public` software release that the pack depends on. Quarter ranges mixed with exact milestones. |
| [closures.md](car/body/closures.md) | Detailed body sub-project: a `[x]` concept milestone, a gate with an approver, a **local id on a task** (`^kinematics`) that another task depends on, and a dependency on the parent's `#public` milestone. |
| [paint-shop.md](car/body/paint-shop.md) | **Coarse on purpose**: quarters only, plus a cancelled task and a public milestone in quarter form (`2028-Q1`). |
| [body.md](car/body/body.md) | A child depending on its parent's public milestone (`../aurora.md#design-freeze`), a gate with an approver, and two sub-projects bounded by dates and by a parent's `#public` milestone. |
| [crash-tests.md](car/homologation/crash-tests.md) | Sub-project of homologation, one level below Aurora. Depends on a `#public` milestone one level up (`../aurora.md#prototypes`). |
| [homologation.md](car/homologation/homologation.md) | Gates with an external approver (vertical `approver:`); a dependency on a **whole project** (`../powertrain/powertrain.md`, no fragment); a sub-project bound by a date. |
| [interior.md](car/interior/interior.md) | Progressive structure: header, a sub-project bound, a **cancelled** task and a `#public` sign-off milestone. |
| [seats.md](car/interior/seats.md) | **Near-plain todo list**: header and checkbox items only, no headings or gates. A supplier file that stays minimal. |
| [plant-readiness.md](car/manufacturing/plant-readiness.md) | Top-level branch of Aurora with its own header ID, a `[x]` milestone, a gate with a committee approver, and dependencies on three other branches' public milestones. |
| [supplier-ppap.md](car/manufacturing/supplier-ppap.md) | Sub-project under plant readiness; depends on a level-5 public milestone and on a remote supplier URL. Exact dates. |
| [software.md](car/software/software.md) | **Coarse dates** throughout (`2027-Q1`, `2027-H2`), a coarse commitment, a non-standard state (`discovery`), sub-projects, and **one conflicting bound** (the ADAS line). |
| [infotainment.md](car/software/infotainment.md) | Vendor-led sub-project with quarter milestones, a cancelled task, and a task with a `ticket:` key. |
| [adas.md](car/software/adas.md) | Exact dates for a safety-relevant function; a **remote** dependency (reference-style link); the **second intentional conflict** against its parent's bound. |
| [ota-backend.md](car/software/ota-backend.md) | **Quarters only** for every milestone and task; one owner handle and no exact dates. |
| [transit-e.md](car/van/transit-e.md) | Second parent of the battery pack. Its link carries the stable id in the link title (`"id:…"`). The **first intentional conflict** is recorded here. |
| [.pmd/profile.md](car/.pmd/profile.md) | The profile: state vocabulary (now with `sourcing` and `qualification`), the extra key `ticket:`, and the `@handle` registry. |
| [aurora.lock](car/aurora.lock) | Tool-written contracts with the projects `aurora.md` links directly (`pmd lock car/aurora.md`): need-by (`by`), commitment, start, public milestones, and `late` when a child forecasts past its commitment (here: Powertrain). The viewer draws collapsed sub-projects from it. Every other file that links projects has its own `NAME.lock` beside it, built bottom-up. |

## Intentional conflicts

Two child commitments exceed their parent's bound on purpose, so that tooling has something to
flag:

1. **Transit-E asks the battery pack for `by: 2027-10-31`** ([transit-e.md](car/van/transit-e.md)),
   but the pack commits to 2027-11-30 ([battery-pack.md](car/powertrain/battery-pack/battery-pack.md)).
2. **Vehicle software asks ADAS for `by: 2028-05-31`** ([software.md](car/software/software.md)),
   but ADAS commits to 2028-08-31 ([adas.md](car/software/adas.md)).

Every other child commitment is at or before its parent's bound.

## Conventions used

Meaning comes from the **shape of a list item**, not from the heading it sits under.
Headings are free-form and only group things for humans (`## Milestones`, `## Work`,
`## Phase 1 …`, `## Risks` are all equally valid).

| Notation | Meaning |
|----------|---------|
| `- [ ] 2027-06-30 Design freeze` | **Milestone**: a list item with a leading date (exact or coarse). `[x]` = achieved; only past dates may be `[x]`. |
| `- [ ] Task` / `- [x] Task` / `- [-] Task` | **Task**: a list item with a checkbox and no leading date. `[x]` = done; `[-]` = cancelled (does not block successors, never overdue). |
| `- [ ] [Title](path.md)  by: …` | **Sub-project**: a checkbox, then a link to a project file (no `#fragment`) or a remote project. Its status lives in its own `Commitment:` and `State:`. |
| `- [ ] Title  2w` | Duration estimate (calendar time). |
| Any other list item or paragraph | **Plain prose**, ignored by planning tools. Example: `## Risks` bullets. |
| `  - approver: …` under a milestone | **Vertical property**: a known key on a sub-bullet sets that item's key. A milestone with an `approver:` is a **gate**. |
| `  - Decision criteria: …` under a gate | **Prose sub-bullet**, ignored by tools. Gate decisions are recorded as `[x]` passed or `[-]` rejected; who and when come from version history. |
| `^design-freeze` | Item id. Only needed when something references the item. |
| `#public` | Exported milestone: other projects may depend on it. Others may not. |
| `replaced-by: ^new-id` | On a tombstone (`[-]`) for a removed or renamed public milestone. |
| `by: <date, ^id or target#id>` on a sub-project | The parent's **bound** for that sub-project. A milestone reference uses that milestone's date. |
| `Commitment:` in a header | The project's own promise to all its parents: a date (`2028-11-30`) or a range (`2027-01-04 .. 2028-11-30`). Absent means it accepts the parent's `by:`. |
| `after: <ref>` | Dependency: local (`^id`), remote or sibling (`path.md#id`), or a whole project (`path.md`, no fragment: after its Commitment). Repeat the key for several dependencies. |
| `ID: cars.example.com/…` | A readable, namespaced stable id, not a UUID. Carried in a link title (`"id:…"`) when a file links to the project by path. |
| `ticket: PACK-311` | Extra known key declared in `.pmd/profile.md`. |
| `2w`, `12w`, `3d` | Duration estimate. |
| `2027-Q2`, `2027-H2`, `2027-06` | Coarse date, alias for an explicit date range. |
| `a .. b` | Date range. |
| `@sanne` | Person handle, resolved through the people registry in `.pmd/profile.md`; otherwise use `Name <email>`. |
| `[text][ref]` with `[ref]: https://…` at the end | Reference-style link, used for remote targets and for item lines longer than about 110 characters. |
| `NAME.lock` | Tool-written sidecar beside `NAME.md`: the contracts with the projects it links directly. Never hand-edited. |
