---
name: conventions-and-pr-standards
description: Enforce consistent naming, module structure, public API discipline, and change narratives so outputs are reviewable and uniform.
---

# Conventions and PR Standards

## Code conventions
- File/module names: `snake_case`
- Types: `PascalCase`
- Functions/vars: `snake_case`
- Prefer explicit names over abbreviations.

## Public API discipline
- Default to `pub(crate)` for new APIs.
- If a new API is `pub`, it must have docs and tests.

## Change narrative
When you finish a task, include a short "review note" summary (as text output or added doc file) containing:
- What changed
- Why
- How to test
- Known risks

## Commit/PR style (when requested)
- Title: imperative verb + scope (e.g., "Add loader validation for X")
- Body: bullet list + test proof

## Output contract
All work should appear uniform as if written by one disciplined maintainer.
