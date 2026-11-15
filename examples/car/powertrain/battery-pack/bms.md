# Battery management system (BMS)

- Owner: Ruben Okafor <ruben.okafor@example.com>
- Stakeholders: Priya Nair <priya.nair@example.com>, Jan de Boer <jan.deboer@example.com>
- Commitment: 2027-09-15
- State: in-development
- ID: cars.example.com/platform/bms

Battery management hardware and software. Owned by the electronics team, not the pack team, so
this file records only the handover points the pack depends on. Detail lives in the electronics
backlog.

## Milestones

- [x] 2026-11-02 BMS architecture approved  ^arch
- [ ] 2027-03-31 State-of-charge model v1  ^soc-v1
- [ ] 2027-06-30 BMS hardware release  #public  ^hw-release
- [ ] 2027-09-15 BMS software release  #public  ^sw-release
  - approver: @priya

## Work

- [ ] Cell balancing algorithm  6w  after: ^soc-v1  owner: @ruben
- [ ] Isolation monitoring  4w  after: ^hw-release
- [ ] Charger interface  after: [Charger interface spec][cis]
- [ ] Functional safety case  2027-04-01 .. 2027-09-15  owner: @ruben
- [ ] Pack design release check  after: [Pack design release](battery-pack.md#pack-design-release)
- [ ] Field diagnostic logging  2027-07-01 .. 2027-09-15  ticket: ELEC-87
- [-] Second BMS supplier evaluation  dropped

The electronics team reports to a different board. Changes to the handover points above go
through the pack team first.

[cis]: https://charging.example.net/specs/charger-interface-v2.md#bms
