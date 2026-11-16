# Lunar module plan: integrator, test articles and flight vehicle (private)

- Owner: Elena Brandt <elena.brandt@lm-co.example.net>
- Commitment: 1962-11-07 .. ^landing-lm

Assumption: lunar-orbit rendezvous (LOR) as adopted 1962-07-15. No vehicle has yet been built to land
on another body, so every stage is new design. Durations are real calendar estimates with new-design
allowance. The throttleable descent engine and the flight guidance computer set the critical path.

The descent stage team has its own plan in `teams/` (own owner); the ascent stage and guidance work are
kept here as plain tasks. The structures mass programme is `weight-watch.md` (Kofi Mensah). This file is
the integrator: it keeps the requirements, reviews, test articles and flight vehicle, and links the
team's dated deliverables one line per need.

What happened: the LM ran about 15% over its mass budget, which started the weight-watch list. Guidance
qualification and a descent-engine combustion fix took the margin, and the fire of 1967 added a
flammable-materials review; flight-ready was still reached within five days of its promise.

## Requirements and interfaces
- [x] 1962-11-07 Contract award, lunar module  done: 1962-11-07  ^award
- [x] 1963-01-31 LM requirements baseline: mission profile, mass budget, envelope, landing sites, abort  after: ^award  done: 1963-05-10  ^req
- [x] 1963-02-15 LM interface data to CM/SM stack, preliminary: envelope, attach points, mass estimate  2w  after: ^req  done: 1963-03-20  ^if-prelim

## Team deliverables
- [x] [Descent stage: preliminary design](teams/descent-stage.md#ds-pd)  by: 1963-08-30  ^ds-pd
- [x] [Descent engine: preliminary design and development plan](teams/descent-stage.md#de-plan)  by: 1963-08-30  ^de-plan
- [x] [Landing gear: concept and crush sizing](teams/descent-stage.md#lg-concept)  by: 1963-08-30  ^lg-concept
- [x] Ascent stage: preliminary design  by: 1963-08-30  ^as-pd
- [x] Ascent engine: preliminary design  by: 1963-08-30  ^ae-pd
- [x] Guidance and navigation: concept  by: 1963-08-30  ^gnc-concept
- [x] [Descent stage: detailed design](teams/descent-stage.md#dd-ds)  by: 1964-09-30  ^dd-ds
- [x] Ascent stage: detailed design  by: 1964-09-30  ^dd-as
- [x] [Landing gear: drop and crush test](teams/descent-stage.md#lg-test)  by: 1964-09-30  ^lg-test
- [x] Ascent engine: qualified  by: 1965-10-04  ^ae-qual
- [x] [Descent engine: qualified](teams/descent-stage.md#de-qual)  by: 1965-12-10  ^de-qual
- [x] Guidance flight computer and radar: qualified  by: 1966-07-26  done: 1966-08-10  ^gnc-qual
- [x] [Descent engine: combustion-instability fix](teams/descent-stage.md#de-fix)  by: 1966-12-14  done: 1966-12-20  ^de-fix

## Design reviews
- [x] 1963-08-30 Preliminary design review, LM  after: ^gnc-concept  after: ^ds-pd  after: ^as-pd  after: ^de-plan  after: ^ae-pd  after: ^lg-concept  done: 1963-12-12  ^pdr
- [x] 1964-09-30 Critical design review, LM  after: ^dd-ds  after: ^dd-as  after: ^lg-test  done: 1965-01-25  ^cdr

## Mass
- [x] [Mass recovery: structures programme, weight-watch list](weight-watch.md)  by: 1966-07-26  done: 1966-07-29  ^weight
- [x] 1965-06-30 Mass-recovery estimate and forecast issued to spacecraft centre  done: 1965-07-15  ^mass-est

## Test articles
- [x] Test article fabrication: flight-standard structure, tanks, plumbing, engineering-unit descent engine  36w  after: ^cdr  ^ta-fab
- [x] 1965-11-30 Engineering test article for integrated uncrewed test: qualified ascent engine, descent engine engineering unit, guidance engineering model  6w  after: ^ta-fab  after: ^ae-qual  done: 1965-12-10  ^ta-int
- [x] 1966-10-31 Qualified LM test article: qualified descent engine installed and re-checked  8w  after: ^ta-int  after: ^de-qual  ^ta-qual

## Flight vehicle
- [x] First flight LM build  52w  after: 1966-08-11  after: ^gnc-qual  after: ^weight  after: ^de-qual  after: ^ae-qual  after: ^ta-qual  done: 1967-10-15  ^fa-build
- [x] LM flammability materials review: cabin, wiring, insulation  8w  after: 1967-02-15  ^flam-rev
- [x] Flammable-material changes to the flight article  8w  after: ^fa-build  after: ^flam-rev  ^flam-chg
- [x] Guidance software fix: descent engine cut-off seen on the uncrewed flight (1968-01-22)  6w  after: 1968-01-22  done: 1968-06-10  ^sw-fix
- [x] Ground test campaign on flight article: thrust, landing, abort  40w  after: 1967-12-11  ^gt
- [x] Flight acceptance review  6w  after: ^gt  after: ^sw-fix  done: 1968-12-05  ^fa-accept
- [x] 1969-03-20 Lander crew-rating campaign on the flight article, for the spacecraft centre's need (by 1969-02-10)  after: ^fa-accept  done: 1969-03-20  ^lander-crew
- [x] 1969-10-30 Landing LM, flight article for the landing flight, accepted  done: 1969-10-30  ^landing-lm
