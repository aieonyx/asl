# ASL Changelog

## [v0.1.0-asl-m18] — 2026-06-27 — DataTier-Enforcer Encryption

### What was built

**asl-crypto-bridge** — new crate

A sovereign crypto bridge providing AES-256-GCM encryption and Argon2id
key derivation for use inside seL4 Protection Domains. Designed no_std
so it can run directly in the DataTier-Enforcer PD without OS dependencies.

Public API:
- `derive_key(passphrase, salt)` — Argon2id KDF, returns 32-byte zeroizing key
- `encrypt(key, nonce, plaintext)` — AES-256-GCM with CRITICAL_AAD bound
- `decrypt(key, nonce, ciphertext)` — authenticated decryption
- `nonce_from_counter(u64)` — monotonic counter → 12-byte nonce

**asl-datatier** — DataTier-Enforcer PD updated

The DataTier-Enforcer PD now enforces three-tier policy with encryption:

| Tier     | Policy                                          |
|----------|-------------------------------------------------|
| Critical | AES-256-GCM encrypted at rest                  |
| Personal | ARPi provenance header, cleartext               |
| Noise    | Ephemeral — no persistence guarantee            |

The PD holds a single session key derived at boot via Argon2id. A monotonic
nonce counter prevents nonce reuse within a session.

### AUDIT-001 resolved

Critical tier data is no longer stored plaintext. This closes the audit
finding opened during Onyxia Browser C9 development.

### Formal verification

Six Kani harnesses in `asl-kani/src/crypto_proofs.rs`:

1. `key_len_invariant` — derived key always 32 bytes
2. `nonce_counter_monotonic` — nonce(n) ≠ nonce(n+1) for all n < u64::MAX
3. `encrypt_output_longer` — ciphertext always plaintext_len + 16
4. `decrypt_wrong_key_fails` — one-bit key flip → DecryptError (GCM auth)
5. `empty_input_rejected` — derive_key rejects empty passphrase or salt
6. `nonce_encoding_correctness` — counter encodes big-endian in nonce[0..8]

### Test counts

| Crate              | Tests |
|--------------------|-------|
| asl-crypto-bridge  |    10 |
| asl-datatier       |    11 |
| Kani harnesses     |     6 |
| **M18 total**      |**27** |
| Prior (M1–M17)     |   404 |
| **Workspace total**|**431**|

---

## [v0.1.0-asl-m17] — QEMU boot demo
## [v0.1.0-asl-m16] — AxonScript REPL
## [v0.1.0-asl-m15] — Phoenix Lite ISO first boot
