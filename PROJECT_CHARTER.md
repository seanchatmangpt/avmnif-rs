# Project Charter — avmnif-rs AtomVM POC

## Objective
Prove that AtomVM can be hosted and exercised from Rust within avmnif-rs
without modifying any existing code.

## Scope

**IN:**
- Add-only integration of AtomVM runtime support
- Rust host APIs to load, run, and observe AtomVM execution
- Proof via tests and examples
- New directories and modules only

**OUT:**
- Refactors of existing code in `src/`
- Performance optimization beyond proof-level
- Production hardening or stability guarantees
- Breaking changes to existing public APIs

## Constraints
- **Add-only rule**: No existing files may be modified
- **Rust-first API surface**: All public interfaces are safe Rust
- **Deterministic behavior**: No flaky tests, reproducible results
- **Minimal new dependencies**: Avoid expanding beyond `paste` crate

## Success Criteria

The POC is complete when:

1. **Boot**: AtomVM can be initialized from Rust code
2. **Load**: A compiled Erlang/Elixir module can be loaded into the running instance
3. **Execute**: Functions within loaded modules can be called and return results
4. **Observe**: All failure modes are observable via typed `Result` values
5. **Boundary**: Terms cross the Rust ↔ Erlang boundary correctly
6. **Test**: Proof is demonstrated via tests that verify all above capabilities
7. **Example**: A minimal runnable example exists showing boot → load → execute → observe

## Non-Goals
- Full BEAM compatibility
- Erlang ecosystem completeness
- Hot code reloading
- Distributed Erlang
- API stability guarantees (this is a POC)

## Definition of Done

Work is complete when:
- All 5 document files exist and are committed
- `src/atomvm_support/` hierarchy is populated per `ARCHITECTURAL_SHAPE.md`
- Tests pass: `cargo test --features atomvm_support`
- Example runs: `cargo run --example atomvm_host_example`
- All invariants in `INVARIANTS.md` hold

---

**Acceptance**: POC is accepted when someone external to this conversation successfully:
- Reads PROJECT_CHARTER.md and understands what was built and why
- Reads the 4 supporting documents and finds zero ambiguity about what is allowed/forbidden
- Runs the test/example without investigation
