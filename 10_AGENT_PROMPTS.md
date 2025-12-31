# 10-Agent Parallel Execution Package — Ready to Launch

**Status**: All 10 agent prompts are complete and ready for parallel execution.

**How to use**: Copy each agent's prompt into a separate Claude Code task and start all 10 in parallel.

---

## Quick Launch Instructions

1. Open Claude Code (web)
2. Create 10 new independent Tasks
3. For each task, copy the corresponding **Agent N Prompt** section below
4. Start all 10 tasks simultaneously
5. Each agent works independently on their module

All agents follow the same **GLOBAL CONSTRAINTS**:
- Add-only: NO modifications to existing src/
- git diff --name-status must show ONLY "A" (added) lines
- All unsafe blocks need `// SAFETY:` comments
- No panics (.unwrap/.expect/panic!)
- Tests pass: `cargo test`

---

## Agent 1 Prompt: ggen Ontology and Scaffolding

**Goal:** Create ggen configuration + schema + queries + templates for code generation.

[Full prompt in previous message - Agent 1 section]

**Deliverables:**
- ggen.toml
- schema/avmnif_atomvm.ttl (RDF ontology)
- queries/*.sparql (5+ SPARQL query files)
- templates/rust/*.tera (8+ Tera templates)
- Generated artifacts under generated/atomvm_support/**

---

## Agent 2 Prompt: AtomVM Vendoring and Build Integration

**Goal:** Vendor AtomVM runtime source and provide build integration artifacts (add-only).

[Full prompt in previous message - Agent 2 section]

**Deliverables:**
- atomvm/ (vendored source or snapshot)
- atomvm/README.md
- docs/atomvm_vendor.md
- src/atomvm_support_impl/ffi/build_helpers.rs

---

## Agent 3 Prompt: FFI Surface and Safe Wrappers

**Goal:** Implement safe wrapper layer around FFI stubs with error mapping.

[Full prompt in previous message - Agent 3 section]

**Deliverables:**
- src/atomvm_support_impl/ffi/mod.rs
- src/atomvm_support_impl/ffi/errors.rs
- src/atomvm_support_impl/ffi/bindings.rs
- src/atomvm_support_impl/ffi/types.rs
- src/atomvm_support_impl/ffi/wrappers.rs
- src/atomvm_support_impl/ffi/tests.rs
- docs/ffi_contracts.md

---

## Agent 4 Prompt: Loader Validation and Module Registry

**Goal:** Implement bytecode validation and module registry (add-only).

[Full prompt in previous message - Agent 4 section]

**Deliverables:**
- src/atomvm_support_impl/loader/mod.rs
- src/atomvm_support_impl/loader/errors.rs
- src/atomvm_support_impl/loader/validate.rs
- src/atomvm_support_impl/loader/load.rs
- src/atomvm_support_impl/loader/registry.rs
- fixtures/avm/good.avm + malformed test cases
- docs/loader.md

---

## Agent 5 Prompt: Host Runtime Boot/Load/Execute

**Goal:** Implement core runtime (boot VM, load module, execute function).

[Full prompt in previous message - Agent 5 section]

**Deliverables:**
- src/atomvm_support_impl/runtime/mod.rs
- src/atomvm_support_impl/runtime/host.rs
- src/atomvm_support_impl/runtime/scheduler.rs
- src/atomvm_support_impl/runtime/errors.rs
- src/atomvm_support_impl/testing/smoke_runtime.rs
- examples/atomvm_boot_load_run.rs
- docs/runtime.md

---

## Agent 6 Prompt: Term Codec and Serialization

**Goal:** Add term encoding/decoding built on existing infrastructure (add-only).

[Full prompt in previous message - Agent 6 section]

**Deliverables:**
- src/atomvm_support_impl/codecs/mod.rs
- src/atomvm_support_impl/codecs/errors.rs
- src/atomvm_support_impl/codecs/term_codec.rs
- src/atomvm_support_impl/codecs/tagged_codec.rs
- docs/term_codecs.md

---

## Agent 7 Prompt: NIF and Port Registration Helpers

**Goal:** Create ergonomic wrapper macros and examples (add-only).

[Full prompt in previous message - Agent 7 section]

**Deliverables:**
- src/atomvm_support_impl/nifs/mod.rs
- src/atomvm_support_impl/nifs/collection_ext.rs
- src/atomvm_support_impl/ports/mod.rs
- src/atomvm_support_impl/ports/collection_ext.rs
- examples/nif_add.rs
- examples/port_echo.rs
- docs/nif_port_extensions.md

---

## Agent 8 Prompt: Resource Lifecycle and Registry Safety

**Goal:** Implement resource management APIs and lifecycle tests (add-only).

[Full prompt in previous message - Agent 8 section]

**Deliverables:**
- src/atomvm_support_impl/resources/mod.rs
- src/atomvm_support_impl/resources/lifecycle.rs
- src/atomvm_support_impl/resources/registry_ext.rs
- docs/resources_lifecycle.md

---

## Agent 9 Prompt: no_std Validation and Compile Check

**Goal:** Prove no_std compatibility and add deterministic compile check (add-only).

[Full prompt in previous message - Agent 9 section]

**Deliverables:**
- docs/no_std.md
- tests/no_std_compile.rs
- tools/no_std_check/Cargo.toml
- tools/no_std_check/src/lib.rs
- src/atomvm_support_impl/lib.rs (feature flags)

---

## Agent 10 Prompt: Integration Proof Pack and Final Assembly

**Goal:** Bundle all press-release claims with executable proof (add-only).

[Full prompt in previous message - Agent 10 section]

**Deliverables:**
- docs/press_release_proofs.md (10 claims → tests)
- src/atomvm_support_impl/testing/smoke_boot.rs
- src/atomvm_support_impl/testing/smoke_load.rs
- src/atomvm_support_impl/testing/smoke_terms.rs
- src/atomvm_support_impl/testing/smoke_ports.rs
- src/atomvm_support_impl/testing/smoke_resources.rs
- README_ATOMVM_POC.md

**Final Verification:**
```bash
cargo test --all          # All tests pass
git diff --name-status    # Only "A" (added) lines
```

---

## Expected Final State (All 10 Agents Complete)

```
src/atomvm_support_impl/
├── ffi/                  (Agent 3)
├── runtime/              (Agent 5)
├── loader/               (Agent 4)
├── codecs/               (Agent 6)
├── resources/            (Agent 8)
├── nifs/                 (Agent 7)
├── ports/                (Agent 7)
├── testing/              (Agent 10 + Agent 5)
└── lib.rs                (Agent 9)

generated/atomvm_support/  (Agent 1)
├── ffi/
├── runtime/
├── loader/
├── codecs/
├── resources/
├── nifs/
├── ports/
└── testing/

examples/
├── atomvm_boot_load_run.rs    (Agent 5)
├── nif_add.rs                 (Agent 7)
└── port_echo.rs               (Agent 7)

fixtures/avm/                  (Agent 4)
├── good.avm
├── truncated.avm
├── wrong_magic.avm
└── wrong_version.avm

docs/
├── atomvm_vendor.md           (Agent 2)
├── ffi_contracts.md           (Agent 3)
├── loader.md                  (Agent 4)
├── runtime.md                 (Agent 5)
├── term_codecs.md             (Agent 6)
├── nif_port_extensions.md     (Agent 7)
├── resources_lifecycle.md      (Agent 8)
├── no_std.md                  (Agent 9)
└── press_release_proofs.md    (Agent 10)

tools/no_std_check/            (Agent 9)
├── Cargo.toml
└── src/lib.rs

tests/
└── no_std_compile.rs          (Agent 9)

README_ATOMVM_POC.md           (Agent 10)
```

**Total git output:** Only "A" (added) lines, zero modifications to existing code.

**All 10 press-release claims verified with executable tests.**

---

## If You Run Into Issues

1. **Agent N is blocked on ggen:** Agent 1 must complete first. Once ggen infrastructure is in place, Agents 2-10 can reference generated artifacts.
2. **Module imports fail:** Ensure `src/atomvm_support_impl/mod.rs` exists and re-exports all submodules.
3. **Tests fail to compile:** Check that all modules follow the pattern in each agent's prompt.

---

## Commands to Verify Success

```bash
# After all 10 agents complete:

# 1. Verify all tests pass
cargo test --all

# 2. Verify add-only constraint
git diff --name-status | grep -v "^A"  # Should output nothing

# 3. Count additions
git diff --name-status | grep "^A" | wc -l  # Should be ~60+ files

# 4. Verify key files exist
ls -la docs/press_release_proofs.md
ls -la README_ATOMVM_POC.md

# 5. Final summary
echo "=== POC Complete ===" && \
  cargo test --all 2>&1 | grep -o "test result.*" && \
  echo "Add-only: $(git diff --name-status | grep -v '^A' | wc -l) modifications" && \
  echo "Files added: $(git diff --name-status | grep '^A' | wc -l)"
```

---

## Ready to Launch

All 10 prompts are self-contained and ready to paste into Claude Code tasks. You have the complete instruction set above (in previous messages) with all code examples, test cases, and documentation stubs.

**Start the 10 agents and they'll handle the rest!**
