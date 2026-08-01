# ggen Integration for avmnif-rs

**Ontology-Driven Code Generation for NIF Functions**

This document describes how to use ggen to generate safe, working Rust NIF code from RDF ontologies.

---

## Overview

**ggen** transforms RDF ontologies into working Rust code, tests, and documentation. Instead of manually writing boilerplate for each NIF function, you:

1. **Define your functions as RDF triples** in Turtle format
2. **Run the code generator** to produce working Rust code
3. **Implement the business logic** in the generated stub functions
4. **Tests and docs are automatically generated** from the ontology

### Benefits

- **Single Source of Truth**: Define NIF functions once, generate everywhere
- **Type Safety**: Ontologies capture signatures, arity, and error modes
- **Consistency**: All generated code follows the same patterns
- **Maintenance**: Regenerate from updated ontologies without manual edits
- **Documentation**: API docs auto-generated from descriptions in RDF

---

## Quick Start (5 minutes)

### 1. Create an Ontology

Add your NIF functions to `schema/example-math-nifs.ttl`:

```turtle
@prefix ex: <https://avmnif-rs.io/ns#> .

ex:add a ex:NifFunction ;
    rdfs:label "Add" ;
    rdfs:comment "Add two integers" ;
    ex:erlangName "add" ;
    ex:rustName "nif_add" ;
    ex:arity 2 ;
    ex:canFail true ;
    ex:returnType ex:SmallInt ;
    ex:category "Arithmetic" ;
    ex:description "Adds two signed 32-bit integers" .
```

### 2. Run the Generator

```bash
python3 ggen_codegen.py sync
```

Output:
```
✅ Found 2 ontology file(s)
📖 Parsed example-math-nifs.ttl: 5 NIF(s)
✨ Total NIFs loaded: 5
✏️  Generated: src/generated/math_nifs.rs
✏️  Generated: src/generated/tests.rs
📄 Generated: docs/generated/generated-math-nifs.md
📄 Generated: src/generated/math_nifs.erl
✅ Code generation complete!
```

### 3. Implement Your Functions

Edit `src/generated/math_nifs.rs` and fill in the TODOs:

```rust
/// Add
pub fn nif_add(ctx: &mut Context, args: &[Term]) -> NifResult<Term> {
    if args.len() != 2 {
        return Err(NifError::BadArity);
    }

    let a = args[0].to_value()?.as_int().ok_or(NifError::BadArg)?;
    let b = args[1].to_value()?.as_int().ok_or(NifError::BadArg)?;

    let result = TermValue::int(a + b);
    Ok(Term::from_value(result, &mut ctx.heap)?)
}
```

### 4. Test It

```bash
cargo test
```

---

## Architecture

```
schema/                          # RDF Ontologies
├── avmnif-core.ttl             # Core vocabulary
└── example-math-nifs.ttl       # Your NIF definitions

templates/                       # Tera templates (for future use)
├── nif-functions.tera          # NIF function template
├── nif-tests.tera              # Test template
├── nif-docs.tera               # Documentation template
├── nif-collection.tera         # Collection registration
└── erlang-module.tera          # Erlang stub template

ggen_codegen.py                  # Python code generator

src/generated/                   # Generated artifacts (gitignored)
├── math_nifs.rs                # Generated NIF functions
├── tests.rs                     # Generated test stubs
└── math_nifs.erl               # Generated Erlang module

docs/generated/                  # Generated documentation
└── generated-math-nifs.md      # Auto-generated API docs
```

---

## Ontology Structure

### Core Concepts

**RDF Ontologies describe:**

1. **NifFunction** - A native function callable from Erlang
2. **PortDriver** - A custom I/O port (for future use)
3. **ResourceType** - A managed resource (for future use)
4. **ErrorType** - Error conditions (for future use)
5. **TermType** - AtomVM term types

### Example: Complete NIF Definition

```turtle
@prefix ex: <https://avmnif-rs.io/ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

ex:add a ex:NifFunction ;
    rdfs:label "Add" ;
    rdfs:comment "Add two integers" ;
    ex:erlangName "add" ;              # Function name in Erlang
    ex:rustName "nif_add" ;            # Function name in Rust
    ex:arity 2 ;                       # Number of arguments
    ex:canFail true ;                  # Can return error
    ex:returnType ex:SmallInt ;        # Return type
    ex:category "Arithmetic" ;         # For documentation
    ex:description "Adds two signed 32-bit integers" ;
    ex:parameter [
        rdf:value "arg0" ;
        ex:paramIndex 0 ;
        ex:paramName "a" ;
        ex:paramType ex:SmallInt ;
        ex:paramDoc "First integer"
    ] ;
    ex:parameter [
        rdf:value "arg1" ;
        ex:paramIndex 1 ;
        ex:paramName "b" ;
        ex:paramType ex:SmallInt ;
        ex:paramDoc "Second integer"
    ] ;
    ex:examples "math:add(5, 3)  # => 8" .
```

### Available Term Types

| Type | Rust | Erlang |
|------|------|--------|
| `SmallInt` | `i32` | `integer` |
| `Atom` | `AtomIndex` | `atom` |
| `Nil` | `()` | `[]` |
| `Tuple` | `Vec<TermValue>` | `tuple` |
| `List` | `Vec<TermValue>` | `list` |
| `Map` | `Vec<(TermValue, TermValue)>` | `map` |
| `Binary` | `Vec<u8>` | `binary` |
| `Float` | `f64` | `float` |
| `Pid` | `ProcessId` | `pid` |
| `Port` | `PortId` | `port` |

---

## Code Generation Commands

### Generate All Artifacts

```bash
python3 ggen_codegen.py sync
```

Options:
- `--dry-run`: Preview changes without writing files
- `--verbose`: Show detailed output

### Dry Run (Preview)

```bash
python3 ggen_codegen.py sync --dry-run
```

Output:
```
📝 [DRY-RUN] Would generate: src/generated/math_nifs.rs
📝 [DRY-RUN] Would generate: src/generated/tests.rs
📝 [DRY-RUN] Would generate: docs/generated/generated-math-nifs.md
```

---

## Workflow

### Adding a New NIF Function

1. **Edit** `schema/example-math-nifs.ttl` or create a new ontology file
2. **Add** a new `ex:NifFunction` resource with all properties
3. **Run** `python3 ggen_codegen.py sync`
4. **Implement** the function in the generated `src/generated/*.rs`
5. **Test** with `cargo test`

### Regenerating After Changes

The generator overwrites `src/generated/*.rs` files. If you've made changes:

**Option A: Regenerate and re-implement**
```bash
python3 ggen_codegen.py sync
# Then re-implement the TODOs
```

**Option B: Manual merge**
```bash
git diff src/generated/math_nifs.rs  # See what changed
# Manually integrate changes
```

### CI/CD Integration

In your `.github/workflows/codegen.yml`:

```yaml
name: Code Generation Check
on: [push, pull_request]

jobs:
  ggen:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run ggen
        run: python3 ggen_codegen.py sync --dry-run
      - name: Check for changes
        run: git diff --exit-code
```

---

## Implementation Guide

### Step 1: Parse Arguments

```rust
pub fn nif_add(ctx: &mut Context, args: &[Term]) -> NifResult<Term> {
    if args.len() != 2 {
        return Err(NifError::BadArity);
    }

    // Convert FFI terms to Rust values
    let a = args[0].to_value()?.as_int().ok_or(NifError::BadArg)?;
    let b = args[1].to_value()?.as_int().ok_or(NifError::BadArg)?;

    // ... your logic ...
}
```

### Step 2: Implement Logic

```rust
    // Safe arithmetic with overflow checking
    let result = a.checked_add(b).ok_or(NifError::OutOfMemory)?;
    let result = TermValue::int(result);
```

### Step 3: Return Result

```rust
    // Convert back to FFI term
    Ok(Term::from_value(result, &mut ctx.heap)?)
}
```

### Full Working Example

```rust
pub fn nif_add(ctx: &mut Context, args: &[Term]) -> NifResult<Term> {
    if args.len() != 2 {
        return Err(NifError::BadArity);
    }

    let a = args[0].to_value()?.as_int().ok_or(NifError::BadArg)?;
    let b = args[1].to_value()?.as_int().ok_or(NifError::BadArg)?;

    let result = a.checked_add(b).ok_or(NifError::OutOfMemory)?;
    let result = TermValue::int(result);

    Ok(Term::from_value(result, &mut ctx.heap)?)
}
```

---

## Testing Generated Code

### Generated Test Structure

```rust
#[cfg(test)]
mod math_nifs_tests {
    use avmnif_rs::testing::*;

    #[test]
    fn test_nif_add() {
        // TODO: Implement
        assert!(true);
    }
}
```

### Implement Tests

```rust
#[test]
fn test_nif_add() {
    let table = MockAtomTable::new();
    let mut ctx = test_context();

    let args = vec![
        TermValue::int(5),
        TermValue::int(3),
    ];

    let result = nif_add(&mut ctx, &args);
    assert!(result.is_ok());

    let value = result.unwrap().to_value().unwrap();
    assert_eq!(value.as_int(), Some(8));
}
```

---

## Extending ggen

### Adding New Term Types

1. Add to `schema/avmnif-core.ttl`:

```turtle
ex:MyType a ex:TermType ;
    rdfs:label "My Type" ;
    ex:rustType "MyRustType" ;
    ex:erlangType "my_type" .
```

2. Update templates and generator

### Supporting Port Drivers

The ontology already defines `ex:PortDriver`. Extend the generator to create:

```rust
pub trait PortData {
    fn handle_message(&mut self, message: &Message) -> PortResult;
}
```

### Supporting Resources

Similarly for `ex:ResourceType`:

```rust
pub fn register_resource<T: Send + Sync>(
    env: *mut ErlNifEnv,
    name: &str,
) -> Result<*mut ErlNifResourceType, ResourceError>
```

---

## Troubleshooting

### "No ontology files found"

```bash
# Make sure schema/ directory exists
mkdir -p schema/

# And has .ttl files
ls schema/
```

### "No NIF functions found"

- Check that ontology files use `a ex:NifFunction`
- Verify Turtle syntax is valid

### Generated code won't compile

- Check that all required properties are in the ontology
- Verify type names match `ex:TermType` definitions
- Look for TODO comments in generated code

---

## File Structure After Setup

```
avmnif-rs/
├── ggen.toml                      # ggen configuration
├── ggen_codegen.py                # Python generator
├── GGEN.md                        # This file
│
├── schema/                        # RDF ontologies
│   ├── avmnif-core.ttl           # Core vocabulary
│   └── example-math-nifs.ttl     # Example NIFs
│
├── templates/                    # Tera templates
│   ├── nif-functions.tera        # NIF template
│   ├── nif-tests.tera
│   ├── nif-docs.tera
│   ├── nif-collection.tera
│   └── erlang-module.tera
│
├── src/
│   ├── generated/                # Generated code (auto)
│   │   ├── math_nifs.rs          # Generated NIFs
│   │   ├── tests.rs              # Generated tests
│   │   └── math_nifs.erl         # Generated Erlang
│   └── ... (existing code)
│
└── docs/
    ├── generated/                # Generated docs
    │   └── generated-math-nifs.md
    └── ... (existing docs)
```

---

## Next Steps

1. **Define your NIFs** in RDF ontologies
2. **Run the generator**: `python3 ggen_codegen.py sync`
3. **Implement functions** in generated code
4. **Test thoroughly**: `cargo test`
5. **Document** with ontology descriptions

For questions about ontology syntax, see `CLAUDE.md` and `schema/avmnif-core.ttl`.

---

## References

- **RDF/Turtle**: https://www.w3.org/TR/turtle/
- **avmnif-rs API**: See docs/ directory
- **AtomVM NIF Guide**: https://github.com/atomvm/atomvm/wiki/Nifs
- **CLAUDE.md**: Architecture and patterns guide
