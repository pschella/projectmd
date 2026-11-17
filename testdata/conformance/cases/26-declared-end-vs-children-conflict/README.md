Pins FORMAT §7.3 rule 3c, §7.6 and §7.7 rules 1-2 as ruled (ruling B: the text follows the tool).
Phase A (by 04-10): A1 is 2w after 04-01, 04-01 .. 04-14; A2 is 1w after ^a1, 04-15 .. 04-21. A node with
live sub-items ends at the latest sub-item end, so Phase A ends 04-21. Its declared `by:` gives only slack,
04-10 − 04-21 = −11. A1 and A2 each exceed the container's 04-10 and are reported (E-CONFLICT on lines 6
and 7); Phase A's own conflict is not repeated because its children carry it.
Phase B (by 05-31): B1 is 2w after 04-01, 04-01 .. 04-14, so Phase B ends 04-14. Its `by:` gives slack
05-31 − 04-14 = +47 and no conflict.
