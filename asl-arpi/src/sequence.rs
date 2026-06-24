// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Monotonic sequence counter — per-source PD anti-replay protection.
// Each source PD has an independent counter that only increments.
// A message with seq <= last_seen is rejected as a replay attempt.


/// Maximum number of PDs tracked by the sequence table.
/// Covers all 11 PDs (6 mandatory + 5 optional).
pub const MAX_PD_COUNT: usize = 16;

/// Per-PD sequence counter entry.
#[derive(Debug, Clone, Copy)]
pub struct SeqEntry {
    pub pd_id:    u8,
    pub last_seq: u64,
    pub active:   bool,
}

impl SeqEntry {
    pub const fn empty() -> Self {
        Self { pd_id: 0, last_seq: 0, active: false }
    }
}

/// Sequence counter table — one entry per registered source PD.
pub struct SequenceTable {
    entries: [SeqEntry; MAX_PD_COUNT],
    count:   usize,
}

impl SequenceTable {
    pub const fn new() -> Self {
        Self {
            entries: [SeqEntry::empty(); MAX_PD_COUNT],
            count:   0,
        }
    }

    /// Register a PD in the sequence table.
    /// Returns Err if table is full or PD already registered.
    pub fn register(&mut self, pd_id: u8) -> Result<(), SeqError> {
        if self.count >= MAX_PD_COUNT {
            return Err(SeqError::TableFull);
        }
        if self.find(pd_id).is_some() {
            return Err(SeqError::AlreadyRegistered);
        }
        self.entries[self.count] = SeqEntry {
            pd_id,
            last_seq: 0,
            active: true,
        };
        self.count += 1;
        Ok(())
    }

    /// Validate and advance sequence for a message from src_pd.
    /// Returns Err if seq is not strictly greater than last_seen.
    pub fn validate_and_advance(
        &mut self,
        src_pd: u8,
        seq: u64,
    ) -> Result<(), SeqError> {
        match self.find_mut(src_pd) {
            None => Err(SeqError::UnknownPd),
            Some(entry) => {
                if seq <= entry.last_seq {
                    Err(SeqError::ReplayDetected {
                        last_seen: entry.last_seq,
                        received:  seq,
                    })
                } else {
                    entry.last_seq = seq;
                    Ok(())
                }
            }
        }
    }

    /// Returns current last_seq for a PD, or None if unregistered.
    pub fn last_seq(&self, pd_id: u8) -> Option<u64> {
        self.find(pd_id).map(|e| e.last_seq)
    }

    /// Returns number of registered PDs.
    pub fn registered_count(&self) -> usize {
        self.count
    }

    fn find(&self, pd_id: u8) -> Option<&SeqEntry> {
        self.entries[..self.count]
            .iter()
            .find(|e| e.active && e.pd_id == pd_id)
    }

    fn find_mut(&mut self, pd_id: u8) -> Option<&mut SeqEntry> {
        let count = self.count;
        self.entries[..count]
            .iter_mut()
            .find(|e| e.active && e.pd_id == pd_id)
    }
}

/// Sequence validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SeqError {
    TableFull,
    AlreadyRegistered,
    UnknownPd,
    ReplayDetected { last_seen: u64, received: u64 },
}
