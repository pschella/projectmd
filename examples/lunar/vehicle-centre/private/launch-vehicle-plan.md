# Launch-vehicle plan (private)

- Owner: Ingrid Hollander <ingrid.hollander@lv.example.org>
- Commitment: 1961-09-15 .. 1967-09-15

## Requirements and configuration
- [x] Vehicle requirements baseline (payload, orbit, heavy-lift class)  1961-12-01 .. 1962-02-23  done: 1962-02-23  ^reqs
- [x] Stage and engine trade study (propellants, diameters, staging)  1962-02-24 .. 1962-06-28  after: ^reqs  done: 1962-06-28  ^trade
- [x] 1962-06-30 Vehicle interface spec issued (height, diameter, mass per configuration; umbilical and pad points)  after: ^trade  done: 1962-06-28  ^if-spec
- [x] Payload interface definition (envelope, adapter, mass limit)  1962-02-24 .. 1962-04-20  after: ^reqs  done: 1962-04-20  ^payload-work
- [x] 1962-05-14 Payload interface frozen  after: ^payload-work  done: 1962-05-03  ^payload-if
- [x] 1962-08-31 Payload interface and mass baseline revised for lunar-orbit rendezvous (CSM and LM)  done: 1962-08-28  ^payload-lor
- [x] Vehicle configuration freeze  1962-06-29 .. 1962-07-12  after: ^trade  done: 1962-07-12  ^config-freeze
- [x] 1962-09-20 Launch-centre interface control (pad, umbilicals, transport)  after: ^config-freeze  done: 1962-09-18  ^pad-if

## First stage and its engine
- [x] First-stage engine (E1) development  80w  after: ^config-freeze  ^e1-dev
- [x] 1963-11-30 E1 design freeze  done: 1963-11-01  ^e1-freeze
- [x] Single-engine test stand (T1) design and build  60w  after: ^config-freeze  ^t1
- [x] E1 hot-fire on T1  6w  after: ^t1  after: ^e1-freeze  done: 1964-01-28  ^e1-test
- [x] 1964-01-31 E1 first hot-fire done  done: 1964-01-28  ^e1-hotfire
- [x] First-stage (S1) design  40w  after: ^config-freeze  done: 1963-12-15  ^s1-design
- [x] S1 flight-standard hardware built  72w  after: ^s1-design  ^s1-hw
- [x] S1 flight stage static-fired by its builder  after: ^s1-hw  done: 1965-02-20  ^s1-builder-fire
- [-] Stage test stand (T3) design and build  110w  after: ^config-freeze  ^t3
- [-] S1 full-length static firing on T3  8w  after: ^s1-hw  after: ^t3  ^s1-test
- [-] 1964-12-31 First-stage full-length static firing done  ^s1-static

## Second and third stages
- [x] Second-stage engine (E2, hydrogen) development  96w  after: ^config-freeze  ^e2-dev
- [x] E2 design freeze  2w  after: ^e2-dev  ^e2-freeze
- [x] Hydrogen test stand (T2) design and build  96w  after: ^config-freeze  ^t2
- [x] E2 hot-fire on T2  8w  after: ^t2  after: ^e2-freeze  done: 1964-05-20  ^e2-test
- [x] 1964-06-30 Second-stage engine hot-fire done  done: 1964-05-20  ^e2-hotfire
- [x] 1964-10-30 Flight E2 engines delivered to S-II contractor  16w  after: ^e2-hotfire  done: 1964-11-20  ^e2-flight
- [x] 1963-04-18 S-II interface definition (interstage, aft skirt, propellant lines)  after: ^config-freeze  done: 1963-04-15  ^s2-if
- [x] S-II stage contract  after: ^config-freeze  by: 1966-06-30  done: 1966-08-15  ^s-ii
- [x] E3 restart derivative of E2  24w  after: ^e2-freeze  ^e3
- [x] Third-stage (S3) design and hardware  100w  after: ^s2-if  ^s3-hw
- [-] S3 static test on T2  6w  after: ^s3-hw  after: ^e3  ^s3-test

## Instrument unit
- [x] Instrument unit (IU) design  36w  after: ^config-freeze  ^iu-design
- [x] IU qualification testing  40w  after: ^iu-design  done: 1964-10-12  ^iu-test
- [x] 1964-09-30 Instrument unit qualified  done: 1964-10-12  ^iu-qual

## Integration and readiness
- [x] All-up vehicle integration (S1, S2, S3, IU)  20w  after: ^s1-hw  after: ^s-ii  after: ^s3-hw  after: ^e3  after: ^e1-test  after: ^e2-flight  after: ^iu-qual  after: 1966-10-03  ^integ
- Risk (all-up testing, 1963-11-01): no stage is fired at the launch site or flown on its own before the first full-vehicle flight. The first test of stacked stages is the integration above; engine hot-fires (E1 on T1, E2 on T2) and the E3 restart derivative are still tested first. Exception recorded 1965-03-31: the S1 flight stage was static-fired by its builder (`^s1-builder-fire`), a builder's test, not a launch-site firing.
- [x] Ground-support checkout at launch centre  20w  after: ^integ  after: ^pad-if  ^checkout
- [x] Flight-readiness review  4w  after: ^checkout  ^review
- [x] 1967-09-15 Uncrewed test vehicle ready at launch centre  after: ^review  done: 1967-09-05  ^vehicle-ready
