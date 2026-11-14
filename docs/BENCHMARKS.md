# Parser scaling benchmarks

Run on 2026-11-14 on an Apple Silicon Mac, with a release build of `parser/` (Rust) and
synthetic trees from `cmd/pmdgen` (`-fanout 10 -items 10 -xrefs 0.1 -seed 1`). Each file
has about 34 lines, 1.2 KB, 16 nodes, and 10% of its tasks depend on a public milestone in
another branch.

## Full graph (`pmd stats ROOT.md`: load everything, parse, check across files)

| Depth | Files | Size | Nodes | Read | Parse | Check | Peak memory |
|---|---|---|---|---|---|---|---|
| 2 | 111 | 0.13 MB | 1.8k | 10 ms | 2 ms | 0.3 ms | 5 MB |
| 3 | 1,111 | 1.3 MB | 18k | 66 ms | 14 ms | 3 ms | 26 MB |
| 4 | 11,111 | 13.6 MB | 179k | 0.65 s | 0.13 s | 35 ms | 236 MB |
| 5 | 111,111 | 137 MB | 1.79M | 6.95 s | 1.27 s | 0.51 s | 2.37 GB |

## Top-level view (`pmd tree ROOT.md --depth 1`)

Under 10 ms at every size, including 111,111 files. With a `pmd.lock` the view also shows each
child's commitment and public milestones, still without opening any child file.

## Findings

1. **The locality design works.** The top-level view costs the same at any size, because it
   reads one file and its lock file. This is the main requirement (R1 in the spec).
2. **Parsing is cheap and linear.** It takes about 11 µs per file (about 105 MB/s), and checks
   across files are linear too. Parsing is not the bottleneck.
3. **Reading files dominates** (about 80% of full-load time). Files are read one at a time,
   single-threaded. Parallel reading in the command-line tool should cut this several times.
   It only matters for whole-graph operations, which the design avoids.
4. **Memory is too high when everything is loaded.** It's about 21 KB per file, because every
   parsed tree is kept with owned strings. Fixes, if needed: keep only each project's
   interface (commitment, public milestones, outgoing references) after checking it, or use
   borrowed strings and interned keys. A viewer never loads 100k files at once, but a check
   across a whole organization might.
5. A 2 KB file parses in about 12 µs, so parsing in the browser (WebAssembly) will be instant
   for anything a person opens.

## Reproduce

```bash
go run ./cmd/pmdgen -out /tmp/pmd-bench -depth 4 -fanout 10 -items 10 -xrefs 0.1
cd parser && cargo build --release && ./target/release/pmd stats /tmp/pmd-bench/root.md
```
