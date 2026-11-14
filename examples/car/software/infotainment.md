# Infotainment — head unit and connectivity

- Owner: @thijs
- Stakeholders: Marco Rossi <marco.rossi@example.com>, Lena Hoffmann <lena.hoffmann@example.com>
- Commitment: 2028-Q1
- State: planned

Head unit, smartphone projection and navigation for Aurora. Vendor-led: most of the scope is a
supplier deliverable, and this plan tracks our integration gates.

## Milestones

- [ ] 2027-Q1 Platform selected  ^platform
- [ ] 2027-Q4 Head unit hardware frozen  #public  ^hu-frozen
- [ ] 2028-Q1 Infotainment feature complete  #public  ^feature-complete

## Work

- [x] Requirements workshop  2026-10-05 .. 2026-11-06  owner: @marco
- [ ] Head-unit supplier RFQ  6w  owner: @marco
- [ ] Smartphone projection integration  10w  after: ^platform  after: ../aurora.md#design-freeze
- [ ] Navigation map data licence  2027-Q3 .. 2027-Q4  ticket: INFO-14
- [ ] Hardware validation on mule car  6w  after: ^hu-frozen
- [ ] Functional integration with ECUs  after: ^hu-frozen
- [-] In-house voice assistant  dropped in favour of the vendor cloud service

The navigation licence is the one open commercial point. Without it the feature-complete gate
slips by a quarter.
