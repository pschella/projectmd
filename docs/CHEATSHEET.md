# projectmd cheat sheet

A projectmd file is markdown. Planning meaning comes only from the shapes below; everything
else is free text.

```markdown
# Battery pack — 60 kWh platform pack

- Owner: Priya Nair <priya.nair@example.com>
- Commitment: 2026-09-01 .. 2027-11-30
- State: in-development
- ID: cars.example.com/platform/battery-pack

## Work
- [x] 2026-10-31 Cell chemistry selected  ^chemistry
- [ ] Module & pack design  12w  after: ^chemistry  owner: @priya
- [ ] 2027-06-30 Design freeze  ^design-freeze
- [ ] Thermal validation  8w  after: ^design-freeze
  - after: [Inverter A-samples][inv-a]
- [ ] [Cooling loop](cooling/cooling.md)  by: 2027-03-31
- [ ] [Pack B-samples](pack.md#b-samples)  by: 2027-09-30
- Supplier prefers Q2 delivery.        <- prose: no checkbox, no date, ignored

[inv-a]: https://supplier-portal.example.net/projects/inv-2207.md#a-samples
```

| Token | Meaning |
|---|---|
| `- [ ]` `[x]` `[-]` | open · done · will not happen (cancelled, rejected, withdrawn) |
| `done: 2027-05-28` | completion date; write it when ticking `[x]` |
| leading `2027-06-30` | milestone; its date is the declared end |
| `2027-Q3` `2027-H2` `2027-06` `2027-W14` | coarse date; deadlines use the end of the span |
| `a .. b` | range: declared start a, declared end b |
| `12w` `3d` `2m` | estimate in real calendar time (1w = 7 days, 1m = 1 calendar month); no story points |
| `^id` | defines an id for local references |
| `file.md#id` | reference to an interface milestone in another file |
| `#private` / `#public` | keep a milestone out of / put a task or link into the interface (milestones are in by default) |
| `owner: @h` | who does it |
| `after: x` | starts after x; repeatable; `after: file.md` = that project's end |
| `by: date` / `by: ^id` | the declared end asked of this item (same on tasks and sub-projects) |
| `approver: @h` | gate sign-off; repeatable |
| `replaced-by: ^id` | tombstone for a removed interface milestone |
| `[Title](path.md)` | sub-project; add `"id:ns/name"` after the path for a stable id |
| `[Title](path.md#id)` | deliverable: an interface milestone of another project; `by:` is the contract |
| header `Done` `Late` `As-of` `Plan` `Parent` | delivered date · will end late · freshness · fuller plan · who asks (informational) |
| `` `code` `` | never parsed as a token |

Rules:
1. Read the line from the end: keys and tokens form the suffix; the rest is the title.
   Only known keys count (`owner`, `after`, `by`, `approver`, `replaced-by`, profile keys).
2. Sub-bullets `key: value` under an item set its keys; the H1 sub-bullets are the header.
3. Links are real paths or URLs. Never write a forecast; write only declared dates.
4. Cross-file references must target interface items (milestones unless `#private`; others if `#public`).
5. Do not edit `*.lock` files; tools write them. Publish your promise with `pmd export` (`NAME.public.md`).

Rules for agents (FORMAT §14):
6. Read `AGENTS.md` if the folder has one. Refer to items by `^id`; run `pmd fmt --ids` if needed.
7. Edit only files you own. Ask for a date with `by:` on your own link; never edit a Commitment elsewhere.
8. Write canonical form (`pmd fmt`), use `--if-hash` to avoid overwriting others, and finish with
   `pmd check` clean (`--strict` in CI). Say why in the commit message when a Commitment, `by:` or interface milestone moves.

