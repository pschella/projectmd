# Format examples

Worked examples of the format (see [../docs/FORMAT.md](../docs/FORMAT.md)).

- [lunar/](lunar/README.md): a trimmed, fictionalised 1961-1969 crewed-lunar-landing programme in seven stores (programme office,
  spacecraft centre, two contractors, launch-vehicle centre, launch centre, research centre). Each store has a front file for its
  parent and a `private/` plan. Open it in the viewer with `?root=/examples/lunar/hq/lunar-programme.md&today=1967-11-15`.
- [story/](story/README.md): the plan behind [the Story page](../docs/STORY.md) and [Behind the story](../docs/STORY-DETAILS.md): a product launch across four teams, files in different places, people and agents.
  Open it with `?root=/examples/story/launch.md&today=2027-07-07&cp=1`.
- [org/](org/README.md): a company in three stores (sales, units, teams), described at the end of this file.

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
