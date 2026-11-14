# Plant readiness — Aurora assembly plant

- Owner: Omar Haddad <omar.haddad@example.com>
- Stakeholders: Sanne Visser <sanne.visser@example.com>, @purchasing-board
- Commitment: 2028-09-30
- State: planned
- ID: cars.example.com/aurora/manufacturing/plant-readiness

Plant layout, capex, line installation and commissioning for Aurora. It is a branch of its own
under the programme because the plant also builds the Transit-E van; van-only work lives in
the van plan.

## Milestones

- [x] 2026-10-30 Site selected  ^site
- [ ] 2027-11-30 Capex approved  ^capex
  - approver: @purchasing-board
- [ ] 2028-06-30 Line installation complete  #public  ^line-installed
- [ ] 2028-09-30 Plant ready for SOP  #public  ^plant-ready
  - approver: @omar

## Work

- [x] Site selection study  2026-09-01 .. 2026-10-30  owner: @omar
- [ ] Brownfield survey  2027-Q1  owner: @omar
- [ ] Plant layout  8w  owner: @omar  after: ../aurora.md#design-freeze
- [ ] Battery pack assembly cell  16w  after: ^capex  after: [Pack validated][pack-val]
- [ ] Paint shop commissioning  6w  after: [Paint ready](../body/paint-shop.md#paint-ready)
- [ ] [Supplier PPAP](supplier-ppap.md)  by: 2028-05-31
- [ ] Operator training  4w  after: ^line-installed
- [ ] Permits and works council consultation  2027-Q2 .. 2027-Q3  owner: @omar

The brownfield permit is not granted yet. Until it is, the layout date is a plan, not a promise.
[pack-val]: ../powertrain/battery-pack/battery-pack.md#validated