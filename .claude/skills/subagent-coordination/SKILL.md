---
name: subagent-coordination
description: Coordinate parallel agents by forcing clear task boundaries, file ownership, and merge-safe outputs when multiple agents work on the POC.
---

# Subagent Coordination

## Goals
- Prevent duplicated work
- Prevent conflicting additions
- Enable safe parallelism

## Task claiming protocol
When starting work:
- Declare:
  - objective
  - the new files/directories you will add
  - the acceptance tests you will run

## File ownership rule
- If another agent is adding within a directory, do not add files there.
- Prefer creating a new subdirectory for your work to avoid collisions.

## Merge-safe habits
- Keep modules small and isolated.
- Prefer additive test files rather than editing existing tests.

## Output contract
Every agent's output should be mergeable with minimal conflict risk.
