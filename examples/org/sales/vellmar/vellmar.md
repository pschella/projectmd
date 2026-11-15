# Vellmar passenger information system

- Owner: Elena Fischer <elena.fischer@example.com>
- Commitment: 2028-06-30
- State: contracted
- ID: pis.example.com/sales/vellmar

Station displays and rider apps for the Vellmar Transit Authority (customer contract of
2026-10-01). Pilot at 3 stations, then full rollout.

## Customer milestones

- [x] 2026-10-01 Contract signed with Vellmar Transit Authority  #private  done: 2026-10-01  ^signed
- [ ] 2027-09-30 Pilot live at 3 stations  ^pilot-live
- [ ] 2028-06-30 Full rollout complete  ^rollout-complete

## Deliveries from the business units

- [ ] [Pilot displays](../../units/hardware/hardware.public.md#pilot-displays)  by: 2027-07-31
- [ ] [Pilot software ready](../../units/software/software.public.md#pilot-ready)  by: 2027-08-31
- [ ] [Hardware](../../units/hardware/hardware.public.md)  by: 2028-03-31
- [ ] [Software](../../units/software/software.public.md)  by: 2028-04-30

## Own work

- [ ] Pilot installation at 3 stations  4w  after: [Pilot displays][pilot-displays]  after: [Pilot software ready][pilot-ready]  ^pilot-install
- [ ] Rollout installation  8w  after: [Hardware][hardware]  after: [Software][software]  ^rollout-install

## Notes

- The rollout installation is forecast to end 2028-06-25, five days before the Commitment.
  That is the slack that absorbs Hardware's delay.

[pilot-displays]: ../../units/hardware/hardware.public.md#pilot-displays
[pilot-ready]: ../../units/software/software.public.md#pilot-ready
[hardware]: ../../units/hardware/hardware.public.md
[software]: ../../units/software/software.public.md
