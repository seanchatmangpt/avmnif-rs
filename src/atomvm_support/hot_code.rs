//! Portable hot-code admission core derived from `unrdf/packages/atomvm/hot-code-loader.mjs`.
//!
//! File loading, hashing, callbacks, tracing, and supervisor notification stay outside this
//! module. The NIF layer owns only the deterministic proposal/commit/rollback state machine.

/// Opaque caller-computed module signature.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleSignature(pub [u8; 32]);

/// An admitted module revision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ModuleRevision {
    pub generation: u64,
    pub signature: ModuleSignature,
}

/// Per-module hot-code state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModuleStatus {
    Empty,
    Loaded,
    Reloading,
    Error,
}

/// Refusals produced before external loading or swap actuation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotCodeRefusal {
    ReloadAlreadyPending,
    NoPendingRevision,
    GenerationMismatch,
    UnchangedSignature,
    GenerationOverflow,
}

/// One module's hot-code slot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HotCodeSlot {
    current: Option<ModuleRevision>,
    pending: Option<ModuleRevision>,
    status: ModuleStatus,
}

impl Default for HotCodeSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl HotCodeSlot {
    pub const fn new() -> Self {
        Self {
            current: None,
            pending: None,
            status: ModuleStatus::Empty,
        }
    }

    pub const fn current(&self) -> Option<ModuleRevision> {
        self.current
    }

    pub const fn pending(&self) -> Option<ModuleRevision> {
        self.pending
    }

    pub const fn status(&self) -> ModuleStatus {
        self.status
    }

    /// Propose a caller-validated signature for initial load or reload.
    ///
    /// This does not load bytes or swap executable code. It manufactures only a candidate
    /// revision that the caller may independently verify before commit.
    pub fn propose(
        &mut self,
        signature: ModuleSignature,
    ) -> Result<ModuleRevision, HotCodeRefusal> {
        if self.pending.is_some() {
            return Err(HotCodeRefusal::ReloadAlreadyPending);
        }
        if self.current.map(|revision| revision.signature) == Some(signature) {
            return Err(HotCodeRefusal::UnchangedSignature);
        }

        let generation = match self.current {
            Some(revision) => revision
                .generation
                .checked_add(1)
                .ok_or(HotCodeRefusal::GenerationOverflow)?,
            None => 1,
        };
        let candidate = ModuleRevision {
            generation,
            signature,
        };
        self.pending = Some(candidate);
        self.status = ModuleStatus::Reloading;
        Ok(candidate)
    }

    /// Commit exactly the expected pending generation after external swap succeeds.
    pub fn commit(&mut self, generation: u64) -> Result<ModuleRevision, HotCodeRefusal> {
        let pending = self.pending.ok_or(HotCodeRefusal::NoPendingRevision)?;
        if pending.generation != generation {
            return Err(HotCodeRefusal::GenerationMismatch);
        }
        self.current = Some(pending);
        self.pending = None;
        self.status = ModuleStatus::Loaded;
        Ok(pending)
    }

    /// Abort a candidate without altering the last admitted revision.
    pub fn rollback(&mut self) -> Result<Option<ModuleRevision>, HotCodeRefusal> {
        if self.pending.is_none() {
            return Err(HotCodeRefusal::NoPendingRevision);
        }
        self.pending = None;
        self.status = if self.current.is_some() {
            ModuleStatus::Loaded
        } else {
            ModuleStatus::Empty
        };
        Ok(self.current)
    }

    /// Preserve failure as a distinct state; no implicit rollback or retry is performed.
    pub fn mark_error(&mut self) {
        self.pending = None;
        self.status = ModuleStatus::Error;
    }

    /// Forget the admitted revision. Executable unload remains caller-owned actuation.
    pub fn unload(&mut self) -> Option<ModuleRevision> {
        self.pending = None;
        self.status = ModuleStatus::Empty;
        self.current.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn signature(byte: u8) -> ModuleSignature {
        ModuleSignature([byte; 32])
    }

    #[test]
    fn initial_load_and_reload_are_monotonic() {
        let mut slot = HotCodeSlot::new();
        let first = slot.propose(signature(1)).unwrap();
        assert_eq!(first.generation, 1);
        slot.commit(1).unwrap();
        let second = slot.propose(signature(2)).unwrap();
        assert_eq!(second.generation, 2);
        slot.commit(2).unwrap();
        assert_eq!(slot.current().unwrap().generation, 2);
    }

    #[test]
    fn concurrent_or_stale_commit_is_refused() {
        let mut slot = HotCodeSlot::new();
        slot.propose(signature(1)).unwrap();
        assert_eq!(
            slot.propose(signature(2)),
            Err(HotCodeRefusal::ReloadAlreadyPending)
        );
        assert_eq!(slot.commit(2), Err(HotCodeRefusal::GenerationMismatch));
        assert_eq!(slot.pending().unwrap().generation, 1);
    }

    #[test]
    fn rollback_preserves_last_admitted_revision() {
        let mut slot = HotCodeSlot::new();
        slot.propose(signature(1)).unwrap();
        slot.commit(1).unwrap();
        slot.propose(signature(2)).unwrap();
        let current = slot.rollback().unwrap().unwrap();
        assert_eq!(current.generation, 1);
        assert_eq!(slot.status(), ModuleStatus::Loaded);
    }

    #[test]
    fn unchanged_signature_is_not_manufactured_as_new_code() {
        let mut slot = HotCodeSlot::new();
        slot.propose(signature(7)).unwrap();
        slot.commit(1).unwrap();
        assert_eq!(
            slot.propose(signature(7)),
            Err(HotCodeRefusal::UnchangedSignature)
        );
    }
}
