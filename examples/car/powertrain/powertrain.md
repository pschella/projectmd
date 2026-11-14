# Powertrain

- Owner: Jan de Boer <jan.deboer@example.com>
- Stakeholders: Sanne Visser <sanne.visser@example.com>
- Commitment: 2028-02-15
- State: in-development

Electric drive unit, battery pack and charging for Aurora.

## Key dates

- [ ] 2027-05-31 Powertrain concept frozen  #public  ^concept-frozen
- [ ] 2028-02-15 Powertrain validated  #public  ^validated

## Phase 1 — Concept & design  2026-Q4 .. 2027-Q4

- [ ] [Battery pack](battery-pack/battery-pack.md)  by: 2027-12-15
- [ ] [Inverter (supplier)][inv]  by: 2027-09-30
- [ ] E-motor  owner: Priya Nair <priya.nair@example.com>
  - [ ] Electromagnetic design  6w  ^em-design
  - [ ] Dyno testing  8w  ^dyno-testing  after: ^em-design

## Phase 2 — Validation  2027-Q4 .. 2028-Q1

- [ ] On-board charger integration  6w  after: [Inverter A-samples][inv-a]
- [ ] Powertrain validation  8w  after: ^dyno-testing  after: [Pack validated][pack-val]

## Risks

- The inverter comes from a supplier; its A-samples are tracked on the supplier's server.

[inv]: https://supplier-portal.example.net/projects/inv-2207.md
[inv-a]: https://supplier-portal.example.net/projects/inv-2207.md#a-samples
[pack-val]: battery-pack/battery-pack.md#validated