// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// DataTierEnforcer — central enforcement engine.
// Every data flow in the sovereign stack passes through here.

use asl_common::datatier::DataTier;
use crate::audit::{AuditEvent, AuditLog};
use crate::flow::{DataFlow, FlowResult};
use crate::grant::{GrantRegistry, TierGrant, GrantError};
use crate::erasure::{ErasureQueue, ErasureError};

/// The DataTier-Enforcer instance.
pub struct DataTierEnforcer {
    grants:  GrantRegistry,
    audit:   AuditLog,
    erasure: ErasureQueue,
    /// Vault PD — the only PD allowed to hold Critical plaintext.
    vault_pd: u8,
}

impl DataTierEnforcer {
    pub const fn new(vault_pd: u8) -> Self {
        Self {
            grants:   GrantRegistry::new(),
            audit:    AuditLog::new(),
            erasure:  ErasureQueue::new(),
            vault_pd,
        }
    }

    /// Check and enforce a data flow.
    pub fn check_flow(&mut self, flow: &DataFlow) -> FlowResult {
        // Self-flows always permitted
        if flow.is_internal() {
            let _ = self.audit.append(
                AuditEvent::FlowPermitted,
                flow.src_pd, flow.dst_pd,
                flow.src_tier, flow.dst_tier,
            );
            return FlowResult::Permitted;
        }

        // Critical data leaving vault PD is blocked
        if flow.src_tier == DataTier::Critical
            && flow.src_pd != self.vault_pd
        {
            let _ = self.audit.append(
                AuditEvent::TierViolation,
                flow.src_pd, flow.dst_pd,
                flow.src_tier, flow.dst_tier,
            );
            return FlowResult::CriticalVaultViolation;
        }

        // Same tier or downgrade — always permitted
        if !flow.is_tier_upgrade() {
            let _ = self.audit.append(
                AuditEvent::FlowPermitted,
                flow.src_pd, flow.dst_pd,
                flow.src_tier, flow.dst_tier,
            );
            return FlowResult::Permitted;
        }

        // Tier upgrade — check for grant
        match self.grants.lookup(flow.src_pd, flow.src_tier, flow.dst_tier) {
            Some(_) => {
                let _ = self.audit.append(
                    AuditEvent::FlowGranted,
                    flow.src_pd, flow.dst_pd,
                    flow.src_tier, flow.dst_tier,
                );
                FlowResult::PermittedWithGrant
            }
            None => {
                let _ = self.audit.append(
                    AuditEvent::FlowBlocked,
                    flow.src_pd, flow.dst_pd,
                    flow.src_tier, flow.dst_tier,
                );
                FlowResult::RequiresGrant
            }
        }
    }

    /// Register a cross-tier grant.
    pub fn register_grant(&mut self, grant: TierGrant) -> Result<(), GrantError> {
        let result = self.grants.register(grant);
        if result.is_ok() {
            let _ = self.audit.append(
                AuditEvent::GrantRegistered,
                grant.grantee_pd, 0xFF,
                grant.src_tier, grant.dst_tier,
            );
        }
        result
    }

    /// Submit a GDPR Art.17 erasure request.
    pub fn request_erasure(
        &mut self,
        requestor_pd: u8,
        tier: DataTier,
    ) -> Result<u64, ErasureError> {
        let id = self.erasure.submit(requestor_pd, tier)?;
        let _ = self.audit.append(
            AuditEvent::ErasureRequested,
            requestor_pd, 0xFF,
            tier, tier,
        );
        Ok(id)
    }

    /// Authorize and complete an erasure request.
    pub fn complete_erasure(&mut self, request_id: u64) -> Result<(), ErasureError> {
        self.erasure.authorize(request_id)?;
        self.erasure.complete(request_id)?;
        let _ = self.audit.append(
            AuditEvent::ErasureCompleted,
            0xFF, 0xFF,
            DataTier::Critical, DataTier::Critical,
        );
        Ok(())
    }

    pub fn audit_count(&self) -> usize { self.audit.count() }
    pub fn grant_count(&self) -> usize { self.grants.active_count() }
    pub fn pending_erasures(&self) -> usize { self.erasure.pending_count() }
    pub fn vault_pd(&self) -> u8 { self.vault_pd }
}
