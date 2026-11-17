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

## Resource planning and other tools

The format does one thing: it holds the plan. That is the current state, the dependencies, the
forecast, and who owns each step. Capacity, calendars, skills, cost and allocation belong to other
tools. Those tools read the plan; the plan never stores their data.

A resource tool starts from one command:

```
pmd schedule plan.md --deep --rows
```

It prints one JSON line per item, with the keys `file`, `line`, `id`, `title`, `kind`, `state`,
`owner`, `owner_inherited`, `start`, `end`, `declared_end`, `slack_days`, `estimate_days`, `tags`,
`parent_line` and `leaf`. The schema is in [schema/rows.schema.json](schema/rows.schema.json).
`owner` is inherited from the enclosing item, or from the file's `Owner:` when the item has none;
`owner_inherited` says which. On a team file, an inherited owner means "not yet assigned".

Take this three-item plan (the file's `Owner:` line is left out):

```markdown
- [ ] Draft invoice template  3d  owner: Bo  ^template
- [ ] Migrate ledger  5d  after: ^template  owner: Bo  #skill-sql  ^ledger
- [ ] Check ledger totals  after: ^ledger  owner: Mara Lind  ^check
```

Its three rows, wrapped here for reading (the scratch file had a title and an `Owner:` line above, so the lines start at 5):

```json
{"file":"plan.md","line":5,"id":"template","title":"Draft invoice template","kind":"task",
 "state":"open","owner":"Bo","owner_inherited":false,"start":"2027-03-10","end":"2027-03-12",
 "declared_end":null,"slack_days":null,"estimate_days":3,"tags":[],"parent_line":null,"leaf":true}
{"file":"plan.md","line":6,"id":"ledger","title":"Migrate ledger","kind":"task",
 "state":"open","owner":"Bo","owner_inherited":false,"start":"2027-03-15","end":"2027-03-19",
 "declared_end":null,"slack_days":null,"estimate_days":5,"tags":["skill-sql"],"parent_line":null,"leaf":true}
{"file":"plan.md","line":7,"id":"check","title":"Check ledger totals","kind":"task",
 "state":"open","owner":"Mara Lind","owner_inherited":false,"start":"2027-03-19","end":"2027-03-19",
 "declared_end":null,"slack_days":null,"estimate_days":null,"tags":[],"parent_line":null,"leaf":true}
```

Need calendar days? Use weeks (`1w` is 7 days), a range `A .. B` or dates. A `d` estimate counts
weekdays, so it is not calendar days. Agents leave the estimate off unless an owner gave one.

Tags are the extension point. A `#tag` means nothing to the scheduler, and tools pass it through
untouched. A team may agree on conventions, for example `#team-backend` for the team that does the
work, `#fte-50` for half of a person, or `#skill-welding` for a skill a step needs. An estimate is
time: `5d` means five weekdays, Monday to Friday, whatever the effort. If a tool
needs effort, put it in a tag such as `#effort-2d`. Unknown `key: value` pairs are not an extension
point. The parser ignores them and warns, so use a tag instead.

`pmd load FILE --deep` shows, per owner and week, where someone has more than one item at once:

```
Bo  2027-W10  2 at once: plan.md:6 Build; plan.md:9 Migrate data
```

The forecast never levels work. It reports the overlap and stops there. The fix is the owner's edit:
add `after: ^other` to do one item after the other, or `after: DATE` to start on a given day.

Results come back into the plan only as the owner's own edit: an `after:`, a longer estimate, or a
moved promise. No tool writes someone else's file. A tool that finds a problem tells the owner, and
the owner decides.

Order is priority. Items higher in a file come first. In a parent, the order of its link lines sets
the priority between sub-projects. No field is needed.

Agents often finish work in minutes. For that kind of work, leave the estimate off. An item with no
estimate is a point on the day its predecessors end, so a chain of small steps fits in one day. The
format has no hours: what matters is the day something is done.

An owner may be written `Name <email>` when tools need a stable identity, for example
`owner: Bo Osei <bo@example.org>`. Plain names still work for people and teams that do not need one.

## FAQ

**Do we need the tools?** No. The files are readable and editable by hand. Tools add forecasts and checks.

**Can a parent see a child's whole plan?** No, only its Commitment and non-`#private` milestones.

**How do we handle holidays?** A `d` estimate counts weekdays with no holidays. Put the real constraint in as a date.

**What if a child ignores `by:`?** It stays "requested" in `pmd ready` and `pmd tree` until answered.

**Can we migrate from a tracker?** Export milestones and epics as list items; do not import every ticket.
