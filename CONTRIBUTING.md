# Contributing

The **format is the product**. The files, their grammar and the scaling model come first; the tools
here (`pmd`, the viewer) are reference implementations. Anyone can write another.

## Proposing a format change

The bar is minimalism: nothing enters the format or the tools unless a core requirement needs it.
Open an issue that states:

1. **The requirement** it serves, in a sentence, and why the current format cannot meet it.
2. **A FORMAT/GRAMMAR diff**: the exact wording you would change in `docs/FORMAT.md` and `docs/GRAMMAR.md`.
3. **Conformance cases**: input files and expected results, as added under `testdata/conformance`.

A change that alters the meaning of an existing valid file also needs a `pmd fmt` migration.
Removing a form is welcome when the requirement is met without it.

## Tools

Bug fixes and improvements to `pmd` or the viewer are welcome as pull requests. The conformance suite
is the arbiter between the specification and a tool. Keep dependencies out unless they are needed.

## Development setup

```
cd parser
cargo test --all
cargo clippy --all-targets -- -D warnings
cargo build --release          # parser/target/release/pmd
./build-wasm.sh                # rebuild viewer/pmd.wasm
python3 ../site/build.py out   # render the website into out/ (needs `pip install markdown`)
```

Examples must stay canonical: `pmd fmt --check` on every example file. Examples are checked in CI with
a fixed `PMD_TODAY`.

## Style

Docs are short and concrete. Examples use fictional people and companies. No personal or employer
information goes into the repository.

## Licence

The project is licensed under either of MIT or Apache-2.0, at your option. Unless you state otherwise,
any contribution you submit is dual licensed as above, without additional terms or conditions.
