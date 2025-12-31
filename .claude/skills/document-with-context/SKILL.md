---
name: document-with-context
description: Write code with explicit motivation and invariants: add doc comments, module notes, and short docs explaining why, what, how, and failure modes.
---

# Document With Context

## Goals
- Make intent durable
- Make invariants explicit
- Reduce future archaeology

## Required documentation per new module
At the top of each new module file, add a short module doc:

```rust
//! Purpose: ...
//! Guarantees: ...
//! Failure modes: ...
//! Where to extend: ...
```

## Doc comment rules

For public functions/types:

* State:

  * what it does
  * inputs accepted
  * error cases
  * any safety invariants

For unsafe wrappers:

* restate invariants in both the wrapper and unsafe blocks.

## Minimal docs artifacts

When adding a new subsystem (e.g., AtomVM host support):

* Add a new markdown doc under `docs/` (new file only) describing:

  * module layout
  * key entrypoints
  * testing approach

## Output contract

Each significant addition must include:

* module docs
* public item docs
* a small markdown note for subsystem-level additions
