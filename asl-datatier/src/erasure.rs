// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// GDPR Art.17 erasure hook — Critical tier data erasure pathway.

use asl_common::datatier::DataTier;

pub const MAX_ERASURE_REQUESTS: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErasureStatus {
    Pending,
    Authorized,
    Completed,
    Rejected,
}

#[derive(Debug, Clone, Copy)]
pub struct ErasureRequest {
    pub requestor_pd: u8,
    pub tier:         DataTier,
    pub request_id:   u64,
    pub status:       ErasureStatus,
}

impl ErasureRequest {
    pub fn new(requestor_pd: u8, tier: DataTier, request_id: u64) -> Self {
        Self { requestor_pd, tier, request_id, status: ErasureStatus::Pending }
    }

    pub fn requires_dual_auth(&self) -> bool {
        self.tier == DataTier::Critical
    }
}

pub struct ErasureQueue {
    requests: [Option<ErasureRequest>; MAX_ERASURE_REQUESTS],
    count:    usize,
    next_id:  u64,
}

impl ErasureQueue {
    pub const fn new() -> Self {
        Self { requests: [None; MAX_ERASURE_REQUESTS], count: 0, next_id: 1 }
    }

    pub fn submit(&mut self, requestor_pd: u8, tier: DataTier) -> Result<u64, ErasureError> {
        if self.count >= MAX_ERASURE_REQUESTS {
            return Err(ErasureError::QueueFull);
        }
        let id = self.next_id;
        self.next_id += 1;
        for slot in self.requests.iter_mut() {
            if slot.is_none() {
                *slot = Some(ErasureRequest::new(requestor_pd, tier, id));
                self.count += 1;
                return Ok(id);
            }
        }
        Err(ErasureError::QueueFull)
    }

    pub fn authorize(&mut self, request_id: u64) -> Result<(), ErasureError> {
        for slot in self.requests.iter_mut() {
            if let Some(r) = slot {
                if r.request_id == request_id {
                    r.status = ErasureStatus::Authorized;
                    return Ok(());
                }
            }
        }
        Err(ErasureError::NotFound)
    }

    pub fn complete(&mut self, request_id: u64) -> Result<(), ErasureError> {
        // First pass: validate and update status
        let mut found = false;
        for slot in self.requests.iter_mut() {
            if let Some(r) = slot {
                if r.request_id == request_id {
                    if r.status != ErasureStatus::Authorized {
                        return Err(ErasureError::NotAuthorized);
                    }
                    r.status = ErasureStatus::Completed;
                    found = true;
                    break;
                }
            }
        }
        if !found {
            return Err(ErasureError::NotFound);
        }
        // Second pass: update count separately — no borrow conflict
        self.count = self.count.saturating_sub(1);
        Ok(())
    }

    pub fn reject(&mut self, request_id: u64) -> Result<(), ErasureError> {
        for slot in self.requests.iter_mut() {
            if let Some(r) = slot {
                if r.request_id == request_id {
                    r.status = ErasureStatus::Rejected;
                    return Ok(());
                }
            }
        }
        Err(ErasureError::NotFound)
    }

    pub fn pending_count(&self) -> usize {
        self.requests.iter()
            .filter_map(|r| r.as_ref())
            .filter(|r| r.status == ErasureStatus::Pending)
            .count()
    }

    pub fn queue_count(&self) -> usize { self.count }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErasureError {
    QueueFull,
    NotFound,
    NotAuthorized,
}
