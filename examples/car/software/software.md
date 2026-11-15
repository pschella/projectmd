# Vehicle software

- Owner: Ahmed El Amrani <ahmed.elamrani@example.com>
- Stakeholders: Marco Rossi <marco.rossi@example.com>
- Commitment: 2028-Q3
- State: discovery

Infotainment, driver assistance, over-the-air updates and integration of all ECUs.
Plans are still coarse, so most dates are quarters. The sub-projects below carry their own plans;
the driver assistance one has exact dates and disagrees with the bound given here.

## Milestones

- [ ] 2027-Q2 Software architecture defined  ^architecture
- [ ] 2028-Q2 Feature complete  #public  ^feature-complete
- [ ] 2028-08-31 Release for SOP  #public  ^sop-release
  - approver: @marco

## Work

- [ ] [Infotainment](infotainment.md)  by: 2028-Q1
- [ ] [Over-the-air update backend](ota-backend.md)  by: 2028-Q1
- [ ] [Driver assistance (ADAS)](adas.md)  by: 2028-05-31
- [ ] Integrate battery management  after: [BMS release][bms-sw]
- [ ] Cybersecurity audit  4w  after: ^feature-complete
[bms-sw]: ../powertrain/battery-pack/bms.md#sw-release
