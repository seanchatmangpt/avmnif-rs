---
name: avmvm-bytecode-loaders
description: Standardize how AtomVM bytecode/modules are loaded, validated, and registered when adding loaders, registries, or module introspection.
---

# AtomVM Bytecode Loaders

## Goals
- Deterministic loading: same bytes => same outcome
- Reject malformed input early with clear errors
- Keep validation separate from execution

## Required structure for loaders
Add new loader code under `src/atomvm_support/loader/` with:
- `validate.rs` — structural checks, header checks, size checks
- `load.rs` — actual load/registration functions
- `errors.rs` — typed errors for loader failures

## Validation requirements
At minimum:
- Check minimum length and header/magic (if applicable)
- Check version compatibility (if the format exposes it)
- Check declared lengths do not exceed buffer length
- Fail with a typed error that includes:
  - reason code
  - offset/field name if known

## API shape
Prefer:
- `fn validate(bytes: &[u8]) -> Result<ValidatedModule, LoaderError>`
- `fn load(vm: &mut VmHandle, module: ValidatedModule) -> Result<ModuleId, LoaderError>`

Keep `ValidatedModule` small and cheap (references into bytes OK if lifetime-safe).

## Tests
For each loader feature:
- "golden" test vectors: known-good bytes load successfully
- negative vectors: truncated, wrong header, wrong length fields
- fuzz-friendly entrypoint: a function that accepts arbitrary bytes and must never panic

## Output contract
Whenever adding module loading support:
- Add validation
- Add typed errors
- Add tests (golden + negative)
