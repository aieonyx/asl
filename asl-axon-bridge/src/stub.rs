// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// AXON-STUB-001 — FFI stub resolution pattern.
//
// AXON source files use FFI stubs (functions that return 0)
// as placeholders for seL4 syscall boundaries. The bridge
// resolves these stubs at link time via Rust shims.
//
// From ipc.ax (confirmed working in axon_sel4 rewrite):
//   fn sel4_sys_send(dest: i64, label: i64, length: i64) -> i64
//   fn sel4_sys_recv(src: i64) -> i64
//   fn sel4_sys_call(dest: i64, label: i64, length: i64) -> i64
//   fn sel4_mr_get(idx: i64) -> i64
//   fn sel4_mr_set(idx: i64, val: i64) -> i64
//
// The bridge stub registry tracks which FFI stubs are registered
// and whether they have been resolved by a Rust shim.

/// Known AXON FFI stub names from ipc.ax.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StubId {
    Sel4SysSend  = 0x01,
    Sel4SysRecv  = 0x02,
    Sel4SysCall  = 0x03,
    Sel4MrGet    = 0x04,
    Sel4MrSet    = 0x05,
}

impl StubId {
    /// Returns the canonical FFI name for this stub.
    pub fn name(self) -> &'static str {
        match self {
            StubId::Sel4SysSend => "sel4_sys_send",
            StubId::Sel4SysRecv => "sel4_sys_recv",
            StubId::Sel4SysCall => "sel4_sys_call",
            StubId::Sel4MrGet   => "sel4_mr_get",
            StubId::Sel4MrSet   => "sel4_mr_set",
        }
    }

    /// Returns true if this stub is required for IPC operation.
    pub fn is_required_for_ipc(self) -> bool {
        matches!(self,
            StubId::Sel4SysSend
            | StubId::Sel4SysCall
            | StubId::Sel4MrGet
            | StubId::Sel4MrSet
        )
    }
}

/// Resolution status of a stub.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StubStatus {
    /// Stub registered but not yet resolved by Rust shim.
    Unresolved,
    /// Stub resolved — Rust shim address registered.
    Resolved,
    /// Stub marked as unavailable on this platform.
    Unavailable,
}

/// A registered FFI stub entry.
#[derive(Debug, Clone, Copy)]
pub struct StubEntry {
    pub stub_id: StubId,
    pub status:  StubStatus,
    /// Shim address (non-zero when Resolved)
    pub shim_addr: u64,
}

/// AXON-STUB-001 registry.
pub struct StubRegistry {
    entries: [Option<StubEntry>; 16],
    count:   usize,
}

impl StubRegistry {
    pub const fn new() -> Self {
        Self { entries: [None; 16], count: 0 }
    }

    /// Register a stub as known but unresolved.
    pub fn register(&mut self, stub_id: StubId) -> Result<(), StubError> {
        if self.count >= 16 {
            return Err(StubError::RegistryFull);
        }
        if self.find(stub_id).is_some() {
            return Err(StubError::AlreadyRegistered);
        }
        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(StubEntry {
                    stub_id,
                    status: StubStatus::Unresolved,
                    shim_addr: 0,
                });
                self.count += 1;
                return Ok(());
            }
        }
        Err(StubError::RegistryFull)
    }

    /// Resolve a stub with a Rust shim address.
    pub fn resolve(&mut self, stub_id: StubId, shim_addr: u64) -> Result<(), StubError> {
        if shim_addr == 0 {
            return Err(StubError::ZeroShimAddr);
        }
        for slot in self.entries.iter_mut() {
            if let Some(e) = slot {
                if e.stub_id == stub_id {
                    e.status = StubStatus::Resolved;
                    e.shim_addr = shim_addr;
                    return Ok(());
                }
            }
        }
        Err(StubError::NotFound)
    }

    /// Register all five ipc.ax stubs.
    pub fn register_ipc_stubs(&mut self) -> Result<(), StubError> {
        let stubs = [
            StubId::Sel4SysSend,
            StubId::Sel4SysRecv,
            StubId::Sel4SysCall,
            StubId::Sel4MrGet,
            StubId::Sel4MrSet,
        ];
        for stub in stubs {
            self.register(stub)?;
        }
        Ok(())
    }

    /// Returns true if all IPC-required stubs are resolved.
    pub fn ipc_ready(&self) -> bool {
        let required = [
            StubId::Sel4SysSend,
            StubId::Sel4SysCall,
            StubId::Sel4MrGet,
            StubId::Sel4MrSet,
        ];
        required.iter().all(|s| {
            self.find(*s)
                .map(|e| e.status == StubStatus::Resolved)
                .unwrap_or(false)
        })
    }

    pub fn registered_count(&self) -> usize { self.count }

    pub fn resolved_count(&self) -> usize {
        self.entries.iter()
            .filter_map(|e| e.as_ref())
            .filter(|e| e.status == StubStatus::Resolved)
            .count()
    }

    fn find(&self, stub_id: StubId) -> Option<&StubEntry> {
        self.entries.iter()
            .filter_map(|e| e.as_ref())
            .find(|e| e.stub_id == stub_id)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StubError {
    RegistryFull,
    AlreadyRegistered,
    NotFound,
    ZeroShimAddr,
}
