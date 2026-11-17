# Your first plan in ten minutes

A project manager thinks in people, work and dates: who does what, how long it takes, and when it must
be done. A projectmd plan is a Markdown file that says exactly that, and `pmd` answers the questions a
project manager asks of it.

Noor Haddad is opening a bakery on 2027-06-01. We build her plan in eight steps. The finished
files are in [examples/bakery/](../examples/bakery/README.md). Commands assume `pmd` is on your
path and `PMD_TODAY=2027-03-01` (a fixed "today", so the output below is reproducible).

## 1. A todo list is already a plan

Start with the work. Make a file `bakery.md`: a heading, the person responsible for the project, and
checkbox items. Nothing else is required.

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

## 2. Add people and time where it helps

For each piece of work, say who does it (`owner:`), how long it takes (`3w`, real calendar time) and
what it waits for (`after:`). Give an item an `^id` when other items refer to it. Add only what you
know. In days, `d` counts weekdays (`3d` from a Thursday ends on Monday) and `w` counts calendar weeks
(`1w` is seven days). `after:` also takes a real date: the earliest day the work can start.

```markdown
- [ ] Hire two bakers  3w  owner: Noor Haddad  ^hire
- [ ] Test menu  2w  owner: Noor Haddad  ^menu
  Done when three bakes in a row pass the tasting panel.
- [ ] Go/no-go decision  after: ^menu  after: ^hire  owner: Noor  ^gono
- [ ] Stock the shop  1w  after: 2027-05-17  owner: Noor Haddad
- [ ] Bakery opens  after: ^gono
```

The indented line is the item's description: free text, ideal for "done when ...". The go/no-go has no
estimate and no date: it is a decision Noor owes, so it counts as due today and what waits on it slips.
Stocking waits for 2027-05-17, so the flour is fresh for opening.

What can start today?

```
$ pmd ready bakery.md
bakery.md:6  DO  Hire two bakers  owner: Noor Haddad  due: 2027-03-21
bakery.md:7  DO  Test menu  owner: Noor Haddad  due: 2027-03-14

2 ready (1 file read)
```

The go/no-go is not there: it waits for both. Stocking waits for its date, and the opening waits for
the go/no-go. Is anyone doing too much at once?

```
$ pmd load bakery.md
Noor Haddad  2027-W09  2 at once: bakery.md:6 Hire two bakers; bakery.md:7 Test menu
Noor Haddad  2027-W10  2 at once: bakery.md:6 Hire two bakers; bakery.md:7 Test menu
2 owner-weeks with more than one item (1 file read)
```

Noor hires and tests the menu in the same two weeks. If that is too much, `after: ^hire` on the test
menu puts them in sequence.

## 3. Dates: milestones and the commitment

Some dates are fixed: the opening is 2027-06-01. A date in front of an item makes it a milestone. The
`Commitment:` header is the date the whole project promises.

```markdown
- Commitment: 2027-06-01
...
- [ ] 2027-06-01 Bakery opens  after: ^gono
```

You write only the dates you decide. Everything else is forecast from the durations and the order, and
never stored.

```
$ pmd schedule bakery.md
Bakery opening  2027-03-01 .. 2027-05-23
  Work  2027-03-01 .. 2027-05-23
    Hire two bakers  2027-03-01 .. 2027-03-21
    Test menu  2027-03-01 .. 2027-03-14
    Go/no-go decision  2027-03-21 .. 2027-03-21
    Stock the shop  2027-05-17 .. 2027-05-23
    Bakery opens  2027-03-21 .. 2027-03-21  (declared 2027-06-01)  slack +72d
```

Slack is the margin between the date promised and the date forecast. With only Noor's own work in the
plan, the opening looks easy. The kitchen will change that.

## 4. Work other people own

The kitchen is Dmitri's job and the food licence is Ama's. A short task for someone else is just an
item with their name as `owner:`. Dmitri and Ama run plans of their own, though, so Noor does not copy
their work into hers. She adds one line per result she needs, links it to their file, and says when
she needs it with `by:`. `Test menu` can start once the ovens are in.

```markdown
- [ ] [Ovens installed](fitout/fitout.md#ovens)  by: 2027-05-10  ^ovens
- [ ] [Kitchen fit-out complete](fitout/fitout.md#complete)  by: 2027-05-15  ^fitout
- [ ] [Food licence granted](licences.md#food-licence)  by: 2027-05-15  ^licence
- [ ] Test menu  2w  after: ^ovens  owner: Noor Haddad  ^menu
```

The go/no-go now also waits for the licence, and the opening for the fit-out. Commands that should
follow these links take `--deep`. Dmitri sees his own work:

```
$ pmd ready bakery.md --deep --owner Dmitri
fitout/fitout.md:7  DO  Design  owner: Dmitri Volkov  due: 2027-03-14  started: 2027-03-01

1 ready (3 files read)
```

## 5. Negotiate a date

Dmitri answers with dates in his own file: ovens on 2027-05-04, fit-out complete on 2027-05-25. Noor's
file does not change, but her schedule does:

```
$ pmd schedule bakery.md
Bakery opening  2027-03-01 .. 2027-05-25
  Work  2027-03-01 .. 2027-05-25
    Hire two bakers  2027-03-01 .. 2027-03-21
    Ovens installed  2027-05-04 .. 2027-05-04  (declared 2027-05-10)  slack +6d
    Kitchen fit-out complete  2027-05-25 .. 2027-05-25  (declared 2027-05-15)  slack -10d
    Food licence granted  2027-04-12 .. 2027-04-12  (declared 2027-05-15)  slack +33d
    Test menu  2027-05-05 .. 2027-05-18
    Go/no-go decision  2027-05-18 .. 2027-05-18
    Stock the shop  2027-05-17 .. 2027-05-23
    Bakery opens  2027-05-25 .. 2027-05-25  (declared 2027-06-01)  slack +7d
```

The ovens and the licence come in time. The fit-out does not: Noor asked for 05-15, Dmitri can do 05-25.
The date Dmitri gives is what he can do; Noor's `by:` is only what she asked for. So settling it is
Noor's decision, and it shows up in her queue:

```
$ pmd ready bakery.md --deep --owner Noor
bakery.md:7  DO  Hire two bakers  owner: Noor Haddad  due: 2027-03-21
bakery.md:9  DECIDE  Kitchen fit-out complete  owner: Noor Haddad  due: 2027-05-15  promises 2027-05-25, asked by 2027-05-15

2 ready (3 files read)
```

The opening still has seven days of slack, so Noor can accept (change her `by:` to 2027-05-25),
re-plan, or talk to Dmitri. `pmd check --deep` keeps the warning (`W-DISPUTED`) until she does.

## 6. Each person's queue: `pmd ready`

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
