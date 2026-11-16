# Lunar programme: a trimmed example

A fictionalised 1961-1969 crewed-lunar-landing programme, planned in projectmd by several organisations.
All people and organisations are invented (addresses are `example.org` / `example.net`); the files hold the
final record, so every promise is ticked and dated. Use `today=` in the viewer to see earlier states.

## Stores

Each top folder is a separate store with its own owner and access rules. A store exposes a **front** file
at its top (header, dated promises) and keeps its real breakdown in `private/`; parents link only the
front, and the front's deliverable lines point into the private plan. The owner's own check reads both.

```
examples/lunar/
├── hq/lunar-programme.md                  programme office: the top project
├── spacecraft-centre/
│   ├── spacecraft.md                      front
│   ├── private/spacecraft-plan.md         plan, links both contractors
│   └── rendezvous/rendezvous.md           Earth-orbit practice programme (own contractor)
├── csm-co/{csm.md, private/csm-plan.md}  command and service module contractor
├── lm-co/
│   ├── lm.md                              front
│   └── private/{lm-plan.md, weight-watch.md, teams/descent-stage.md}
├── vehicle-centre/
│   ├── launch-vehicle.md, crewed-launch-vehicle.md      fronts
│   └── private/{launch-vehicle-plan.md, crewed-launch-vehicle-plan.md}
├── launch-centre/{launch-ops.md, private/launch-ops-plan.md}
└── research-centre/lor-study.md
```

## What each part shows

- **Bottom-up study pulled in**: `research-centre/lor-study.md` is a small closed study of lunar-orbit
  rendezvous; the programme links its report as a deliverable (`^lor-report`) and then adopts LOR.
- **Weight watch**: `lm-co/private/weight-watch.md` is one engineer's bottom-up list of mass savings, one
  change per line, promised upward as a mass forecast with a firm need-by.
- **Team split**: the LM integrator plan hands descent-stage work to `teams/descent-stage.md`, a team with
  its own owner and Commitment; the integrator links its dated deliverables. Ascent and guidance work stays
  as plain tasks in the integrator plan (their team files are not part of this trimmed copy).
- **Project succession**: `launch-vehicle.md` closes 1967-09-05 and `crewed-launch-vehicle.md` continues it
  after the second uncrewed flight failed; HQ re-points its need at the new project.
- **The fire re-plan**: the 1967 pad fire stops crewed work. `csm-plan.md` adds a redesign block, the
  spacecraft plan moves crew-ready and lander-ready, and HQ's `^decide` gate picks option B and makes the
  launch complex the binding input.
- **Recurring per-flight needs**: flights 1 to 3 each need their own vehicle and countdown, so HQ has one
  deliverable line per need (`^crew-vehicle`, `-2`, `-3`; `^lo`, `-2`, `-3`) with a `by:` each.
- **Sub-project with its own contractor**: the rendezvous programme runs ahead of the crew-rated CSM.
- **Coarse private structure**: `spacecraft-plan.md` keeps cancelled direct-ascent items as `[-]` record.

## Explore

From the repository root, after `cd parser && cargo build --release`:

```bash
export PMD_TODAY=1969-12-31
parser/target/release/pmd check --deep examples/lunar/hq/lunar-programme.md
parser/target/release/pmd tree examples/lunar/hq/lunar-programme.md --depth 2
parser/target/release/pmd schedule examples/lunar/hq/lunar-programme.md
parser/target/release/pmd fmt --check examples/lunar/*/*.md
```

The deep check reports no errors. In the viewer (`python3 -m http.server 8642` at the repo root):

http://localhost:8642/viewer/?root=/examples/lunar/hq/lunar-programme.md&today=1967-11-15

`today=1967-11-15` shows the programme mid-crisis: the fire is behind it, option B is decided and the
uncrewed flight is just done, while the crewed flights are still ahead. Try `&today=1964-06-30` for the
early design phase or omit `today` for the real date.
