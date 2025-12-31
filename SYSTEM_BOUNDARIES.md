# System Boundaries

## The Existing System is Immutable

All code in `src/` (modules: atom.rs, term.rs, context.rs, resource.rs, port.rs, tagged.rs, registry.rs, log.rs, testing/*) **is not modified**.

These modules remain the foundation.
New code is never inserted into them.

## Allowed Additions

**Locations where code may be added:**

- `src/atomvm_support/` — New top-level module hierarchy
- `examples/` — Runnable demonstration code
- Documentation files (`.md`) — New feature guides
- Test harnesses using `testing::*` module

**Types of additions:**

- New Rust modules and traits
- FFI bindings and safe wrappers around them
- Implementation of trait bounds defined in existing modules
- Test utilities and integration harnesses
- Example programs
- Configuration files (if needed for build)

## Forbidden Actions

**These are not permitted:**

- Editing any file in `src/atom.rs`, `src/term.rs`, `src/context.rs`, `src/resource.rs`, `src/port.rs`, `src/tagged.rs`, `src/registry.rs`, `src/log.rs`
- Editing any file in `src/testing/`
- Moving existing files
- Renaming existing modules
- Changing existing public function signatures
- Changing existing trait definitions
- Adding dependencies to `Cargo.toml` (except if essential for FFI)
- Modifying `Cargo.toml` version or metadata

**Exception:** Documentation within existing modules (doc comments) may be clarified, never expanded with new sections.

## The Boundary Rule

All new behavior must be **layered around** the existing system, never inside it.

**Allowed pattern:**
```
existing module (untouched)
       ↑
       |
   new code
```

**Forbidden pattern:**
```
existing module (edited)
       ↑
       |
   new code
```

## Why This Boundary Exists

1. **Verification**: Proves new code works without reshaping the foundation
2. **Reversibility**: If the POC fails, the codebase is unchanged
3. **Clarity**: Future contributors see exactly what is "proven" vs "existing"
4. **Testability**: Old tests pass unchanged; new tests prove new capability

## File Organization Authority

**`src/`** — Existing library core (untouchable)
**`src/atomvm_support/`** — POC integration layer (new, add-only)
**`examples/`** — Runnable proofs (add-only)
**`docs/`** — POC-specific guides (add-only)

This mirrors the current structure and respects it.
