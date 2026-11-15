# projectmd cheat sheet

A projectmd file is markdown. Planning meaning comes only from the shapes below; everything
else is free text.

```markdown
# Battery pack — 60 kWh platform pack

- Owner: Priya Nair <priya.nair@example.com>
- Commitment: 2026-09-01 .. 2027-11-30

## Work
- [x] 2026-10-31 Cell chemistry selected  ^chemistry
- [ ] Module & pack design  12w  after: ^chemistry  owner: Priya
- [ ] 2027-06-30 Design freeze  ^design-freeze
- [ ] [Inverter A-samples][inv-a]  ^inv-a
- [ ] Thermal validation  8w  after: ^design-freeze  after: ^inv-a
- [ ] [Cooling loop](cooling/cooling.md)  by: 2027-03-31
- [ ] [Pack B-samples](pack.md#b-samples)  by: 2027-09-30  ^b-samples
- Supplier prefers Q2 delivery.        <- prose: no checkbox, no date, ignored

[inv-a]: https://supplier-portal.example.net/projects/inv-2207.md#a-samples
```

| Token | Meaning |
|---|---|
| `- [ ]` `[x]` `[-]` | open · done · will not happen (cancelled, rejected, withdrawn); a gate is a milestone its owner decides |
| `done: 2027-05-28` | completion date; write it when ticking `[x]` |
| leading `2027-06-30` | milestone; its date is the declared end |
| `#private` | keeps a milestone out of its parents' view (milestones are exposed by default) |
| `2027-Q3` `2027-06` | coarse date; deadlines use the end of the span |
| `a .. b` | range: declared start a, declared end b |
| `12w` `3d` `2m` | estimate in real calendar time (1w = 7 days, 1m = 1 calendar month) |
| `^id` | defines an id for local references |
| `owner: Name` | who does it (`Name <email>` or any text) |
| `after: ^id` | starts the day after that local item ends; repeatable; a date also works |
| `by: date` / `by: ^id` | the declared end; on a link line, the need-by asked of the child |
| `[Title](path.md)` | sub-project (a link line); the child's Commitment is its promise |
| `[Title](path.md#id)` | deliverable: a milestone of another project; `by:` is the contract |
| `` `code` `` | never parsed as a token |

Rules:
1. Read the line from the end: keys and tokens form the suffix; the rest is the title. Known keys: `owner`, `after`, `by`, `done`.
2. Sub-bullets `key: value` under an item set its keys. The H1 sub-bullets are the header
   (`Owner`, `Commitment`, `Done`).
3. `after:` and `by:` take local `^id`s or dates only. To depend on another file, add a link
   line with an `^id` and point at it.
4. To grow an item into a sub-project, replace its title with a link; nothing else changes.
5. Write only declared dates, never a forecast. To hide a plan from parents, keep a thin front
   file (header, milestones, deliverable lines into the plan).

Rules for agents (FORMAT §13):
6. Read `AGENTS.md` if the folder has one. Refer to items by `^id`.
7. Edit only files you own. Ask for a date with `by:` on your own link line; never edit a Commitment elsewhere.
8. Write canonical form (`pmd fmt`) and finish with `pmd check` clean (`--strict` in CI).
   Say why in the commit message when a Commitment, `by:` or milestone moves.
