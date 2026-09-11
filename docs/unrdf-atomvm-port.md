# unrdf AtomVM → avmnif-rs port

This port moves the **portable runtime semantics** from `unrdf/packages/atomvm` into the `no_std` Rust boundary that can be reused by AtomVM NIF/port integrations. It does not copy environment-specific JavaScript machinery or RDF-domain semantics into `avmnif-rs`.

## DfCM boundary

The port preserves the decomposition:

```text
validation != planning != admission != actuation != observation
```

A validated message is not authority to call it. A restart plan is not a process restart. A hot-code candidate is not executable code replacement. A circuit admission is not the external operation. Runtime lifecycle state does not manufacture load/execute authority.

This keeps reversible information available until an explicit caller-owned actuation boundary.

## Source → destination

| unrdf source | avmnif-rs destination | Ported semantics |
| --- | --- | --- |
| `atomvm-runtime.mjs` | `src/atomvm_support/runtime_lifecycle.rs` | `Uninitialized → Loading → Ready → Executing → Ready`, explicit Error/Destroyed states and typed invalid transitions |
| `circuit-breaker.mjs` | `src/atomvm_support/circuit_breaker.rs` | threshold/open/half-open/close state machine, reset interval, exactly one half-open probe |
| `otp/supervisor.mjs`, `supervisor-tree.mjs` | `src/atomvm_support/supervisor.rs` | one-for-one / one-for-all / rest-for-one selection, permanent/transient/temporary restart policy, restart intensity |
| `hot-code-loader.mjs` | `src/atomvm_support/hot_code.rs` | signature identity, candidate generation, serialized pending revision, commit/rollback/error/unload state |
| `sla-monitor.mjs`, `roundtrip-sla.mjs` | `src/atomvm_support/sla.rs` | thresholded latency samples and deterministic aggregate evidence |
| `message-validator.mjs` | `src/atomvm_support/message_validation.rs` | generic RPC call/result and node-health structural contracts |

## Intentionally not ported

The following remain adapters above this crate rather than becoming additional core authority:

- browser and Service Worker lifecycle glue;
- Node.js child-process orchestration;
- Emscripten/WebAssembly module bootstrapping;
- filesystem/network module loading and executable hot swap;
- OpenTelemetry provider/exporter configuration;
- RDF triples, SPARQL, Oxigraph, and other `unrdf` domain contracts;
- JavaScript serialization and Zod-specific error representation;
- wall-clock ownership.

`avmnif-rs` is `no_std`, so time-sensitive controls consume caller-provided monotonic ticks. Hot-code signatures are opaque `[u8; 32]` values computed by an external verifier. RPC argument payloads remain owned by the existing AtomVM `Term`/message transport layer.

## Authority model

```text
untrusted/environment observation
        ↓
structural validation
        ↓
pure state/policy selection
        ↓
candidate/admission
        ↓
EXPLICIT CALLER AUTHORITY
        ↓
NIF / port / AtomVM / filesystem / process actuation
        ↓
receipt / health / SLA observation
```

The modules in this port stop before the actuation line. They can answer **what transition is admissible** or **what should be selected**, but cannot themselves perform external I/O, start processes, replace BEAM code, or call distributed nodes.

## Semantic non-duplication

Only generic AtomVM runtime controls were moved. RDF/SPARQL schemas remain in `unrdf` (or another domain owner), preventing `avmnif-rs` from becoming a second registry for knowledge-graph semantics. Existing `avmnif-rs` support modules remain source-compatible because the new modules are namespaced rather than glob-reexported.

## Verification targets

The port carries unit tests for:

1. valid and refused runtime lifecycle transitions;
2. circuit threshold, timeout, single half-open probe, success/failure transitions;
3. all three OTP supervisor restart strategies and restart types;
4. restart-intensity N+1 blocking and manual-restart selection;
5. hot-code monotonic generation, concurrent proposal refusal, stale commit refusal, rollback, unchanged signature refusal;
6. deterministic SLA aggregates without wall-clock access;
7. generic RPC and health-message structural validation.

A successful Rust test/CI run verifies only these pure boundaries. It does **not** by itself prove a real AtomVM node, NIF, executable code swap, network call, or supervisor restart occurred.
