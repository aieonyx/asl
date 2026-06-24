// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Trust graph — directed capability graph.
// Nodes: PDs. Edges: granted capability types.
// TrustGraph-Gate validates tokens against registered edges.

use asl_common::pd::PdId;
use crate::token::{CapToken, CapabilityType, TokenResult, validate};
use crate::trust_score::{TrustRegistry, TrustError};

/// Maximum edges in the capability graph.
pub const MAX_EDGES: usize = 64;

/// A capability edge in the trust graph.
#[derive(Debug, Clone, Copy)]
pub struct CapEdge {
    pub src_pd:   u8,
    pub dst_pd:   u8,
    pub cap_type: CapabilityType,
    pub active:   bool,
    /// Sequence of last validated token on this edge
    pub last_seq: u64,
}

impl CapEdge {
    pub const fn empty() -> Self {
        Self {
            src_pd: 0, dst_pd: 0,
            cap_type: CapabilityType::Read,
            active: false, last_seq: 0,
        }
    }
}

/// Result of token validation against the graph.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GraphResult {
    /// Token valid — capability granted.
    Granted,
    /// Token structurally invalid.
    InvalidToken(TokenResult),
    /// No edge exists for this capability.
    NoEdge,
    /// Token seq is a replay.
    ReplayDetected,
    /// Source PD not trusted.
    UntrustedSource,
}

/// The trust graph — central capability authority.
pub struct TrustGraph {
    edges:  [CapEdge; MAX_EDGES],
    edge_count: usize,
    trust:  TrustRegistry,
}

impl TrustGraph {
    pub const fn new() -> Self {
        Self {
            edges: [CapEdge::empty(); MAX_EDGES],
            edge_count: 0,
            trust: TrustRegistry::new(),
        }
    }

    /// Initialize the graph with all mandatory PDs.
    /// Called by GENESIS during commissioning.
    pub fn commission(&mut self) -> Result<(), TrustError> {
        let mandatory = [
            PdId::Genesis as u8,
            PdId::ArpiBroker as u8,
            PdId::DataTierEnforcer as u8,
            PdId::TrustGraphGate as u8,
            PdId::InvertedAdmin as u8,
            PdId::AxonBridge as u8,
        ];
        for pd in mandatory {
            self.trust.register(pd, true)?;
        }
        // Register core capability edges for mandatory PDs
        // GENESIS → ARPi-Broker: Execute
        self.add_edge(
            PdId::Genesis as u8,
            PdId::ArpiBroker as u8,
            CapabilityType::Execute,
        )?;
        // ARPi-Broker → all mandatory PDs: Read + Execute
        let targets = [
            PdId::DataTierEnforcer as u8,
            PdId::TrustGraphGate as u8,
            PdId::InvertedAdmin as u8,
            PdId::AxonBridge as u8,
        ];
        for dst in targets {
            self.add_edge(PdId::ArpiBroker as u8, dst, CapabilityType::Execute)?;
        }
        Ok(())
    }

    /// Add a capability edge to the graph.
    pub fn add_edge(
        &mut self,
        src_pd: u8,
        dst_pd: u8,
        cap_type: CapabilityType,
    ) -> Result<(), TrustError> {
        if self.edge_count >= MAX_EDGES {
            return Err(TrustError::RegistryFull);
        }
        self.edges[self.edge_count] = CapEdge {
            src_pd, dst_pd, cap_type,
            active: true, last_seq: 0,
        };
        self.edge_count += 1;
        Ok(())
    }

    /// Validate a capability token against the graph.
    pub fn validate_token(&mut self, token: &CapToken) -> GraphResult {
        // Structural validation
        let tv = validate(token);
        if tv != TokenResult::Valid {
            return GraphResult::InvalidToken(tv);
        }

        // Source must be trusted
        if !self.trust.is_trusted(token.src_pd) {
            return GraphResult::UntrustedSource;
        }

        // Find matching edge
        let edge = self.find_edge_mut(token.src_pd, token.dst_pd, token.cap_type);
        match edge {
            None => GraphResult::NoEdge,
            Some(e) => {
                // Anti-replay
                if token.seq <= e.last_seq {
                    return GraphResult::ReplayDetected;
                }
                e.last_seq = token.seq;
                // Record grant in trust registry
                let _ = self.trust.record_grant(token.dst_pd);
                GraphResult::Granted
            }
        }
    }

    /// Revoke a capability edge.
    pub fn revoke_edge(
        &mut self,
        src_pd: u8,
        dst_pd: u8,
        cap_type: CapabilityType,
    ) -> Result<(), TrustError> {
        let count = self.edge_count;
        self.edges[..count]
            .iter_mut()
            .find(|e| e.active && e.src_pd == src_pd
                && e.dst_pd == dst_pd && e.cap_type == cap_type)
            .map(|e| {
                e.active = false;
            })
            .ok_or(TrustError::UnknownPd)?;
        let _ = self.trust.record_revocation(dst_pd);
        Ok(())
    }

    /// Returns trust score for a PD.
    pub fn trust_score(&self, pd_id: u8) -> Option<u8> {
        self.trust.score(pd_id)
    }

    pub fn edge_count(&self) -> usize { self.edge_count }
    pub fn registered_pd_count(&self) -> usize { self.trust.registered_count() }

    fn find_edge_mut(
        &mut self,
        src_pd: u8,
        dst_pd: u8,
        cap_type: CapabilityType,
    ) -> Option<&mut CapEdge> {
        let count = self.edge_count;
        self.edges[..count].iter_mut().find(|e| {
            e.active && e.src_pd == src_pd
                && e.dst_pd == dst_pd && e.cap_type == cap_type
        })
    }
}
