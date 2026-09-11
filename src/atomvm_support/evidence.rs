//! Portable evidence sealing without owning cryptographic authority.
//!
//! `avmnif-rs` is deliberately agnostic about which digest algorithm or keying
//! scheme an embedding environment uses. The caller computes a digest and this
//! module binds it to an immutable body. Verification likewise requires an
//! explicit caller-supplied digest function.

/// Opaque 256-bit digest supplied by an external verifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Digest32(pub [u8; 32]);

impl Digest32 {
    pub const ZERO: Self = Self([0; 32]);
}

/// Immutable body plus externally manufactured digest evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sealed<T> {
    body: T,
    digest: Digest32,
}

impl<T> Sealed<T> {
    /// Bind caller-computed evidence to a body.
    ///
    /// This does not compute or attest the digest. The caller retains that
    /// authority and is responsible for using a canonical representation.
    pub const fn new(body: T, digest: Digest32) -> Self {
        Self { body, digest }
    }

    pub const fn body(&self) -> &T {
        &self.body
    }

    pub const fn digest(&self) -> Digest32 {
        self.digest
    }

    /// Recompute evidence using caller-owned machinery and compare it to the
    /// sealed digest. Verification is observation, not actuation authority.
    pub fn verify_with<F>(&self, digest_body: F) -> bool
    where
        F: FnOnce(&T) -> Digest32,
    {
        digest_body(&self.body) == self.digest
    }

    /// Consume the sealed value without manufacturing any verification claim.
    pub fn into_parts(self) -> (T, Digest32) {
        (self.body, self.digest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seal_does_not_imply_verification() {
        let sealed = Sealed::new(7_u32, Digest32([7; 32]));
        assert!(sealed.verify_with(|value| Digest32([*value as u8; 32])));
        assert!(!sealed.verify_with(|_| Digest32([8; 32])));
    }

    #[test]
    fn body_is_immutable_through_public_api() {
        let sealed = Sealed::new("intent", Digest32([1; 32]));
        assert_eq!(sealed.body(), &"intent");
        assert_eq!(sealed.digest(), Digest32([1; 32]));
    }
}
