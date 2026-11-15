# Cell supplier qualification — NCM cells for the module

- Owner: Ingrid Haas <ingrid.haas@example.com>
- Stakeholders: @jonas, Priya Nair <priya.nair@example.com>
- Commitment: 2027-04-15

Qualification of the nominated cell supplier for module builds. This is the detailed level of
the plan: every gate has an owner and an exact date. The supplier's own plan is linked, not
copied.

## Gates and milestones

- [x] 2026-11-10 Supplier process audit done  #private  ^audit
- [ ] 2027-01-29 Process FMEA accepted  #private  ^pfmea
- [ ] 2027-03-12 PPAP level 3 submitted for cells  #private  ^ppap-submit
- [ ] 2027-04-15 Supplier qualified for module builds  ^qualified
  - Decision criteria: audit findings closed; PPAP accepted; two lots within capability.

## Work

- [x] Supplier questionnaire returned  2026-10-20 .. 2026-11-06  owner: @ingrid
- [ ] Cell line capability study  3w  after: ^audit  owner: @ingrid
- [ ] Cell lot traceability review  1w  owner: @ingrid
- [ ] Process FMEA workshop  4w  after: ^audit  owner: @jonas
- [ ] [Cell spec v3][cell-spec]  ^ncm-811-2027-spec-v3
- [ ] Cell specification v3 agreed with supplier  after: ^ncm-811-2027-spec-v3
- [ ] Dimensional and sample run at supplier  3w  after: ^pfmea  owner: @ingrid
- [ ] Qualification report  2w  after: ^ppap-submit
- [-] Second-source line audit  dropped after the single-source decision

The single-source decision is recorded in the pack risks. If it reverses, this plan needs a
second line audit before the qualified gate can close.

[cell-spec]: https://cells.example.org/plans/ncm-811-2027.md#spec-v3
