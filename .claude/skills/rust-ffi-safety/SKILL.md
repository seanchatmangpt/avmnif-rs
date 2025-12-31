---
name: rust-ffi-safety
description: Apply strict safety and correctness patterns when adding any Rust<->C FFI, unsafe blocks, pointers, buffers, or ownership transfer.
---

# Rust FFI Safety

## Goals
- Make unsafe code **small, audited, and wrapped** by safe APIs.
- Make ownership and lifetime rules explicit and testable.

## Rules for any `unsafe` block
Every `unsafe` block must be preceded by a comment section:

```rust
// SAFETY:
// - Invariant 1 ...
// - Invariant 2 ...
// - Caller guarantees ...
```

If you cannot state invariants precisely, redesign the interface.

## FFI boundary patterns

* Prefer **opaque handles** over exposing raw pointers widely.
* Prefer `NonNull<T>` over `*mut T` in wrappers.
* For buffers crossing the boundary:

  * pass pointer + length
  * validate length
  * never assume NUL-termination unless explicitly guaranteed
* Do not panic across FFI. Convert to error returns.

## Error handling

* Convert errors to a stable error representation at the boundary:

  * numeric codes + optional static string
  * or a Rust enum mapped to error atoms on the VM side (if applicable)
* Never return uninitialized memory.

## Threading and reentrancy

* Do not assume callbacks are single-threaded unless guaranteed.
* Document whether calls are re-entrant; if not, guard state.

## Testing requirements

For each new FFI wrapper:

* Add a unit test that exercises:

  * nominal path
  * invalid pointer/len handling (as far as safely possible)
  * error propagation path
* Add a "no UB by construction" check: wrappers refuse to create invalid states.

## Output contract

Whenever you add FFI:

* Add the safe wrapper
* Add tests
* Add minimal docs describing ownership and error mapping
