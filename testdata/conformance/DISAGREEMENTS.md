# Disagreements between the golden cases and the reference tool

None open. Every golden case is active (`SKIPPED` in `parser/pmd-core/tests/golden.rs` is empty).

The entries and rulings below are kept as history.

## Entries (history)

- **09-form-by**: slack. Derivation: `Report  by: 2027-06-30` forecasts 06-30, so slack 0 (FORMAT §7.6:
  declared end minus forecast end). Tool: slack null, because it only computes slack when an estimate or
  a predecessor supplies an early end. Spec or tool?
- **10-form-range**: slack. Derivation: range end 03-31 equals the forecast, slack 0 (§7.6, §7.1 declares
  range ends). Tool: null (same cause as 09).
- **26-declared-end-vs-children-conflict**: Phase B end. Derivation: §7.3 rule 3c gives
  max(latest child end, own declared end) and `by:` is a declared end (§7.1), so 05-31. Tool: 04-14,
  because its children branch only takes a leading date or range end, not `by:`.
- **27-overdue-not-marked-done**: slack. Derivation: -28 (02-01 minus 03-01, §7.6). Tool: null (same
  cause as 09). The conflict itself agrees.
- **39-conflict-commitment**: line of the Commitment conflict. Derivation: §7.7 rule 2 reports it once
  against the file's Commitment, i.e. line 4. Tool: line 1 (the H1). §11 makes lines normative and
  the spec does not say which line; a repeated header key is reported on its own line (case 64), so
  4 is the consistent choice. Spec gap.
- **44-link-after-keeps-promise**: start of a sub-project line to a date-only Commitment. §7.5 says start
  is the child's Commitment start, moved later by `after:`. A date-only Commitment has no start, so the
  derivation is 04-22 (after: 04-21 plus one day). Tool: 05-31 (treats the date as a point). Spec gap.
- **66-w-keylike-no-checkbox**: `- 2027-06-30 Foo` with no checkbox. FORMAT §11 and GRAMMAR §2 item 3 both say
  a list item with a leading date and no checkbox gets W-KEYLIKE. Tool: emits it only when the item also has
  `^id`, a key or a duration (parser/pmd-core/src/parse.rs, the "leading date plus a suffix token" branch).
  Tool narrower than the spec.

## Rulings (2026-11-16)

- **09-form-by**: tool changes. Slack is defined for every node with a declared end (FORMAT §7.6), so a `by:`-only item has slack 0.
- **10-form-range**: tool changes. Same rule as 09; range end 03-31 gives slack 0.
- **26-declared-end-vs-children-conflict**: text changes. A node's end is its latest sub-item end; its own declared end gives only slack and conflict (§7.3 3c). Phase B ends 04-14, not 05-31.
- **27-overdue-not-marked-done**: tool changes. Slack is -28 (§7.6); the conflict already agrees.
- **39-conflict-commitment**: text changes. A project's Commitment conflict is reported on its H1 line, line 1 (§7.7 rule 2, §11). The tool already agrees.
- **44-link-after-keeps-promise**: tool changes. A date-only Commitment gives no start, so the start is the `after:` start, 04-22; the end stays 05-31, the promise (§7.5).
- **66-w-keylike-no-checkbox**: text changes. W-KEYLIKE needs a leading date plus a planning suffix (§11, GRAMMAR §2 item 3); a bare dated note is prose. The tool already agrees.
