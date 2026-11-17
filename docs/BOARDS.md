# Working board-style: for teams used to kanban or agile

A board is a view of the plan, not a second system. Its columns come from what the file already
says. An item is waiting, ready, in progress or done because of its `after:` and `done:` lines, so
you move a card by editing one line. Inside the team's file, work can flow the way a kanban team
likes it. Outside the file, the team still makes dated promises. The team boundary is the file
boundary: flow inside, promises outside.

## The four columns

| Column | An item is here when | Derived from | To move it |
|---|---|---|---|
| Waiting | open, and an `after:` it names is not met yet | `after: ^id`, or a future `after: DATE` | nothing: it moves when the blocker is ticked |
| Ready | open, every `after:` is met, no start recorded | no `after: DATE` on or before today | pull it: add `after: <today>` and `owner:` |
| Doing | open, with a start recorded | `after: DATE` on or before today | finish it: tick `[x]` and add `done: <today>` |
| Done | ticked | `done:` | none |

Dropped items (`[-]`) do not show. The order of the list is the priority: the top item is first in
its column.

## An example

A payments team works in `team.md`. Today is 10 March 2027.

```markdown
# Payments team

- Owner: Ana Ruiz
- Commitment: ^release

## Work
- [x] Refund API design  owner: Ana Ruiz  done: 2027-03-05
- [ ] Refund endpoint  3d  after: 2027-03-08  owner: Ben Osei  ^api
  Done when the endpoint passes the contract tests.
- [ ] Refund receipts  2d  after: 2027-03-09  owner: Chen Wei  ^receipts
- [ ] Refund notice email  2d  owner: Ben Osei  ^email
- [ ] Review refund endpoint  after: ^api  owner: Ana Ruiz  ^review
- [ ] 2027-03-26 Refunds live for all shops  after: ^review  after: ^receipts  ^release
```

The board shows:

- **Done:** Refund API design.
- **Doing:** Refund endpoint (Ben, since 8 March) and Refund receipts (Chen, since 9 March).
- **Ready:** Refund notice email. Ben can pull it now.
- **Waiting:** Review refund endpoint, for the endpoint; Refunds live, for the review and the receipts.

`Commitment: ^release` makes the last line the team's promise. Everything above it is the team's own
flow. The review is an item with an owner, so it appears in Ana's queue.

## Where your tracker concepts went

| Tracker concept | In projectmd |
|---|---|
| Backlog and priority | the order of the items in the file |
| Assignee | `owner:` |
| Ticket id | `^id`. Keep the ticket id in the description if the tracker still holds the detail. |
| Description, acceptance criteria | indented text under the item. Write "Done when …". |
| Comments, history | version control. Say why a date moved in the description or the commit message. |
| Epic | a parent item, or its own file when it grows. |
| Sprint | a parent item with a range, such as `Sprint 12  2027-01-18 .. 2027-01-29`. A child that runs past the end is a conflict. |
| Review or QA column | not a state. It is an item someone must do: `Review X  after: ^x  owner: Ana`. It shows in that person's queue, and it can be late. A column cannot be late. |
| Blocked | Waiting, with the blocker named. |
| WIP limit | a view option: Doing items per owner, in the board footer. It is not a field in the file. |
| Velocity, burndown | throughput and median cycle time, from the start and `done:` dates. |

## A team that does not estimate

A plain prioritised list with no estimates is a valid plan. Its promise to the project is a dated
milestone. Without estimates the scheduler cannot warn about that date, but the board can use the
file's own history. This is the flow forecast:

1. **Throughput:** items finished per week, from the `done:` dates over recent weeks.
2. **Items left:** the open items the milestone waits on, found through `after:`.
3. **Likely date:** items left divided by throughput, counted in weeks from today. It is shown
   beside the declared date. For example, 9 items left at 3 a week is about 2027-03-31, shown
   beside the declared 2027-03-26.

The flow forecast is information only. It changes no schedule. The level above still sees only the
promise. With no finished items there is no throughput, so no forecast.

## What you do not get, on purpose

There are no custom workflow states, ticket types, story points, sprint ceremonies, permissions or
attachments. Anything that could be computed from the files is not stored, so nothing can go out of
step with the files ([DESIGN §4.2](DESIGN.md#42-the-three-laws)). Permissions belong to the store
the files live in. Put attachments wherever the team keeps them, and link to them from the
description.

## Commands

```
pmd board FILE... [--deep] [--owner NAME] [--json]     (new in 0.9)
pmd ready FILE... [--deep] [--owner NAME]
```

`pmd board` prints the four columns in file order. Waiting items name their blocker, and Doing
items show their start date. The footer gives work in progress per owner, throughput, median cycle
time, and the flow forecast for each open milestone. `--owner` shows one person's items. `--json`
gives the same data to scripts and agents. The viewer's Board tab shows the same view, read-only
(new in 0.9).

`pmd ready --owner NAME` is your personal pull queue. It lists the work you can start now, and the
moves you owe: decide, answer, deliver.

## Daily routine

**Team member**

1. Run `pmd ready --owner NAME` to see what is yours.
2. Pull one Ready item: add `after: <today>` in the same edit.
3. When it is done, tick `[x]` and add `done: <today>`.
4. Run `pmd fmt` and `pmd check`, then commit.

**Team lead**

1. Check each promise's forecast on the board. If the likely date is past the declared one, act now.
2. Move the promise rather than miss it. Change the date before it passes, and say why in the description.
3. The parent sees a later date as a dispute in its own queue. It accepts, re-plans or talks to you.
   You do not edit its file. Tell the parent first.
