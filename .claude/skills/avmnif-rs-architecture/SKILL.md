---
name: avmnif-rs-architecture
description: Enforce avmnif-rs module boundaries and patterns when adding AtomVM support, NIFs, ports, terms, resources, host runtime, loaders, or tests.
---

# avmnif-rs Architecture

## Hard constraints
- **Add-only rule**: do not modify or delete any existing files. Only add new files/directories.
- Keep new code discoverable: prefer **small focused modules** over mega-files.
- Keep public API surface minimal: prefer `pub(crate)` unless required.

## Where new code should go
Add new files under new directories as needed (do not restructure existing `src/` files):
- `src/atomvm_support/` — host-facing AtomVM support (boot, load, run)
- `src/atomvm_support/ffi/` — FFI bindings + safe wrappers
- `src/atomvm_support/loader/` — module/bytecode loading
- `src/atomvm_support/runtime/` — host runtime orchestration primitives
- `src/atomvm_support/testing/` — integration harnesses that need VM boot

Keep existing areas intact:
- `src/term.rs`, `src/atom.rs`, `src/tagged.rs`, `src/ports.rs`, `src/nifs.rs`, `src/resource*.rs` remain unchanged.

## Architectural layering rules
1. **FFI layer** (lowest): raw bindings + minimal types. No business logic.
2. **Safe wrapper layer**: Rust-safe functions/types that enforce invariants.
3. **Host/runtime layer**: orchestration, module loading, runtime control.
4. **Examples/tests**: consume host/runtime via safe wrappers only.

## Naming rules
- Prefer descriptive module names: `loader`, `ffi`, `runtime`, `errors`, `validate`.
- Avoid ambiguous names like `common`, `utils`, `misc`.
- Avoid crate/module names that collide with Rust built-ins (e.g., `core`).

## Output contract
When adding features:
- Add code + tests + a short doc stub (under `docs/` or `src/atomvm_support/README.md`).
- Ensure any new public function has a short `///` doc comment stating invariants and failure modes.
