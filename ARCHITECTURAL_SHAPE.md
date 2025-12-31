# Architectural Shape

## Directory Structure (Required Additions)

```
src/atomvm_support/
├── mod.rs                 # Module organization, public exports
├── ffi/
│   ├── mod.rs            # FFI bindings organization
│   ├── bindings.rs       # Raw C FFI imports (extern "C" only)
│   ├── errors.rs         # FFI error type mapping
│   └── types.rs          # Safe wrapper types around FFI structs
├── runtime/
│   ├── mod.rs            # Runtime orchestration
│   ├── host.rs           # AtomVM host initialization & management
│   ├── executor.rs       # Module execution, function calls
│   └── state.rs          # Global state management (if needed)
├── loader/
│   ├── mod.rs            # Module loading organization
│   ├── validation.rs     # Bytecode format validation
│   └── registry.rs       # Module registration and lookup
└── testing/
    ├── mod.rs            # Test integration harnesses
    └── fixtures.rs       # Test scenario builders
```

## Dependency Direction (Strict)

```
ffi/
  ↑
  |
runtime/
  ↑
  |
loader/
  ↑
  |
examples/ & tests/
```

**Rules:**
- `ffi/` has no dependencies outside its module
- `runtime/` depends only on `ffi/` and `avmnif::*`
- `loader/` depends on `runtime/` and `ffi/`
- Tests/examples depend on all above
- **No backward dependencies allowed** (runtime cannot import from loader, etc.)
- **No sibling imports** (loader ↔ runtime forbidden)

## Prohibited Shapes

These structures are forbidden:

- Circular imports between sub-layers
- Shared "utils" or "helpers" module used by multiple layers
- Global state accessible from multiple layers without explicit handoff
- Implicit initialization in module load-time code
- Cross-boundary unsafe code (unsafe blocks must be in `ffi/` only)

## Module Boundaries (Public Interfaces)

### FFI Module (`ffi/mod.rs`)

**Exports:**
- Safe error type: `AtomVmError`
- Safe wrapper type: `AtomVmHost` (opaque handle)
- Initialization function: `initialize_atomvm()` → `Result<AtomVmHost, AtomVmError>`
- Cleanup (via Drop impl on guard type)

**Does NOT export:**
- Raw C pointers
- Unvalidated bytecode
- Process IDs without wrapping

### Runtime Module (`runtime/mod.rs`)

**Exports:**
- `AtomVmContext` (opaque, from FFI)
- `ExecutionResult` enum (success, error, halted, etc.)
- `execute_function()` → `Result<TermValue, AtomVmError>`
- `load_module_bytes()` → `Result<(), AtomVmError>`

**Does NOT export:**
- Raw memory pointers
- Unloaded module references
- Unvalidated terms

### Loader Module (`loader/mod.rs`)

**Exports:**
- `validate_bytecode()` → `Result<&[u8], BytecodeError>`
- `ModuleInfo` struct (name, arity, public functions)
- `inspect_module()` → `Result<ModuleInfo, BytecodeError>`

**Does NOT export:**
- Parsed internal bytecode structures
- Unvalidated bytes

## Layer Invariants

**FFI Layer:**
- All unsafe code is isolated and documented
- Every C call is wrapped with error mapping
- Null pointer checks before any dereference

**Runtime Layer:**
- No unsafe code (receives wrapped types from FFI)
- All errors are observable (no silent failures)
- State transitions are explicit (init → load → execute)

**Loader Layer:**
- No unsafe code (pure validation)
- Bytecode is validated before runtime sees it
- Errors are specific (format, version, corruption, etc.)

## No Implicit State

- AtomVM instance is represented by an explicit handle (`AtomVmHost`)
- Execution context is passed as parameter, never global
- Loaded modules are tracked explicitly (not discovered at runtime)

## Growth Path (for future)

This structure supports later additions:
- `optimizer/` — Sits above `loader/`
- `debugger/` — Reads state from `runtime/`
- `profiler/` — Instruments `executor/`

All additions follow the same dependency direction.
