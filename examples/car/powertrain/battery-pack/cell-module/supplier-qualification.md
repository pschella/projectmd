# Cell supplier qualification — NCM cells for the module

- Owner: Ingrid Haas <ingrid.haas@example.com>
- Stakeholders: @jonas, Priya Nair <priya.nair@example.com>
- Commitment: 2027-04-15
- State: qualification
- ID: cars.example.com/platform/battery-pack/cell-module/supplier-qualification

Qualification of the nominated cell supplier for module builds. This is the detailed level of
the plan: every gate has an owner and an exact date. The supplier's own plan is linked, not
copied.

## Gates and milestones

- [x] 2026-11-10 Supplier process audit done  ^audit
- [ ] 2027-01-29 Process FMEA accepted  ^pfmea
  - approver: @jonas
- [ ] 2027-03-12 PPAP level 3 submitted for cells  ^ppap-submit
- [ ] 2027-04-15 Supplier qualified for module builds  #public  ^qualified
  - approver: @purchasing-board
  - Decision criteria: audit findings closed; PPAP accepted; two lots within capability.

## Work

- [x] Supplier questionnaire returned  2026-10-20 .. 2026-11-06  owner: @ingrid
- [ ] Cell line capability study  3w  owner: @ingrid  after: ^audit
- [ ] Cell lot traceability review  1w  owner: @ingrid  ticket: PACK-322
- [ ] Process FMEA workshop  4w  owner: @jonas  after: ^audit
- [ ] Cell specification v3 agreed with supplier  after: [Cell spec v3][cell-spec]
- [ ] Dimensional and sample run at supplier  3w  owner: @ingrid  after: ^pfmea
- [ ] Qualification report  2w  after: ^ppap-submit
- [-] Second-source line audit  dropped after the single-source decision

The single-source decision is recorded in the pack risks. If it reverses, this plan needs a
second line audit before the qualified gate can close.

[cell-spec]: https://cells.example.org/plans/ncm-811-2027.md#spec-v3
