# projectmd grammar (normative)

This is the precise, implementable definition of the projectmd syntax (format 0.9, beta).
[FORMAT.md](FORMAT.md) explains the meaning; this file decides how text is read. Where they
differ, this file wins for syntax and FORMAT.md wins for semantics.

Notation: regular expressions are RE2 syntax (Go `regexp`), matched case-sensitively unless
marked `(?i)`. "Whitespace" means space or tab. A tab counts as 4 columns for indentation.

---

## 1 Document structure (line level)

A file is UTF-8 text, split into lines on `\n` (a trailing `\r` is removed). The parser walks
the lines top to bottom and classifies each, in this priority order:

| # | Line class | Recognised by | Effect |
|---|---|---|---|
| 1 | Fence | `^\s{0,3}(```+\|~~~+)` | Toggles a fenced code; lines inside are ignored until a closing fence of the same character and at least the same length. |
| 2 | HTML comment | a line whose first non-whitespace text is `<!--` | Ignored up to and including the line containing `-->`. |
| 3 | Heading | `^(#{1,6})\s+(.*?)(?:\s+#+)?\s*$` | §3. Only a closing run of `#` preceded by whitespace is stripped. |
| 4 | Reference definition | `^\s{0,3}\[([^\]]+)\]:\s+(\S+)(?:\s+"([^"]*)")?\s*$` | Records `label → (target, title)`. Labels are case-insensitive. Applies to the whole file. |
| 5 | List item | `^(\s*)([-*+])\s+(.*)$` | §2. Indent = width of group 1 in columns. |
| 6 | Other | anything else | Prose. Ignored, except as a description (§2.4). A blank line does not end a list. |

An ordered-list line (`1. text`) is class 6.

## 2 List items

### 2.1 Nesting

Each list item has an **indent**. Its parent is the nearest preceding list item with a smaller
indent, as long as no heading lies between them; otherwise its parent is the current heading
(§3) or, before any heading, the project itself. A planning item whose parent is prose becomes a
child of the nearest enclosing planning item (else the heading or project); it is never dropped.

### 2.2 Classification

Let `body` be group 3 of the list-item regex. Apply in order:

1. **Checkbox.** If `body` matches `^\[( |x|X|-)\]\s+(.*)$`, the item has a checkbox:
   ` ` = open, `x`/`X` = done, `-` = cancelled. `body` becomes group 2.
2. **Leading date** (only with a checkbox). If `body` starts with a date expression (§5.1)
   followed by whitespace and more text, that date is the item's **leading date** and is removed
   from `body`. A leading range is not a form: only the first date leads, and the rest is the
   title.
3. **Planning item** = has a checkbox. It is parsed per §4 and becomes a node. Without a checkbox,
   a leading date is plain text and the item is prose. `W-KEYLIKE` only when it has a leading date
   and a planning suffix (`^id`, a known key or an estimate); a bare dated note is prose with no diagnostic.
4. **Property bullet** = not a planning item, its parent is a planning item, a heading, or the
   project, and `body` matches `^([A-Za-z][A-Za-z0-9-]*):\s+(.+)$` where group 1 is a known key
   (§4.4, case-insensitive). The **value runs to the end of the line** (`- owner: Henrik Osei`);
   it is typed per §4.5 and added to the parent's keys. (The project header is a special case, §6.)
5. Otherwise the item is **prose**. Its text may be a description (§2.4). Planning items nested
   under it are still planning items (§2.1).

A planning item's own children may be planning items, property bullets or prose.

### 2.3 Node kinds

| Kind | Rule |
|---|---|
| `milestone` | planning item with a leading date |
| `subproject` | planning item without leading date whose **entire title** is one link (§4.2) whose resolved target has no `#fragment` and whose path ends in `.md` (case-insensitive; query string allowed for URLs) |
| `deliverable` | planning item without leading date whose **entire title** is one link whose target **has** a `#fragment` (the id of a milestone in another file) |
| `task` | any other planning item |
| `heading` | §3 |

A **dated deliverable** is a `milestone` (leading date) whose entire title is such a link with a
`#fragment`. It keeps kind `milestone` (so it is exposed unless `#private`), stores the link's text as
title and the link in `link`. Without its own `id` it takes the link's fragment when that id is free in
the file. A leading date on a link without a `#fragment` is `E-SYNTAX`. A milestone with an estimate
(`2027-06-30 Qualify  26w`) is work with a deadline (FORMAT §7).

### 2.4 Descriptions

Prose lines (class 6) and prose bullets (rule 5) that follow a planning item and are indented
deeper than it are its **description**, kept verbatim, joined with `\n`. The description ends at
the next planning item, or at a line indented at or above the item. A blank line does not end it.
Property bullets (rule 4) are keys, not description.

## 3 Headings

- The **first H1** in the file is the project title.
- Every other heading (any level, including later H1s) is a **heading node**. Its parent is the
  nearest preceding heading node of a lower level, else the project.
- A heading is **only a title**. Its text is taken verbatim after the closing-`#` rule of §1 #3
  (`## Use C#` keeps its `#`). No suffix is parsed: a date, estimate, `^id` or range in a heading
  is title text. A known key followed by a value-like atom (`## Phase  by: 2026-12-01`) gets
  `W-KEYLIKE`. The H1 is the same: its facts are the header keys (§6).
- A heading node with no planning-item descendants is dropped after parsing (e.g. `## Risks`).

## 4 Title and suffix

### 4.1 Atomisation

The text of a planning item (body after checkbox and leading date) is split left to right into
**atoms**. At each position, skip whitespace, then take the first match:

| Atom | Pattern (at current position) |
|---|---|
| code | `` `[^`]*` `` (unclosed: rest of text) |
| inline link | `\[([^\]]*)\]\(\s*(\S+?)(?:\s+"([^"]*)")?\s*\)` |
| reference link | `\[([^\]]*)\]\[([^\]]*)\]` — empty label means the text is the label |
| email | `<[^<>\s]+@[^<>\s]+>` |
| word | `\S+` up to the next whitespace (a word may contain `[`, `(` etc. if no link matched) |

Each atom remembers its start/end offset in the text, so the title can be cut verbatim.
An unclosed bracket or `<` is plain text, not an error.

### 4.2 Atom tests

| Test | Definition |
|---|---|
| tag | word matching `^#[A-Za-z][A-Za-z0-9_-]*$` (so `#3` is not a tag); case-insensitive for `#private` |
| id | word matching `^\^[A-Za-z0-9][A-Za-z0-9_-]*$` |
| estimate | word matching `^[1-9][0-9]*[dwm]$` |
| date | word that is a date expression (§5.1) |
| range | either one word `D..D`, or three atoms `D` `..` `D`, where D are date expressions |
| key | word matching `(?i)^([a-z][a-z0-9-]*):$` whose name is a known key (§4.4) |
| link | inline link or reference link atom; a reference link must resolve via a definition (§1 #4), else it is text |

### 4.3 Suffix extraction (right to left)

Let `A[0..n)` be the atoms and `end = n`. Repeat:

1. If `end - 1 ≤ 0`, stop (the title must keep at least one atom).
2. Let `a = A[end-1]`.
3. **Owner.** If `a` is not a tag, id, estimate, range part or known key, scan
   `k` leftwards from `end-2` to 1 over atoms, stopping at the first atom that is a tag, id,
   estimate, range part (`..`, `D..D`) or known key. If `A[k]` is the key `owner`, the
   value is the text of `A[k+1..end)` verbatim (any atoms; a person or one word keeps that
   kind); record `owner → value`, set `end = k`, continue.
4. **Key + value.** Otherwise find the smallest `k < end-1` such that `A[k]` is a key and `A[k+1..end)`
   is a valid value (§4.5), trying `k = end-2`, then `end-4` (a range after `done:`), then for an
   email atom `a`, every `k` scanning left over plain words. If found and `k ≥ 1`, record `key → value`,
   set `end = k`, continue.
5. **Range.** If `A[end-3..end)` (or `a` alone) is a range and that leaves ≥ 1 atom, record it,
   move `end`, continue.
6. **Bare token.** If `a` is a tag, id or estimate, record it, `end = end-1`, continue.
7. Stop.

The **title** is the text from the start of `A[0]` to the end of `A[end-1]`, verbatim. For a
`subproject` or `deliverable` node (§2.3) the stored title is the link's text (`[Lunar module](…)` →
`Lunar module`); the link itself is kept in the node's `link`. Suffix elements are stored in their
original left-to-right order. Code atoms are never tokens.

A bare date is **not** a suffix token. If the last atom of the title is a date expression,
`W-KEYLIKE` is reported ("write `by: DATE`, or lead with the date"). If a known key is the last
atom of the title (no value), `W-KEYLIKE` is reported.

**Declared-end combinations.** A node has at most one declared end among: leading date, range, `by:`.
Any of these pairs is `E-SYNTAX` ("more than one declared
end"): range and estimate, range and `by:`, leading date and `by:`, leading date and range.
An estimate with `by:` is allowed.

Examples (known keys: owner, after, by, done):

| Text | Title | Suffix |
|---|---|---|
| `Fix issue: retry logic  2w` | `Fix issue: retry logic` | est 2w |
| `Test 12m cable harness  3d  owner: @jan` | `Test 12m cable harness` | est 3d, owner @jan |
| `Review #3 comments  1w  ^review` | `Review #3 comments` | est 1w, id review |
| `Review  owner: Henrik Osei  3d  ^rev` | `Review` | owner `Henrik Osei`, est 3d, id rev |
| `Test  by: Friday  ^t` | `Test  by: Friday` | id t (the bad key stays in the title with `W-KEYLIKE`; tokens to its right are kept) |
| `Tooling release  #private ^tr  owner: Marco Rossi <m@example.com>` | `Tooling release` | tag private, id tr, owner person |
| `Rendezvous concept  2027-Q1 .. 2027-Q2` | `Rendezvous concept` | range |
| `Ship  2027-06-30` | `Ship  2027-06-30` | none; `W-KEYLIKE` (a date at the end is title text) |
| `2w` | `2w` | none (the title may not be empty) |
| `Ship it  after: ^a  after: ^b` | `Ship it` | after ^a, after ^b |
| `` Explain `after: ^x` syntax `` | `` Explain `after: ^x` syntax `` | none |

### 4.4 Known keys

Known keys: `owner`, `after`, `by`, `done`. Only `after` is repeatable. Repeating any other key on
one node is `E-SYNTAX` (the last value wins for tooling that continues). `done` on an open item
gets `W-KEYLIKE`. Any other `word:` is text.

### 4.5 Values

A value is exactly one of:

| Value | Atoms |
|---|---|
| person | one or more word atoms followed by one email atom: `Marco Rossi <m@example.com>` |
| range | a range |
| word | one word atom (`@name`, dates, ids, free words) |

No key takes a link: a link atom is never a value. `after: file.md#id` and `by: [x](y.md#z)` stay
in the title with `W-KEYLIKE`; a cross-file edge is a link line (FORMAT §6).

Each key type-checks its value; a value of the wrong type means the key is not a key (it stays in
the title with `W-KEYLIKE`):

| Key | Accepted values |
|---|---|
| `owner` | any text (§4.3 step 3), e.g. `Jan Doe`, `Jan <j@x.org>`, `@jan` |
| `after` | an id (`^x`) or a date expression |
| `by` | a date expression or an id (`^x`); not a range |
| `done` | a date expression or a range |

A property bullet's value (§2.2 rule 4) is the rest of the line, typed the same way.

## 5 Lexical forms

### 5.1 Dates

| Form | Regex | Expands to |
|---|---|---|
| exact | `^(\d{4})-(\d{2})-(\d{2})$` | that day (must be a real calendar date) |
| month | `^(\d{4})-(\d{2})$` | first..last day of month |
| quarter | `^(\d{4})-Q([1-4])$` | first..last day of quarter |

A string matching the shape but invalid (`2027-02-30`, `2027-13`) is a date atom with error
`E-DATE`. A range `A .. B` expands to start(A)..end(B); end before start is `E-DATE`.
A coarse date used as a start (`after:`, a range's start) is its first day; used as an end
(`by:`, leading date, a range's end, `done:`) it is its last day.

### 5.2 Estimates

`Nd` = N weekdays (Monday to Friday), `Nw` = 7N calendar days, `Nm` = N calendar months. How an
estimate becomes an end date is defined once, in FORMAT §7.1.

### 5.3 Names

Ids and tags: see §4.2. People: `Name <email>` or any text.

## 6 Project header

The **header** is the run of list items at the indent of the first list item after the H1 line
(only blank lines in between). It continues while each item has no checkbox and matches
`^([A-Za-z][A-Za-z0-9 _-]*):\s*(.*)$`. It ends at the first item that does not match, or at any
non-blank, non-list line (a heading or a paragraph). Deeper lines under a header item are ignored;
a checkbox item nested there is a planning item of the project (§2.1).

Header keys are **any** key (case-insensitive, internal spaces allowed: `Start date`); values are raw
strings, interpreted for core keys. A prose bullet with a colon inside the run (`- Note: x`) is a
header key. A header key set twice is `E-SYNTAX`.

| Key | Interpretation |
|---|---|
| `Owner` | a person (`Name <email>`) or any text |
| `Commitment` | date expression or range; or `^id` / `DATE .. ^id`: a local item with a declared end (leading date or dated deliverable), whose date (range end) is the Commitment; an unknown or undated id is `E-REF-LOCAL` |
| `Done` | exact date: the project was delivered that day |

Any other key (`Stakeholders`, ...) is kept in `header` but has no meaning in the core. If the first
item does not match, there is no header. There is no YAML front matter.

## 7 Interface, contracts and fronts

Nothing beyond §1-§6 is parsed. A project's interface (FORMAT §8) and a contract state (FORMAT §9)
are computed from project files at read time. A front file (FORMAT §8.1) is an ordinary project file.

## 8 Parse output (data model)

```text
Project  { path, title, owner, commitment, done, header{k:v},
           nodes[] (top level), refdefs{label:(target,title)}, diagnostics[] }
Node     { kind (task|milestone|subproject|deliverable|heading), line,
           checkbox (none|open|done|cancelled), title, date (leading date), range,
           duration (the estimate), private (bool), id, done, keys{ name: [values] },
           link (target + link title), description, children[] }
Value    { kind (word|person|range), text, name?, email? }
Diagnostic { severity (E|W|I), code, line, message }
```

Scheduling adds `forecast` to nodes and `link_resolved` to link nodes; JSON shapes are given by the
schemas in `docs/schema/`.

Diagnostic codes produced by the parser itself: `E-SYNTAX`, `E-DATE`, `E-DUP-ID`, `E-REF-LOCAL`
(an `^id` value with no matching id in the file, or `by: ^id` to an item with no date),
`W-NO-OWNER`, `W-KEYLIKE`. Cross-file checks (`E-REF-PUBLIC`, `E-REF-MISSING`, `E-CYCLE`) and
contract states (`W-OVERDUE`, `W-DISPUTED`, `W-UNKNOWN`, `I-REQUESTED`) need the graph loader;
`E-CONFLICT` and the `after:` cycle need scheduling. The conditions are in FORMAT §11.
