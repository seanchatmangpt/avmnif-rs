//! Thesis Support Module
//!
//! Implements the complete "Boundary-Complete Determinism" PhD thesis system.
//! Contains working implementations of all thesis findings and contributions.
//!
//! # Authority Stack (DFLSS-Lean)
//!
//! - PROJECT_CHARTER.md: POC scope and success criteria
//! - SYSTEM_BOUNDARIES.md: Add-only rule, forbidden actions
//! - ARCHITECTURAL_SHAPE.md: Directory layout and dependencies
//! - INVARIANTS.md: Global truths that must never break
//! - CLAUDE.md: Execution doctrine for development
//!
//! # 10-Agent System
//!
//! Agent 2: Boundary testing with PingPongLoop
//! Agent 3: FFI safety ledger
//! Agent 4: Term morphism proofs
//! Agent 5: Error reconciliation
//! Agent 6: KGC calculus (core finding)
//! Agent 7: Proof pack with evidence
//! Agent 8: Invariant ledger
//! Agent 9: Stress and determinism tests
//! Agent 10: Final algebra and integration

pub mod boundary_testing;
pub mod safety_ledger;
pub mod kgc;
pub mod proof_pack;
