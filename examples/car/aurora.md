# Aurora — compact electric hatchback

- Owner: Sanne Visser <sanne.visser@example.com>
- Stakeholders: Marco Rossi <marco.rossi@example.com>, @lena, @product-board
- Commitment: 2028-11-30
- State: concept
- ID: cars.example.com/aurora

A compact five-door EV for the European market: 400 km range, priced under €30k,
start of production in Q4 2028.

## Milestones

- [ ] 2027-01-29 Concept approval  ^concept-approval
  - approver: @product-board
  - Decision criteria: 400 km range confirmed by the powertrain team;
    cost target under €30k; styling clinic done.
  - Outcome: `[x]` passed or `[-]` rejected, recorded here with the date.
    Who decided and when come from version history.
- [ ] 2027-06-30 Design freeze  #public  ^design-freeze
- [ ] 2027-Q4 Prototype build complete  #public  ^prototypes
- [ ] 2028-03-31 Tooling release  #public  ^tooling-release
  - approver: Marco Rossi <marco.rossi@example.com>
- [ ] 2028-09-30 Type approval granted  #public  ^type-approval
- [ ] 2028-11-30 Start of production (SOP)  #public  ^sop

## Work

- [x] Market study  2026-10-01 .. 2026-11-13  owner: @lena
- [ ] Concept & styling  2026-11-16 .. 2027-01-22  owner: @sanne
  - [ ] Exterior styling clinic  2027-01-11 .. 2027-01-15
- [ ] [Powertrain](powertrain/powertrain.md)  by: 2028-02-28
- [ ] [Body & chassis](body/body.md)  by: ^tooling-release
- [ ] [Interior](interior/interior.md)  by: 2028-02-28
- [ ] [Vehicle software](software/software.md)  by: 2028-Q3
- [ ] [Homologation](homologation/homologation.md)  by: ^type-approval
- [ ] [Plant readiness](manufacturing/plant-readiness.md)  by: 2028-10-31
- [ ] Prototype build  8w  owner: @jan  after: [Pack B-samples][pack-b]
- [ ] Production readiness  2028-10-01 .. 2028-11-15  after: ^tooling-release  after: [Plant ready][plant-ready]

## Risks

- The 400 km range target is not yet confirmed by the powertrain team.
- Battery pack timing sets the powertrain schedule; see the pack's own risks.

[plant-ready]: manufacturing/plant-readiness.md#plant-ready
[pack-b]: powertrain/battery-pack/battery-pack.md#b-samples