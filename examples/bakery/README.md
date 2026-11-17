# Bakery

A small example: Noor Haddad opens a bakery on 2027-06-01. Three owners, three files:

- [bakery.md](bakery.md): Noor's plan. A todo list with estimates and `after:`, a milestone (the opening), a decision
  she owes (`owner: Noor`), one item with a description ("done when ..."), and three links:
  `by:` asks of other people, one of them a deliverable (`fitout/fitout.md#ovens`) that `Test menu` is `after:`.
- [fitout/fitout.md](fitout/fitout.md): Dmitri Volkov, the contractor. His Commitment is the kitchen fit-out.
- [licences.md](licences.md): Ama Owusu, the accountant. One done item (`done:`).

The contract on "Kitchen fit-out complete" is disputed: Noor asked for 2027-05-15, Dmitri promises 2027-05-25.
It is Noor's move (accept, re-plan or talk).

Try it (today is 2027-03-01):

```
export PMD_TODAY=2027-03-01
pmd check --deep examples/bakery/bakery.md      # exactly one W-DISPUTED
pmd tree examples/bakery/bakery.md              # contract states per link
pmd ready examples/bakery/bakery.md --deep      # the queue: do, decide, answer
pmd why examples/bakery/bakery.md ^gono --deep  # what fixes the go/no-go date
```

Viewer: `viewer/?root=examples/bakery/bakery.md&today=2027-03-01`

Walk through it step by step in [docs/TUTORIAL.md](../../docs/TUTORIAL.md).
