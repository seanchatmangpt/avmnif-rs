# CLAUDE.md - avmnif-rs Development Guide for AI Assistants

> **Last Updated**: 2025-12-31
> **Project**: avmnif-rs v0.4.0 - Safe NIF toolkit for AtomVM written in Rust
> **Purpose**: This document provides AI assistants (Claude Code) with comprehensive guidance on the codebase structure, conventions, and best practices.

---

## 🎯 Skill-Driven Development System

**This project uses a Skill-based governance system that overrides general guidelines.**

### Prime Directive
- **Add-only**: Never modify or delete existing files unless explicitly required by a skill.
- **Preserve**: All existing `src/` modules remain unchanged.
- **Extend**: New functionality is added through new directories and files.

### How Skills Work

Project Skills are located in `.claude/skills/<skill-name>/SKILL.md`. Each skill:
- Has a specific scope (e.g., FFI safety, testing discipline, error reporting)
- Contains binding constraints and output contracts
- Is loaded automatically when matching task descriptions

When working on a task, the applicable Skills **override** any conflicting guidance in this document.

### Skill Priority Order

If multiple Skills apply to a task, resolve conflicts in this order:

1. **avmnif-rs-architecture** — Module boundaries, add-only rule, directory structure
2. **rust-ffi-safety** — Unsafe code, FFI boundaries, pointer safety
3. **term-codec-correctness** — Term conversions, roundtrip testing, error handling
4. **error-reporting-patterns** — Typed errors, boundary mapping, error tests
5. **tests-and-benchmarks** — Testing discipline, test completeness
6. **document-with-context** — Documentation, inline comments, design intent
7. **examples-for-every-feature** — Runnable examples, minimal demos
8. **ci-readiness** — Determinism, reproducibility, clear commands
9. **conventions-and-pr-standards** — Naming, code style, review narratives
10. **subagent-coordination** — Task claiming, file ownership, merge safety

**Golden Rule**: Safety, add-only, and testability always win in a conflict.

### Where New Code Goes

When adding code, use these directories (do not restructure existing `src/`):
- `src/atomvm_support/` — Host-facing AtomVM integrations
- `src/atomvm_support/ffi/` — FFI bindings and safe wrappers
- `src/atomvm_support/loader/` — Module/bytecode loading
- `src/atomvm_support/runtime/` — Runtime orchestration
- `src/atomvm_support/testing/` — Integration harnesses
- `examples/` — Runnable demonstrations
- `docs/` — Feature documentation (new files only)
- `.claude/skills/` — Additional governance skills

### Using Skills in Practice

When you encounter a task:

1. **Identify applicable Skills** by matching task scope
2. **Read the relevant SKILL.md files** in full
3. **Apply their constraints** before writing code
4. **Output contract**: Every task must satisfy its Skill's output contract
5. **If conflicts arise**: Use the priority order above to decide

**Example**: If adding FFI code:
- Read `rust-ffi-safety/SKILL.md` first
- Then read `tests-and-benchmarks/SKILL.md`
- Apply both; if they conflict, safety wins

---

## Table of Contents

1. [Project Overview](#project-overview)
2. [Architecture & Design Principles](#architecture--design-principles)
3. [Directory Structure](#directory-structure)
4. [Core Modules](#core-modules)
5. [Module Dependency Graph](#module-dependency-graph)
6. [Naming Conventions](#naming-conventions)
7. [Code Style & Patterns](#code-style--patterns)
8. [Testing Infrastructure](#testing-infrastructure)
9. [Development Workflow](#development-workflow)
10. [Common Patterns & Idioms](#common-patterns--idioms)
11. [Error Handling](#error-handling)
12. [Documentation Standards](#documentation-standards)
13. [Build Configuration](#build-configuration)
14. [Frequently Used Patterns](#frequently-used-patterns)
15. [Key Gotchas & Tips](#key-gotchas--tips)

---

## Project Overview

### What is avmnif-rs?

**avmnif-rs** is a type-safe Rust library for building Native Implemented Functions (NIFs) and port drivers for [AtomVM](https://github.com/atomvm/atomvm), an open-source virtual machine for running Erlang/Elixir code on embedded systems and microcontrollers.

**Key Characteristics:**
- **Edition**: Rust 2021
- **MSRV**: Rust 1.70 or later
- **no_std Compatible**: Core functionality works in embedded contexts with `alloc` crate
- **License**: MIT
- **Single Dependency**: Only `paste` crate for macro utilities
- **~7,500 LOC**: Core library + comprehensive testing infrastructure

### Target Use Cases

1. **NIF Development**: Building native functions that extend AtomVM's capabilities
2. **Port Drivers**: Creating custom I/O drivers for hardware communication
3. **Resource Management**: Type-safe wrapper around AtomVM's resource system
4. **Data Serialization**: Seamless Rust ↔ Erlang data exchange via tagged maps

### Codebase Statistics

| Metric | Value |
|--------|-------|
| Core Modules | 10 files (~3,600 LOC) |
| Testing Infrastructure | 8 files (~3,900 LOC) |
| Documentation | 7 markdown files (~3,000 lines) |
| Test Functions | 99+ test cases |
| Public Traits | 6 major traits |
| Error Types | 5 custom enum types |

---

## Architecture & Design Principles

### Five Core Design Principles

The entire codebase is built on these principles. Reference these when making design decisions:

#### 1. **Generic by Design (Dependency Injection)**

No global state. All operations accept trait implementations as parameters:

```rust
// Good: Works with any AtomTableOps implementation
fn create_error_reply<T: AtomTableOps>(reason: &str, table: &T) -> Result<Term, NifError>

// Bad: Would be hardcoded to specific implementation
fn create_error_reply(reason: &str) -> Result<Term, NifError>
```

**Why**: Enables testing with `MockAtomTable` without runtime overhead. Production code uses `AtomTable`, test code uses mocks.

#### 2. **Memory Safety as Default**

- Unsafe code is minimized and only at FFI boundaries
- Public API surface is entirely safe Rust
- FFI calls are wrapped with null pointer checks and error mapping

**Guideline**: New code should follow this pattern. Mark unsafe functions with `// Safety:` comments explaining invariants.

#### 3. **Explicit Error Handling**

Every fallible operation returns a `Result` type. No panics or `.unwrap()` in production code paths.

**Error Type Hierarchy**:
- `NifError` - Generic NIF operation errors (maps to Erlang atoms)
- `AtomError` - Atom table operation failures
- `ResourceError` - Resource management failures
- `PortError` - Port communication failures
- `TaggedError` - Serialization errors with rich context

**Guideline**: Use appropriate error type for each domain. Implement `From` traits for automatic error conversion.

#### 4. **Type Safety Over Runtime Convenience**

Newtype wrappers prevent type confusion:

```rust
pub struct AtomIndex(pub u32);     // Not just u32
pub struct ProcessId(pub u32);     // Not just u32
pub struct PortId(pub u32);        // Not just u32
pub struct Term(pub usize);        // Not just usize
```

**Guideline**: When representing domain concepts, create newtype wrappers. Type system prevents mixing incompatible types.

#### 5. **Zero-Cost Abstractions**

All abstractions compile to the same code as hand-written FFI:

- Trait dispatch is monomorphized (not virtual)
- Generic code generates specialized versions
- Wrappers compile away with optimization

**Guideline**: Use generics and traits freely; they don't add runtime overhead with `-C opt-level=3`.

---

## Directory Structure

```
/home/user/avmnif-rs/
├── Cargo.toml              # Project manifest, version, dependencies
├── Cargo.lock              # Locked dependency versions
├── README.md               # Main documentation and quick start
├── LICENSE                 # MIT license
├── .gitignore              # Excludes /target, .DS_Store
│
├── src/                    # Main source code
│   ├── lib.rs              # Library entry point, module organization
│   ├── atom.rs             # Atom table interface (482 LOC)
│   ├── term.rs             # Term representation & conversion (791 LOC) ⭐ LARGEST
│   ├── context.rs          # Context management (378 LOC)
│   ├── resource.rs         # Resource type registration (728 LOC)
│   ├── port.rs             # Port communication utilities (636 LOC)
│   ├── tagged.rs           # Type-safe ADT serialization (516 LOC)
│   ├── registry.rs         # NIF registration macros (64 LOC)
│   ├── log.rs              # Logging utilities (25 LOC)
│   ├── macros.rs           # Placeholder for macro definitions (empty)
│   ├── c/
│   │   └── logshim.c       # C logging integration (small)
│   └── testing/            # Test infrastructure (only compiled in test mode)
│       ├── mod.rs          # Testing module organization
│       ├── mocks.rs        # Mock implementations (1,102 LOC) ⭐ TESTING CORE
│       ├── helpers.rs      # Test helper functions (428 LOC)
│       ├── fixtures.rs     # Test fixtures & data (570 LOC)
│       ├── nifs.rs         # NIF test utilities (398 LOC)
│       ├── resources.rs    # Resource testing (408 LOC)
│       ├── tagged.rs       # Tagged serialization tests (548 LOC)
│       └── ports.rs        # Port communication tests (415 LOC)
│
└── docs/                   # Feature documentation (7 markdown files)
    ├── atoms.md            # Atom table operations guide
    ├── tagged.md           # Type-safe ADT serialization guide
    ├── resources.md        # Resource management
    ├── ports.md            # Port communication patterns
    ├── nif_collection.md   # NIF registration macros
    ├── port_collection.md  # Port registration macros
    ├── port_memory.md      # Memory management for ports
    └── testing.md          # Testing infrastructure guide
```

### Key Observations

1. **Largest Module**: `term.rs` (791 LOC) - handles term encoding/decoding
2. **Test Infrastructure**: Comparable size to core code (~3,900 LOC) - reflects testing-first approach
3. **Minimal Dependencies**: Only `paste` crate for macro utilities
4. **Documentation**: 3,000+ lines of markdown guides for features
5. **Modular Organization**: Clear separation by responsibility (atom, term, port, resource, context, tagged)

---

## Core Modules

### Module Relationships

```
lib.rs (exports all public APIs)
 ├── atom.rs          [Foundation] Generic atom table interface
 ├── log.rs           [Utility] Logging infrastructure
 ├── term.rs          [Core] Term representation & conversion
 ├── context.rs       [Core] Port/NIF context management
 ├── resource.rs      [Feature] Resource lifecycle management
 ├── port.rs          [Feature] Port driver utilities
 ├── tagged.rs        [Feature] Erlang ADT serialization
 ├── registry.rs      [Macro] NIF collection registration
 └── testing/ (cfg(test) only)
     ├── mocks.rs     [Core] Mock implementations
     ├── helpers.rs   [Utilities] Test assertion helpers
     └── fixtures.rs  [Data] Pre-built test scenarios
```

### Module Summaries

#### `atom.rs` - Atom Table Operations (482 LOC)

**Purpose**: Safe Rust interface to AtomVM's atom storage system

**Core Trait**:
```rust
pub trait AtomTableOps {
    fn count(&self) -> usize;
    fn get_atom_string(&self, index: AtomIndex) -> Result<AtomRef<'_>, AtomError>;
    fn ensure_atom(&self, atom_data: &[u8]) -> Result<AtomIndex, AtomError>;
    fn find_atom(&self, atom_data: &[u8]) -> Result<AtomIndex, AtomError>;
    fn ensure_atom_str(&self, atom_str: &str) -> Result<AtomIndex, AtomError>;
    fn atom_equals(&self, atom_index: AtomIndex, data: &[u8]) -> bool;
    fn compare_atoms(&self, atom1: AtomIndex, atom2: AtomIndex) -> i32;
    // ... more methods
}
```

**Key Implementations**:
- `AtomTable` - Production FFI wrapper
- `MockAtomTable` (in testing) - Test implementation

**Guideline**: Always accept `impl AtomTableOps` rather than concrete types. This enables seamless testing.

**Common Pattern**:
```rust
fn my_function<T: AtomTableOps>(table: &T) -> Result<()> {
    let ok_atom = table.ensure_atom_str("ok")?;
    // Use atom...
    Ok(())
}
```

#### `term.rs` - Term Representation & Conversion (791 LOC) ⭐ LARGEST

**Purpose**: Bridge between Erlang's binary term format and Rust's high-level ADT

**Dual Representation**:
1. **Low-level**: `Term(usize)` - Direct bit-level encoding for FFI
2. **High-level**: `TermValue` enum - Functional, easier to manipulate

**TermValue ADT** (the most important type to understand):
```rust
pub enum TermValue {
    SmallInt(i32),                           // 32-bit integers
    Atom(AtomIndex),                         // Reference to atom table
    Nil,                                     // Empty list
    Pid(ProcessId),                          // Erlang process ID
    Port(PortId),                            // Erlang port ID
    Reference(RefId),                        // Erlang reference
    Tuple(Vec<TermValue>),                   // Ordered collection
    List(Box<TermValue>, Box<TermValue>),    // Cons cell: (Head, Tail)
    Map(Vec<(TermValue, TermValue)>),        // Key-value pairs
    Binary(Vec<u8>),                         // Binary data
    Function(FunctionRef),                   // Erlang function reference
    Resource(ResourceRef),                   // NIF resource
    Float(f64),                              // 64-bit floating point
    Invalid,                                 // Invalid/error term
}
```

**Key Methods**:
```rust
impl TermValue {
    // Constructors
    pub fn int(value: i32) -> Self
    pub fn atom<T: AtomTableOps>(name: &str, table: &T) -> Self
    pub fn tuple(elements: Vec<TermValue>) -> Self
    pub fn list(elements: Vec<TermValue>) -> Self

    // Pattern matching
    pub fn as_int(&self) -> Option<i32>
    pub fn as_atom(&self) -> Option<AtomIndex>
    pub fn as_tuple(&self) -> Option<&[TermValue]>
    pub fn tuple_arity(&self) -> usize

    // Functional operations (Rust-style)
    pub fn fold_list<T, F>(&self, init: T, f: F) -> T where F: Fn(T, &TermValue) -> T
    pub fn map_list<F>(&self, f: F) -> TermValue where F: Fn(&TermValue) -> TermValue + Clone
    pub fn filter_list<F>(&self, predicate: F) -> TermValue where F: Fn(&TermValue) -> bool + Clone
    pub fn list_to_vec(&self) -> Vec<TermValue>

    // Map operations
    pub fn map_get(&self, key: &TermValue) -> Option<&TermValue>
    pub fn map_set(&self, key: TermValue, value: TermValue) -> TermValue
}
```

**FFI Conversion**:
```rust
impl Term {
    pub fn to_value(self) -> NifResult<TermValue>           // Decode from FFI
    pub fn from_value(value: TermValue, heap: &mut Heap) -> NifResult<Self>  // Encode to FFI
}
```

**Guideline**: Use `TermValue` for manipulation, `Term` only at FFI boundaries. Never decode/encode unnecessarily.

**Common Pattern**:
```rust
// In a NIF function
pub fn my_nif(env: *mut ErlNifEnv, argc: c_int, argv: *const ERL_NIF_TERM) -> ERL_NIF_TERM {
    // At boundary: convert FFI Term to TermValue
    let input = Term(argv as usize).to_value()?;

    // In logic: manipulate TermValue
    let result = input.map_list(|v| v.as_int().map(|i| TermValue::int(i * 2)).unwrap_or(v.clone()));

    // At boundary: convert back to FFI Term
    Term::from_value(result, heap)?
}
```

#### `context.rs` - Context Management (378 LOC)

**Purpose**: Safe wrappers for AtomVM port/NIF contexts with platform data storage

**Opaque Context**:
```rust
#[repr(C)]
pub struct Context {
    _private: [u8; 0],
}
```

**Extension Trait**:
```rust
pub trait ContextExt {
    unsafe fn set_platform_data(&mut self, data: *mut c_void);
    unsafe fn get_platform_data(&self) -> *mut c_void;
    unsafe fn get_platform_data_as<T>(&self) -> *mut T;
    unsafe fn set_platform_data_box<T>(&mut self, data: Box<T>);
    unsafe fn take_platform_data_box<T>(&mut self) -> Option<Box<T>>;
    // ... more methods
}
```

**Helper Functions**:
```rust
pub fn with_platform_data<T: PlatformData, R, F>(ctx: &Context, f: F) -> Option<R>
    where F: FnOnce(&T) -> R

pub fn with_platform_data_mut<T: PlatformData, R, F>(ctx: &mut Context, f: F) -> Option<R>
    where F: FnOnce(&mut T) -> R
```

**RAII Pattern**:
```rust
pub struct ContextGuard { /* ... */ }
impl Drop for ContextGuard {
    fn drop(&mut self) { /* Auto cleanup */ }
}
```

**Guideline**: Use `ContextGuard` for automatic context cleanup. Use `with_platform_data` for type-safe access.

#### `resource.rs` - Resource Management (728 LOC)

**Purpose**: Type-safe wrapper around AtomVM's NIF resource system

**Resource Trait**:
```rust
pub trait ResourceManager: Send + Sync {
    fn init_resource_type(&mut self, env: *mut ErlNifEnv, name: &str,
        init: &ErlNifResourceTypeInit, flags: ErlNifResourceFlags)
        -> Result<*mut ErlNifResourceType, ResourceError>;
    fn alloc_resource(&self, resource_type: *mut ErlNifResourceType, size: c_uint)
        -> Result<*mut c_void, ResourceError>;
    fn make_resource(&self, env: *mut ErlNifEnv, obj: *mut c_void)
        -> Result<ERL_NIF_TERM, ResourceError>;
    // ... more methods
}
```

**Key Macros**:
```rust
resource_type!(RESOURCE_NAME, RustType, destructor_fn);
create_resource!(resource_type, data_expression);
get_resource!(env, term, resource_type);
make_resource_term!(env, resource_ptr);
```

**Guideline**: Register resource types at module initialization. Use macros to minimize FFI boilerplate.

#### `port.rs` - Port Communication (636 LOC)

**Purpose**: Safe Rust wrappers for AtomVM's port driver interface

**Core Functions**:
```rust
pub fn parse_gen_message(message: &Message) -> Result<(Term, Term, Term), NifError>
pub fn send_reply(ctx: &Context, pid: Term, reference: Term, reply: Term)
pub fn send_async_message(pid: u32, message: Term)
pub fn create_error_reply<T: AtomTableOps>(reason: &str, table: &T) -> Result<Term, NifError>
pub fn create_ok_reply<T: AtomTableOps>(data: Term, table: &T) -> Result<Term, NifError>
```

**Port Data Trait**:
```rust
pub trait PortData: PlatformData {
    fn handle_message(&mut self, message: &Message) -> PortResult {
        PortResult::Continue
    }
    fn is_active(&self) -> bool { true }
}
```

**Key Macros**:
```rust
port_collection!(port_name, init = fn, destroy = fn, create_port = fn, handler = fn)
simple_port!(port_name, data = Type, init_data = expr, init = fn, destroy = fn)
port_data!(PortName { field1: Type1, field2: Type2 })
```

**Guideline**: Use `port_collection!` macro to register drivers. Implement `PortData` for custom data.

#### `tagged.rs` - Type-Safe Serialization (516 LOC)

**Purpose**: Automatic Rust ↔ Erlang data exchange with type validation

**Core Trait**:
```rust
pub trait TaggedMap: Sized {
    fn to_tagged_map<T: AtomTableOps>(&self, table: &T) -> TaggedResult<TermValue>;
    fn from_tagged_map<T: AtomTableOps>(map: TermValue, table: &T) -> TaggedResult<Self>;
    fn type_name() -> &'static str;
}
```

**Implementations**:
- Primitives: `i32`, `String`, `bool`
- Collections: `Option<U>`, `Vec<U>`
- Custom types via derive macro (not implemented inline, but pattern supported)

**Example**:
```rust
impl TaggedMap for MyStruct {
    fn to_tagged_map<T: AtomTableOps>(&self, table: &T) -> TaggedResult<TermValue> {
        let mut map = vec![];
        map.push((TermValue::atom("type", table), TermValue::atom("my_struct", table)));
        map.push((TermValue::atom("field1", table), self.field1.to_tagged_map(table)?));
        Ok(TermValue::Map(map))
    }

    fn from_tagged_map<T: AtomTableOps>(map: TermValue, table: &T) -> TaggedResult<Self> {
        validate_type_discriminator(&map, "my_struct", table)?;
        let field1 = extract_string_field(&map, "field1", table)?;
        Ok(MyStruct { field1 })
    }

    fn type_name() -> &'static str { "my_struct" }
}
```

**Helper Functions**:
```rust
pub fn validate_type_discriminator<T: AtomTableOps>(map: &TermValue, expected: &str, table: &T) -> TaggedResult<()>
pub fn extract_string_field<T: AtomTableOps>(map: &TermValue, field: &str, table: &T) -> TaggedResult<String>
pub fn extract_int_field<T: AtomTableOps>(map: &TermValue, field: &str, table: &T) -> TaggedResult<i32>
pub fn to_snake_case(name: &str) -> String
```

**Guideline**: Use for structured data exchange. Type discriminators prevent runtime type errors.

---

## Module Dependency Graph

```
Testing Code (cfg(test) only):
├── testing/mocks.rs        (MockAtomTable, MockResourceManager)
├── testing/helpers.rs      (assert_*, test helpers)
├── testing/fixtures.rs     (predefined test data)
├── testing/nifs.rs
├── testing/resources.rs
├── testing/tagged.rs
└── testing/ports.rs

Core Production Dependencies:
lib.rs
 ├─→ atom.rs               [No dependencies]
 ├─→ log.rs                [No dependencies]
 ├─→ term.rs               (depends on: atom)
 ├─→ context.rs            [No dependencies]
 ├─→ resource.rs           (depends on: error mapping)
 ├─→ port.rs               (depends on: term, atom)
 ├─→ tagged.rs             (depends on: term, atom)
 └─→ registry.rs           [Macro, no dependencies]

External Dependencies:
└─→ paste = "1.0.15"        (For token pasting macros)
```

**Dependency Principle**: Modules depend on simpler modules. Circular dependencies are avoided.

---

## Naming Conventions

### Variables & Parameters (snake_case)

```rust
let atom_index = AtomIndex(42);
let result_code = 0u32;
let translate_table = &table;
let boxed_data = Box::new(data);
```

### Constants (UPPER_CASE)

```rust
const TERM_PRIMARY_MASK: usize = 0x3;
const TERM_INTEGER_TAG: usize = 0xF;
const TERM_BOXED_TUPLE: usize = 0x00;
```

### Functions (snake_case with verb prefixes)

```rust
pub fn get_atom_string(...)      // Accessor
pub fn set_platform_data(...)    // Mutator
pub fn ensure_atom_str(...)      // Create if not exists
pub fn find_atom(...)            // Search
pub fn create_port_context(...)  // Constructor
pub fn destroy_port_context(...) // Destructor
pub fn is_valid(...)             // Query
pub fn has_platform_data(...)    // Query
pub fn with_platform_data(...)   // Closure with context
```

### Types (CamelCase)

**Structs**:
```rust
pub struct AtomIndex(pub u32);           // Newtype
pub struct AtomRef<'a> { ... }          // Reference type
pub struct ContextGuard { ... }         // RAII guard
pub struct PortBuilder<T> { ... }       // Builder pattern
```

**Enums**:
```rust
pub enum AtomError { NotFound, AllocationFailed, ... }
pub enum TermValue { SmallInt(i32), Atom(AtomIndex), ... }
```

**Traits** (suffix with Ops, Ext, or leave generic):
```rust
pub trait AtomTableOps { ... }        // Operations trait
pub trait ContextExt { ... }          // Extension trait
pub trait ResourceManager { ... }     // Capability trait
pub trait TaggedMap { ... }           // Behavior trait
pub trait PlatformData { ... }        // Marker trait
pub trait PortData { ... }            // Behavior trait
```

**Type Aliases** (CamelCase):
```rust
pub type NifResult<T> = Result<T, NifError>;
pub type TaggedResult<T> = Result<T, TaggedError>;
```

### Modules & Files (snake_case)

```
src/atom.rs         → mod atom
src/term.rs         → mod term
src/port.rs         → mod port
src/testing/        → mod testing
src/testing/mocks.rs   → mod mocks (inside testing)
```

### Macros (snake_case)

```rust
nif_collection!(...)
port_collection!(...)
resource_type!(...)
atom_with_table!(...)
tuple!(...)
list!(...)
map!(...)
```

### Prefixes & Suffixes Pattern

| Pattern | Meaning | Examples |
|---------|---------|----------|
| `get_*` | Read-only accessor | `get_atom_string`, `get_resource_manager` |
| `set_*` | Mutable setter | `set_platform_data` |
| `ensure_*` | Create if absent | `ensure_atom`, `ensure_atom_str` |
| `find_*` | Search/lookup | `find_atom` |
| `create_*` | Constructor | `create_port_context` |
| `destroy_*` | Destructor | `destroy_port_context` |
| `is_*` | Boolean query | `is_valid`, `is_nil` |
| `has_*` | Existence check | `has_platform_data` |
| `with_*` | Closure/context | `with_platform_data` |
| `*_str` | String variant | `ensure_atom_str` (vs `ensure_atom`) |
| `*_safe` | Safe wrapper | `create_port_context_safe` |
| `Mock*` | Test implementation | `MockAtomTable`, `MockResourceManager` |
| `*Error` | Error type | `AtomError`, `NifError` |
| `*Result` | Result alias | `NifResult<T>`, `TaggedResult<T>` |

---

## Code Style & Patterns

### Module Organization

Every module follows this structure:

```rust
//! Module-level documentation explaining purpose and usage

// ── Section Comment ──────────────────────────────────────────────────────
// Core Trait/Type Definitions

pub trait MyTrait { /* ... */ }
pub struct MyStruct { /* ... */ }

// ── Another Section ──────────────────────────────────────────────────────
// Implementation Details

impl MyTrait for MyStruct { /* ... */ }

// ── Tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_something() { /* ... */ }
}
```

### Safety Annotations

Unsafe code must have `// Safety:` comments explaining invariants:

```rust
/// # Safety
/// The pointer must be valid and point to a real AtomVM atom table
pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
    AtomTable(ptr)
}
```

### Error Handling Pattern

Always use `Result` types for fallible operations:

```rust
pub fn my_function<T: AtomTableOps>(table: &T) -> Result<TermValue, NifError> {
    // Propagate errors with ?
    let atom_idx = table.ensure_atom_str("hello")?;

    // Match on specific errors if needed
    match table.find_atom_str("world") {
        Ok(idx) => { /* ... */ }
        Err(AtomError::NotFound) => {
            // Handle not found specifically
        }
        Err(e) => {
            // Convert other errors
            return Err(NifError::BadArg);
        }
    }

    Ok(TermValue::atom(atom_idx))
}
```

### Trait Implementations with Generics

```rust
// Accept trait objects instead of concrete types
pub fn process<T: AtomTableOps>(table: &T) -> Result<()> {
    let idx = table.ensure_atom_str("key")?;
    Ok(())
}

// Test calls it with MockAtomTable, production with AtomTable
// Both work seamlessly - this is the power of Rust generics!
```

### Newtype Pattern

Wrap domain concepts:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AtomIndex(pub u32);

impl AtomIndex {
    pub fn new(index: u32) -> Self { AtomIndex(index) }
    pub fn get(self) -> u32 { self.0 }
    pub fn is_valid(self) -> bool { self.0 != 0 }
}
```

### Builder Pattern

For complex construction:

```rust
pub struct PortBuilder<T> {
    data: T,
    user_data: u64,
    user_term: Term,
}

impl<T> PortBuilder<T> {
    pub fn new(data: T) -> Self { /* ... */ }
    pub fn build(self, global: &GlobalContext) -> *mut Context { /* ... */ }
    pub fn build_with_user_data(self, global: &GlobalContext, data: u64) -> *mut Context { /* ... */ }
    pub fn build_with_user_term(self, global: &GlobalContext, term: Term) -> *mut Context { /* ... */ }
}
```

### RAII Pattern

Automatic resource cleanup:

```rust
pub struct ContextGuard {
    ctx: *mut Context,
}

impl ContextGuard {
    pub unsafe fn new(ctx: *mut Context) -> Self { ContextGuard { ctx } }
    pub fn context(&self) -> &Context { unsafe { &*self.ctx } }
}

impl Drop for ContextGuard {
    fn drop(&mut self) {
        unsafe { destroy_port_context_safe(self.ctx); }
    }
}
```

### Functional Patterns

Prefer functional style for term manipulation:

```rust
// ✅ Good: Functional
let doubled = term.map_list(|v| {
    v.as_int().map(|i| TermValue::int(i * 2)).unwrap_or(v.clone())
});

// ✅ Also good: Fold for accumulation
let sum = list.fold_list(0i32, |acc, v| {
    acc + v.as_int().unwrap_or(0)
});

// ❌ Avoid: Imperative mutations
let mut result = Vec::new();
for item in list.list_to_vec() {
    result.push(transform(item));
}
```

---

## Testing Infrastructure

### Testing Philosophy

Tests are first-class citizens with ~3,900 LOC of test code comparable to core code size.

**Principles**:
1. **No Global State**: Each test gets fresh mock instances
2. **Dependency Injection**: Mocks implement same traits as production code
3. **Fixture-Based**: Pre-built test data for common scenarios
4. **Isolation**: Tests don't affect each other

### Running Tests

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_atom_table

# Run with output
cargo test -- --nocapture

# Run tests matching pattern
cargo test fixtures
```

### Test Organization

```
src/testing/
├── mod.rs              # Re-exports all testing utilities
├── mocks.rs            # MockAtomTable, MockResourceManager
├── helpers.rs          # assert_*, test assertion functions
├── fixtures.rs         # user_fixture(), config_fixture(), etc.
├── nifs.rs             # NIF testing utilities
├── resources.rs        # Resource management tests
├── tagged.rs           # Tagged serialization tests
└── ports.rs            # Port communication tests
```

### Mock Implementations

#### MockAtomTable

```rust
use avmnif::testing::*;

#[test]
fn test_with_mock_atom_table() {
    let table = MockAtomTable::new();

    // Works just like AtomTable from production
    let idx = table.ensure_atom_str("hello").unwrap();
    let atom_ref = table.get_atom_string(idx).unwrap();
    assert_eq!(atom_ref.as_str().unwrap(), "hello");
}
```

**Available Constructors**:
```rust
MockAtomTable::new()           // Pre-populated with common atoms
MockAtomTable::new_empty()     // Empty atom table
MockAtomTable::new_with_atoms(&["custom", "atoms"])  // Custom atoms
```

#### MockResourceManager

```rust
#[test]
fn test_resource_allocation() {
    let mut manager = MockResourceManager::new();

    // Simulate allocation failure
    manager.set_fail_alloc(true);

    // Test error handling
    let result = manager.alloc_resource(null_mut(), 100);
    assert!(result.is_err());
}
```

### Fixture System

Pre-built test data for common scenarios:

```rust
use avmnif::testing::*;

#[test]
fn test_with_fixture() {
    let table = MockAtomTable::new();

    // Pre-built user data
    let user = user_fixture(&table);

    // Extract and verify
    assert_int(user.tuple_get(0), 123);  // ID field
}
```

**Available Fixtures**:
```rust
user_fixture(&table)                           // User data
admin_user_fixture(&table)                     // Admin user
config_fixture(&table)                         // Configuration
mixed_data_list_fixture(&table)                // Various types
nested_structure_fixture(&table)               // Deep nesting
large_list_fixture(&table, 1000)              // Performance testing
large_map_fixture(&table, 100)                // Large maps

// Specialized fixtures
binary_fixtures::empty_binary()
binary_fixtures::text_binary()
pid_fixtures::self_pid()
ref_fixtures::local_ref()
scenarios::user_session_scenario(&table)
```

### Test Helper Functions

```rust
use avmnif::testing::*;

// Atom creation
let ok_atom = atom("ok", &table);
let atoms = atoms(&["ok", "error"], &table);

// Term creation
let int_list = int_list(&[1, 2, 3]);
let int_tuple = int_tuple(&[1, 2, 3]);
let atom_map = atom_map(&[("key", value)], &table);

// Assertions
assert_term_eq(&expected, &actual);
assert_atom_str(&term, "ok", &table);
assert_int(&term, 42);
assert_list_length(&term, 3);
assert_tuple_arity(&term, 2);
assert_map_has_key(&map, "key", &table);
assert_map_contains(&map, "key", &expected_value, &table);
```

### Writing New Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use avmnif::testing::*;

    #[test]
    fn test_my_function() {
        // Setup
        let table = MockAtomTable::new();

        // Execute
        let result = my_function(&table);

        // Verify
        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value.as_int(), Some(42));
    }
}
```

---

## Development Workflow

### Git Workflow

**Current Status**: Simple linear development on feature branches

```bash
# Current branch
git branch
# * claude/add-claude-documentation-fmTD8

# Before making changes
git pull origin claude/add-claude-documentation-fmTD8

# After changes
git add .
git commit -m "Descriptive commit message"
git push origin claude/add-claude-documentation-fmTD8
```

### Commit Message Conventions

**Pattern**: No strict convention, but follow these guidelines:

```
# Good: Clear, imperative mood
Add CLAUDE.md development guide
Implement tagged serialization for user types
Fix atom table initialization race condition
Update documentation for port drivers

# Avoid: Vague, past tense
fixed stuff
updated code
changed things
```

### Version Management

Current version in `Cargo.toml`:
```toml
[package]
version = "0.4.0"
```

**Semantic Versioning**:
- `0.4.0` = MAJOR.MINOR.PATCH
- Increment MAJOR for breaking changes
- Increment MINOR for new features
- Increment PATCH for bug fixes

**Release Process**:
1. Update `Cargo.toml` version
2. Commit with message: `v0.4.0 release`
3. Push to main branch
4. Tag: `git tag v0.4.0` (optional, not currently used)

### Building & Testing

```bash
# Development build
cargo build

# Optimized release build
cargo build --release

# Run all tests
cargo test

# Generate documentation
cargo doc --open

# Check code without building
cargo check

# Lint with Clippy
cargo clippy

# Format code
cargo fmt
```

### Minimum Supported Rust Version (MSRV)

**MSRV**: Rust 1.70 or later

Ensure changes work with:
```bash
rustup install 1.70
cargo +1.70 test
```

### Documentation Generation

```bash
# Generate and open HTML docs
cargo doc --no-deps --open

# Uses these settings from Cargo.toml:
# [package.metadata.docs.rs]
# all-features = true
```

---

## Common Patterns & Idioms

### Pattern 1: Dependency Injection with Traits

```rust
// ✅ Production code accepts trait
pub fn handle_message<T: AtomTableOps>(message: &str, table: &T) -> Result<()> {
    let atom = table.ensure_atom_str(message)?;
    Ok(())
}

// In tests
#[test]
fn test_handle_message() {
    let table = MockAtomTable::new();
    let result = handle_message("test", &table);
    assert!(result.is_ok());
}

// In production
let table = AtomTable::from_global();
handle_message("test", &table)?;
```

### Pattern 2: FFI Boundary Management

```rust
// At the FFI boundary (NIF function)
pub fn my_nif(env: *mut ErlNifEnv, argc: c_int, argv: *const ERL_NIF_TERM) -> ERL_NIF_TERM {
    // Convert FFI to Rust types
    let arg_term = Term(unsafe { *argv } as usize);
    match arg_term.to_value() {
        Ok(TermValue::SmallInt(n)) => {
            // Process in Rust
            let result = do_something(n);

            // Convert back to FFI
            match Term::from_value(TermValue::int(result), &mut heap) {
                Ok(term) => term.raw() as ERL_NIF_TERM,
                Err(_) => make_error_term("computation_failed", env),
            }
        }
        Err(_) => make_error_term("badarg", env),
    }
}
```

### Pattern 3: Error Mapping

```rust
// Convert domain error to NIF error
pub fn my_operation<T: AtomTableOps>(table: &T) -> Result<TermValue, NifError> {
    table.ensure_atom_str("key")
        .map_err(|_| NifError::BadArg)  // Map AtomError → NifError
        .map(|idx| TermValue::atom(idx))
}

// Or implement From for automatic conversion
impl From<AtomError> for NifError {
    fn from(_: AtomError) -> Self {
        NifError::BadArg
    }
}
```

### Pattern 4: Type-Safe Context Access

```rust
pub struct MyPortData {
    counter: u32,
}

impl PlatformData for MyPortData {
    fn cleanup(&mut self) {
        // Custom cleanup if needed
    }
}

// In port message handler
pub fn handle_port_message(ctx: &mut Context, message: &Message) {
    with_platform_data_mut::<MyPortData, _, _>(ctx, |data| {
        data.counter += 1;
    });
}
```

### Pattern 5: Functional List Processing

```rust
let list = TermValue::list(vec![
    TermValue::int(1),
    TermValue::int(2),
    TermValue::int(3),
]);

// Map over list
let doubled = list.map_list(|v| {
    v.as_int().map(|i| TermValue::int(i * 2)).unwrap_or(v.clone())
});

// Fold (accumulate)
let sum = list.fold_list(0i32, |acc, v| {
    acc + v.as_int().unwrap_or(0)
});

// Filter
let evens = list.filter_list(|v| {
    v.as_int().map(|i| i % 2 == 0).unwrap_or(false)
});
```

### Pattern 6: Tagged Map Serialization

```rust
// Convert Rust struct to Erlang map
let user_data = UserData {
    id: 123,
    name: "Alice".to_string(),
};

let map = user_data.to_tagged_map(&table)?;

// Convert Erlang map back to Rust struct
let recovered = UserData::from_tagged_map(map, &table)?;
assert_eq!(recovered.id, 123);
```

### Pattern 7: Result Chaining

```rust
fn process<T: AtomTableOps>(table: &T) -> Result<TermValue, NifError> {
    // Chain operations with ?
    let atom1 = table.ensure_atom_str("step1")?;
    let atom2 = table.ensure_atom_str("step2")?;

    Ok(TermValue::tuple(vec![
        TermValue::atom(atom1),
        TermValue::atom(atom2),
    ]))
}

// Caller handles error
match process(&table) {
    Ok(value) => println!("Success: {:?}", value),
    Err(e) => println!("Error: {:?}", e),
}
```

### Pattern 8: Guard Types for Resource Cleanup

```rust
pub fn use_context(global: &GlobalContext) -> Result<()> {
    let ctx_ptr = create_port_context_safe(global);
    let mut guard = unsafe { ContextGuard::new(ctx_ptr) };

    // Use context
    set_platform_data_box::<MyData>(&mut guard, Box::new(my_data))?;

    // Automatic cleanup on drop
    Ok(())
} // guard dropped here, context cleaned up automatically
```

---

## Error Handling

### Error Type Reference

| Type | Module | Use For | Maps To |
|------|--------|---------|----------|
| `NifError` | term.rs | Generic NIF errors | Erlang atoms |
| `AtomError` | atom.rs | Atom table failures | Rust code errors |
| `ResourceError` | resource.rs | Resource allocation/lifecycle | NifError |
| `PortError` | port.rs | Port communication | PortResult |
| `TaggedError` | tagged.rs | Serialization problems | Nested error context |
| `TermError` | term.rs | Term encoding/decoding | Internal errors |

### NifError Variants

```rust
pub enum NifError {
    BadArg,           // Invalid argument (maps to :badarg)
    BadArity,         // Wrong number of arguments (maps to :badarity)
    OutOfMemory,      // Allocation failed (maps to :enomem)
    SystemLimit,      // Hit system limit (maps to :system_limit)
    InvalidTerm,      // Term is invalid (maps to :badarg)
    Other(&'static str),  // Custom error message
}
```

**When Each Occurs**:
- `BadArg`: Wrong argument type, invalid value
- `BadArity`: Function expects N args, got M
- `OutOfMemory`: Heap allocation failed
- `SystemLimit`: Term too large, atom limit reached, etc.
- `InvalidTerm`: Term doesn't match expected structure
- `Other`: Custom errors from application logic

### Error Conversion Pattern

```rust
// Automatic conversion via From
impl From<AtomError> for NifError {
    fn from(_: AtomError) -> Self {
        NifError::BadArg
    }
}

// In code: just use ? to propagate
let idx = table.ensure_atom_str("key")?;  // AtomError → NifError automatically
```

### Error Handling in FFI Boundaries

```rust
pub fn my_nif(env: *mut ErlNifEnv, argc: c_int, argv: *const ERL_NIF_TERM) -> ERL_NIF_TERM {
    let table = AtomTable::from_global();

    match process(&table) {
        Ok(result) => {
            match Term::from_value(result, &mut heap) {
                Ok(term) => term.raw() as ERL_NIF_TERM,
                Err(e) => make_error_term("encoding_failed", env),
            }
        }
        Err(NifError::BadArg) => {
            unsafe {
                enif_make_badarg(env)
            }
        }
        Err(NifError::OutOfMemory) => {
            unsafe {
                enif_raise_exception(env, make_atom("enomem", env))
            }
        }
        Err(NifError::Other(msg)) => {
            make_error_term(msg, env)
        }
        // ... handle other error variants
    }
}
```

---

## Documentation Standards

### Module-Level Documentation

Every module starts with `//!` comments explaining:

```rust
//! # Module Name
//!
//! Brief description of what the module does.
//!
//! ## Design Philosophy
//!
//! Explain the design principles used in this module.
//!
//! ## Examples
//!
//! ```
//! use avmnif::my_module::*;
//! // Example code
//! ```
```

### Function Documentation

```rust
/// Brief description (one line).
///
/// More detailed explanation if needed.
///
/// # Arguments
///
/// * `param1` - Description
/// * `param2` - Description
///
/// # Returns
///
/// Description of return value.
///
/// # Errors
///
/// Description of when/why errors occur.
///
/// # Examples
///
/// ```
/// let result = my_function(args);
/// assert!(result.is_ok());
/// ```
pub fn my_function(param1: Type1, param2: Type2) -> Result<ReturnType, ErrorType> {
    // Implementation
}
```

### Safety Documentation

```rust
/// # Safety
///
/// The pointer must be:
/// - Non-null
/// - Properly aligned for the type
/// - Pointing to a valid instance of the type
pub unsafe fn from_raw(ptr: *mut c_void) -> Self {
    // ...
}
```

### Guideline: Documentation Gaps

Current documentation priorities:
- ✅ **Excellent**: Architecture, design, feature guides (in docs/)
- ✅ **Good**: Testing infrastructure, common patterns
- ⚠️ **Needs Work**: Inline function documentation, doc comments
- ⚠️ **Missing**: API reference document, error handling guide

### Documenting Your Changes

When adding code:
1. Add `///` doc comments to public items
2. Include examples for non-obvious functions
3. Document `// Safety:` for unsafe code
4. Update relevant markdown guides if adding features
5. Add tests that serve as usage examples

---

## Build Configuration

### Cargo.toml Structure

```toml
[package]
name = "avmnif-rs"
version = "0.4.0"
edition = "2021"
description = "Safe NIF toolkit for AtomVM written in Rust"
license = "MIT"
repository = "https://github.com/HeroesLament/avmnif-rs"
documentation = "https://docs.rs/avmnif-rs"
readme = "README.md"

[dependencies]
paste = "1.0.15"  # Only dependency: for macro token pasting

[dev-dependencies]
# None specified - tests use internal mocks

[lib]
# Default configuration, no special settings needed

[package.metadata.docs.rs]
all-features = true  # Documentation builds with all features enabled
```

### Building

```bash
# Development (unoptimized)
cargo build
# Output: target/debug/libavmnif_rs.a

# Release (optimized)
cargo build --release
# Output: target/release/libavmnif_rs.a

# For specific target
cargo build --target aarch64-unknown-linux-gnu --release
```

### Important: no_std Compatibility

The core library is `no_std` + `alloc`:

```rust
#![no_std]
extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;
```

**Guideline**: When adding code:
- Use `alloc::*` for heap allocations
- Avoid `std::*` for core functionality
- Use `core::*` for language primitives
- Test with: `cargo build --no-default-features`

---

## Frequently Used Patterns

### Creating Terms

```rust
// With helper functions
let int_term = TermValue::int(42);
let atom_term = TermValue::atom("ok", &table);
let nil_term = TermValue::Nil;
let empty_list = TermValue::list(vec![]);

// With macros
let tuple = tuple!(TermValue::int(1), TermValue::int(2));
let list = list!(1, 2, 3);  // Requires helper macro
let map = map!([("key", value)] &table);  // Requires helper macro

// Manual construction
let tuple = TermValue::tuple(vec![
    TermValue::int(1),
    TermValue::atom("key", &table),
]);
```

### Extracting Values

```rust
// Pattern matching
match term {
    TermValue::SmallInt(n) => println!("Int: {}", n),
    TermValue::Atom(idx) => println!("Atom index: {:?}", idx),
    TermValue::Tuple(elements) => println!("Tuple with {} elements", elements.len()),
    TermValue::Nil => println!("Empty list"),
    TermValue::List(head, tail) => println!("List with head: {:?}", head),
    _ => println!("Other term"),
}

// Using as_* methods
if let Some(n) = term.as_int() {
    println!("Integer: {}", n);
}

if let Some(atom_idx) = term.as_atom() {
    println!("Atom: {:?}", atom_idx);
}

// Tuple access
if let Some(elements) = term.as_tuple() {
    println!("Arity: {}", elements.len());
    if let Some(first) = elements.first() {
        println!("First element: {:?}", first);
    }
}
```

### Working with Atom Tables

```rust
// Ensure an atom exists
let ok_idx = table.ensure_atom_str("ok")?;

// Find an existing atom
let error_idx = table.find_atom_str("error")?;

// Compare atoms
if table.atom_equals_str(idx, "target") {
    println!("Atom matches!");
}

// Get atom string
let atom_ref = table.get_atom_string(ok_idx)?;
let s = atom_ref.as_str()?;
println!("Atom string: {}", s);

// Common atoms
let ok = atoms::ok(&table)?;
let error = atoms::error(&table)?;
let true_atom = atoms::true_atom(&table)?;
```

### Working with Ports

```rust
// Receive and parse message
let (pid, reference, command) = parse_gen_message(message)?;

// Send reply
send_reply(ctx, pid, reference, reply_term);

// Send async message
send_async_message(target_pid, message_term);

// Create responses
let ok_reply = create_ok_reply(data, &table)?;
let error_reply = create_error_reply("reason", &table)?;
```

### Working with Resources

```rust
// Register resource type
let res_type = resource_manager.init_resource_type(
    env,
    "my_resource",
    &resource_type_init(),
    0,
)?;

// Allocate resource
let resource = resource_manager.alloc_resource(res_type, size_of::<MyType>())?;

// Wrap in Erlang term
let term = resource_manager.make_resource(env, resource)?;

// Later: extract resource from term
let resource = resource_manager.get_resource(env, term, res_type)?;
```

### Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use avmnif::testing::*;

    #[test]
    fn test_my_feature() {
        // Setup mocks
        let table = MockAtomTable::new();

        // Use fixtures
        let user = user_fixture(&table);

        // Call code under test
        let result = my_function(&table, &user);

        // Verify results
        assert!(result.is_ok());
        assert_int(result.unwrap(), 42);
    }
}
```

---

## Key Gotchas & Tips

### ⚠️ Gotcha 1: Atom Table Ownership

**Problem**: Each function needs its own atom table implementation, but atom indices are only valid for their table.

```rust
let table1 = MockAtomTable::new();
let ok_idx = table1.ensure_atom_str("ok")?;

let table2 = MockAtomTable::new();
// ❌ WRONG: ok_idx from table1 is invalid in table2!
let atom = table2.get_atom_string(ok_idx)?;  // Could fail
```

**Solution**: Pass the same table instance through all operations.

```rust
fn process<T: AtomTableOps>(table: &T) -> Result<()> {
    let ok_idx = table.ensure_atom_str("ok")?;
    use_atom(ok_idx, table)?;  // Pass table along
    Ok(())
}
```

### ⚠️ Gotcha 2: List vs List Cell Confusion

**Problem**: `TermValue::List(head, tail)` is ONE cons cell, not a whole list.

```rust
// ❌ WRONG: This is a single list node, not a 3-element list
let list = TermValue::List(
    Box::new(TermValue::int(1)),
    Box::new(TermValue::int(2)),
);

// ✅ RIGHT: Use list() helper or construct properly
let list = TermValue::list(vec![
    TermValue::int(1),
    TermValue::int(2),
    TermValue::int(3),
]);

// ✅ Or manually build cons cells
let list = TermValue::List(
    Box::new(TermValue::int(1)),
    Box::new(TermValue::List(
        Box::new(TermValue::int(2)),
        Box::new(TermValue::Nil),  // Important!
    )),
);
```

### ⚠️ Gotcha 3: Test Isolation

**Problem**: Tests share no state, but mocks must be set up fresh each time.

```rust
#[test]
fn test_one() {
    let table = MockAtomTable::new();  // Fresh instance
    let idx = table.ensure_atom_str("key").unwrap();
    assert_eq!(idx, ...);
}

#[test]
fn test_two() {
    let table = MockAtomTable::new();  // Different instance!
    // Atom indices will be different from test_one
}
```

**Solution**: Create fresh mocks in each test. Don't share state between tests.

### 💡 Tip 1: Use Functional Patterns for Terms

Instead of imperative loops, use functional patterns:

```rust
// ✅ GOOD: Functional style
let processed = term.map_list(|v| v.as_int().map(|i| TermValue::int(i * 2)))
                    .filter_list(|v| v.as_int().map(|i| i > 10).unwrap_or(false));

// ❌ LESS IDIOMATIC: Imperative approach
let mut result = vec![];
for item in term.list_to_vec() {
    if let Some(i) = item.as_int() {
        if i * 2 > 10 {
            result.push(TermValue::int(i * 2));
        }
    }
}
```

### 💡 Tip 2: Type-Safe Indices

Always use wrapper types for indices:

```rust
// ✅ GOOD: Type-safe
fn use_atom(idx: AtomIndex, table: &impl AtomTableOps) -> Result<()> {
    let s = table.get_atom_string(idx)?;
    Ok(())
}

// ❌ WRONG: Could accidentally pass process ID as atom index
fn use_atom(idx: u32, table: &impl AtomTableOps) -> Result<()> {
    let s = table.get_atom_string(AtomIndex(idx))?;
    Ok(())
}
```

### 💡 Tip 3: Leverage Error Propagation

Use `?` to propagate errors up the stack:

```rust
// ✅ CLEAN
pub fn complex_operation<T: AtomTableOps>(table: &T) -> Result<TermValue, NifError> {
    let atom1 = table.ensure_atom_str("step1")?;
    let atom2 = table.ensure_atom_str("step2")?;
    let atom3 = table.ensure_atom_str("step3")?;

    Ok(TermValue::tuple(vec![
        TermValue::atom(atom1),
        TermValue::atom(atom2),
        TermValue::atom(atom3),
    ]))
}

// ❌ VERBOSE
pub fn complex_operation<T: AtomTableOps>(table: &T) -> Result<TermValue, NifError> {
    let atom1 = match table.ensure_atom_str("step1") {
        Ok(a) => a,
        Err(e) => return Err(NifError::BadArg),
    };
    let atom2 = match table.ensure_atom_str("step2") {
        Ok(a) => a,
        Err(e) => return Err(NifError::BadArg),
    };
    // ...
}
```

### 💡 Tip 4: Test Error Paths

Don't just test happy path:

```rust
#[test]
fn test_error_handling() {
    let table = MockAtomTable::new_empty();  // No atoms!

    // Test that it fails gracefully
    let result = my_function(&table);
    assert!(result.is_err());
}
```

### 💡 Tip 5: Use Builder Pattern for Complex Setup

```rust
// ✅ READABLE
let port = PortBuilder::new(my_data)
    .build_with_user_term(global, my_term);

// ❌ CONFUSING: What does each argument mean?
let port = create_port_with_data_and_term(global, my_data, my_term);
```

### 💡 Tip 6: Keep Unsafe Code Minimal

All unsafe is in FFI boundaries:

```rust
// ✅ GOOD: Unsafe only where necessary
pub fn from_raw(ptr: *mut c_void) -> Self {
    AtomTable(ptr)  // Minimal unsafe
}

// ❌ BAD: Unnecessary unsafe in business logic
pub fn process(data: *mut MyData) {
    unsafe {
        (*data).field = 42;  // Could use &mut instead
    }
}
```

### 💡 Tip 7: Document Design Decisions

Use module-level comments to explain why:

```rust
//! # Port Driver Module
//!
//! This module provides safe wrappers around AtomVM's port driver interface.
//!
//! ## Design Notes
//!
//! We use a trait-based approach (`PortData` trait) to allow different port
//! implementations without code duplication. This enables testing with mock
//! implementations while production code uses real FFI.
```

---

## Summary: AI Assistant Guidelines

When working with avmnif-rs:

### Do ✅

1. **Accept trait bounds** over concrete types: `impl AtomTableOps` not `&AtomTable`
2. **Use ? operator** for error propagation
3. **Prefer functional patterns** for term manipulation (`map_list`, `fold_list`, etc.)
4. **Create tests with mocks** - don't skip test writing
5. **Document unsafe code** with `// Safety:` comments
6. **Use newtype wrappers** for domain types (`AtomIndex`, not `u32`)
7. **Follow naming conventions** - they help readers
8. **Reference module docs** when confused about design

### Don't ❌

1. **Call .unwrap()** in production code - use `?` instead
2. **Create global state** - use dependency injection
3. **Mix Term and TermValue** at FFI boundaries - convert at boundary only
4. **Share atom indices** between different atom tables
5. **Write imperative code** for term manipulation - use functional patterns
6. **Ignore error types** - use appropriate error type for domain
7. **Duplicate atom table** instances - pass single reference
8. **Use unsafe unnecessarily** - only at FFI boundary

### Reference Material

- **Architecture**: This file (CLAUDE.md)
- **Design Details**: `docs/` directory (atoms.md, tagged.md, port_collection.md, etc.)
- **API Surface**: Source files with `///` doc comments
- **Examples**: Tests in `src/testing/`
- **Patterns**: Existing code in modules

---

**End of CLAUDE.md**

> **File Version**: 1.0
> **Generated**: 2025-12-31
> **Based on**: 10-agent codebase analysis
> **Project**: avmnif-rs v0.4.0

This document should be updated whenever significant architectural changes or new patterns emerge in the codebase.
