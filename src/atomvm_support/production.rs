//! Production admission primitives for v26.9.11.
//!
//! This module does not mint authority. It verifies caller-supplied authority
//! evidence through a caller-owned verifier, enforces intent binding, expiry,
//! and bounded replay protection, and emits an [`ExecutionPermit`] that native
//! host adapters may require before crossing the DO boundary.

use alloc::collections::VecDeque;
use alloc::string::{String, ToString};

use super::evidence::Digest32;

/// Wire/protocol generation for externally serialized production envelopes.
pub const PRODUCTION_PROTOCOL_VERSION: u16 = 1;

/// Opaque externally-issued authority evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthorityEvidence {
    pub protocol_version: u16,
    pub authority_ref: String,
    pub intent_digest: Digest32,
    pub nonce: [u8; 16],
    pub expires_tick: u64,
}

impl AuthorityEvidence {
    pub fn new(
        authority_ref: &str,
        intent_digest: Digest32,
        nonce: [u8; 16],
        expires_tick: u64,
    ) -> Result<Self, ProductionRefusal> {
        if authority_ref.trim().is_empty() {
            return Err(ProductionRefusal::AuthorityRefMissing);
        }
        if nonce == [0; 16] {
            return Err(ProductionRefusal::ZeroNonce);
        }
        Ok(Self {
            protocol_version: PRODUCTION_PROTOCOL_VERSION,
            authority_ref: authority_ref.to_string(),
            intent_digest,
            nonce,
            expires_tick,
        })
    }
}

/// Evidence whose external verifier has admitted identity/authority semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedAuthority {
    evidence: AuthorityEvidence,
}

impl VerifiedAuthority {
    pub fn evidence(&self) -> &AuthorityEvidence {
        &self.evidence
    }
}

/// A permit proving authority verification + replay admission for one intent.
/// It remains evidence; possession does not itself perform actuation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionPermit {
    pub protocol_version: u16,
    pub authority_ref: String,
    pub intent_digest: Digest32,
    pub nonce: [u8; 16],
    pub admitted_tick: u64,
    pub expires_tick: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionRefusal {
    AuthorityRefMissing,
    ZeroNonce,
    ProtocolVersionMismatch,
    IntentDigestMismatch,
    AuthorityExpired,
    AuthorityVerificationFailed,
    ReplayDetected,
    ReplayCapacityZero,
}

/// Verify an externally-issued authority envelope without choosing a signature
/// algorithm, key store, identity provider, or HSM implementation in core.
pub fn verify_authority<F>(
    evidence: AuthorityEvidence,
    expected_intent_digest: Digest32,
    now_tick: u64,
    verify: F,
) -> Result<VerifiedAuthority, ProductionRefusal>
where
    F: Fn(&AuthorityEvidence) -> bool,
{
    if evidence.protocol_version != PRODUCTION_PROTOCOL_VERSION {
        return Err(ProductionRefusal::ProtocolVersionMismatch);
    }
    if evidence.intent_digest != expected_intent_digest {
        return Err(ProductionRefusal::IntentDigestMismatch);
    }
    if now_tick > evidence.expires_tick {
        return Err(ProductionRefusal::AuthorityExpired);
    }
    if !verify(&evidence) {
        return Err(ProductionRefusal::AuthorityVerificationFailed);
    }
    Ok(VerifiedAuthority { evidence })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ReplayEntry {
    intent_digest: Digest32,
    nonce: [u8; 16],
    expires_tick: u64,
}

/// Deterministic bounded replay guard. Expired entries are pruned first; when
/// capacity is reached, the oldest live entry is evicted. Production callers
/// should size capacity to exceed their maximum in-flight authority window.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayGuard {
    capacity: usize,
    entries: VecDeque<ReplayEntry>,
}

impl ReplayGuard {
    pub fn new(capacity: usize) -> Result<Self, ProductionRefusal> {
        if capacity == 0 {
            return Err(ProductionRefusal::ReplayCapacityZero);
        }
        Ok(Self {
            capacity,
            entries: VecDeque::with_capacity(capacity),
        })
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn admit(
        &mut self,
        authority: VerifiedAuthority,
        now_tick: u64,
    ) -> Result<ExecutionPermit, ProductionRefusal> {
        let evidence = authority.evidence;
        if now_tick > evidence.expires_tick {
            return Err(ProductionRefusal::AuthorityExpired);
        }

        self.prune(now_tick);
        if self.entries.iter().any(|entry| {
            entry.intent_digest == evidence.intent_digest && entry.nonce == evidence.nonce
        }) {
            return Err(ProductionRefusal::ReplayDetected);
        }

        if self.entries.len() == self.capacity {
            self.entries.pop_front();
        }
        self.entries.push_back(ReplayEntry {
            intent_digest: evidence.intent_digest,
            nonce: evidence.nonce,
            expires_tick: evidence.expires_tick,
        });

        Ok(ExecutionPermit {
            protocol_version: evidence.protocol_version,
            authority_ref: evidence.authority_ref,
            intent_digest: evidence.intent_digest,
            nonce: evidence.nonce,
            admitted_tick: now_tick,
            expires_tick: evidence.expires_tick,
        })
    }

    pub fn prune(&mut self, now_tick: u64) {
        self.entries.retain(|entry| now_tick <= entry.expires_tick);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionStanding {
    PortableAlive,
    NativeHostAlive,
    IntegrationUnknown,
    Blocked,
}

/// Explicit deployment evidence dimensions. This prevents one successful test
/// from being projected into an untested global/hardware claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DeploymentEvidence {
    pub portable_policy: bool,
    pub native_host_process: bool,
    pub real_atomvm_fixture: bool,
    pub real_nif_fixture: bool,
    pub hot_code_replacement: bool,
}

impl DeploymentEvidence {
    pub const fn standing(&self) -> ProductionStanding {
        if !self.portable_policy {
            ProductionStanding::Blocked
        } else if self.native_host_process && self.real_atomvm_fixture {
            ProductionStanding::NativeHostAlive
        } else {
            ProductionStanding::IntegrationUnknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: u8) -> Digest32 {
        Digest32([byte; 32])
    }

    #[test]
    fn authority_requires_external_verifier_and_exact_intent() {
        let evidence = AuthorityEvidence::new("hsm:key/7", digest(1), [7; 16], 100).unwrap();
        assert_eq!(
            verify_authority(evidence.clone(), digest(2), 1, |_| true),
            Err(ProductionRefusal::IntentDigestMismatch)
        );
        assert_eq!(
            verify_authority(evidence.clone(), digest(1), 1, |_| false),
            Err(ProductionRefusal::AuthorityVerificationFailed)
        );
        assert!(verify_authority(evidence, digest(1), 1, |_| true).is_ok());
    }

    #[test]
    fn replay_is_refused_until_entry_expires() {
        let evidence = AuthorityEvidence::new("authority:1", digest(3), [3; 16], 10).unwrap();
        let verified = verify_authority(evidence.clone(), digest(3), 1, |_| true).unwrap();
        let mut guard = ReplayGuard::new(4).unwrap();
        guard.admit(verified, 1).unwrap();
        let replay = verify_authority(evidence.clone(), digest(3), 2, |_| true).unwrap();
        assert_eq!(guard.admit(replay, 2), Err(ProductionRefusal::ReplayDetected));

        guard.prune(11);
        let renewed = AuthorityEvidence::new("authority:1", digest(3), [4; 16], 20).unwrap();
        let renewed = verify_authority(renewed, digest(3), 11, |_| true).unwrap();
        assert!(guard.admit(renewed, 11).is_ok());
    }

    #[test]
    fn standing_does_not_overclaim_unverified_surfaces() {
        let evidence = DeploymentEvidence {
            portable_policy: true,
            ..DeploymentEvidence::default()
        };
        assert_eq!(evidence.standing(), ProductionStanding::IntegrationUnknown);
    }
}
