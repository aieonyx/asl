
Copyright (c) 2026 Edison Lepiten / AIEONYX
SPDX-License-Identifier: Apache-2.0

Updated at each milestone completion.

## §1 Project Identity
- Name: AIEONYX ASL-seL4 mKernel
- Repository: github.com/aieonyx/asl
- License: Apache 2.0 (permanent — Community Promise)
- Framework: S4+i (Security · Sovereignty · Simplicity · Speed · +i Intelligence)

## §2 Milestone Evidence

### ASL-M0 — Workspace Scaffold (2026-06-24)
- Full Cargo workspace initialized
- Six mandatory sovereign PDs stubbed
- Post Doctrine gate active on all commits
- SOVEREIGNTY-LEDGER.md live


## §3 ASL-M14 Security Audit (2026-06-25)

### Kani Formal Verification Coverage
- ARPi header: size=78, magic=0xA291, valid-magic-validates, PD ID bounds
- DataTier: grant-iff-upgrade, same-tier-no-grant, total-order, from_u8-total
- SOMA: hash-size=32, incomplete-fails, distinct-layers-distinct-hashes, threshold-requires-all-three
- ABI: v1-always-valid, zero-missing, wrong-prefix-invalid
- Admin: devmode-always-not-active, all-actions-dual-key
- TrustGraph: self-grant-invalid, zero-seq-invalid, zero-sig-invalid, valid-validates

### Security Audit Summary
- 10 findings total: 9 CLEAR + 1 MITIGATED
- 0 DEFERRED findings
- A1 Memory Safety: CLEAR
- A2 Integer Overflow: CLEAR
- A3 Information Leakage: CLEAR
- A4 Replay Attacks: CLEAR
- A5 Privilege Escalation: CLEAR
- A6 Side Channels: MITIGATED (@constant_time at bridge boundary)
- A7 Spectre/Meltdown: CLEAR (seL4 formal proof)
- A8 AUDIT-001: CLEAR (resolved M12)
- A9 KNOWN-BUG-002/003: CLEAR (resolved M13)

Target: cs.AR — Computer Architecture
Title: ASL-seL4: A Formally Isolated Sovereign Microkernel with
       Capability-Flow Static Analysis and TriSec Identity Binding
