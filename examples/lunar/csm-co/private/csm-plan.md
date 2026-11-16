# CSM contract — engineering plan (private)

- Owner: Dale Ferreira <dale.ferreira@csm-co.example.net>
- Commitment: 1962-04-15 .. 1969-03-31

Scope: command module (CM) and service module (SM): heat shield, landing system, service
propulsion (SPS engine, subcontracted), reaction control, fuel cells, environmental control.
Since LOR was adopted (1962-07-15): docking system (CM side of the docking, probe, tunnel,
latches) and the CM/SM side of the LM interfaces. Ground test articles: integration articles
(not qualified, for the spacecraft centre's integrated test), flight-standard qualification
articles, a structural test article, a thermal-vacuum chamber, parachute drop-test units, a
docking mechanism qualification set.

## Decision: preliminary design baseline moves to 1963-05-31
- LOR adds the docking system and the CM/SM side of the LM interfaces. The docking
  preliminary design (3 months, from today) and the LM interface control document (3 weeks,
  from the LM data on 1963-02-15) are on the baseline's path. The baseline moves from 1963-03-15
  to 1963-05-31, about 6 weeks of float on the forecast of 1963-04-28.
- To keep the qualification and crew-rating dates, detailed design and integration-article
  fabrication start when the internal preliminary design review closes (1963-04-28), not at the
  formal baseline. The baseline freezes the design; detailed design starts before it. The risk is
  rework if the baseline review changes the design.
- Cost: detailed design data moves from 1964-01-31 to 1964-02-29 (engineering forecast 1964-02-11,
  about 18 days of float). The schedule chains qualification from the promised 1964-02-29, so
  it shows qualified CM and SM with zero float; from the engineering forecast it is about
  three weeks early. Qualified CM and SM stays 1965-06-30. The crew-rating amendment 1966-06-30
  is promised as the spacecraft centre asked; it has no float, and any slip in qualification
  moves it one-for-one.

## Sales proposal against engineering view
- Sales proposal promised: preliminary design 1962-12-14, detailed design 1963-10-31,
  qualified CM and SM 1964-08-14.
- Front-file promises are the forecast plus about one month of float. The sales dates are
  not given to the centre.

## Risks (engineering view)
- LOR docking and LM interfaces are new scope. The docking design needs the LM docking
  and umbilical interface data from the lunar module contractor by 1963-02-15 (asked by the
  spacecraft centre). Without it, the LM interface definition cannot close.
- Ablator for lunar-return heating: screening is done (screening done), results still to be
  reviewed against the lunar-return speed.
- SPS engine: one supplier, no flight-type hot-fire yet. The 15-month estimate is ours, not the supplier's.
- Thermal-vacuum chamber: none in-house large enough for the SM. Procure and install (9 months)
  is on the qualification path.
- Staffing: about 40 engineers now; the plan needs about 300 by early 1963. The ramp is the
  largest single risk, and LOR scope adds to it.
- Qualification durations are first estimates. A failure in thermal-vacuum test restarts the chain.

## Award and system definition
- [x] 1962-04-15 Contract award  done: 1962-04-15  ^award
- [x] Subsystem trade studies (structure, heat shield, SPS, power, ECS)  4m  after: ^award  done: 1962-12-10  ^trades
- [x] Ablator screening tests, arc-jet  2m  after: ^trades  done: 1962-12-10  ^hs-screen
- [x] Requirements flow-down and interface definition, incl. LOR docking  2m  after: ^trades  done: 1962-12-10  ^reqs
- [x] Interface data package: mass, envelope, umbilicals  1m  after: ^reqs  done: 1962-12-10  ^if-task
- [x] Thermal-vacuum chamber, procure and install  9m  after: ^award  ^tvac

## Preliminary design
- [x] Preliminary design, command module (structure, heat shield, landing system)  4m  after: ^hs-screen  ^pd-cm
- [x] Preliminary design, service module (SPS, RCS, power, ECS)  4m  after: ^reqs  ^pd-sm
- [x] Docking system, CM and SM: preliminary design (probe, drogue, tunnel, latches, loads)  3m  ^dock-pd
- [x] LM interface control document, CM and SM side (docking, umbilicals, crew transfer)  3w  after: 1963-02-15  ^lm-def
- [x] LOR cost estimate, CM and SM (docking, LM interfaces, schedule)  3w  ^lor-cost-work
- [x] 1963-02-15 LOR cost and schedule impact to spacecraft centre  after: ^lor-cost-work  done: 1963-03-01  ^lor-cost
- [x] Internal preliminary design review  2w  after: ^pd-cm  after: ^pd-sm  after: ^dock-pd  after: ^lm-def  ^pd-rev
- [x] 1963-05-31 Preliminary design baseline, CM and SM  after: ^pd-rev  done: 1963-06-14  ^pdb
- [x] Parachute landing system, development and drop tests  6m  after: ^pdb  ^chute-dev
- [x] SPS engine development with supplier, hot-fire acceptance  15m  after: ^pdb  after: 1964-11-01  ^sps-dev

## Detailed design and integration articles
- [x] Detailed design, command module  9m  after: ^pd-rev  ^dd-cm
- [x] Detailed design, service module  9m  after: ^pd-rev  ^dd-sm
- [x] Detailed design, docking system and LM interfaces  4m  after: ^pd-rev  ^dd-dock
- [x] Drawing release and design data package  2w  after: ^dd-cm  after: ^dd-sm  after: ^dd-dock  ^drel
- [x] 1964-02-29 Detailed design data, CM and SM  after: ^drel  done: 1964-04-22  ^dd-data
- [x] Integration articles, fabrication and systems fit (CM and SM, not qualified, with docking mechanism)  after: 1964-01-06  by: 1965-05-31  done: 1965-06-10  ^int-fab

## Qualification
- [x] Flight-standard qualification articles, fabrication (one CM, one SM)  8m  after: ^dd-data  ^qa-fab
- [x] Docking mechanism qualification, dock and undock under load  3m  after: ^qa-fab  after: ^dd-dock  ^dock-qual
- [x] Vibration and acoustic test, CM and SM  2m  after: ^qa-fab  ^vib
- [x] Structural static test, structural test article  3m  after: ^qa-fab  ^struct
- [x] Landing system qualification drops  3m  after: ^qa-fab  after: ^chute-dev  ^landing
- [x] Thermal-vacuum qualification test  5m  after: ^vib  after: ^tvac  after: ^sps-dev  after: 1966-02-01  ^tv
- [x] Qualification report and review  1m  after: ^tv  after: ^struct  after: ^landing  after: ^dock-qual  ^qual-rep
- [x] 1966-08-31 CM and SM qualified for uncrewed flight  after: ^qual-rep  done: 1966-09-20  ^qualified

## Crew rating amendment (promised)
- [x] Crew-rating interface definition: life-support and escape interface  3m  after: ^dd-data  ^crew-if-work
- [x] 1965-09-01 Crew-rating interface definition delivered to spacecraft centre  after: ^crew-if-work  ^crew-if
- [x] Crew-rating amendment: life support, escape system, cabin materials  12m  after: 1966-01-15  ^crew-amend

## CM redesign after the fire (1967-02-15)
Context: the command-module ground-test fire of 1967-01-27 killed three crew. Crewed work stops. The
crew-rating amendment (`^crew-amend`, planned end 1967-01-15, ticked as delivered with no done date) chose
cabin materials that are now in question, so the amendment is reopened by the redesign. Uncrewed work
continues: `^qualified` (done 1966-09-20) and the uncrewed flights are not changed by this block.
Estimate from the event: about 18 months extra on the crew rating. Plan below: materials review now (8w,
starts today), redesign after the board report (gate 1967-04-30), then redesign qualification and the
revised amendment. Forecast crew rating about 1969-02-28; promise 1969-03-31 (forecast plus one month).
Open: the board may change the redesign scope (rework risk); whether the uncrewed CM needs the new hatch
and wiring is not yet decided (it is not in these items).

- [x] Fire cause and cabin materials flammability review, CM  8w  after: 1967-02-15  ^mat-rev
- [x] 1967-04-30 Fire investigation board report (external; gates the design freeze)  done: 1967-04-05  ^board
- [x] 1967-04-30 CM design freeze, redesign  after: ^board  done: 1967-05-15  ^design-freeze
- [x] Redesign, CM: new crew hatch, wiring, non-flammable cabin materials  6m  after: 1967-05-01  after: ^mat-rev  ^redesign
- [x] Redesign qualification: hatch, wiring and materials tests, CM  9m  after: 1967-11-01  after: ^redesign  done: 1968-08-01  ^redesign-qual
- [x] 1968-08-01 First crew-rated CSM, amendment configuration, delivered to spacecraft centre  done: 1968-08-01  ^csm-delivered
- [x] 1968-06-30 Crew-rating amendment: configuration freeze for the spacecraft centre  after: ^redesign  done: 1968-04-15  ^amend-freeze
- [x] Crew-rating amendment, revised for the CM redesign  7m  after: ^redesign-qual  done: 1969-03-05  ^crew-revise

After the fire the redesign was qualified and the crew-rating amendment revised for it; the board reported early, the
design froze 15 days after the date the spacecraft centre asked for, and the first crew-rated CSM was delivered 1968-08-01.

## Answers owed to the spacecraft centre
- [x] 1968-08-29 Written answer to spacecraft centre on the test configuration: frozen amendment configuration vs test configuration (1968-03-21)  owner: Dale Ferreira  done: 1968-08-27  ^sc-answers
- [x] 1969-01-31 Written answers still owed to spacecraft centre: configuration that flew 1967-11-09; uncrewed CM retrofit estimate (asked 1967-03-31)  owner: Dale Ferreira  ^sc-answers-2

## Flights 2 and 3 CSMs
- [x] CSM for flight 2, lunar-orbit flight without LM, delivered to spacecraft centre  ^csm-f2
- [x] CSM for flight 3, landing flight, delivered to spacecraft centre  ^csm-f3
