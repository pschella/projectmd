# Benchmarks

Machine: Apple M2 (`arm64`, `sysctl machdep.cpu.brand_string`), release build of `parser/`.
Data: synthetic trees from `cmd/pmdgen` (`-fanout 10 -items 10 -xrefs 0.1 -seed 1`). Each
file has about 34 lines, 1.2 KB, 16 nodes; 10% of tasks are deliverable lines into another
branch. Timings are medians of warm runs.

| Depth | Files | Size | `pmd check TOP` (one hop, default) | `pmd check --deep TOP` | Peak memory (deep) |
|---|---|---|---|---|---|
| 3 | 1,111 | 1.3 MB | 3 ms | 0.06 s | 30 MB |
| 4 | 11,111 | 13.5 MB | 3 ms | 0.6 s | 273 MB |
| 5 | 111,111 | 137 MB | 3 ms | 11 s | 2.7 GB |

Parse time: 10 µs per file (about 120 MB/s), the same at every size.

## What this shows

1. **Default operations are one hop and do not grow with the graph.** `check`, `schedule` and
   `tree` read the file and its direct links (here 11 files). Time is process start-up plus
   about 11 small reads, 3 ms at 1k and at 100k files. There is no index, cache or stored
   state to maintain.
2. **`--deep` is linear in the number of files** (about 100 µs per file here, mostly reading;
   the run at 100k is slower per file because of memory pressure). It is for CI over a whole
   graph, not for everyday use.
3. **Parsing is not the bottleneck.** A 1.2 KB file parses in 10 µs, so parsing in the
   browser (WebAssembly) is instant for anything a person opens.
4. Deep mode keeps every parsed project in memory (about 24 KB per file). That is acceptable
   for one repository; a check across a very large organisation should shard by store.

## Reproduce

```bash
go run ./cmd/pmdgen -out /tmp/pmd-bench -depth 4 -fanout 10 -items 10 -xrefs 0.1
cd parser && cargo build --release
time ./target/release/pmd check /tmp/pmd-bench/root.md            # one hop
time ./target/release/pmd check --deep /tmp/pmd-bench/root.md     # whole graph
PMD_BENCH_DIR=/tmp/pmd-bench cargo test --release -p pmd-core --test examples -- --ignored --nocapture
```
