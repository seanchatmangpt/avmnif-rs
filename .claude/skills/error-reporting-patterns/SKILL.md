---
name: error-reporting-patterns
description: Standardize error types, mapping, and propagation across boundary layers: term decoding, ports, NIFs, resources, FFI wrappers, loaders, runtime host.
---

# Error Reporting Patterns

## Goals
- Typed errors internally
- Stable mapping at boundaries
- No silent swallowing

## Rules
- Use `Result<T, E>` everywhere; avoid sentinel values.
- Define error enums per subsystem when helpful:
  - `LoaderError`
  - `FfiError`
  - `CodecError`
- Implement `Display` for user-readable messages.
- Keep error variants actionable (include context fields).

## Boundary mapping
When crossing into VM-facing surfaces:
- Map errors deterministically to:
  - `BadArg`, `BadArity`, `OutOfMemory`, `SystemLimit`, `InvalidTerm`, or a stable custom atom/message.
- Never leak internal panic messages into the boundary.

## Tests
For each error mapping:
- Add a test proving the mapping is correct for representative variants.
- Add at least one "unknown error" mapping test to ensure stability.

## Output contract
Any new subsystem must include:
- its error type
- boundary mapping
- tests proving mapping correctness
