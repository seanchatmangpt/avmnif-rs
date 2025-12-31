---
name: term-codec-correctness
description: Enforce correct, tested conversions between Rust types and AtomVM terms (atoms, ints, tuples, lists, maps, tagged ADTs), including negative cases.
---

# Term Codec Correctness

## Goals
- Round-trip correctness where defined
- Predictable failure modes for invalid input
- No allocation surprises across term creation

## Conversion rules
- Decoders must reject:
  - wrong term type
  - wrong arity
  - out-of-range numeric values
  - malformed tagged maps
- Encoders must:
  - build terms using existing heap/context facilities
  - return explicit errors on OOM or system limits

## Required tests per new type
For any new conversion or tagged ADT:
1. **Round trip**: Rust -> term -> Rust equals original
2. **Type mismatch**: wrong term kind returns `BadArg` (or equivalent)
3. **Boundary values**: min/max, empty, nested
4. **Tag mismatch** (for tagged maps): wrong discriminator rejects

## Tagged ADT rules
- Discriminator atom must be stable and documented
- Field naming must be explicit
- Missing fields must error unless explicitly optional

## Performance guardrails
- Avoid deep recursion without depth checks
- Prefer iterative decoding for lists if large
- Keep allocations visible and minimal

## Output contract
Whenever adding term conversions:
- Add tests for roundtrip + negative cases
- Add a brief doc comment stating accepted shapes and failure conditions
