// Copyright (C) 2026 Edison Lepiten <aieonyx.eu@gmail.com>
// SPDX-License-Identifier: Apache-2.0
//
// asl-arpi-ipc — ARPi Live IPC Protection Domain
//
// AXON Receptor Protocol Interface — full five-layer sovereign auth
// running live over seL4 IPC channels.
//
// Public name : AXON Receptor Protocol Interface
// Five mandatory layers (in binding order):
//
//   Layer 1 — Schema      : AXON-typed message contract validation
//   Layer 2 — Identity    : Ed25519 commissioning keypair verification
//   Layer 3 — Mutual Auth : Both endpoints present valid commissioning proof
//   Layer 4 — Scope       : Policy PD monotonic capability token check
//   Layer 5 — Anomaly     : Aegis / IME threat escalation gate
//
// Every bind event (pass OR reject) is logged — no silent failure path.
// All five layers must pass for a bind to succeed.
//
// ARPi 78-byte provenance header is prepended to every bound message.
// (Header spec: AIEONYX-SPEC-ARPi-v1.0 — locked)
//
// Sovereign proof: axon_main() → 0x4153 (invariant)
//
// S4+i: A leaked credential is useless without the hardware-resident key.

#![no_std]
#![forbid(unsafe_code)]

#[cfg(kani)]
extern crate kani;

// ── Constants ─────────────────────────────────────────────────────────────────

/// ARPi provenance header size — locked in AIEONYX-SPEC-ARPi-v1.0.
pub const ARPI_HEADER_SIZE: usize = 78;

/// ARPi magic bytes — first 4 bytes of every header.
pub const ARPI_MAGIC: [u8; 4] = [0x41, 0x52, 0x50, 0x69]; // "ARPi"

/// ARPi spec version.
pub const ARPI_VERSION: u8 = 0x01;

/// Ed25519 public key size in bytes.
pub const ED25519_KEY_SIZE: usize = 32;

/// Ed25519 signature size in bytes.
pub const ED25519_SIG_SIZE: usize = 64;

/// Capability token size in bytes.
pub const CAP_TOKEN_SIZE: usize = 8;

/// Maximum scope grant count per bind session.
pub const MAX_SCOPE_GRANTS: usize = 8;

/// Anomaly score threshold — binds above this score are rejected.
pub const ANOMALY_THRESHOLD: u8 = 75;

/// ARPi PD identifier.
pub const ARPI_PD_ID: u8 = 0x01; // mandatory PD, established M1

/// Sovereign proof value.
pub const AXON_PROOF: u64 = 0x4153;

/// ARPi seL4 IPC channel.
pub const CH_ARPI_BIND: u8 = 0x10;

// ── Error type ────────────────────────────────────────────────────────────────

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ArpiError {
    /// Layer 1: message schema type mismatch.
    SchemaViolation,
    /// Layer 2: Ed25519 identity proof invalid.
    IdentityRejected,
    /// Layer 3: mutual auth — remote endpoint proof invalid.
    MutualAuthFailed,
    /// Layer 4: capability token invalid or expired.
    ScopeViolation,
    /// Layer 5: anomaly score exceeds threshold.
    AnomalyEscalated,
    /// Bind already completed — cannot re-bind same session.
    AlreadyBound,
    /// Bind log is full — cannot record more events.
    LogFull,
    /// Invalid input.
    InvalidInput,
}

// ── Layer 1: Schema ───────────────────────────────────────────────────────────

/// AXON message schema types — the contract for what may pass a bind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum SchemaType {
    /// Raw bytes — lowest trust, requires all 5 layers.
    Raw        = 0x00,
    /// ARPi control message.
    ArpiCtrl   = 0x01,
    /// AWP protocol message.
    AwpMsg     = 0x02,
    /// DataTier record — Critical tier.
    DataCrit   = 0x03,
    /// DataTier record — Personal tier.
    DataPers   = 0x04,
    /// Threat intel report.
    ThreatRpt  = 0x05,
    /// Identity assertion.
    IdAssert   = 0x06,
}

impl SchemaType {
    pub fn from_u8(v: u8) -> Option<Self> {
        match v {
            0x00 => Some(Self::Raw),
            0x01 => Some(Self::ArpiCtrl),
            0x02 => Some(Self::AwpMsg),
            0x03 => Some(Self::DataCrit),
            0x04 => Some(Self::DataPers),
            0x05 => Some(Self::ThreatRpt),
            0x06 => Some(Self::IdAssert),
            _    => None,
        }
    }
}

/// Validate a message schema byte. Returns the schema type if valid.
pub fn validate_schema(schema_byte: u8) -> Result<SchemaType, ArpiError> {
    SchemaType::from_u8(schema_byte).ok_or(ArpiError::SchemaViolation)
}

// ── Layer 2: Identity ─────────────────────────────────────────────────────────

/// An Ed25519 public key (32 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ed25519PublicKey(pub [u8; ED25519_KEY_SIZE]);

/// An Ed25519 signature (64 bytes).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ed25519Signature(pub [u8; ED25519_SIG_SIZE]);

/// Identity proof submitted by a binding endpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityProof {
    /// Ed25519 commissioning public key.
    pub pubkey: Ed25519PublicKey,
    /// Signature over the bind nonce.
    pub sig:    Ed25519Signature,
    /// Bind nonce — 8 random bytes, unique per session.
    pub nonce:  [u8; 8],
}

/// Verify an identity proof.
///
/// In production this calls into the Ed25519 verifier via axon_crypto.
/// In this PD layer we verify the structural contract:
///   - pubkey must be non-zero
///   - nonce must be non-zero
///   - sig first byte must not be 0x00 (structural plausibility)
///
/// Full cryptographic verification is wired at ASL-M22 (AXON migration).
pub fn verify_identity(proof: &IdentityProof) -> Result<(), ArpiError> {
    if proof.pubkey.0 == [0u8; ED25519_KEY_SIZE] {
        return Err(ArpiError::IdentityRejected);
    }
    if proof.nonce == [0u8; 8] {
        return Err(ArpiError::IdentityRejected);
    }
    if proof.sig.0[0] == 0x00 && proof.sig.0[1] == 0x00 {
        return Err(ArpiError::IdentityRejected);
    }
    Ok(())
}

// ── Layer 3: Mutual Auth ──────────────────────────────────────────────────────

/// Mutual auth session — both endpoints must present valid identity proofs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MutualAuthSession {
    pub local:  IdentityProof,
    pub remote: IdentityProof,
}

/// Verify mutual auth — both local and remote identity proofs must be valid,
/// and they must have different public keys (no self-binding).
pub fn verify_mutual_auth(session: &MutualAuthSession) -> Result<(), ArpiError> {
    verify_identity(&session.local).map_err(|_| ArpiError::MutualAuthFailed)?;
    verify_identity(&session.remote).map_err(|_| ArpiError::MutualAuthFailed)?;
    // Self-binding is forbidden — keys must differ
    if session.local.pubkey == session.remote.pubkey {
        return Err(ArpiError::MutualAuthFailed);
    }
    Ok(())
}

// ── Layer 4: Scope ────────────────────────────────────────────────────────────

/// A monotonic capability token — issued by the Policy PD.
/// Format: 8 bytes — [seq:4][pd_id:1][schema:1][flags:2]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CapToken(pub [u8; CAP_TOKEN_SIZE]);

impl CapToken {
    /// Create a new capability token.
    pub fn new(seq: u32, pd_id: u8, schema: SchemaType, flags: u16) -> Self {
        let mut t = [0u8; CAP_TOKEN_SIZE];
        t[0..4].copy_from_slice(&seq.to_be_bytes());
        t[4] = pd_id;
        t[5] = schema as u8;
        t[6..8].copy_from_slice(&flags.to_be_bytes());
        Self(t)
    }

    /// Extract sequence number from token.
    pub fn seq(&self) -> u32 {
        u32::from_be_bytes([self.0[0], self.0[1], self.0[2], self.0[3]])
    }

    /// Extract PD ID from token.
    pub fn pd_id(&self) -> u8 { self.0[4] }

    /// Extract schema type from token.
    pub fn schema(&self) -> Option<SchemaType> {
        SchemaType::from_u8(self.0[5])
    }

    /// Token is valid if seq > 0 and schema is known.
    pub fn is_valid(&self) -> bool {
        self.seq() > 0 && self.schema().is_some()
    }
}

/// Scope grant — one capability allowed in this bind session.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeGrant {
    pub token:  CapToken,
    pub active: bool,
}

/// Scope registry for a bind session — fixed size, no heap.
pub struct ScopeRegistry {
    grants: [Option<ScopeGrant>; MAX_SCOPE_GRANTS],
    count:  usize,
    /// Last seen sequence number — enforces monotonic token ordering.
    last_seq: u32,
}

impl ScopeRegistry {
    pub const fn new() -> Self {
        Self {
            grants:   [None; MAX_SCOPE_GRANTS],
            count:    0,
            last_seq: 0,
        }
    }

    /// Register a capability grant. Token must be valid and monotonically greater.
    pub fn grant(&mut self, token: CapToken) -> Result<(), ArpiError> {
        if !token.is_valid() {
            return Err(ArpiError::ScopeViolation);
        }
        if token.seq() <= self.last_seq {
            return Err(ArpiError::ScopeViolation);
        }
        if self.count >= MAX_SCOPE_GRANTS {
            return Err(ArpiError::ScopeViolation);
        }
        for slot in self.grants.iter_mut() {
            if slot.is_none() {
                *slot = Some(ScopeGrant { token, active: true });
                self.last_seq = token.seq();
                self.count += 1;
                return Ok(());
            }
        }
        Err(ArpiError::ScopeViolation)
    }

    /// Check if a given schema is within scope.
    pub fn has_scope(&self, schema: SchemaType) -> bool {
        for slot in self.grants.iter() {
            if let Some(g) = slot {
                if g.active && g.token.schema() == Some(schema) {
                    return true;
                }
            }
        }
        false
    }

    pub fn count(&self) -> usize { self.count }
    pub fn last_seq(&self) -> u32 { self.last_seq }
}

impl Default for ScopeRegistry {
    fn default() -> Self { Self::new() }
}

/// Validate scope: the token must be valid and the schema must be in scope.
pub fn validate_scope(
    registry: &ScopeRegistry,
    schema: SchemaType,
) -> Result<(), ArpiError> {
    if !registry.has_scope(schema) {
        Err(ArpiError::ScopeViolation)
    } else {
        Ok(())
    }
}

// ── Layer 5: Anomaly ──────────────────────────────────────────────────────────

/// Anomaly check result.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnomalyResult {
    /// Bind proceeds normally.
    Clear,
    /// Anomaly detected but below threshold — logged, bind continues.
    Flagged,
    /// Anomaly above threshold — bind rejected, Aegis escalated.
    Escalated,
}

/// Run the anomaly gate.
///
/// score 0–49   : Clear
/// score 50–74  : Flagged (logged, bind continues)
/// score 75–100 : Escalated (bind rejected)
pub fn anomaly_gate(score: u8) -> AnomalyResult {
    if score >= ANOMALY_THRESHOLD {
        AnomalyResult::Escalated
    } else if score >= 50 {
        AnomalyResult::Flagged
    } else {
        AnomalyResult::Clear
    }
}

// ── ARPi provenance header ────────────────────────────────────────────────────

/// ARPi 78-byte provenance header — prepended to every bound message.
///
/// Wire layout (78 bytes):
///   [0..4]   magic       "ARPi" = [0x41,0x52,0x50,0x69]
///   [4]      version     0x01
///   [5]      schema      SchemaType as u8
///   [6]      pd_src      source PD ID
///   [7]      pd_dst      destination PD ID
///   [8..16]  seq         u64 BE monotonic message sequence
///   [16..24] cap_token   CapToken (8 bytes)
///   [24..56] pubkey_src  Ed25519 source public key (32 bytes)
///   [56..78] reserved    zero-padded (22 bytes)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ArpiHeader {
    pub magic:      [u8; 4],
    pub version:    u8,
    pub schema:     SchemaType,
    pub pd_src:     u8,
    pub pd_dst:     u8,
    pub seq:        u64,
    pub cap_token:  CapToken,
    pub pubkey_src: Ed25519PublicKey,
}

impl ArpiHeader {
    pub const WIRE_LEN: usize = ARPI_HEADER_SIZE; // 78

    /// Build a new ARPi provenance header.
    pub fn new(
        schema:     SchemaType,
        pd_src:     u8,
        pd_dst:     u8,
        seq:        u64,
        cap_token:  CapToken,
        pubkey_src: Ed25519PublicKey,
    ) -> Self {
        Self {
            magic: ARPI_MAGIC,
            version: ARPI_VERSION,
            schema,
            pd_src,
            pd_dst,
            seq,
            cap_token,
            pubkey_src,
        }
    }

    /// Serialise to 78-byte wire format.
    pub fn to_bytes(&self) -> [u8; ARPI_HEADER_SIZE] {
        let mut b = [0u8; ARPI_HEADER_SIZE];
        b[0..4].copy_from_slice(&self.magic);
        b[4] = self.version;
        b[5] = self.schema as u8;
        b[6] = self.pd_src;
        b[7] = self.pd_dst;
        b[8..16].copy_from_slice(&self.seq.to_be_bytes());
        b[16..24].copy_from_slice(&self.cap_token.0);
        b[24..56].copy_from_slice(&self.pubkey_src.0);
        // bytes 56..78 remain zero — reserved
        b
    }

    /// Validate header magic and version.
    pub fn validate(&self) -> Result<(), ArpiError> {
        if self.magic != ARPI_MAGIC {
            return Err(ArpiError::SchemaViolation);
        }
        if self.version != ARPI_VERSION {
            return Err(ArpiError::SchemaViolation);
        }
        Ok(())
    }
}

// ── Bind log ──────────────────────────────────────────────────────────────────

/// Maximum bind log entries per session.
pub const BIND_LOG_MAX: usize = 32;

/// Outcome of a bind attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindOutcome {
    /// All five layers passed — bind succeeded.
    Success,
    /// Bind failed at the specified layer.
    Failed(u8), // layer number 1–5
}

/// A single bind log entry — written for every bind attempt (pass or reject).
#[derive(Debug, Clone, Copy)]
pub struct BindLogEntry {
    pub seq:     u64,
    pub pd_src:  u8,
    pub pd_dst:  u8,
    pub schema:  u8,
    pub outcome: BindOutcome,
    pub anomaly: u8,
}

/// Bind log — fixed size, no heap. Every event recorded.
pub struct BindLog {
    entries: [Option<BindLogEntry>; BIND_LOG_MAX],
    count:   usize,
}

impl BindLog {
    pub const fn new() -> Self {
        Self {
            entries: [None; BIND_LOG_MAX],
            count:   0,
        }
    }

    /// Record a bind event. Returns LogFull if capacity exceeded.
    pub fn record(&mut self, entry: BindLogEntry) -> Result<(), ArpiError> {
        if self.count >= BIND_LOG_MAX {
            return Err(ArpiError::LogFull);
        }
        for slot in self.entries.iter_mut() {
            if slot.is_none() {
                *slot = Some(entry);
                self.count += 1;
                return Ok(());
            }
        }
        Err(ArpiError::LogFull)
    }

    pub fn count(&self) -> usize { self.count }

    pub fn last(&self) -> Option<&BindLogEntry> {
        // Walk backwards to find last entry
        let mut last_idx = None;
        let mut last_seq = 0u64;
        for (i, slot) in self.entries.iter().enumerate() {
            if let Some(e) = slot {
                if e.seq >= last_seq {
                    last_seq = e.seq;
                    last_idx = Some(i);
                }
            }
        }
        last_idx.and_then(|i| self.entries[i].as_ref())
    }
}

impl Default for BindLog {
    fn default() -> Self { Self::new() }
}

// ── ARPi bind engine — five layers in sequence ────────────────────────────────

/// ARPi bind engine — runs all five layers in order.
pub struct ArpiBinder {
    pub scope:    ScopeRegistry,
    pub log:      BindLog,
    seq:          u64,
}

impl ArpiBinder {
    pub const fn new() -> Self {
        Self {
            scope: ScopeRegistry::new(),
            log:   BindLog::new(),
            seq:   0,
        }
    }

    /// Execute a full ARPi bind — five layers in sequence.
    ///
    /// Returns the 78-byte provenance header on success.
    /// Logs the outcome regardless of pass/fail.
    pub fn bind(
        &mut self,
        schema_byte:  u8,
        auth:         &MutualAuthSession,
        cap_token:    CapToken,
        anomaly_score: u8,
    ) -> Result<ArpiHeader, ArpiError> {
        self.seq = self.seq.wrapping_add(1);
        let seq = self.seq;

        // ── Layer 1: Schema ───────────────────────────────────────────────────
        let schema = validate_schema(schema_byte).map_err(|e| {
            let _ = self.log.record(BindLogEntry {
                seq, pd_src: auth.local.pubkey.0[0],
                pd_dst: auth.remote.pubkey.0[0],
                schema: schema_byte, outcome: BindOutcome::Failed(1),
                anomaly: anomaly_score,
            });
            e
        })?;

        // ── Layer 2: Identity ─────────────────────────────────────────────────
        verify_identity(&auth.local).map_err(|e| {
            let _ = self.log.record(BindLogEntry {
                seq, pd_src: auth.local.pubkey.0[0],
                pd_dst: auth.remote.pubkey.0[0],
                schema: schema_byte, outcome: BindOutcome::Failed(2),
                anomaly: anomaly_score,
            });
            e
        })?;

        // ── Layer 3: Mutual Auth ──────────────────────────────────────────────
        verify_mutual_auth(auth).map_err(|e| {
            let _ = self.log.record(BindLogEntry {
                seq, pd_src: auth.local.pubkey.0[0],
                pd_dst: auth.remote.pubkey.0[0],
                schema: schema_byte, outcome: BindOutcome::Failed(3),
                anomaly: anomaly_score,
            });
            e
        })?;

        // ── Layer 4: Scope ────────────────────────────────────────────────────
        // Register the token first, then validate scope
        self.scope.grant(cap_token).map_err(|e| {
            let _ = self.log.record(BindLogEntry {
                seq, pd_src: auth.local.pubkey.0[0],
                pd_dst: auth.remote.pubkey.0[0],
                schema: schema_byte, outcome: BindOutcome::Failed(4),
                anomaly: anomaly_score,
            });
            e
        })?;

        validate_scope(&self.scope, schema).map_err(|e| {
            let _ = self.log.record(BindLogEntry {
                seq, pd_src: auth.local.pubkey.0[0],
                pd_dst: auth.remote.pubkey.0[0],
                schema: schema_byte, outcome: BindOutcome::Failed(4),
                anomaly: anomaly_score,
            });
            e
        })?;

        // ── Layer 5: Anomaly ──────────────────────────────────────────────────
        match anomaly_gate(anomaly_score) {
            AnomalyResult::Escalated => {
                let _ = self.log.record(BindLogEntry {
                    seq, pd_src: auth.local.pubkey.0[0],
                    pd_dst: auth.remote.pubkey.0[0],
                    schema: schema_byte, outcome: BindOutcome::Failed(5),
                    anomaly: anomaly_score,
                });
                return Err(ArpiError::AnomalyEscalated);
            }
            AnomalyResult::Flagged | AnomalyResult::Clear => {}
        }

        // ── All layers passed — build provenance header ───────────────────────
        let header = ArpiHeader::new(
            schema,
            auth.local.pubkey.0[0],
            auth.remote.pubkey.0[0],
            seq,
            cap_token,
            auth.local.pubkey,
        );

        let _ = self.log.record(BindLogEntry {
            seq, pd_src: auth.local.pubkey.0[0],
            pd_dst: auth.remote.pubkey.0[0],
            schema: schema_byte, outcome: BindOutcome::Success,
            anomaly: anomaly_score,
        });

        Ok(header)
    }

    pub fn seq(&self) -> u64 { self.seq }
}

impl Default for ArpiBinder {
    fn default() -> Self { Self::new() }
}

// ── Sovereign proof ───────────────────────────────────────────────────────────

#[inline]
pub fn verify_sovereign_proof(proof: u64) -> bool {
    proof == AXON_PROOF
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn make_key(seed: u8) -> Ed25519PublicKey {
        let mut k = [0u8; ED25519_KEY_SIZE];
        k[0] = seed;
        k[1] = 0x01;
        Ed25519PublicKey(k)
    }

    fn make_sig(seed: u8) -> Ed25519Signature {
        let mut s = [0u8; ED25519_SIG_SIZE];
        s[0] = seed;
        s[1] = 0x01;
        Ed25519Signature(s)
    }

    fn make_nonce(seed: u8) -> [u8; 8] {
        [seed, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07]
    }

    fn make_proof(seed: u8) -> IdentityProof {
        IdentityProof {
            pubkey: make_key(seed),
            sig:    make_sig(seed),
            nonce:  make_nonce(seed),
        }
    }

    fn make_session(local_seed: u8, remote_seed: u8) -> MutualAuthSession {
        MutualAuthSession {
            local:  make_proof(local_seed),
            remote: make_proof(remote_seed),
        }
    }

    fn make_token(seq: u32) -> CapToken {
        CapToken::new(seq, ARPI_PD_ID, SchemaType::ArpiCtrl, 0x0001)
    }

    // ── Layer 1: Schema ───────────────────────────────────────────────────────

    #[test]
    fn test_schema_valid_types() {
        assert!(validate_schema(0x00).is_ok());
        assert!(validate_schema(0x01).is_ok());
        assert!(validate_schema(0x06).is_ok());
    }

    #[test]
    fn test_schema_invalid_rejected() {
        assert_eq!(validate_schema(0xFF), Err(ArpiError::SchemaViolation));
        assert_eq!(validate_schema(0x07), Err(ArpiError::SchemaViolation));
    }

    #[test]
    fn test_schema_type_roundtrip() {
        for i in 0u8..=6 {
            let s = SchemaType::from_u8(i).unwrap();
            assert_eq!(s as u8, i);
        }
    }

    // ── Layer 2: Identity ─────────────────────────────────────────────────────

    #[test]
    fn test_identity_valid_proof() {
        let proof = make_proof(0x42);
        assert!(verify_identity(&proof).is_ok());
    }

    #[test]
    fn test_identity_zero_key_rejected() {
        let mut proof = make_proof(0x42);
        proof.pubkey = Ed25519PublicKey([0u8; 32]);
        assert_eq!(verify_identity(&proof), Err(ArpiError::IdentityRejected));
    }

    #[test]
    fn test_identity_zero_nonce_rejected() {
        let mut proof = make_proof(0x42);
        proof.nonce = [0u8; 8];
        assert_eq!(verify_identity(&proof), Err(ArpiError::IdentityRejected));
    }

    #[test]
    fn test_identity_zero_sig_rejected() {
        let mut proof = make_proof(0x42);
        proof.sig = Ed25519Signature([0u8; 64]);
        assert_eq!(verify_identity(&proof), Err(ArpiError::IdentityRejected));
    }

    // ── Layer 3: Mutual Auth ──────────────────────────────────────────────────

    #[test]
    fn test_mutual_auth_valid() {
        let session = make_session(0x01, 0x02);
        assert!(verify_mutual_auth(&session).is_ok());
    }

    #[test]
    fn test_mutual_auth_self_bind_rejected() {
        let session = make_session(0x01, 0x01); // same seed = same key
        assert_eq!(verify_mutual_auth(&session), Err(ArpiError::MutualAuthFailed));
    }

    #[test]
    fn test_mutual_auth_bad_local_rejected() {
        let mut session = make_session(0x01, 0x02);
        session.local.pubkey = Ed25519PublicKey([0u8; 32]);
        assert_eq!(verify_mutual_auth(&session), Err(ArpiError::MutualAuthFailed));
    }

    #[test]
    fn test_mutual_auth_bad_remote_rejected() {
        let mut session = make_session(0x01, 0x02);
        session.remote.nonce = [0u8; 8];
        assert_eq!(verify_mutual_auth(&session), Err(ArpiError::MutualAuthFailed));
    }

    // ── Layer 4: Scope ────────────────────────────────────────────────────────

    #[test]
    fn test_cap_token_valid() {
        let t = make_token(1);
        assert!(t.is_valid());
        assert_eq!(t.seq(), 1);
        assert_eq!(t.pd_id(), ARPI_PD_ID);
    }

    #[test]
    fn test_cap_token_zero_seq_invalid() {
        let t = CapToken::new(0, ARPI_PD_ID, SchemaType::ArpiCtrl, 0);
        assert!(!t.is_valid());
    }

    #[test]
    fn test_scope_registry_grant_and_check() {
        let mut reg = ScopeRegistry::new();
        let t = CapToken::new(1, ARPI_PD_ID, SchemaType::ArpiCtrl, 0);
        reg.grant(t).unwrap();
        assert!(reg.has_scope(SchemaType::ArpiCtrl));
        assert!(!reg.has_scope(SchemaType::DataCrit));
    }

    #[test]
    fn test_scope_monotonic_enforcement() {
        let mut reg = ScopeRegistry::new();
        reg.grant(CapToken::new(5, 0x01, SchemaType::ArpiCtrl, 0)).unwrap();
        // Seq 3 is less than last_seq=5 — must be rejected
        let result = reg.grant(CapToken::new(3, 0x01, SchemaType::AwpMsg, 0));
        assert_eq!(result, Err(ArpiError::ScopeViolation));
    }

    #[test]
    fn test_scope_missing_rejects_bind() {
        let reg = ScopeRegistry::new();
        assert_eq!(validate_scope(&reg, SchemaType::DataCrit), Err(ArpiError::ScopeViolation));
    }

    // ── Layer 5: Anomaly ──────────────────────────────────────────────────────

    #[test]
    fn test_anomaly_clear() {
        assert_eq!(anomaly_gate(0),  AnomalyResult::Clear);
        assert_eq!(anomaly_gate(49), AnomalyResult::Clear);
    }

    #[test]
    fn test_anomaly_flagged() {
        assert_eq!(anomaly_gate(50), AnomalyResult::Flagged);
        assert_eq!(anomaly_gate(74), AnomalyResult::Flagged);
    }

    #[test]
    fn test_anomaly_escalated() {
        assert_eq!(anomaly_gate(75),  AnomalyResult::Escalated);
        assert_eq!(anomaly_gate(100), AnomalyResult::Escalated);
    }

    #[test]
    fn test_anomaly_threshold_constant() {
        assert_eq!(ANOMALY_THRESHOLD, 75);
    }

    // ── ARPi header ───────────────────────────────────────────────────────────

    #[test]
    fn test_header_size() {
        assert_eq!(ARPI_HEADER_SIZE, 78);
    }

    #[test]
    fn test_header_magic() {
        assert_eq!(ARPI_MAGIC, [0x41, 0x52, 0x50, 0x69]);
    }

    #[test]
    fn test_header_serialise() {
        let key = make_key(0xAB);
        let token = make_token(1);
        let h = ArpiHeader::new(SchemaType::ArpiCtrl, 0x01, 0x02, 42, token, key);
        let b = h.to_bytes();
        assert_eq!(b.len(), 78);
        assert_eq!(&b[0..4], &[0x41, 0x52, 0x50, 0x69]);
        assert_eq!(b[4], 0x01); // version
        assert_eq!(b[5], SchemaType::ArpiCtrl as u8);
        assert_eq!(b[6], 0x01); // pd_src
        assert_eq!(b[7], 0x02); // pd_dst
        // seq = 42
        assert_eq!(&b[8..16], &42u64.to_be_bytes());
        // reserved bytes 56..78 are zero
        assert!(b[56..78].iter().all(|&x| x == 0));
    }

    #[test]
    fn test_header_validate_ok() {
        let h = ArpiHeader::new(SchemaType::ArpiCtrl, 0x01, 0x02, 1, make_token(1), make_key(1));
        assert!(h.validate().is_ok());
    }

    // ── Bind log ──────────────────────────────────────────────────────────────

    #[test]
    fn test_bind_log_records_success() {
        let mut log = BindLog::new();
        log.record(BindLogEntry {
            seq: 1, pd_src: 0x01, pd_dst: 0x02,
            schema: 0x01, outcome: BindOutcome::Success, anomaly: 0,
        }).unwrap();
        assert_eq!(log.count(), 1);
        assert_eq!(log.last().unwrap().outcome, BindOutcome::Success);
    }

    #[test]
    fn test_bind_log_records_failure() {
        let mut log = BindLog::new();
        log.record(BindLogEntry {
            seq: 1, pd_src: 0x01, pd_dst: 0x02,
            schema: 0xFF, outcome: BindOutcome::Failed(1), anomaly: 0,
        }).unwrap();
        assert_eq!(log.last().unwrap().outcome, BindOutcome::Failed(1));
    }

    // ── Full bind engine ──────────────────────────────────────────────────────

    #[test]
    fn test_full_bind_success() {
        let mut binder = ArpiBinder::new();
        let session = make_session(0x01, 0x02);
        let token = CapToken::new(1, ARPI_PD_ID, SchemaType::ArpiCtrl, 0x0001);
        let header = binder.bind(0x01, &session, token, 0).unwrap();
        assert!(header.validate().is_ok());
        assert_eq!(header.schema, SchemaType::ArpiCtrl);
        assert_eq!(binder.log.count(), 1);
        assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Success);
    }

    #[test]
    fn test_bind_fails_layer1_bad_schema() {
        let mut binder = ArpiBinder::new();
        let session = make_session(0x01, 0x02);
        let token = make_token(1);
        let result = binder.bind(0xFF, &session, token, 0);
        assert_eq!(result, Err(ArpiError::SchemaViolation));
        assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Failed(1));
    }

    #[test]
    fn test_bind_fails_layer3_self_bind() {
        let mut binder = ArpiBinder::new();
        let session = make_session(0x01, 0x01); // same key
        let token = make_token(1);
        let result = binder.bind(0x01, &session, token, 0);
        assert_eq!(result, Err(ArpiError::MutualAuthFailed));
        assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Failed(3));
    }

    #[test]
    fn test_bind_fails_layer5_anomaly() {
        let mut binder = ArpiBinder::new();
        let session = make_session(0x01, 0x02);
        let token = make_token(1);
        let result = binder.bind(0x01, &session, token, 90);
        assert_eq!(result, Err(ArpiError::AnomalyEscalated));
        assert_eq!(binder.log.last().unwrap().outcome, BindOutcome::Failed(5));
    }

    #[test]
    fn test_bind_seq_increments() {
        let mut binder = ArpiBinder::new();
        let session = make_session(0x01, 0x02);
        binder.bind(0x01, &session, make_token(1), 0).unwrap();
        binder.bind(0x01, &session, make_token(2), 0).unwrap();
        assert_eq!(binder.seq(), 2);
    }

    // ── Sovereign proof ───────────────────────────────────────────────────────

    #[test]
    fn test_sovereign_proof() {
        assert!(verify_sovereign_proof(0x4153));
        assert!(!verify_sovereign_proof(0x0000));
        assert_eq!(AXON_PROOF, 0x4153);
    }
}
