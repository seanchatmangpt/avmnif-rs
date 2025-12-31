# Invariants

These are truths that must never break.

If any invariant is violated, the work is invalid and must be reverted.

## Global Invariants

### G1: Add-Only Rule
Existing files in `src/` are never modified.
New code goes in new directories.

**Verification:**
```bash
git diff src/atom.rs src/term.rs src/context.rs src/resource.rs src/port.rs src/tagged.rs src/registry.rs src/log.rs src/testing/
# Must output: (nothing)
```

### G2: Deterministic Execution
All tests pass the same way every run.
No flaky timeouts, no random failures.
AtomVM instance behavior is reproducible.

**Verification:**
```bash
cargo test --features atomvm_support -- --test-threads=1
# All tests pass in sequence with identical output
```

### G3: No Panic Across Boundaries
Code at FFI boundaries never panics.
All Rust → C transitions are wrapped with error handling.

**Verification:**
- No `panic!()`, `.unwrap()`, `.expect()` in `atomvm_support/ffi/`
- All C calls return `Result<T, AtomVmError>`

### G4: Explicit Ownership
Resources obtained from C are explicitly tracked.
No implicit leaks, no double-frees.
Ownership transfers are documented.

**Verification:**
- FFI wrappers implement `Drop` where needed
- Comments explain ownership at every C boundary
- Tests verify cleanup via `valgrind` or similar (if available)

## FFI Layer Invariants

### F1: Null-Pointer Safety
Every pointer from C is checked before use.

**Pattern:**
```rust
// ❌ Wrong
let value = unsafe { *ptr };

// ✅ Right
let value = if ptr.is_null() {
    return Err(AtomVmError::NullPointer);
} else {
    unsafe { *ptr }
};
```

### F2: Error Mapping
Every C error code is mapped to a Rust error type.
No error is silent.

**Pattern:**
```rust
// C returns int (0 = success, non-zero = error code)
match c_function() {
    0 => Ok(result),
    -1 => Err(AtomVmError::FailedToInitialize),
    code => Err(AtomVmError::UnknownError(code)),
}
```

### F3: Unsafe Documentation
Every `unsafe` block has a `// Safety:` comment.

**Pattern:**
```rust
// Safety: ptr is guaranteed non-null because we checked above.
// It points to a valid ErlNifEnv allocated by AtomVM.
unsafe { enif_make_ok(ptr) }
```

### F4: No Panics in Unsafe
Unsafe blocks do not call functions that might panic.

**Verification:**
- No `.unwrap()` in unsafe blocks
- No panic-prone operations (integer overflow, etc.)

## Runtime Layer Invariants

### R1: Observable Failures
Every function that can fail returns `Result<T, AtomVmError>`.
Failures are never hidden.

**Verification:**
- No functions returning `bool` for success/failure
- All Err paths have specific error types

### R2: Explicit State Transitions
AtomVM state changes are explicit and ordered.
Cannot execute before loading.
Cannot load before initializing.

**Sequence:**
```
Initialize → Load Module → Execute → Observe Result
     ↓            ↓            ↓           ↓
 AtomVmHost  ModuleLoaded  ExecutionResult  TermValue
```

**Verification:**
- Types prevent calling execute before load (at compile time or runtime check)
- No silent no-ops (e.g., execute-when-not-loaded does not return Ok)

### R3: Term Boundary Correctness
Terms crossing Rust ↔ Erlang are valid on both sides.
Roundtrip testing proves this.

**Verification:**
```rust
let original = TermValue::int(42);
let sent = Term::from_value(original.clone(), heap)?;
let received = sent.to_value()?;
assert_eq!(original, received);  // Roundtrip holds
```

## Loader Layer Invariants

### L1: Bytecode Validation Before Use
No module executes without prior validation.
Validation catches corruption, format errors, version mismatches.

**Verification:**
- `load_module()` calls `validate_bytecode()` internally or caller proves validation happened
- Test suite includes corrupted/invalid bytecode files

### L2: Module Metadata is Accurate
Module inspection (functions, arities, exports) matches what executes.
No surprises at execution time.

**Verification:**
- `inspect_module()` returns the same metadata that execution respects
- A function reported as arity-2 accepts exactly 2 arguments

## Test Invariants

### T1: No Shared State Between Tests
Each test gets a fresh AtomVM instance.
Tests do not affect each other.

**Verification:**
```bash
# Any order, any count should work
cargo test --features atomvm_support -- --test-threads=4
cargo test --features atomvm_support test_name1
cargo test --features atomvm_support test_name2
# All pass identically
```

### T2: Test Data is Minimal
Test modules contain only what's needed to exercise the code.
No bloat, no leftover debug code.

**Verification:**
- Example bytecode files are < 1KB
- Test function names are descriptive

## Violations and Recovery

**If any invariant is violated:**

1. **Identify which**: Which invariant from this list?
2. **Revert scope**: Revert all atomvm_support code back to checkpoint
3. **Investigate**: Why did the violation happen?
4. **Fix root cause**: Do not work around it
5. **Verify recovery**: Re-run full test suite

**No merges are allowed with violations.**
