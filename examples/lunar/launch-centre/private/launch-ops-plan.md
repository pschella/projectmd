# Launch centre — private plan

- Owner: Marcus Tan <marcus.tan@lc.example.org>
- Commitment: 1961-09-15 .. 1969-11-28

## Inputs we need
- [x] [Vehicle interface spec](../../vehicle-centre/launch-vehicle.md#if-spec)  by: 1962-06-30  done: 1962-06-28  ^lv-spec
- [x] [Pad and umbilical interface control](../../vehicle-centre/launch-vehicle.md#pad-if)  by: 1962-09-20  done: 1962-09-18  ^pad-if
- [x] [Spacecraft interface spec for launch-control fit-out](../../spacecraft-centre/spacecraft.md#if-spec)  by: 1963-03-31  done: 1964-02-10  ^sc-spec

## Work
- [x] Acquire launch-site land  6m  done: 1962-03-30  ^land
- [x] Assembly-building design  6m  after: ^land  done: 1962-10-15  ^vab-design
- [x] Launch-pad design  6m  after: ^land  done: 1962-11-30  ^pad-design
- [x] Launch-control design  4m  after: ^land  done: 1962-08-20  ^lc-design
- [x] Assembly-building structure, topped out  16m  after: ^vab-design  after: 1963-01-09  done: 1964-04-14  ^vab-top
- [x] Assembly-building fit-out, ready for stacking  3m  after: ^vab-top  done: 1964-09-30  ^vab
- [x] Launch-pad construction  12m  after: ^pad-design  done: 1964-04-15  ^pad
- [x] Launch-control construction  12m  after: ^lc-design  done: 1963-12-20  ^lc
- [x] Crawler-transporter design  5m  after: ^lv-spec  done: 1963-06-14  ^crawler-design
- [x] Crawler-transporter build  12m  after: ^crawler-design  ^crawler-build
- [x] Crawler-transporter test  1m  after: ^crawler-build  done: 1965-01-26  ^crawler
- [x] Integrated ground checkout  6w  after: ^vab  after: ^lc  after: ^pad  after: ^crawler  after: ^sc-spec  done: 1965-03-15  ^checkout

## Launch readiness from 1966 (estimates are ours, not yet checked with the pad crew)
- [x] [Uncrewed test vehicle ready at launch centre](../../vehicle-centre/launch-vehicle.md#vehicle-ready)  by: 1967-09-15  done: 1967-09-05  ^lv-vehicle
- [x] [CM and SM crew-rating interface definition](../../csm-co/csm.md#crew-if)  by: 1965-09-01  ^csm-crew-if
- [x] [CM and SM crew-rating amendment](../../csm-co/csm.md#crew-rating)  by: 1969-03-31  done: 1969-03-05  ^csm-crew
- [x] [Crew-rated vehicle at launch centre](../../vehicle-centre/crewed-launch-vehicle.md#crew-vehicle)  by: 1969-06-16  done: 1969-06-05  ^crew-vehicle
- [x] Uncrewed-vehicle pad and launch-control configuration, umbilicals and consoles  6m  after: ^checkout  after: 1965-06-01  done: 1966-06-20  ^uv-cfg
- [x] Countdown dress rehearsal with the uncrewed vehicle  2w  after: ^lv-vehicle  after: ^uv-cfg  after: 1967-09-16  ^uv-cd
- [x] Crew access tower and escape-system ground interface, design  6m  after: ^csm-crew-if  after: 1965-09-02  done: 1966-09-30  ^crew-design
- [x] Crew access and escape interface installed at pad 1  8m  after: ^crew-design  after: ^uv-cd  after: 1967-11-10  done: 1968-03-20  ^crew-install
- [x] Crewed countdown rehearsal, crew-rated vehicle  2w  after: ^crew-install  after: ^csm-crew  after: ^crew-vehicle  done: 1969-07-10  ^crew-cd

## Countdowns for flights 2 and 3
- [x] [Crewed vehicle 2 at launch centre](../../vehicle-centre/crewed-launch-vehicle.md#crew-vehicle-2)  by: 1969-09-20  done: 1969-09-22  ^cv-2
- [x] [Crewed vehicle 3 at launch centre](../../vehicle-centre/crewed-launch-vehicle.md#crew-vehicle-3)  by: 1969-11-04  done: 1969-11-04  ^cv-3
- [x] 1969-10-15 Countdown for flight 2, crew-rated vehicle 2 on the pad  after: ^cv-2  done: 1969-10-15  ^cd-2
- [x] 1969-11-28 Countdown for flight 3, crew-rated vehicle 3 on the pad  after: ^cv-3  done: 1969-11-28  ^cd-3
