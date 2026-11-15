# Powertrain

- Owner: Jan de Boer <jan.deboer@example.com>
- Stakeholders: Sanne Visser <sanne.visser@example.com>
- Commitment: 2028-02-15

Electric drive unit, battery pack and charging for Aurora.

## Key dates

- [ ] 2027-05-31 Powertrain concept frozen  ^concept-frozen
- [ ] 2028-02-15 Powertrain validated  ^validated

## Plan

- [ ] Phase 1 — Concept & design  2026-Q4 .. 2027-Q4
  - [ ] [Battery pack](battery-pack/battery-pack.md)  by: 2027-12-15
  - [ ] [Inverter (supplier)][inv]  by: 2027-09-30
  - [ ] E-motor  owner: Priya Nair <priya.nair@example.com>
    - [ ] Electromagnetic design  6w  ^em-design
    - [ ] Dyno testing  8w  after: ^em-design  ^dyno-testing
- [ ] Phase 2 — Validation  2027-Q4 .. 2028-Q1
  - [ ] [Inverter A-samples][inv-a]  ^inv-2207-a-samples
  - [ ] On-board charger integration  6w  after: ^inv-2207-a-samples
  - [ ] [Pack validated][pack-val]  ^battery-pack-validated
  - [ ] Powertrain validation  8w  after: ^dyno-testing  after: ^battery-pack-validated

## Risks

- The inverter comes from a supplier; its A-samples are tracked on the supplier's server.

[inv]: https://supplier-portal.example.net/projects/inv-2207.md
[inv-a]: https://supplier-portal.example.net/projects/inv-2207.md#a-samples
[pack-val]: battery-pack/battery-pack.md#validated
