# Battery pack — 60 kWh platform pack

- Owner: Priya Nair <priya.nair@example.com>
- Stakeholders: Jan de Boer <jan.deboer@example.com>, Tom Peeters <tom.peeters@example.com>
- Commitment: 2026-09-01 .. 2027-11-30

Shared 60 kWh pack used by both the Aurora hatchback and the Transit-E van. Its sub-project is
the cell module, which has a supplier plan under it.

## Milestones

- [x] 2026-10-31 Cell chemistry selected  #private  ^chemistry
- [ ] 2027-03-31 Cell supplier nominated  #private  ^cell-nomination
- [-] 2027-06-15 Pack design freeze  ^pack-design-freeze
  - Withdrawn: replaced by "Pack design release" below.
- [ ] 2027-06-15 Pack design release  ^pack-design-release
- [ ] 2027-09-30 B-samples available  ^b-samples
- [ ] 2027-11-30 Pack validated  ^validated

## Work

- [x] Cell chemistry benchmark  2026-09-01 .. 2026-10-31
- [ ] Cell supplier RFQ  8w
- [ ] [Cell module](cell-module.md)  by: 2027-06-30
- [ ] [Cell module sign-off][cm-sign]  ^cell-module-sign-off
- [ ] Module & pack design  12w  after: ^cell-nomination  after: ^cell-module-sign-off
- [ ] [Cell C-samples][c-samples]  ^ncm-811-2027-c-samples
- [ ] Cell samples from supplier  after: ^ncm-811-2027-c-samples
- [ ] Abuse & crash testing  6w  after: ^b-samples
- [ ] Fix issue: thermal runaway test rig  2w

## Risks

- The cell supplier is not nominated yet; the RFQ result could move the pack schedule.
- Open conflict: Transit-E asks for the pack by 2027-10-31, one month before the pack commits to
  2027-11-30. To be negotiated with the van team.

[c-samples]: https://cells.example.org/plans/ncm-811-2027.md#c-samples
[cm-sign]: cell-module.md#sign-off
