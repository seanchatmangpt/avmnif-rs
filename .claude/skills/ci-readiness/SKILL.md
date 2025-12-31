---
name: ci-readiness
description: Ensure new additions are deterministic, testable, and CI-friendly: no hidden local assumptions, stable commands, and clear run instructions.
---

# CI Readiness

## Goals
- Everything runs from a clean checkout
- Tests are deterministic
- No hidden local state required

## Rules
- Do not rely on developer machine state.
- Prefer repository-relative paths.
- Avoid time-sensitive tests unless controlled.
- If a toolchain dependency exists, document it in a new doc file under `docs/` (add-only).

## Test determinism
- Fixed seeds where randomness is used
- No network-required tests unless explicitly marked and isolated
- Avoid ordering-dependent tests

## Output contract
Every new feature must be:
- buildable with `cargo test`
- documented with exact commands to run
- accompanied by tests that pass reliably
