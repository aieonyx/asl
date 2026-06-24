// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Tier audit log — immutable append-only record of every
// tier boundary event: permitted flows, violations, grants.
// Used for GDPR Art.17 compliance and security audit trails.

use asl_common::datatier::DataTier;

/// Maximum audit entries before log is considered full.
pub const MAX_AUDIT_ENTRIES: usize = 256;

/// Type of audit event.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditEvent {
    FlowPermitted,
    FlowGranted,
    FlowBlocked,
    TierViolation,
    ErasureRequested,
    ErasureCompleted,
    GrantRegistered,
    GrantRevoked,
}

/// A single audit log entry.
#[derive(Debug, Clone, Copy)]
pub struct AuditEntry {
    pub event:    AuditEvent,
    pub src_pd:   u8,
    pub dst_pd:   u8,
    pub src_tier: DataTier,
    pub dst_tier: DataTier,
    /// Monotonic entry sequence
    pub seq:      u64,
}

impl AuditEntry {
    pub fn new(
        event: AuditEvent,
        src_pd: u8,
        dst_pd: u8,
        src_tier: DataTier,
        dst_tier: DataTier,
        seq: u64,
    ) -> Self {
        Self { event, src_pd, dst_pd, src_tier, dst_tier, seq }
    }
}

/// Append-only audit log.
pub struct AuditLog {
    entries: [Option<AuditEntry>; MAX_AUDIT_ENTRIES],
    count:   usize,
    seq:     u64,
}

impl AuditLog {
    pub const fn new() -> Self {
        Self {
            entries: [None; MAX_AUDIT_ENTRIES],
            count:   0,
            seq:     0,
        }
    }

    /// Append an audit entry. Returns Err if log is full.
    pub fn append(
        &mut self,
        event: AuditEvent,
        src_pd: u8,
        dst_pd: u8,
        src_tier: DataTier,
        dst_tier: DataTier,
    ) -> Result<u64, AuditError> {
        if self.count >= MAX_AUDIT_ENTRIES {
            return Err(AuditError::LogFull);
        }
        self.seq += 1;
        let entry = AuditEntry::new(event, src_pd, dst_pd, src_tier, dst_tier, self.seq);
        self.entries[self.count] = Some(entry);
        self.count += 1;
        Ok(self.seq)
    }

    /// Returns total entry count.
    pub fn count(&self) -> usize { self.count }

    /// Returns current sequence number.
    pub fn current_seq(&self) -> u64 { self.seq }

    /// Count entries of a specific event type.
    pub fn count_by_event(&self, event: AuditEvent) -> usize {
        self.entries[..self.count]
            .iter()
            .filter_map(|e| e.as_ref())
            .filter(|e| e.event == event)
            .count()
    }

    /// Returns last entry if any.
    pub fn last(&self) -> Option<&AuditEntry> {
        if self.count == 0 { return None; }
        self.entries[self.count - 1].as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditError {
    LogFull,
}
