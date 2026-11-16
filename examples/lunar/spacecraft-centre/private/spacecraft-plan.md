# Spacecraft plan — lunar-orbit rendezvous (private)

- Owner: Henrik Osei <henrik.osei@sc.example.org>

Assumption: HQ adopted lunar-orbit rendezvous on 1962-07-15. The mothership (CM and SM) stays in
lunar orbit; a separate lunar module (LM, prime contract with lm-co) lands. Direct-ascent lander work
is cancelled; the `[-]` items are kept as record.
What happened: the direct-ascent plan was cancelled when LOR was adopted (the `[-]` items), crew rating was re-estimated after the
1967 fire, and the LM test article moved the uncrewed-ready and lander-ready dates.

## Architecture and requirements
- [-] Direct-ascent architecture study  3m  ^arch
- [x] LOR requirements: CM and SM docking, rendezvous, lunar-orbit return  3w  ^arch-lor
- [-] Requirements baseline: mass, envelope, heat shield, abort  1m  after: ^arch  ^req
- [-] 1962-04-14 Requirements baseline frozen (direct ascent)  after: ^req  ^req-da
- [x] 1962-09-14 Requirements baseline frozen (LOR)  after: ^arch-lor  done: 1962-09-20  ^req-frozen

## Prime contracts
- [ ] [Command and service module prime contract](../../csm-co/csm.md)  ^csm
- [x] [Interim interface and mass data, CM and SM](../../csm-co/csm.md#if-data)  by: 1962-12-14  done: 1962-12-10  ^csm-if
- [x] [Preliminary design baseline, CM and SM](../../csm-co/csm.md#pdb)  by: 1963-03-15  done: 1963-06-14  ^csm-pdb
- [x] [Detailed design data, CM and SM](../../csm-co/csm.md#dd-data)  by: 1964-01-31  done: 1964-04-22  ^csm-dd
- [x] [Integration articles, CM and SM](../../csm-co/csm.md#int-art)  by: 1965-05-31  done: 1965-06-10  ^csm-int
- [x] [Crew-rating amendment, CM and SM](../../csm-co/csm.md#crew-rating)  by: 1969-03-31  done: 1969-03-05  ^csm-crew
- [x] [Crew-rating amendment configuration freeze](../../csm-co/csm.md#amend-freeze)  by: 1968-06-30  done: 1968-04-15  ^csm-amend-freeze
- [x] [CM design freeze, redesign](../../csm-co/csm.md#design-freeze)  by: 1967-04-30  done: 1967-05-15  ^csm-freeze
- [x] [CM and SM qualified for uncrewed flight](../../csm-co/csm.md#qualified)  by: 1966-08-31  done: 1966-09-20  ^csm-qual
- [x] Select prime contractor, lunar lander  done: 1962-07-15  ^sel-lander
- [x] [Engineering test article for integrated uncrewed test](../../lm-co/lm.md#ta-int)  by: 1965-11-30  done: 1965-12-10  ^lm-ta-int
- [x] [First flight-ready LM](../../lm-co/lm.md#flight-ready)  by: 1968-11-30  done: 1968-12-05  ^lm-flight
- [x] [Flight-article crew-rating campaign, lander](../../lm-co/lm.md#lander-crew)  by: 1969-02-10  done: 1969-03-20  ^lm-crew
- [x] Heat-shield development, lunar-return speed  14m  after: ^req-frozen  ^heat

## Design
- [-] Preliminary design, direct-ascent spacecraft  5m  after: ^sel-lander  ^pdr-da
- [x] Preliminary design review, CM, SM and LM interfaces  2w  after: ^csm-pdb  done: 1963-12-12  ^pdr
- [x] Centre detailed design, CM, SM and LM interfaces  6m  after: ^pdr  ^dd
- [-] Detailed design, command module, service module and lander  10m  after: ^pdr-da  ^dd-da
- [-] 1963-11-30 Critical design review: command, service module and lander  after: ^dd-da  ^cdr-da
- [x] 1964-09-30 Critical design review: command and service module  2m  after: ^dd  after: ^csm-dd  done: 1964-10-20  ^cdr
- [x] Spacecraft interface spec: dimensions, mass, umbilicals, checkout connectors  2m  after: ^csm-if  done: 1964-02-10  ^if-work
- [x] 1963-03-31 Spacecraft interface spec delivered to launch centre  after: ^if-work  done: 1964-02-10  ^if-spec

## Build and test
- [-] Lander development and qualification test  14m  after: ^dd-da  ^qual-lander
- [-] Integrated uncrewed spacecraft test and acceptance  5m  after: ^qual-lander  ^uncrewed-acc-da
- [x] Integrated uncrewed test on integration articles  5m  after: ^csm-int  after: ^lm-ta-int  after: 1965-12-10  ^uncrewed-test
- [x] Flight acceptance, qualified CM and SM  2m  after: ^csm-qual  after: ^uncrewed-test  ^flight-acc
- [x] 1966-11-30 Uncrewed spacecraft ready for flight  after: ^flight-acc  ^uncrewed-ready

## Crew rating and lunar-orbit test
Risk: the CSM crew-rating amendment (CSM estimate about 1966-06-30) is not promised to us yet. Crew rating here
starts after flight acceptance and does not wait for it. The crewed test runs during the uncrewed flight review;
the review gates the first crewed flight (HQ 1967-12-31), not our test.
Re-plan: the LM engineering test article (needed for the integrated uncrewed test) moves to 1965-10-31, so the test
and flight acceptance move with it: uncrewed-ready 1965-09-30 -> 1966-05-31 (HQ need 1966-12-31 holds). The 36-month
lunar-orbit campaign starts on the same article; the LM flight-ready date (1968-11-30) gates lander-ready, one month
before HQ's 1968-12-31. Crew rating starts on the qualified CM and SM, not on flight acceptance (a bold re-plan, proposed
to HQ): crew-ready holds at 1967-06-30 with zero float, and the crewed test also waits for the CSM amendment (1966-09-30).
- [x] 1967-04-30 Fire investigation board report (external; CM design freeze waits for it)  done: 1967-04-05  ^board
- [x] Crew rating: life support, cabin materials, escape system  10m  after: ^csm-qual  after: ^csm-freeze  done: 1968-03-20  ^crew-rate
- [x] Uncrewed flight-data review, first lunar-distance flight  3m  after: 1967-11-10  ^flight-review
- [x] Crewed integrated test, command and service module  6m  after: 1968-03-21  done: 1968-09-18  ^crew-test
- [x] Lander crew rating and lunar-orbit test campaign  38m  after: 1965-12-11  done: 1969-03-20  ^lander-crew
- [x] Lunar lander ready for crewed lunar-orbit flight  after: ^lander-crew  after: ^lm-flight  after: ^lm-crew  done: 1969-03-28  ^lander-ready
- [x] Command and service module ready for lunar-orbit flight without LM  after: ^crew-test  done: 1969-03-31  ^lo-csm

## Rendezvous practice
Two-crew Earth-orbit flights to practise rendezvous, docking, long flights and spacewalks before the
lunar-orbit flight. They fly on the programme's own practice vehicle, so they do not wait for the crew-rated CSM (1967-06-30).
- [x] [Rendezvous practice programme](../rendezvous/rendezvous.md)  by: 1968-06-30  done: 1966-11-15  ^rdv
