// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ARPi provenance header — 78-byte sovereign IPC prefix.
// Every message routed through ARPi-Broker carries this header.

/// ARPi provenance header.
/// Total size: 78 bytes.
#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct ArpiHeader {
    /// Magic: 0xA2_91 (ARPi marker)
    pub magic:      u16,
    /// ASL version that produced this message
    pub asl_ver:    u8,
    /// Source PD identity
    pub src_pd:     u8,
    /// Destination PD identity
    pub dst_pd:     u8,
    /// Data tier of the payload
    pub data_tier:  u8,
    /// Monotonic sequence counter (anti-replay)
    pub seq:        u64,
    /// Ed25519 signature over [src_pd, dst_pd, data_tier, seq, payload_hash]
    pub signature:  [u8; 64],
}

impl ArpiHeader {
    pub const MAGIC: u16 = 0xA291;
    pub const SIZE:  usize = core::mem::size_of::<ArpiHeader>();

    pub fn new(src: u8, dst: u8, tier: u8, seq: u64, sig: [u8; 64]) -> Self {
        Self {
            magic:     Self::MAGIC,
            asl_ver:   0x01,
            src_pd:    src,
            dst_pd:    dst,
            data_tier: tier,
            seq,
            signature: sig,
        }
    }

    pub fn is_valid_magic(&self) -> bool {
        self.magic == Self::MAGIC
    }
}

// Compile-time size assertion: ArpiHeader must be exactly 78 bytes.
const _: () = assert!(
    core::mem::size_of::<ArpiHeader>() == 78,
    "ArpiHeader must be exactly 78 bytes"
);
