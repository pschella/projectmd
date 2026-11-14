# projectmd

**Project plans as plain markdown files, linked into a graph, shown as one expandable Gantt chart.**

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
- [ ] Module & pack design  12w  after: ^chemistry  owner: @priya
- [ ] 2027-09-30 B-samples available  #public  ^b-samples
- [ ] [Cell module](cell-module.md)  by: 2027-06-30
```

- **Every file is a project.** Its list items are tasks and milestones. A link to another file
  makes that file a sub-project, wherever it lives.
- **Time is built in at every level.** You can use dates, rough dates (`2027-Q3`), durations
  (`12w`) and dependencies (`after:`). A parent states what it needs (`by:`) and a child states
  what it promises (`Commitment:`).
- **It scales by locality.** Only declared dates cross file boundaries. The top-level view reads
  one file plus a lock file, whether the graph has 10 projects or 100,000.

> Status: **early draft.** The format may still change.

## Documentation

| Document | For |
|---|---|
| [docs/CHEATSHEET.md](docs/CHEATSHEET.md) | The whole format on one page. Start here, or hand it to an AI agent. |
| [docs/FORMAT.md](docs/FORMAT.md) | Format reference: the node model, scheduling, interfaces, the lock file, validation. |
| [docs/GRAMMAR.md](docs/GRAMMAR.md) | The precise grammar. It decides how text is parsed. |
| [docs/VIEWER-CONTRACT.md](docs/VIEWER-CONTRACT.md) | How the web viewer calls the parser (WebAssembly interface). |
| [docs/BENCHMARKS.md](docs/BENCHMARKS.md) | Parser scaling measurements. |
| [examples/](examples/README.md) | A worked example: the plan for a fictional electric car, 20 files up to 5 levels deep. |

## Quick start

**Viewer.** Serve the repository root, then open a project:

```bash
python3 -m http.server 8642
```

Open http://localhost:8642/viewer/?root=/examples/car/aurora.md. Click ▸ to expand a sub-project
(it loads lazily). Ctrl/Cmd-click or double-click a sub-project to make it the top level.

**Command-line tool** (needs Rust):

```bash
cd parser && cargo build --release
./target/release/pmd check ../examples/car/aurora.md      # validate the graph
./target/release/pmd schedule ../examples/car/aurora.md   # forecast dates and conflicts
./target/release/pmd tree ../examples/car/aurora.md       # top-level outline
./target/release/pmd lock ../examples/car/aurora.md       # refresh pmd.lock
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
