---
name: tests-and-benchmarks
description: Require tests and (when relevant) benchmarks for every new file/module added, especially for FFI, loaders, term codecs, ports, NIFs, and runtime glue.
---

# Tests and Benchmarks Discipline

## Tests are mandatory
For every added module file:
- Add unit tests in a sibling `*_tests.rs` or `#[cfg(test)] mod tests` section.
- Prefer table-driven tests when many cases exist.

## Test categories to include when applicable
- **Correctness**: expected outputs
- **Negative**: invalid inputs produce correct error
- **Regression**: encode bug scenario as a test
- **Resource**: ensures no leaks / correct drops for resources
- **Boundary**: FFI safety wrapper refuses invalid states

## Benchmarks (only when performance-critical)
Add benchmarks when the new code:
- runs in hot paths (term conversions, message passing)
- crosses boundaries frequently
- could regress latency/throughput

Benchmark rules:
- stable input sizes
- compare against baseline if available
- print results in a consistent format

## Output contract
Every change you make must include:
- new tests
- and benchmarks if the change is plausibly performance-sensitive
