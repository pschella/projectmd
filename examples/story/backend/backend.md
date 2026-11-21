# Backend team

- Owner: Sam Okafor <sam@pantry.example.com>
- Commitment: ^capacity-live

## Work
- [x] Slot data model  owner: Lena Fischer  done: 2027-06-11  ^model
- [ ] Slot booking endpoint  4w  after: 2027-06-14  owner: Lena Fischer  ^booking
  Done when the endpoint passes the contract tests the app team wrote.
- [ ] Load test booking at 5x peak  1w  after: ^booking  owner: Lena Fischer  ^load
- [ ] 2027-08-02 Booking API ready  after: ^load  ^slots-api
- [ ] Courier capacity feed (v3 API)  9w  after: 2027-06-21  owner: Arjun Mehta  ^capacity
  The courier partner retires its v2 API in July: rebuilt against v3 (moved from 3w on 2027-07-05).
- [ ] 2027-08-30 Live courier capacity  after: ^capacity  ^capacity-live
