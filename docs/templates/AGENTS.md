# Agents: working in this folder of plans

This folder holds projectmd plans (markdown). Notation: `docs/CHEATSHEET.md` in the projectmd repo
(https://github.com/pschella/projectmd). Read it before editing.

## Who owns what

| File | Owner |
|---|---|
| `plan.md` | Name <name@example.org> |
| `team-a/plan.md` | Name <name@example.org> |

## Rules

- Edit only files you own (the `Owner:` in the header). Never edit another file's `Commitment` or milestones.
- Refer to items by `^id`; give every milestone others depend on an explicit `^id`. Keep titles stable.
- Ask for a date with `by:` on your own link line. Answer a request by dating your own milestone.
- Never invent a `done:` date. Tick `[x]` with `done: YYYY-MM-DD` only for a date you know.
- When you start work, add `after: <today>` in the same edit that claims it.
- Move a promise rather than miss it. Say why in the item's description or the commit message.
- Write canonical form and finish clean: `pmd fmt FILE` then `pmd check --deep FILE` (exit 0).

## Loop

1. `pmd ready FILE --deep --owner "Your Name"` (your queue: do, decide, fix, answer, deliver).
2. Do the item; tick it with `done:` or move its date.
3. `pmd fmt FILE && pmd check --deep FILE`.
4. When a date moved and you do not know why: `pmd why FILE ^id --deep`.

Set `PMD_TODAY=YYYY-MM-DD` to fix "today" when testing.
