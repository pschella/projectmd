# projectmd

**A plain-markdown format for project plans, and a model for scaling them: from one todo list
to a large programme spread across many teams, repositories and companies.**

Many interdependent projects must come together on strict timelines, with milestones,
decision gates and sign-offs. Ticket trackers are poor at timelines, scheduling tools are heavy
and siloed, and text todo lists show *what* but not *when*. projectmd keeps the todo list and
adds just enough structure to answer "when?" and "what depends on what?":

```markdown
# Battery pack — 60 kWh platform pack

- Owner: Priya Nair <priya.nair@example.com>
- Commitment: 2027-11-30

## Work
- [x] 2026-10-31 Cell chemistry selected  ^chemistry
- [ ] Module & pack design  12w  after: ^chemistry  owner: Priya
- [ ] 2027-09-30 B-samples available  ^b-samples
- [ ] [Cell module](cell-module.md)  by: 2027-06-30
```

- **Every file is a project.** Its list items are tasks and milestones. A link to another file
  makes that file a sub-project, wherever it lives.
- **Time is built in at every level.** You can use dates, rough dates (`2027-Q3`), durations
  (`12w`) and dependencies (`after:`). A parent states what it needs (`by:`) and a child states
  what it promises (`Commitment:`).
- **It scales by locality.** Only declared dates cross file boundaries. Any view reads one file
  plus its direct links, whether the graph has 10 projects or 100,000.

## Grow it gradually, top-down or bottom-up

A plan does not have to start big. Each step is optional, and each keeps the files you already have valid:

1. **A todo list.** `- [ ] Order cells` is already a project.
2. **Add time** where it helps: a date, a duration or an `after:`. Tools forecast and flag conflicts.
3. **Split** an item that grew into its own file: replace its title with a link. Nothing else on the line changes.
4. **Promise.** Give the file a `Commitment:`. Its milestones are what parents see; `#private` hides one.
5. **Distribute.** A sub-project can live anywhere a link can point, kept by its own team with its own tools.

Grow **top-down** (a lead writes the skeleton and links placeholder files) or **bottom-up** (existing
team plans are linked into a new parent). Both meet in the same graph, and no file is special.

**Contracts cross boundaries, nothing else.** A parent states what it needs (`by:`), a child states
what it promises (`Commitment:` and milestones), and the contract state (requested, agreed, disputed,
delivered) is computed live from the two files. Reading a plan never looks further than its direct
links. A plan that must stay private gets a thin front file. See [docs/FORMAT.md](docs/FORMAT.md) §8-§9.

## Tools

The format is the product. The tools here are a first reference implementation: a parser and
validator in Rust (also built to WebAssembly) and a web viewer that draws a project graph as an
expandable Gantt chart. Anyone can write others: the files are plain markdown and the grammar is specified.

![The viewer showing an example programme as a nested Gantt chart](docs/img/gantt.png)

> Status: **early draft.** The format may still change.

## Documentation

| Document | For |
|---|---|
| [docs/CHEATSHEET.md](docs/CHEATSHEET.md) | The whole format on one page. Start here, or hand it to an AI agent. |
| [docs/FORMAT.md](docs/FORMAT.md) | Format reference: the node model, scheduling, interfaces and front files, contracts, validation. |
| [docs/GRAMMAR.md](docs/GRAMMAR.md) | The precise grammar. It decides how text is parsed. |
| [docs/DESIGN.md](docs/DESIGN.md) | Why it is designed this way: requirements, concepts, checks. |
| [docs/VIEWER-CONTRACT.md](docs/VIEWER-CONTRACT.md) | How the reference viewer calls the parser (WebAssembly interface). |
| [docs/BENCHMARKS.md](docs/BENCHMARKS.md) | Parser scaling measurements. |
| [examples/](examples/README.md) | Worked examples: a fictionalised crewed-lunar-landing programme in seven stores (`examples/lunar/`) and a company split across three stores (`examples/org/`). |

## Quick start

**Viewer.** Serve the repository root, then open a project:

```bash
python3 -m http.server 8642
```

Open http://localhost:8642/viewer/?root=/examples/lunar/hq/lunar-programme.md&today=1967-11-15. Click the chevron to expand a
sub-project (loaded on demand) and select a row to see its details. Ctrl/Cmd-click or double-click
a sub-project to make it the top level.

**Command-line tool** (needs Rust):

```bash
cd parser && cargo build --release
./target/release/pmd check ../examples/lunar/hq/lunar-programme.md          # validate; --deep walks all linked files, --strict fails on warnings
./target/release/pmd schedule ../examples/lunar/hq/lunar-programme.md       # forecast dates and conflicts
./target/release/pmd tree ../examples/lunar/hq/lunar-programme.md --depth 2 # outline with contract states
./target/release/pmd ready ../examples/lunar/hq/lunar-programme.md --deep   # open work that may start now
./target/release/pmd fmt ../examples/lunar/hq/lunar-programme.md --check    # canonical form? (without --check, rewrites the file)
```

Rebuild the viewer's WebAssembly module with `parser/build-wasm.sh`.

## Repository layout

```
docs/        user documentation (format, grammar, viewer contract, benchmarks)
examples/    worked example project graph
parser/      Rust workspace: pmd-core (library), pmd-cli (`pmd`), pmd-wasm (browser build)
viewer/      static web Gantt viewer (HTML/CSS/JS + pmd.wasm), no build step
testdata/    language-neutral conformance cases for parsers
cmd/pmdgen/  synthetic project-graph generator for benchmarks (Go)
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in
the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without
any additional terms or conditions.
