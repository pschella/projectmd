# Cell module — 12-cell module for the 60 kWh pack

- Owner: @jonas
- Stakeholders: Priya Nair <priya.nair@example.com>
- Commitment: 2027-06-30
- State: in-development
- ID: cars.example.com/platform/battery-pack/cell-module

Mechanical and thermal module holding twelve cells and their busbars. Owned by the pack team.
The cell supplier's gates are tracked in a sub-project one level further down.

## Milestones

- [x] 2026-11-06 Module concept frozen  ^module-concept
- [ ] 2027-02-26 Module design sign-off  #public  ^sign-off
  - approver: @priya
  - Decision criteria: housing stiffness target met; busbar resistance within budget.
- [ ] 2027-04-30 Module thermal release  ^thermal
- [ ] 2027-06-30 Module production release  ^prod-release

## Work

- [x] Module layout study  2026-10-12 .. 2026-11-06  owner: @jonas
- [ ] Busbar and sensing harness design  6w  owner: @jonas  after: ^module-concept
- [ ] Module housing design  6w  owner: @jonas  after: ^module-concept
- [ ] Cell-to-module thermal test  4w  after: ^sign-off  ^thermal-test
- [ ] [Supplier qualification](cell-module/supplier-qualification.md)  by: 2027-04-15
- [ ] Production readiness review  1w  after: ^thermal-test  after: [Supplier qualified][sq-q]

## Risks

- The housing supplier has not quoted yet; a late quote moves the sign-off, not the release.
- Cell swelling under the thermal test is the main open technical question.

[sq-q]: cell-module/supplier-qualification.md#qualified
