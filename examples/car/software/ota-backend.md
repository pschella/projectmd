# Over-the-air update backend

- Owner: @amara
- Commitment: 2027-Q4
- State: planned

Cloud backend that signs, stores and rolls out vehicle software updates. Coarse plan: quarters
only until the cloud provider is chosen.

## Milestones

- [ ] 2027-Q1 Reference architecture agreed  ^ota-arch
- [ ] 2027-Q3 Signed-update pipeline live  #public  ^signed-pipeline
- [ ] 2027-Q4 Fleet rollout backend ready  #public  ^fleet-ready

## Work

- [ ] Cloud provider shortlist  2027-Q1
- [ ] Key management and signing service  2027-Q2 .. 2027-Q3  after: ^ota-arch
- [ ] Campaign manager UI  2027-H2  after: ^signed-pipeline
- [ ] Backend cybersecurity review  4w  after: ^signed-pipeline
- [ ] Vehicle client integration  2027-Q4  after: ^ota-arch
