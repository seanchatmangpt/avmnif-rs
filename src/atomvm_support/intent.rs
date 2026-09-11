//! Constructed AtomVM intents separated from cryptographic sealing and actuation.
//!
//! `topology != route != intent != seal != actuation`. This module turns an
//! already selected route into a canonical control-plane body. The payload is
//! represented by a digest so the portable layer need not own serialization.

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use super::evidence::{Digest32, Sealed};
use super::swarm::is_stable_id;

/// Intent body before externally supplied sealing evidence is attached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IntentDraft {
    pub cluster_id: String,
    pub source_id: String,
    pub target_id: String,
    pub route: Vec<String>,
    pub operation: String,
    pub payload_digest: Digest32,
    pub constructed_tick: u64,
}

/// An intent body bound to externally-computed digest evidence.
pub type ConstructedIntent = Sealed<IntentDraft>;

/// Typed refusal during intent construction only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntentRefusal {
    InvalidClusterId,
    InvalidSourceId,
    InvalidTargetId,
    EmptyOperation,
    EmptyRoute,
    RouteSourceMismatch,
    RouteTargetMismatch,
}

impl IntentDraft {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        cluster_id: &str,
        source_id: &str,
        target_id: &str,
        route: Vec<String>,
        operation: &str,
        payload_digest: Digest32,
        constructed_tick: u64,
    ) -> Result<Self, IntentRefusal> {
        if !is_stable_id(cluster_id) {
            return Err(IntentRefusal::InvalidClusterId);
        }
        if !is_stable_id(source_id) {
            return Err(IntentRefusal::InvalidSourceId);
        }
        if !is_stable_id(target_id) {
            return Err(IntentRefusal::InvalidTargetId);
        }
        if operation.is_empty() {
            return Err(IntentRefusal::EmptyOperation);
        }
        if route.is_empty() {
            return Err(IntentRefusal::EmptyRoute);
        }
        if route.first().map(String::as_str) != Some(source_id) {
            return Err(IntentRefusal::RouteSourceMismatch);
        }
        if route.last().map(String::as_str) != Some(target_id) {
            return Err(IntentRefusal::RouteTargetMismatch);
        }

        Ok(Self {
            cluster_id: cluster_id.to_string(),
            source_id: source_id.to_string(),
            target_id: target_id.to_string(),
            route,
            operation: operation.to_string(),
            payload_digest,
            constructed_tick,
        })
    }

    /// Attach a digest computed by an external canonicalizer/verifier.
    /// Sealing records identity; it does not prove that any operation ran.
    pub const fn seal(self, digest: Digest32) -> ConstructedIntent {
        Sealed::new(self, digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec;

    fn route() -> Vec<String> {
        vec!["source".to_string(), "gateway".to_string(), "target".to_string()]
    }

    #[test]
    fn route_endpoints_are_part_of_intent_admission() {
        let bad_source = IntentDraft::new(
            "cluster",
            "source",
            "target",
            vec!["other".to_string(), "target".to_string()],
            "atomvm.execute",
            Digest32([1; 32]),
            10,
        );
        assert_eq!(bad_source, Err(IntentRefusal::RouteSourceMismatch));

        let bad_target = IntentDraft::new(
            "cluster",
            "source",
            "target",
            vec!["source".to_string(), "other".to_string()],
            "atomvm.execute",
            Digest32([1; 32]),
            10,
        );
        assert_eq!(bad_target, Err(IntentRefusal::RouteTargetMismatch));
    }

    #[test]
    fn constructed_intent_requires_external_seal_and_can_be_reverified() {
        let body = IntentDraft::new(
            "cluster",
            "source",
            "target",
            route(),
            "atomvm.execute",
            Digest32([2; 32]),
            10,
        )
        .unwrap();
        let sealed = body.seal(Digest32([9; 32]));
        assert!(sealed.verify_with(|_| Digest32([9; 32])));
        assert!(!sealed.verify_with(|_| Digest32([8; 32])));
    }

    #[test]
    fn arbitrary_operation_is_preserved_for_later_policy_selection() {
        let body = IntentDraft::new(
            "cluster",
            "source",
            "target",
            route(),
            "custom.observe",
            Digest32::ZERO,
            1,
        )
        .unwrap();
        assert_eq!(body.operation, "custom.observe");
    }
}
