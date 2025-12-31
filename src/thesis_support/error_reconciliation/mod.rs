//! Agent 5: Error Reconciliation
//!
//! Maps errors from Rust domain to Erlang domain (and vice versa).
//!
//! # Key Insight
//!
//! Errors are boundary operations - they cross domains.
//! Error reconciliation ensures error semantics are preserved across crossing.
//!
//! # Design Pattern: Error Morphism
//!
//! Each error type has encode/decode:
//! - RustError → ErlangError (FFI call fails)
//! - ErlangError → RustError (return value indicates error)
//! - Proof: f(f^-1(x)) = x for all errors

extern crate alloc;
use alloc::collections::HashMap;
use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;

/// Represents an error in the Rust domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DomainError {
    /// Invalid argument
    BadArg,
    /// Out of memory
    OutOfMemory,
    /// Type mismatch
    TypeError,
    /// Custom domain error
    Custom(String),
}

impl DomainError {
    pub fn description(&self) -> &str {
        match self {
            DomainError::BadArg => "bad argument",
            DomainError::OutOfMemory => "out of memory",
            DomainError::TypeError => "type error",
            DomainError::Custom(s) => s,
        }
    }
}

/// Represents an error in the Erlang domain
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErlangError {
    /// Badarg atom
    Badarg,
    /// Enomem atom
    Enomem,
    /// Error tuple {error, Reason}
    Error(String),
    /// Custom Erlang error
    Custom(String),
}

impl ErlangError {
    pub fn atom_name(&self) -> &str {
        match self {
            ErlangError::Badarg => "badarg",
            ErlangError::Enomem => "enomem",
            ErlangError::Error(s) => s,
            ErlangError::Custom(s) => s,
        }
    }
}

/// Error Morphism: bidirectional error mapping
pub trait ErrorMorphism {
    /// Map from Rust domain to Erlang domain
    fn to_erlang(&self, err: &DomainError) -> ErlangError;

    /// Map from Erlang domain to Rust domain
    fn to_rust(&self, err: &ErlangError) -> DomainError;

    /// Prove roundtrip: f(f^-1(x)) = x
    fn verify_morphism(&self, original: &DomainError) -> Result<bool, String> {
        let erlang = self.to_erlang(original);
        let back = self.to_rust(&erlang);

        // Check that we get back an equivalent error
        let equiv = match (original, &back) {
            (DomainError::BadArg, DomainError::BadArg) => true,
            (DomainError::OutOfMemory, DomainError::OutOfMemory) => true,
            (DomainError::TypeError, DomainError::TypeError) => true,
            (DomainError::Custom(a), DomainError::Custom(b)) => a == b,
            _ => false,
        };

        Ok(equiv)
    }
}

/// Standard Error Morphism implementation
pub struct StandardErrorMorphism;

impl ErrorMorphism for StandardErrorMorphism {
    fn to_erlang(&self, err: &DomainError) -> ErlangError {
        match err {
            DomainError::BadArg => ErlangError::Badarg,
            DomainError::OutOfMemory => ErlangError::Enomem,
            DomainError::TypeError => ErlangError::Error("type_error".to_string()),
            DomainError::Custom(s) => ErlangError::Custom(s.clone()),
        }
    }

    fn to_rust(&self, err: &ErlangError) -> DomainError {
        match err {
            ErlangError::Badarg => DomainError::BadArg,
            ErlangError::Enomem => DomainError::OutOfMemory,
            ErlangError::Error(s) if s == "type_error" => DomainError::TypeError,
            ErlangError::Error(s) => DomainError::Custom(s.clone()),
            ErlangError::Custom(s) => DomainError::Custom(s.clone()),
        }
    }
}

/// Error Reconciliation: tracks all error mappings and validates them
#[derive(Debug, Clone)]
pub struct ErrorReconciliation {
    /// All error roundtrips verified
    verified: HashMap<String, bool>,
    /// Total error mappings checked
    total_checks: usize,
    /// Stability proof: errors remain stable across hops
    stability_proof: Vec<String>,
}

impl ErrorReconciliation {
    /// Create new reconciliation
    pub fn new() -> Self {
        ErrorReconciliation {
            verified: HashMap::new(),
            total_checks: 0,
            stability_proof: Vec::new(),
        }
    }

    /// Verify an error morphism
    pub fn verify_error<M: ErrorMorphism>(&mut self, morphism: &M, error: &DomainError) -> bool {
        self.total_checks += 1;

        match morphism.verify_morphism(error) {
            Ok(valid) => {
                self.verified.insert(format!("{:?}", error), valid);
                valid
            }
            Err(_) => {
                self.verified.insert(format!("{:?}", error), false);
                false
            }
        }
    }

    /// Run error through N hops and check stability
    pub fn test_error_stability<M: ErrorMorphism>(
        &mut self,
        morphism: &M,
        error: &DomainError,
        num_hops: u32,
    ) -> bool {
        let mut current_error = error.clone();

        for hop in 1..=num_hops {
            // Cross domain
            let erlang = morphism.to_erlang(&current_error);

            // Cross back
            current_error = morphism.to_rust(&erlang);

            // Check stability
            let stable = current_error == *error;
            self.stability_proof.push(format!("Hop {}: {}", hop, if stable { "stable" } else { "drift" }));

            if !stable {
                return false;
            }
        }

        true
    }

    /// Get success rate
    pub fn success_rate(&self) -> f64 {
        if self.verified.is_empty() {
            return 0.0;
        }
        let successes = self.verified.values().filter(|&&v| v).count();
        successes as f64 / self.verified.len() as f64
    }

    /// Generate reconciliation report
    pub fn report(&self) -> String {
        let mut report = String::from("=== Error Reconciliation Report ===\n\n");
        report.push_str(&format!("Total error checks: {}\n", self.total_checks));
        report.push_str(&format!(
            "Verified errors: {}/{}\n",
            self.verified.values().filter(|&&v| v).count(),
            self.verified.len()
        ));
        report.push_str(&format!("Success rate: {:.2}%\n\n", self.success_rate() * 100.0));

        if !self.stability_proof.is_empty() {
            report.push_str("--- Stability Proof ---\n");
            for proof in &self.stability_proof {
                report.push_str(&format!("  {}\n", proof));
            }
        }

        report.push_str("\n--- Verified Errors ---\n");
        for (error, valid) in &self.verified {
            report.push_str(&format!("  {}: {}\n", error, if *valid { "✓" } else { "✗" }));
        }

        report
    }
}

impl Default for ErrorReconciliation {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_error_badarg() {
        let err = DomainError::BadArg;
        assert_eq!(err.description(), "bad argument");
    }

    #[test]
    fn test_domain_error_custom() {
        let err = DomainError::Custom("custom error".to_string());
        assert_eq!(err.description(), "custom error");
    }

    #[test]
    fn test_erlang_error_badarg() {
        let err = ErlangError::Badarg;
        assert_eq!(err.atom_name(), "badarg");
    }

    #[test]
    fn test_erlang_error_enomem() {
        let err = ErlangError::Enomem;
        assert_eq!(err.atom_name(), "enomem");
    }

    #[test]
    fn test_standard_error_morphism_badarg() {
        let morphism = StandardErrorMorphism;
        let rust_err = DomainError::BadArg;

        let erlang_err = morphism.to_erlang(&rust_err);
        assert_eq!(erlang_err, ErlangError::Badarg);
    }

    #[test]
    fn test_standard_error_morphism_out_of_memory() {
        let morphism = StandardErrorMorphism;
        let rust_err = DomainError::OutOfMemory;

        let erlang_err = morphism.to_erlang(&rust_err);
        assert_eq!(erlang_err, ErlangError::Enomem);
    }

    #[test]
    fn test_standard_error_morphism_roundtrip() {
        let morphism = StandardErrorMorphism;
        let rust_err = DomainError::BadArg;

        let erlang_err = morphism.to_erlang(&rust_err);
        let back = morphism.to_rust(&erlang_err);

        assert_eq!(rust_err, back);
    }

    #[test]
    fn test_error_morphism_verify() {
        let morphism = StandardErrorMorphism;
        let err = DomainError::BadArg;

        let result = morphism.verify_morphism(&err).unwrap();
        assert!(result);
    }

    #[test]
    fn test_error_reconciliation_creation() {
        let recon = ErrorReconciliation::new();
        assert_eq!(recon.total_checks, 0);
    }

    #[test]
    fn test_error_reconciliation_verify() {
        let mut recon = ErrorReconciliation::new();
        let morphism = StandardErrorMorphism;
        let err = DomainError::BadArg;

        let result = recon.verify_error(&morphism, &err);
        assert!(result);
        assert_eq!(recon.total_checks, 1);
    }

    #[test]
    fn test_error_reconciliation_success_rate() {
        let mut recon = ErrorReconciliation::new();
        let morphism = StandardErrorMorphism;

        for _ in 0..5 {
            recon.verify_error(&morphism, &DomainError::BadArg);
        }

        assert!(recon.success_rate() > 0.0);
    }

    #[test]
    fn test_error_stability_test() {
        let mut recon = ErrorReconciliation::new();
        let morphism = StandardErrorMorphism;
        let err = DomainError::BadArg;

        let stable = recon.test_error_stability(&morphism, &err, 5);
        assert!(stable);
    }

    #[test]
    fn test_error_reconciliation_report() {
        let mut recon = ErrorReconciliation::new();
        let morphism = StandardErrorMorphism;
        let _ = recon.verify_error(&morphism, &DomainError::BadArg);

        let report = recon.report();
        assert!(report.contains("Error Reconciliation Report"));
        assert!(report.contains("Total error checks: 1"));
    }

    #[test]
    fn test_multiple_error_types() {
        let mut recon = ErrorReconciliation::new();
        let morphism = StandardErrorMorphism;

        assert!(recon.verify_error(&morphism, &DomainError::BadArg));
        assert!(recon.verify_error(&morphism, &DomainError::OutOfMemory));
        assert!(recon.verify_error(&morphism, &DomainError::TypeError));

        assert_eq!(recon.total_checks, 3);
    }
}
