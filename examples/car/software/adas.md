# Driver assistance (ADAS) — level 2 highway and lane functions

- Owner: Noor Bakker <noor.bakker@example.com>
- Stakeholders: Marco Rossi <marco.rossi@example.com>, Ahmed El Amrani <ahmed.elamrani@example.com>
- Commitment: 2028-08-31

Level 2 driver assistance: adaptive cruise, lane keeping and highway assist, on a camera and
radar set. The vehicle software plan asks for release by 2028-05-31. This file commits to
2028-08-31. The gap is an open conflict, to be negotiated with the software team.

## Milestones

- [ ] 2027-03-31 Sensor set frozen  ^sensors
- [ ] 2027-12-15 Level-2 function spec approved  #private  ^fn-spec
- [ ] 2028-04-30 Track validation complete  #private  ^track
- [ ] 2028-08-31 ADAS release for SOP  ^adas-release

## Work

- [ ] Camera and radar supplier nomination  8w
- [ ] Lane-keeping function design  12w  after: ^sensors
- [ ] Highway-assist function design  10w  after: ^fn-spec
- [ ] Simulation and hardware-in-the-loop tests  2027-Q2 .. 2027-Q4  after: ^sensors  ^hil
- [ ] [Radar A-samples][radar]  ^radar-r3-a-samples
- [ ] Radar A-samples  after: ^radar-r3-a-samples
- [ ] Track validation runs  8w  after: ^hil  after: ^fn-spec

[radar]: https://sensor-vendor.example.net/projects/radar-r3.md#a-samples
