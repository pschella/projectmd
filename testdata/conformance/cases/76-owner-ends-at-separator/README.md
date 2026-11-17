Pins FORMAT §3 (rule 1: an inline `owner:` value ends at a two-space separator) and §3 (rule 2: an unknown
`key: value` after the first suffix part is skipped with W-KEYLIKE, and its value is not parsed). Owner `Bo`
ends before `effort:`. `effort: 6d` is skipped: `6d` is not a second estimate, so there is no E-SYNTAX.
Derivation: no `after:` and not started, so the item starts today (2027-03-01). 3w is 21 days inclusive:
03-01 .. 03-21. The `^b` id is kept. Line 5 gets W-KEYLIKE, the only diagnostic.
