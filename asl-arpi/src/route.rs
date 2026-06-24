// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ARPi route table — maps (src_pd, dst_pd) pairs to route entries.
// Only registered routes are allowed. Unknown routes are rejected.
// All six mandatory PDs are registered at commissioning time.


/// Maximum number of routes in the table.
pub const MAX_ROUTES: usize = 64;

/// A single registered route entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RouteEntry {
    pub src_pd:  u8,
    pub dst_pd:  u8,
    pub active:  bool,
    /// True if this route is on the IPC fastpath.
    /// Fastpath: payload fits in registers (<= FASTPATH_BYTES).
    pub fastpath: bool,
}

impl RouteEntry {
    pub const fn empty() -> Self {
        Self { src_pd: 0, dst_pd: 0, active: false, fastpath: false }
    }
}

/// IPC fastpath payload budget.
/// Messages with total size (header + payload) <= this value
/// stay on the seL4 register-based fastpath.
/// ARPi header = 78 bytes. seL4 fastpath budget ~120 bytes.
/// Payload budget = 120 - 78 = 42 bytes on strict fastpath.
/// We use 44 bytes to allow for alignment.
pub const FASTPATH_PAYLOAD_BYTES: usize = 44;

/// Total fastpath message budget including ARPi header.
pub const FASTPATH_TOTAL_BYTES: usize = FASTPATH_PAYLOAD_BYTES + ARPI_HEADER_SIZE;

use asl_common::arpi::ARPI_HEADER_SIZE;

/// Route table for all registered ARPi communication paths.
pub struct RouteTable {
    entries: [RouteEntry; MAX_ROUTES],
    count:   usize,
}

impl RouteTable {
    pub const fn new() -> Self {
        Self {
            entries: [RouteEntry::empty(); MAX_ROUTES],
            count:   0,
        }
    }

    /// Register a route between two PDs.
    pub fn register(
        &mut self,
        src_pd: u8,
        dst_pd: u8,
        fastpath: bool,
    ) -> Result<(), RouteError> {
        if self.count >= MAX_ROUTES {
            return Err(RouteError::TableFull);
        }
        if self.find(src_pd, dst_pd).is_some() {
            return Err(RouteError::AlreadyExists);
        }
        // Self-routing is not allowed
        if src_pd == dst_pd {
            return Err(RouteError::SelfRoute);
        }
        self.entries[self.count] = RouteEntry {
            src_pd,
            dst_pd,
            active: true,
            fastpath,
        };
        self.count += 1;
        Ok(())
    }

    /// Look up a route. Returns Err if no route exists.
    pub fn lookup(&self, src_pd: u8, dst_pd: u8) -> Result<RouteEntry, RouteError> {
        self.find(src_pd, dst_pd)
            .copied()
            .ok_or(RouteError::NoRoute)
    }

    /// Returns true if route exists and is on fastpath.
    pub fn is_fastpath(&self, src_pd: u8, dst_pd: u8) -> bool {
        self.find(src_pd, dst_pd)
            .map(|e| e.fastpath)
            .unwrap_or(false)
    }

    /// Returns total number of registered routes.
    pub fn route_count(&self) -> usize {
        self.count
    }

    /// Deactivate a route (used when a PD is decommissioned).
    pub fn deactivate(&mut self, src_pd: u8, dst_pd: u8) -> Result<(), RouteError> {
        let count = self.count;
        self.entries[..count]
            .iter_mut()
            .find(|e| e.active && e.src_pd == src_pd && e.dst_pd == dst_pd)
            .map(|e| { e.active = false; })
            .ok_or(RouteError::NoRoute)
    }

    fn find(&self, src_pd: u8, dst_pd: u8) -> Option<&RouteEntry> {
        self.entries[..self.count]
            .iter()
            .find(|e| e.active && e.src_pd == src_pd && e.dst_pd == dst_pd)
    }
}

/// Route table errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RouteError {
    TableFull,
    AlreadyExists,
    SelfRoute,
    NoRoute,
}
