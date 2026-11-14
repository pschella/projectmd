# Driver assistance (ADAS) — level 2 highway and lane functions

- Owner: Noor Bakker <noor.bakker@example.com>
- Stakeholders: Marco Rossi <marco.rossi@example.com>, Ahmed El Amrani <ahmed.elamrani@example.com>
- Commitment: 2028-08-31
- State: planned
- ID: cars.example.com/aurora/software/adas

Level 2 driver assistance: adaptive cruise, lane keeping and highway assist, on a camera and
radar set. The vehicle software plan asks for release by 2028-05-31. This file commits to
2028-08-31. The gap is an open conflict, to be negotiated with the software team.

## Milestones

- [ ] 2027-03-31 Sensor set frozen  #public  ^sensors
- [ ] 2027-12-15 Level-2 function spec approved  ^fn-spec
  - approver: @marco
- [ ] 2028-04-30 Track validation complete  ^track
- [ ] 2028-08-31 ADAS release for SOP  #public  ^adas-release
  - approver: @marco

## Work

- [ ] Camera and radar supplier nomination  8w
- [ ] Lane-keeping function design  12w  after: ^sensors
- [ ] Highway-assist function design  10w  after: ^fn-spec
- [ ] Simulation and hardware-in-the-loop tests  2027-Q2 .. 2027-Q4  after: ^sensors  ^hil
- [ ] Radar A-samples  after: [Radar A-samples][radar]
- [ ] Track validation runs  8w  after: ^hil  after: ^fn-spec

[radar]: https://sensor-vendor.example.net/projects/radar-r3.md#a-samples
