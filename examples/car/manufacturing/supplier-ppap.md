# Supplier PPAP — production part approval

- Owner: Ingrid Haas <ingrid.haas@example.com>
- Stakeholders: Omar Haddad <omar.haddad@example.com>
- Commitment: 2028-05-15
- State: qualification

Production part approval for every purchased part of Aurora, batch by batch. It sits under
plant readiness so that the line cannot start on unapproved parts.

## Milestones

- [ ] 2027-12-15 Part families list frozen  ^families
- [ ] 2028-02-29 PPAP level 3 batch submitted  ^submit
- [ ] 2028-04-30 PPAP approved for all part families  #public  ^ppap-approved
  - approver: @purchasing-board

## Work

- [x] Capability study template agreed  2026-10-12 .. 2026-11-06  owner: @ingrid
- [ ] Control plans per part family  6w  owner: @ingrid  after: ^families
- [ ] Dimensional layout reports  5w  after: ^families
- [ ] Sample run at supplier sites  3w  after: ^families
- [ ] Cell supplier PPAP  after: [Supplier qualified][sq-ref]
- [ ] Inverter A-samples check  after: [Inverter A-samples][inv-a]
- [-] Single-source exception for seats  not needed after the second nomination

[inv-a]: https://supplier-portal.example.net/projects/inv-2207.md#a-samples
[sq-ref]: ../powertrain/battery-pack/cell-module/supplier-qualification.md#qualified