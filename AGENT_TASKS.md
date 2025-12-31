# 10-Agent Parallel Implementation Tasks

This document contains 10 independent agent tasks for implementing the AtomVM POC.

**Use in Claude Code (web):**
1. Create 10 new "Tasks" in the web UI
2. Copy the **Global Rules** (below) into each task
3. Copy the specific **Agent N** section into the corresponding task
4. Start all 10 tasks in parallel
5. Each agent works independently

---

## Global Rules (prepend to EVERY agent task)

```
GLOBAL RULES FOR ALL AGENTS:

1. **Add-only constraint**: Do not modify or delete ANY existing files.
2. Use **ggen** to generate scaffolding:
   - If ggen.toml/schema/templates exist: run `ggen sync`
   - If not: Agent 1 creates them first
3. Implementation pattern:
   - `generated/atomvm_support/**/*.rs` = scaffolded by ggen
   - `src/atomvm_support_impl/**/*.rs` = hand-written implementation
4. Every deliverable includes:
   - Unit tests (in the impl file or `tests/` dir)
   - Integration/smoke tests (at least one per agent)
   - Doc stub (new file in `docs/`)
   - Example (if applicable, new file in `examples/`)
5. All `unsafe` blocks must have `// SAFETY:` invariant comment

ACCEPTANCE CHECK (before declaring done):
- Run: ggen sync (or verify it was run)
- Run: cargo test (all tests pass)
- Run: git diff --name-status
  - Output must show ONLY "A" (added) lines, NO modifications
- Confirm: No existing src/ files were edited
```

---

## Agent 1: ggen Ontology & Scaffolding

**Goal:** Create ggen configuration and templates so generated code provides the entire AtomVM support skeleton.

**Deliverables (add-only only):**

- `ggen.toml` (if missing; configure output → `generated/`)
- `schema/avmnif_atomvm.ttl` (RDF describing AtomVM modules, errors, functions)
- `queries/subsystems.sparql`, `queries/modules.sparql`, `queries/errors.sparql`,
  `queries/ffi_fns.sparql`, `queries/tests.sparql`, `queries/examples.sparql`
- `templates/rust/mod_tree.tera` (generates module tree)
- `templates/rust/error_enum.tera` (generates error enums)
- `templates/rust/ffi_wrapper.tera` (generates FFI wrapper stubs)
- `templates/rust/loader_scaffold.tera`, `runtime_scaffold.tera`
- `templates/rust/test_matrix.tera`, `example.tera`, `doc_stub.tera`

**Output goal:**
After `ggen sync`, should see:
```
generated/atomvm_support/
├── ffi/
├── runtime/
├── loader/
├── codecs/
├── resources/
├── nifs/
├── ports/
└── testing/
```

Each module has error enums, placeholder wrappers, test stubs, example stubs.

**Acceptance:**
- `ggen sync` runs without error
- Generated code compiles (as stubs; may use `unimplemented!()`)
- Next 9 agents can reference generated artifacts

---

## Agent 2: AtomVM Vendoring & Build Glue

**Goal:** Add AtomVM runtime source and build integration (add-only, no existing edits).

**Deliverables:**

- `atomvm/` directory (vendored source, snapshot, or reference)
- `atomvm/README.md` (provenance, version, how it's used)
- `docs/atomvm_vendor.md` (detailed integration plan)
- `src/atomvm_support_impl/ffi/build_helpers.rs` (utilities for linking, if needed)

**Implementation:**

If linking AtomVM requires editing `Cargo.toml` or root `build.rs`:
- Document the blocker in `docs/atomvm_vendor.md`
- Provide a **"hosted mode" POC** plan (e.g., separate binary, shim layer)
- Proceed with code generation anyway (tests can be mocked)

**Acceptance:**
- `cargo test` passes (no existing files edited)
- Documentation explains AtomVM vendoring and any limitations
- No panic or import errors from ggen artifacts

---

## Agent 3: FFI Surface & Safe Wrappers

**Goal:** Implement safe, auditable wrapper layer around generated FFI stubs.

**Deliverables:**

- `src/atomvm_support_impl/ffi/mod.rs`
- `src/atomvm_support_impl/ffi/wrappers.rs` (safe wrapper functions)
- `src/atomvm_support_impl/ffi/errors.rs` (error type + C↔Rust mapping)
- `src/atomvm_support_impl/ffi/tests.rs` (unit tests)
- `docs/ffi_contracts.md`

**Rules:**
- Generated bindings stay as-is (scaffold only)
- Wrappers are hand-written, safe-first
- Every `unsafe` block has `// SAFETY:` explaining invariants
- No `.unwrap()` or `panic!()` in production paths

**Tests must validate:**
- Null pointer rejection
- Invalid length rejection
- Error mapping determinism

**Acceptance:**
- `cargo test --lib atomvm_support_impl::ffi` passes
- All tests are deterministic (no flaky timing)

---

## Agent 4: Loader Validation & Module Registry

**Goal:** Bytecode validation, module loading, registry API.

**Deliverables:**

- `src/atomvm_support_impl/loader/validate.rs` (validate_bytes, format checks)
- `src/atomvm_support_impl/loader/load.rs` (load module into registry)
- `src/atomvm_support_impl/loader/registry.rs` (module storage/lookup)
- `src/atomvm_support_impl/loader/errors.rs` (BytecodeError, LoaderError types)
- `docs/loader.md`
- `fixtures/avm/` (add-only; golden bytecode + 3+ malformed test cases)
- `src/atomvm_support_impl/loader/tests.rs`

**Key function:**
```rust
pub fn validate_bytes(bytes: &[u8]) -> Result<(), BytecodeError>
```
Must never panic, even on malicious input.

**Tests:**
- Golden bytecode → `Ok(())`
- Truncated bytecode → specific `BytecodeError` variant
- Wrong magic → specific error
- Wrong version → specific error
- Registry stores/retrieves modules correctly

**Acceptance:**
- `cargo test --lib atomvm_support_impl::loader` passes
- Golden + negative tests run deterministically

---

## Agent 5: Host Runtime API (Boot, Load, Run, Observe)

**Goal:** Core runtime layer: boot VM, load module, execute function, observe result.

**Deliverables:**

- `src/atomvm_support_impl/runtime/host.rs` (AtomVmHost, boot logic)
- `src/atomvm_support_impl/runtime/scheduler.rs` (minimal execution scheduler)
- `src/atomvm_support_impl/runtime/errors.rs` (RuntimeError, ExecutionError)
- `src/atomvm_support_impl/runtime/tests.rs` (smoke + negative cases)
- `src/atomvm_support_impl/testing/smoke_runtime.rs` (integration harness)
- `examples/atomvm_boot_load_run.rs`
- `docs/runtime.md`

**Core scenario (in tests):**
```rust
let host = AtomVmHost::boot(config)?;
host.load_module("math", bytecode)?;
let result = host.execute_function("math", "add", [int(2), int(3)])?;
assert_eq!(result.as_int(), Some(5));
```

**Acceptance:**
- Smoke test runs and observes a result (success or typed error)
- Example compiles (even if gated by feature flag)
- No panics on bad input

---

## Agent 6: Term Conversion & Tagged ADT Codecs

**Goal:** Add term serialization/deserialization without touching existing `term.rs`/`tagged.rs`.

**Deliverables:**

- `src/atomvm_support_impl/codecs/term_codec.rs` (roundtrip encoder/decoder)
- `src/atomvm_support_impl/codecs/tagged_codec.rs` (tagged ADT serialization)
- `src/atomvm_support_impl/codecs/tests.rs` (roundtrip + negative tests)
- `docs/term_codecs.md`

**Tests must cover:**
- Roundtrip: `value → Term → value` equality
- Roundtrip for: ints, atoms, tuples, maps, lists
- One tagged struct example
- One tagged enum example
- Negative: wrong tag → error
- Negative: missing field → error
- Negative: type mismatch → error

**Acceptance:**
- `cargo test --lib atomvm_support_impl::codecs` passes
- Roundtrip tests are deterministic
- Negative tests all return typed errors (no panics)

---

## Agent 7: NIF & Port Collection Ergonomics

**Goal:** Wrapper macros/utilities for NIF and port registration (without editing existing ones).

**Deliverables:**

- `src/atomvm_support_impl/nifs/collection_ext.rs` (NIF registration helpers)
- `src/atomvm_support_impl/ports/collection_ext.rs` (port registration helpers)
- `examples/nif_add.rs` (example simple NIF)
- `examples/port_echo.rs` (example simple port)
- `docs/nif_port_extensions.md`

**Ergonomic goal:**
Make registering NIFs/ports as simple as possible. Examples:
```rust
// Show how to register a NIF "add" function
// Show how to register a port "echo" with message handler
```

**Acceptance:**
- Examples compile (even if behind feature flag)
- Documentation explains the extension approach
- No edits to existing `src/registry.rs` or port macros

---

## Agent 8: Resource Lifecycle & Registry Safety

**Goal:** Resource type registration, lifecycle testing, drop semantics.

**Deliverables:**

- `src/atomvm_support_impl/resources/lifecycle.rs` (lifecycle APIs)
- `src/atomvm_support_impl/resources/registry_ext.rs` (registry helpers)
- `src/atomvm_support_impl/resources/tests.rs`
- `docs/resources_lifecycle.md`

**Tests must verify:**
- Resource creation (allocate + register)
- Drop semantics (cleanup called on drop)
- Error propagation (invalid resource → typed error)
- Use-after-free rejection (if applicable to AtomVM model)

**Acceptance:**
- `cargo test --lib atomvm_support_impl::resources` passes
- Lifecycle tests are deterministic
- Documentation explains the safety model

---

## Agent 9: no_std Core Validation

**Goal:** Prove which parts are `no_std` compatible; add compile check.

**Deliverables:**

- `docs/no_std.md` (what is/isn't no_std, and why)
- `tests/no_std_compile.rs` OR `tools/no_std_check/Cargo.toml` (add-only)
  - Minimal `#![no_std]` consumer crate that compiles against claimed no_std parts
- If full no_std is impossible: document blockers (don't fix them, just document)

**Content of no_std.md:**
- Which modules/features are no_std-safe
- Which are not, and why
- What would be needed to make full no_std (but don't do it)
- How to run the compile check

**Acceptance:**
- `cargo test` or compile check passes deterministically
- Documentation is clear about the no_std scope

---

## Agent 10: Integration Suite & Press Release Proof Pack

**Goal:** Bundle all claims with executable proof (tests + examples).

**Deliverables:**

- `docs/press_release_proofs.md` (claim → test/example mapping)
- `src/atomvm_support_impl/testing/smoke_boot.rs`
- `src/atomvm_support_impl/testing/smoke_load.rs`
- `src/atomvm_support_impl/testing/smoke_terms.rs`
- `src/atomvm_support_impl/testing/smoke_ports.rs`
- `src/atomvm_support_impl/testing/smoke_resources.rs`
- `examples/` (aligned runnable examples for above)

**Content of press_release_proofs.md:**
```markdown
# AtomVM POC Press Release — Proof Map

## Claim 1: "Boot AtomVM from Rust"
**Evidence:**
- Test: `src/atomvm_support_impl/testing/smoke_boot.rs` → test_boot_initializes
- Example: `examples/atomvm_boot_minimal.rs`
- Run: `cargo test smoke_boot`

## Claim 2: "Load and execute .avm bytecode"
**Evidence:**
- Test: `smoke_load.rs` + `smoke_terms.rs`
- Example: `atomvm_boot_load_run.rs`
- Run: `cargo test smoke_load smoke_terms`

... (and so on for every press-release claim)
```

**Acceptance:**
- Reader can follow the proofs doc and run tests/examples
- All smoke tests pass
- Any claim not provable (requires non-add-only change) is explicitly marked

---

# Running These in Parallel

1. Open Claude Code (web)
2. Create 10 new "Tasks"
3. For each task n ∈ [1..10]:
   - Paste the **Global Rules** block above
   - Paste the **Agent N** section
   - Click "Start Task"
4. Monitor outputs; they run in parallel
5. Once all complete, run `cargo test` to verify everything integrates

---

# Expected Outcome

After all 10 agents complete:

```
generated/atomvm_support/          ← ggen scaffolds the shape
src/atomvm_support_impl/           ← agents implement the substance
├── ffi/
├── runtime/
├── loader/
├── codecs/
├── resources/
├── nifs/
├── ports/
└── testing/

examples/atomvm_*.rs               ← runnable proofs
docs/
├── atomvm_vendor.md
├── ffi_contracts.md
├── loader.md
├── runtime.md
├── term_codecs.md
├── nif_port_extensions.md
├── resources_lifecycle.md
├── no_std.md
└── press_release_proofs.md

fixtures/avm/                      ← test bytecode cases
```

**Total git status:** All additions, zero modifications to existing code.

**Proof:** `cargo test` passes entirely, every claim in press_release_proofs.md is executable.

---

# Questions?

If any agent gets stuck:
1. Check the **Global Rules** (did you edit existing files?)
2. Check the **Authority Hierarchy** from CLAUDE.md (did you respect it?)
3. Check PROJECT_CHARTER.md (did you deliver within scope?)

Good luck!
