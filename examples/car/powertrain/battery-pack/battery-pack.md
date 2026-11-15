# Battery pack — 60 kWh platform pack

- Owner: Priya Nair <priya.nair@example.com>
- Stakeholders: Jan de Boer <jan.deboer@example.com>, Tom Peeters <tom.peeters@example.com>
- Commitment: 2026-09-01 .. 2027-11-30
- State: in-development
- ID: cars.example.com/platform/battery-pack

Shared 60 kWh pack used by both the Aurora hatchback and the Transit-E van. Its sub-projects are
the cell module (which has a supplier plan under it) and the battery management system.

## Milestones

- [x] 2026-10-31 Cell chemistry selected  ^chemistry
- [ ] 2027-03-31 Cell supplier nominated  ^cell-nomination
  - approver: @purchasing-board
- [-] 2027-06-15 Pack design freeze  #public  replaced-by: ^pack-design-release  ^pack-design-freeze
- [ ] 2027-06-15 Pack design release  #public  ^pack-design-release
- [ ] 2027-09-30 B-samples available  #public  ^b-samples
- [ ] 2027-11-30 Pack validated  #public  ^validated

## Work

- [x] Cell chemistry benchmark  2026-09-01 .. 2026-10-31
- [ ] Cell supplier RFQ  8w
- [ ] [Cell module](cell-module.md)  by: 2027-06-30
- [ ] [Battery management system](bms.md)  by: 2027-09-30
- [ ] Module & pack design  12w  after: ^cell-nomination  after: [Cell module sign-off][cm-sign]
- [ ] Cell samples from supplier  after: [Cell C-samples][c-samples]
- [ ] Abuse & crash testing  6w  after: ^b-samples  after: [BMS software release](bms.md#sw-release)
- [ ] Fix issue: thermal runaway test rig  2w  ticket: PACK-311

## Risks

- The cell supplier is not nominated yet; the RFQ result could move the pack schedule.
- Open conflict: Transit-E asks for the pack by 2027-10-31, one month before the pack commits to
  2027-11-30. To be negotiated with the van team.

[c-samples]: https://cells.example.org/plans/ncm-811-2027.md#c-samples
[cm-sign]: cell-module.md#sign-off
