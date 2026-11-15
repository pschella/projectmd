# Cell module — 12-cell module for the 60 kWh pack

- Owner: @jonas
- Stakeholders: Priya Nair <priya.nair@example.com>
- Commitment: 2027-06-30

Mechanical and thermal module holding twelve cells and their busbars. Owned by the pack team.
The cell supplier's gates are tracked in a sub-project one level further down.

## Milestones

- [x] 2026-11-06 Module concept frozen  #private  ^module-concept
- [ ] 2027-02-26 Module design sign-off  ^sign-off
  - Decision criteria: housing stiffness target met; busbar resistance within budget.
- [ ] 2027-04-30 Module thermal release  #private  ^thermal
- [ ] 2027-06-30 Module production release  #private  ^prod-release

## Work

- [x] Module layout study  2026-10-12 .. 2026-11-06  owner: @jonas
- [ ] Busbar and sensing harness design  6w  after: ^module-concept  owner: @jonas
- [ ] Module housing design  6w  after: ^module-concept  owner: @jonas
- [ ] Cell-to-module thermal test  4w  after: ^sign-off  ^thermal-test
- [ ] [Supplier qualification](cell-module/supplier-qualification.md)  by: 2027-04-15
- [ ] [Supplier qualified][sq-q]  ^supplier-qualification-qualified
- [ ] Production readiness review  1w  after: ^thermal-test  after: ^supplier-qualification-qualified

## Risks

- The housing supplier has not quoted yet; a late quote moves the sign-off, not the release.
- Cell swelling under the thermal test is the main open technical question.

[sq-q]: cell-module/supplier-qualification.md#qualified
