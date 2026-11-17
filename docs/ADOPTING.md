# Adopting projectmd in a team or organisation

projectmd is plain markdown, so adopting it is mostly a matter of habits. This page collects the ones that work.

## Start small

Put one file, `plan.md`, in the repository you already have. A heading, an `Owner:` and a checkbox list is a
valid plan. Add durations, dates and `after:` only where they answer a real question. Run
`pmd check plan.md` now and then. Do not plan the structure of the whole organisation first.

## Grow into sub-projects

When an item outgrows a line, give it a file and replace its title with a link; nothing else on the
line changes, and everything that depended on its `^id` keeps working. Let the people who do the work
own the new file. A parent states what it needs (`by:`), the child states what it promises.

## Where files live

- **One repo**: simplest. Plans live next to the code; pull requests are the review and history.
- **Many repos**: each team keeps `plan.md` in its own repo; parents link by relative path or URL.
- **Shared drive or wiki export**: fine for readers; you lose history, so say who changed what in the file.
- **URLs**: a link may point at a raw markdown URL (`https://.../plan.md#id`). Tools only read what a link names.

A plan that must stay private gets a thin **front file**: it carries only the `Owner:`, the `Commitment:` and the
milestones parents may see, and links to the private plan. Parents link to the front file.

## Ownership

Every file has one owner (the `Owner:` header). You write only what you own: never edit another file's
Commitment or milestones. To ask something of another team, write `by:` on your own link line. Put the
owners in `AGENTS.md` ([template](templates/AGENTS.md)) or CODEOWNERS so reviews route correctly.

## Working agreements: the promise protocol

Each state says whose move is next, and `pmd ready` shows each person their own.

1. **Ask** with `by: DATE` on your link line.
2. **Answer** with a date: the child states a Commitment or dates the milestone. Silence is "requested".
3. **A dispute is the parent's move.** If the promise is later than `by:`, the parent accepts (moves `by:`),
   re-plans, or talks to the child. A child does not edit the parent's file.
4. **Move your promise rather than miss it.** Change the date before it passes; `overdue` is the child's move.
5. **Say why.** A moved date gets a sentence in the item's description (indented text under it) or in the commit message.
6. **Tick honestly.** `[x]` with `done: DATE` when you know the day; never invent one.

## CI

Add the projectmd action to a pull request workflow. It checks the canonical form and the plan links, and
fails the job on findings:

```yaml
- uses: actions/checkout@v4
  with:
    fetch-depth: 0   # needed for the "Plan changes" summary
- uses: pschella/projectmd@main
  with:
    files: plans/plan.md
```

`version` will accept a release tag (for example `v0.2.0`) once the first release is published; until then
use `@main` and the action builds `pmd` from source (cached).

| Input | Default | Meaning |
|---|---|---|
| `files` | required | Space-separated root plan files or globs, relative to the repository root |
| `deep` | `true` | Check the whole link graph (`pmd check --deep`) |
| `strict` | `true` | Warnings fail too (`pmd check --strict`) |
| `today` | (empty) | Pretend today is this date, `YYYY-MM-DD` (sets `PMD_TODAY`) |
| `version` | `source` | `source` builds from this action's checkout; any other value is a release tag to download |
| `changes` | `true` | On pull requests, add `pmd changes` output for changed plan files to the job summary |

A failing check means the plan has a broken link, a stale or overdue promise, a dispute, or a file that is
not in canonical form: the conversation it names is owed, so the failure is intentional. Use `strict: "false"`
if you prefer warnings to be advisory. The job summary always shows the files checked and the findings count;
under "Plan changes" it lists the promises, asks and items done or cancelled since the base of the pull request.

## Notifications

`pmd changes OLD NEW` lists what moved between two versions of a file (promises and asks, with day
deltas). On a pull request, compare the base with the head:

```
pmd changes <(git show origin/main:plan.md) plan.md
```

Post the output as a PR comment. `pmd changes` is part of the 0.9 tool set and may not be in your build yet.

## Agents

Point agents at the [cheat sheet](CHEATSHEET.md) and drop the [AGENTS.md template](templates/AGENTS.md)
in the folder of plans. The loop is: `pmd ready --owner me`, do the item, tick it with `done:` (or move a date),
then `pmd fmt` and `pmd check`. Agents edit only files they own, so give an agent its own file to own.

## Humans and agents in one plan

Both are owners. A human delegates to an agent by linking to the agent's file with `by:`; an agent
waits on a human with a decision item in its own file (`- [ ] Decide X  owner: Mara`). An undated decision is due today,
so whatever waits on it slips a day per day: that pressure is deliberate. `pmd ready --owner Mara` is her list of
decisions to make.

## Coming from kanban or agile

Keep your board and its habits. A projectmd file can show the same four columns (Waiting, Ready,
Doing, Done), computed from the file: pulling an item is adding `after: <today>`, and finishing one is
ticking it with `done:`. Your sprints become parent items with a date range, and your review steps
become items with an owner. Tickets, story points and workflow states are not in the format. The
mapping, the commands and a daily routine are in [BOARDS.md](BOARDS.md).

## What projectmd does not do

It has no resources or cost, no working calendars (a week is seven days), no ticket workflow, no comments or
assignments inbox. Pair it with what you have: keep tickets in your tracker, and mirror only the milestones
and promises into plans (a ticket id in a description is enough). Let the tracker own the day-to-day, and the
plan own "when" and "what depends on what".

## FAQ

**Do we need the tools?** No. The files are readable and editable by hand. Tools add forecasts and checks.

**Can a parent see a child's whole plan?** No, only its Commitment and non-`#private` milestones.

**How do we handle holidays?** Estimates are calendar time. Put the real constraint in as a date.

**What if a child ignores `by:`?** It stays "requested" in `pmd ready` and `pmd tree` until answered.

**Can we migrate from a tracker?** Export milestones and epics as list items; do not import every ticket.
