# Your first plan in ten minutes

Noor Haddad is opening a bakery on 2027-06-01. We build her plan in seven steps. The finished
files are in [examples/bakery/](../examples/bakery/README.md). Commands assume `pmd` is on your
path and `PMD_TODAY=2027-03-01` (a fixed "today", so the output below is reproducible).

## 1. A todo list is already a plan

Make a file `bakery.md`. A heading, an owner, and checkbox items. Nothing else is required.

```markdown
# Bakery opening

- Owner: Noor Haddad

## Work
- [ ] Hire two bakers
- [ ] Test menu
- [ ] Go/no-go decision
- [ ] Bakery opens
```

```
$ pmd tree bakery.md
Bakery opening  (bakery.md)
  ## Work
    [ ] Hire two bakers
    [ ] Test menu
    [ ] Go/no-go decision
    [ ] Bakery opens
```

## 2. Add time where it helps

A duration (`3w`, real calendar time), an owner, an `^id` for anything others refer to, and `after:`
for order. Add only what you know. In days, `d` counts weekdays (`3d` from a Thursday ends on Monday)
and `w` counts calendar weeks (`1w` is seven days).

```markdown
- [ ] Hire two bakers  3w  owner: Noor Haddad  ^hire
- [ ] Test menu  2w  owner: Noor Haddad  ^menu
  Done when three bakes in a row pass the tasting panel.
- [ ] Go/no-go decision  after: ^menu  after: ^hire  owner: Noor  ^gono
```

The indented line is the item's description: free text, ideal for "done when ...". The go/no-go has no
estimate and no date: it is a decision Noor owes, so it counts as due today and what waits on it slips.

```
$ pmd ready bakery.md
bakery.md:6  DO  Hire two bakers  owner: Noor Haddad  due: 2027-03-21
bakery.md:7  DO  Test menu  owner: Noor Haddad  due: 2027-03-14

2 ready (1 file read)
```

`ready` lists work that can start today. The go/no-go is not there: it waits for both.

## 3. Milestones and promises

A leading date makes a milestone: a promise that other plans may depend on. Header `Commitment:` names
the promise of the whole project; here it is the opening, by id.

```markdown
- Commitment: ^opening
...
- [ ] 2027-06-01 Bakery opens  after: ^gono  ^opening
```

Only declared dates are written. The forecast is computed, never stored (output here is from the finished example, after step 5).

```
$ pmd schedule bakery.md
Bakery opening  2027-03-01 .. 2027-05-25
  ...
```

## 4. Delegate by linking

The kitchen is Dmitri's job and the food licence is Ama's. Each has a file of their own. Noor links to
what she needs and says when with `by:`. Linking to a milestone (`#ovens`) makes it a deliverable;
`Test menu` depends on it with `after: ^ovens`, a promise rather than a copied date.

```markdown
- [ ] [Ovens installed](fitout/fitout.md#ovens)  by: 2027-05-10  ^ovens
- [ ] [Kitchen fit-out complete](fitout/fitout.md#complete)  by: 2027-05-15  ^fitout
- [ ] [Food licence granted](licences.md#food-licence)  by: 2027-05-15  ^licence
- [ ] Test menu  2w  after: ^ovens  owner: Noor Haddad  ^menu
```

Dmitri's queue lists the work he owns. His promises are already in his file, so there are no requests
for him to answer:

```
$ pmd ready bakery.md --deep --owner Dmitri
fitout/fitout.md:7  DO  Design  owner: Dmitri Volkov  due: 2027-03-14  started: 2027-03-01

1 ready (3 files read)
```

## 5. Negotiate a date

Dmitri answers by dating milestones in his own file (`2027-05-04 Ovens installed`,
`2027-05-25 Kitchen fit-out complete`). Noor's file does not change: the contract states are computed.

```
$ pmd tree bakery.md
Bakery opening  commitment=2027-06-01  (bakery.md)
  ## Work
    [ ] Hire two bakers  3w
    <> Ovens installed  -> fitout/fitout.md#ovens 2027-05-04 agreed → child
    <> Kitchen fit-out complete  -> fitout/fitout.md#complete 2027-05-25 disputed → parent
    <> Food licence granted  -> licences.md#food-licence 2027-04-12 agreed → child
    ...
```

The ovens are agreed (promise before `by:`). The fit-out is disputed: asked 05-15, promised 05-25. A
dispute is the parent's move, because a promise is what the child can do and `by:` is only a wish.

```
$ pmd ready bakery.md --deep --owner Noor
bakery.md:7  DO  Hire two bakers  owner: Noor Haddad  due: 2027-03-21
bakery.md:9  DECIDE  Kitchen fit-out complete  owner: Noor Haddad  due: 2027-05-15  promises 2027-05-25, asked by 2027-05-15

2 ready (3 files read)
```

Noor can accept (change her `by:` to 2027-05-25), re-plan, or talk to Dmitri. `pmd check --deep` keeps
the warning (`W-DISPUTED`) until she does.

## 6. Find your work with `pmd ready`

Each person (or agent) asks for their own queue: `do` for work, `decide` for disputes they must settle,
`answer` for requests they have not answered, `deliver` for overdue promises, `fix` for unreadable links.

```
$ pmd ready bakery.md --deep --owner Ama
licences.md:8  DO  Food licence granted  owner: Ama Owusu  due: 2027-04-11  started: 2027-02-15  (declared 2027-04-12)

1 ready (3 files read)
```

Finish an item by ticking it and writing the day: `- [x] ... done: 2027-02-20`. Without `done:` the item is
finished with an unknown date; never invent one. To start work, add `after: <today>`.

## 7. Ask `pmd why` when a date moves

The go/no-go is forecast for 2027-05-18. Why?

```
$ pmd why bakery.md ^gono --deep
bakery.md:13  Go/no-go decision  forecast 2027-05-18  after
  <- bakery.md:11  Test menu  2027-05-18  estimate, starts 2027-05-05 after the next item
  <- bakery.md:8  Ovens installed  2027-05-04  promise of fitout/fitout.md ^ovens
  == fitout/fitout.md ==
  <- fitout/fitout.md:9  Ovens installed  2027-04-25  after
  <- fitout/fitout.md:8  Building work  2027-04-25  estimate, starts 2027-03-15 after the next item
  <- fitout/fitout.md:7  Design  2027-03-14  estimate, starts 2027-03-01 (after: date)
```

The chain ends in Dmitri's file, not in Noor's: if Noor wants the date earlier, that is the conversation to have.
Each hop is local, so this works however large the plan grows.

## 8. Repair a date with `pmd replan`, review with `pmd changes` and `pmd board`

`pmd replan` proposes the smallest edits to a file's own dates that make it consistent. It is a dry run
unless you add `--write`. On the bakery it finds one disputed promise: Noor asked for the fit-out by
2027-05-15, and Dmitri promised 2027-05-25.

```bash
cd parser && PMD_TODAY=2027-03-01 ./target/release/pmd replan ../examples/bakery/bakery.md
# ../examples/bakery/bakery.md:9  accept dispute  2027-05-15 → 2027-05-25 (+10d)
# 1 edits (dry run; --write to apply)
```

`replan` edits only the file it is given. It never changes another team's dates, so the fit-out
promise stays with Dmitri's file. Noor can accept the date, or ask Dmitri to move it.

`pmd changes OLD NEW` lists what moved between two versions of one file. Give it the old version
from git:

```bash
pmd changes <(git show HEAD~1:bakery.md) bakery.md
```

`pmd board` shows one file as a board: what waits, what is ready, what is in progress and what is done.
Its footer gives the flow forecast for each open milestone.

```bash
pmd board fitout/fitout.md
```

Every command is a view. None of them stores state, so you can run them at any time.

## Next steps

- [CHEATSHEET.md](CHEATSHEET.md): the whole notation on one page.
- [FORMAT.md](FORMAT.md): the specification.
- [ADOPTING.md](ADOPTING.md): using projectmd in a team or organisation.
