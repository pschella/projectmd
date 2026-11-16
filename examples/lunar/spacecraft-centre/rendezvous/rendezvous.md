# Rendezvous practice programme

- Owner: Grace Adeyemi <grace.adeyemi@sc.example.org>
- Commitment: 1963-11-01 .. 1967-09-30
- Done: 1966-11-15

Two-crew Earth-orbit flights to practise rendezvous, docking, spacewalks and long-duration flight before the lunar-orbit flights. The flights fly on our own practice spacecraft and target vehicles, built by our own contractor, so they do not wait for the crew-rated CSM (1967-06-30). All ten crewed flights finish by 1967-09-30, nine months before the 1968-06-30 need date.

Re-planned 1963-11-01: the uncrewed test moves from 1964-04 to 1965-02-28 and the first crewed flight from 1965-03 to 1965-06-30. A new practice vehicle cannot be designed, built and tested in the four months after contract award (1963-12-31).

## Flights

- [x] 1965-02-28 Flight 0, uncrewed test: launch, orbit, recovery of the practice vehicle  after: ^pv-test-deliv  done: 1965-03-23  ^f0
- [x] 1965-07-31 Flight 1, first crewed: 3 days, systems and crew checkout  after: ^f0  after: ^cv1-deliv  after: ^train-sys  done: 1965-08-21  ^f1
- [x] 1965-10-31 Flight 2, first rendezvous with target vehicle 1: 3 days  after: ^f1  after: ^tv1-deliv  after: ^train-rdv  done: 1965-12-04  ^f2
- [x] 1966-03-31 Flight 3, first docking with target vehicle 2: 3 days  after: ^f2  after: ^tv2-deliv  done: 1966-03-16  ^f3
- [x] 1966-04-29 Flight 4, first spacewalk (EVA): 3 days  after: ^f3  after: ^train-eva  done: 1966-06-03  ^f4
- [x] 1966-07-20 Flight 5, first long-duration flight: 14 days  after: ^f4  after: ^cv2-deliv  after: ^train-long  done: 1966-07-21  ^f5
- [x] 1966-09-30 Flight 6, rendezvous and docking with active target vehicle 3: 3 days  after: ^f5  after: ^tv3-deliv  done: 1966-09-14  ^f6
- [x] 1966-12-15 Flight 7, second spacewalk, long EVA: 3 days  after: ^f6  ^f7
- [x] 1967-03-15 Flight 8, second long-duration flight: 14 days  after: ^f7  after: ^cv3-deliv  ^f8
- [x] 1967-06-15 Flight 9, docking with manual takeover and abort test: 3 days  after: ^f8  after: ^tv4-deliv  ^f9
- [x] 1967-09-15 Flight 10, final full-profile practice: rendezvous, docking, EVA: 8 days  after: ^f9  done: 1966-11-15  ^f10

## Work

### Practice spacecraft (own contractor)
- [x] Contract award, practice spacecraft  2m  done: 1963-12-01  ^pv-contract
- [x] Practice spacecraft design  8m  after: ^pv-contract  ^pv-design
- [x] Test vehicle build and ground test  5m  after: ^pv-design  ^pv-test-build
- [x] 1965-01-31 Test vehicle delivered  after: ^pv-test-build  done: 1965-01-20  ^pv-test-deliv
- [x] Crew vehicle 1 build and acceptance  8m  after: ^pv-design  ^cv1-build
- [x] 1965-05-15 Crew vehicle 1 delivered  after: ^cv1-build  ^cv1-deliv
- [x] Crew vehicle 2 build and acceptance  6m  after: ^cv1-deliv  ^cv2-build
- [x] 1966-05-15 Crew vehicle 2 delivered  after: ^cv2-build  ^cv2-deliv
- [x] Crew vehicle 3 build and acceptance  6m  after: ^cv2-deliv  after: 1966-05-16  ^cv3-build
- [x] 1966-12-15 Crew vehicle 3 delivered  after: ^cv3-build  ^cv3-deliv

### Target vehicles
- [x] Contract award, target vehicles  done: 1964-02-14  ^tv-contract
- [x] Target vehicle design  5m  after: ^tv-contract  ^tv-design
- [x] Target vehicle 1 build (rendezvous target)  6m  after: ^tv-design  ^tv1-build
- [x] 1965-09-30 Target vehicle 1 delivered  after: ^tv1-build  ^tv1-deliv
- [x] Target vehicle 2 build, docking system fitted  6m  after: ^tv1-build  ^tv2-build
- [x] 1966-03-31 Target vehicle 2 delivered  after: ^tv2-build  ^tv2-deliv
- [x] Target vehicle 3 build, powered active target  6m  after: ^tv2-build  after: 1965-07-16  ^tv3-build
- [x] 1966-09-30 Target vehicle 3 delivered  after: ^tv3-build  ^tv3-deliv
- [x] Target vehicle 4 build, abort-test variant  6m  after: ^tv3-build  after: 1966-01-17  ^tv4-build
- [x] 1967-05-31 Target vehicle 4 delivered  after: ^tv4-build  ^tv4-deliv

### Crew selection and training
- [x] Crew selection, two crews  3m  ^crew-sel
- [x] Practice vehicle simulator  5m  after: ^pv-design  ^sim
- [x] Procedures and systems training  6m  after: ^sim  after: ^crew-sel  ^train-sys
- [x] Rendezvous and docking training  3m  after: ^train-sys  ^train-rdv
- [x] EVA suit and water-tank training  4m  after: ^train-sys  ^train-eva
- [x] Long-duration life-support training  3m  after: ^train-sys  ^train-long
