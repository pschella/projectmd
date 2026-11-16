# projectmd grammar (normative)

This is the precise, implementable definition of the projectmd syntax. [FORMAT.md](FORMAT.md)
explains the meaning; this file decides how text is read. Where they differ, this file wins
for syntax and FORMAT.md wins for semantics.

Notation: regular expressions are RE2 syntax (Go `regexp`), matched case-sensitively unless
marked `(?i)`. "Whitespace" means space or tab. A tab counts as 4 columns for indentation.

---

## 1 Document structure (line level)

A file is UTF-8 text, split into lines on `\n` (a trailing `\r` is removed). The parser walks
the lines top to bottom and classifies each, in this priority order:

| # | Line class | Recognised by | Effect |
|---|---|---|---|
| 1 | Fence | `^\s{0,3}(```+\|~~~+)` | Toggles a code block; lines inside are ignored until a closing fence of the same character and at least the same length. |
| 2 | HTML comment | a line starting `<!--` | Ignored up to and including the line containing `-->`. |
| 3 | Heading | `^(#{1,6})\s+(.*?)\s*#*\s*$` | §3. |
| 4 | Reference definition | `^\s{0,3}\[([^\]]+)\]:\s+(\S+)(?:\s+"([^"]*)")?\s*$` | Records `label → (target, title)`. Labels are case-insensitive. Applies to the whole file. |
| 5 | List item | `^(\s*)([-*+])\s+(.*)$` | §2. Indent = width of group 1 in columns. |
| 6 | Other | anything else | Prose. Ignored. A blank line does not end a list. |

## 2 List items

### 2.1 Nesting

Each list item has an **indent**. Its parent is the nearest preceding list item with a smaller
indent, as long as no heading lies between them; otherwise its parent is the current heading
(§3) or, before any heading, the project itself.

### 2.2 Classification

Let `body` be group 3 of the list-item regex. Apply in order:

1. **Checkbox.** If `body` matches `^\[( |x|X|-)\]\s+(.*)$`, the item has a checkbox:
   ` ` = open, `x`/`X` = done, `-` = cancelled. `body` becomes group 2.
2. **Leading date** (only with a checkbox). If `body` now starts with a range (§4.2: `D..D` or `D .. D`) followed by
   whitespace and more text, that range is the item's **leading range**: it is removed, and the
   item is a planning item of kind `task` with that declared range. Otherwise, if `body` starts
   with a date expression (§5.1) followed by whitespace and more text, that date is the item's
   **leading date** and is removed from `body`. (Only on items, not headings.)
3. **Planning item** = has a checkbox. Without one, a leading date is plain text and the item is prose (`W-KEYLIKE` when it has a leading date and a suffix token: `^id`, known key, duration). It is parsed per §4 and becomes a node.
4. **Property bullet** = not a planning item, its parent is a planning item, a heading, or the
   project, and `body` matches `^([A-Za-z][A-Za-z0-9-]*):\s+(.+)$` where group 1 is a known key
   (§4.4, case-insensitive). The value (group 2) is parsed as one value (§4.5) and added to the
   parent's keys. (The project header is a special case, §6.1.)
5. Otherwise the item is **prose**: ignored, and its children are ignored too.

A planning item's own children may be planning items, property bullets or prose.

### 2.3 Node kinds

| Kind | Rule |
|---|---|
| `milestone` | planning item with a leading date |
| `subproject` | planning item without leading date whose **entire title** is one link (§4.2) whose resolved target has no `#fragment` and whose path ends in `.md` (case-insensitive; query string allowed for URLs) |
| `deliverable` | planning item without leading date whose **entire title** is one link whose target **has** a `#fragment` (the id of a milestone in another file) |

A **dated deliverable** is a `milestone` (leading date) whose entire title is such a link: it
keeps kind `milestone` (so it is exposed unless `#private`), stores the link's text as title and
the link in `link`. Its declared date is the promise; its forecast end is the linked milestone's
date, and `E-CONFLICT` is reported when that is later than the declared date. Without its own
`id` it takes the link's fragment when that id is free in the file. A milestone with a duration
(`2027-06-30 Qualify  26w`) is scheduled as work ending by its date (FORMAT §7).
| `task` | any other planning item |
| `heading` | §3 |

## 3 Headings

- The **first H1** in the file is the project title. Its text (after suffix parsing, §4) is the
  project name; a suffix on the H1 is allowed and applies to the project.
- Every other heading (any level, including later H1s) is a **heading node**. Its parent is the
  nearest preceding heading node of a lower level, else the project.
- Headings carry **no dates**: they only group. A date, range or duration at the end of a heading
  is ignored (use an item with children, `- [ ] Phase 1  2026-11 .. 2027-06`).
- A heading node with no planning-item descendants is dropped after parsing (e.g. `## Risks`).

## 4 Title and suffix

### 4.1 Atomisation

The text (item body after checkbox/leading date, or heading text) is split left to right into
**atoms**. At each position, skip whitespace, then take the first match:

| Atom | Pattern (at current position) |
|---|---|
| code | `` `[^`]*` `` (unclosed: rest of text) |
| inline link | `\[([^\]]*)\]\(\s*(\S+?)(?:\s+"([^"]*)")?\s*\)` |
| reference link | `\[([^\]]*)\]\[([^\]]*)\]` — empty label means the text is the label |
| email | `<[^<>\s]+@[^<>\s]+>` |
| word | `\S+` up to the next whitespace (a word may contain `[`, `(` etc. if no link matched) |

Each atom remembers its start/end offset in the text, so the title can be cut verbatim.

### 4.2 Atom tests

| Test | Definition |
|---|---|
| tag | word matching `^#[A-Za-z][A-Za-z0-9_-]*$` (so `#3` is not a tag) |
| id | word matching `^\^[A-Za-z0-9][A-Za-z0-9_-]*$` |
| duration | word matching `^[1-9][0-9]*[dwm]$` |
| date | word that is a date expression (§5.1) |
| range | either one word `D..D`, or three atoms `D` `..` `D`, where D are date expressions |
| key | word matching `(?i)^([a-z][a-z0-9-]*):$` whose name is a known key (§4.4) |
| link | inline link or reference link atom; a reference link must resolve via a definition (§1 #4), else it is not a link (it is a word-like text atom) |

### 4.3 Suffix extraction (right to left)

Let `A[0..n)` be the atoms and `end = n`. Repeat:

1. If `end - 1 ≤ 0`, stop (the title must keep at least one atom).
2. Let `a = A[end-1]`.
3. **Owner.** If `a` is not a tag, id, duration, date, range part or known key, scan
   `k` leftwards from `end-2` to 1 over atoms, stopping at the first atom that is a tag, id,
   duration, date, range part (`..`, `D..D`) or known key. If `A[k]` is the key `owner`, the
   value is the text of `A[k+1..end)` verbatim (any atoms; a person or one word keeps that
   kind); record `owner → value`, set `end = k`, continue.
4. **Key + value.** Otherwise find the smallest `k < end-1` such that `A[k]` is a key and `A[k+1..end)`
   is a valid value (§4.5) — trying `k = end-2`, then `end-4` (range), then for an email atom
   `a`, every `k` scanning left over plain words. If found and `k ≥ 1`, record `key → value`,
   set `end = k`, continue.
5. **Range.** If `A[end-3..end)` (or `a` alone) is a range and that leaves ≥ 1 atom, record it,
   move `end`, continue.
6. **Bare token.** If `a` is a tag, id, duration or date, record it, `end = end-1`, continue.
7. Stop.

The **title** is the text from the start of `A[0]` to the end of `A[end-1]`, verbatim. For a `subproject` or `deliverable` node (§2.3) the stored title is the link's text (`[Powertrain](…)` → `Powertrain`); the link itself is kept in the node's `link`.
Suffix elements are stored in their original left-to-right order. Code atoms are never tokens.

Examples (known keys: owner, after, by, done):

| Text | Title | Suffix |
|---|---|---|
| `Fix issue: retry logic  2w` | `Fix issue: retry logic` | dur 2w |
| `Test 12m cable harness  3d  owner: @jan` | `Test 12m cable harness` | dur 3d, owner @jan |
| `Review #3 comments  1w  ^review` | `Review #3 comments` | dur 1w, id review |
| `Review  owner: Henrik Osei  3d  ^rev` | `Review` | owner `Henrik Osei`, dur 3d, id rev |
| `Test  by: Friday  ^t` | `Test  by: Friday` | id t (the bad key stays in the title with `W-KEYLIKE`; tokens to its right are kept) |
| `Tooling release  #private ^tr  owner: Marco Rossi <m@example.com>` | `Tooling release` | tag private, id tr, owner person |
| `Seat concept  2027-Q1 .. 2027-Q2` | `Seat concept` | range |
| `2w` | `2w` | (none — title may not be empty) |
| `Ship it  after: ^a  after: ^b` | `Ship it` | after ^a, after ^b |
| `` Explain `after: ^x` syntax `` | `` Explain `after: ^x` syntax `` | (none) |

### 4.4 Known keys

Known keys: `owner`, `after`, `by`, `done` (any date expression, §5.1; its end is the done date; only meaningful on a done item).
Any other `word:` is text. Repeatable: `after`. Repeating any other key on one node is an error
`E-SYNTAX` (the last value wins for tooling that continues).

### 4.5 Values

A value is exactly one of:

| Value | Atoms |
|---|---|
| person | one or more word atoms followed by one email atom: `Marco Rossi <m@example.com>` |
| link | one link atom |
| range | a range |
| word | one word atom (`@name`, dates, ids, free words) |

A key whose following atoms do not form a value is not a key (it stays in the title).

Each key also **type-checks** its value; a value of the wrong type means the key is not a
key (it stays in the title):

| Key | Accepted values |
|---|---|
| `owner` | any text (§4.3 step 3), e.g. `Jan Doe`, `Jan <j@x.org>`, `@jan` |
| `after` | word that is an id (`^x`) or a date expression |
| `by` | date expression, range or id (`^x`) |
| `done` | date expression or range |

So `by: Friday` and cross-file forms (`after: file.md#id`,
`by: [x](y.md#z)`) stay in the title; `W-KEYLIKE` is reported when a known key's value does not fit.

## 5 Lexical forms

### 5.1 Dates

| Form | Regex | Expands to |
|---|---|---|
| exact | `^(\d{4})-(\d{2})-(\d{2})$` | that day (must be a real calendar date) |
| month | `^(\d{4})-(\d{2})$` | first..last day of month |
| quarter | `^(\d{4})-Q([1-4])$` | first..last day of quarter |

A string matching the shape but invalid (`2027-02-30`, `2027-13`) is a date atom
with error `E-DATE`. A range `A .. B` expands to start(A)..end(B); end before start is `E-DATE`.

### 5.2 Durations

Durations are **real calendar time** (no working days, no abstract effort units):
`Nd` = N days, `Nw` = 7N days, `Nm` = N calendar months. Month arithmetic adds months to the
start date and clamps the day to the target month's length (2027-01-31 + `1m` = 2027-02-28;
2027-11-15 + `3m` = 2028-02-15).

### 5.3 Names

Ids and tags: see §4.2. People: `Name <email>` or any text.

## 6 Project header

### 6.1 Property-list header

The **header** is the run of consecutive top-level list items that starts with the first list
item after the H1 line (only blank lines in between) and continues while each item matches
`^([A-Za-z][A-Za-z0-9 _-]*):\s*(.*)$` and has no checkbox. The run ends at the first item that
does not match (which, with everything after it, is parsed normally), or at any non-blank,
non-list line. A todo list directly after the header is therefore fine. Header keys are **any** key
(case-insensitive, internal spaces allowed: `Start date`); values are raw strings, interpreted
for core keys:

| Key | Interpretation |
|---|---|
| `Owner` | a person (`Name <email>`) or any text |
| `Commitment` | date expression or range; or `^id` / `DATE .. ^id`: a local item with a declared end (leading date or dated deliverable), whose date (range end) is the Commitment; unknown or undated id is E-REF-LOCAL |
| `Done` | exact date: the project was delivered that day |

Any other key (`Stakeholders`, ...) is kept in `header` but has no meaning in the core.

If the first item does not match, there is no property header.

There is no YAML front matter; the property list is the only header form.

## 7 Interface and contracts

There is no interface file and no contracts file. A project's interface is derived from its own
file (FORMAT §8): header `Owner`, `Commitment`, `Done`, and the milestones not tagged
`#private`. A contract state (FORMAT §9) is computed from the parent's `by:` and the child's
file at read time. A **front file** (FORMAT §8.1) is an ordinary project file and uses no
grammar beyond §1-§6.

## 8 Parse output (data model)

```text
Project  { path, title, owner, commitment, done, header{k:v},
           suffix (from H1), nodes[] (top level), refdefs{label:(target,title)}, diagnostics[] }
Node     { kind (task|milestone|subproject|deliverable|heading), line, checkbox (none|open|done|cancelled),
           title, date (leading or suffix date), range, duration, tags[], id, done,
           keys{ name: [values] }, link (subproject/deliverable target + link title), children[] }
Value    { kind (word|person|link|range), text, target?, title?, name?, email? }
Diagnostic { severity (E|W), code, line, message }
```

Diagnostic codes produced by the parser itself: `E-SYNTAX`, `E-DATE`, `E-DUP-ID` (an id
defined twice in a file), `E-REF-LOCAL` (an `^id` value with no matching id in the file),
`W-NO-OWNER`, `W-KEYLIKE` (also for a list item with a known key, or a leading date plus a suffix token, and no checkbox). Cross-file checks (`E-REF-PUBLIC`, `E-REF-MISSING`, `E-CYCLE`) and contract states
(`W-OVERDUE`, `W-DISPUTED`, `W-UNKNOWN`, `I-REQUESTED`) need the graph loader; `E-CONFLICT` needs
scheduling. All of these are outside the parser.
