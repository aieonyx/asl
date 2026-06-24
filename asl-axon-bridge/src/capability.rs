// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// Capability translation — AXON capability-flow types → seL4 objects.
//
// AXON's capability-flow type system proves at compile time which
// PD communications are valid. At the bridge, these compile-time
// proofs are translated into seL4 capability objects that enforce
// the same constraints at runtime.
//
// AXON capability annotation: #[cap(name)]
// Bridge maps: cap name → seL4 CPtr slot
//
// ASL-M5: structural mapping with slot registry.
// ASL-M7+: real seL4 CNode capability derivation.

use asl_common::pd::PdId;

/// Maximum capability mappings in the bridge registry.
pub const MAX_CAP_MAPPINGS: usize = 64;

/// AXON capability names (from #[cap(name)] annotations).
/// These must match exactly what the AXON compiler emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AxonCapName {
    /// seL4 IPC endpoint capability
    Endpoint    = 0x01,
    /// Notification capability
    Notification = 0x02,
    /// Shared memory frame capability
    SharedFrame = 0x03,
    /// IRQ handler capability
    IrqHandler  = 0x04,
    /// TCB (Thread Control Block) capability
    Tcb         = 0x05,
    /// CNode capability
    CNode       = 0x06,
}

/// A mapping from AXON cap name to seL4 CPtr slot.
#[derive(Debug, Clone, Copy)]
pub struct CapMapping {
    pub pd_id:    u8,
    pub cap_name: AxonCapName,
    /// seL4 CPtr slot number
    pub cptr:     u64,
    pub active:   bool,
}

impl CapMapping {
    pub const fn empty() -> Self {
        Self { pd_id: 0, cap_name: AxonCapName::Endpoint, cptr: 0, active: false }
    }
}

/// Capability translation registry.
pub struct CapRegistry {
    mappings: [CapMapping; MAX_CAP_MAPPINGS],
    count:    usize,
}

impl CapRegistry {
    pub const fn new() -> Self {
        Self { mappings: [CapMapping::empty(); MAX_CAP_MAPPINGS], count: 0 }
    }

    /// Register a capability mapping for a PD.
    pub fn register(
        &mut self,
        pd_id: u8,
        cap_name: AxonCapName,
        cptr: u64,
    ) -> Result<(), CapError> {
        if self.count >= MAX_CAP_MAPPINGS {
            return Err(CapError::RegistryFull);
        }
        if cptr == 0 {
            return Err(CapError::ZeroCptr);
        }
        if self.find(pd_id, cap_name).is_some() {
            return Err(CapError::AlreadyMapped);
        }
        self.mappings[self.count] = CapMapping {
            pd_id, cap_name, cptr, active: true,
        };
        self.count += 1;
        Ok(())
    }

    /// Translate an AXON cap name to a seL4 CPtr for a PD.
    pub fn translate(
        &self,
        pd_id: u8,
        cap_name: AxonCapName,
    ) -> Result<u64, CapError> {
        self.find(pd_id, cap_name)
            .map(|m| m.cptr)
            .ok_or(CapError::NoMapping)
    }

    /// Revoke all capability mappings for a PD.
    pub fn revoke_pd(&mut self, pd_id: u8) -> usize {
        let mut revoked = 0;
        for m in self.mappings[..self.count].iter_mut() {
            if m.active && m.pd_id == pd_id {
                m.active = false;
                revoked += 1;
            }
        }
        revoked
    }

    pub fn mapping_count(&self) -> usize { self.count }

    /// Register all mandatory PD endpoint capabilities.
    pub fn register_mandatory_pds(&mut self) -> Result<(), CapError> {
        let pds = [
            PdId::Genesis as u8,
            PdId::ArpiBroker as u8,
            PdId::DataTierEnforcer as u8,
            PdId::TrustGraphGate as u8,
            PdId::InvertedAdmin as u8,
        ];
        for (i, &pd_id) in pds.iter().enumerate() {
            // CPtr slots start at 0x100 for mandatory PDs
            let cptr = 0x100u64 + i as u64;
            self.register(pd_id, AxonCapName::Endpoint, cptr)?;
        }
        Ok(())
    }

    fn find(&self, pd_id: u8, cap_name: AxonCapName) -> Option<&CapMapping> {
        self.mappings[..self.count]
            .iter()
            .find(|m| m.active && m.pd_id == pd_id && m.cap_name == cap_name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapError {
    RegistryFull,
    ZeroCptr,
    AlreadyMapped,
    NoMapping,
}
