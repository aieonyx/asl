# SOVEREIGNTY LEDGER — ASL-seL4 mKernel

Copyright (c) 2026 Edison Lepiten / AIEONYX
SPDX-License-Identifier: Apache-2.0

This ledger records every milestone completion for the ASL-seL4 mKernel.
Each entry requires: GPG-signed tag, test count, Post Doctrine gate pass.

---

## Format

| Field | Value |
|---|---|
| Milestone | ASL-Mx |
| Version | ASL vX.X.X [seL4 15.0.0] |
| GPG Tag | vX.X.X-asl-mx |
| Tests | N passing / 0 failures |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| NLNet Evidence | EXHIBIT.md §X |
| Date | YYYY-MM-DD |

---

## Ledger

### ASL-M0 — Workspace Scaffold
| Field | Value |
|---|---|
| Milestone | ASL-M0 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| GPG Tag | pending |
| Tests | 0 (scaffold only) |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Date | 2026-06-24 |


### ASL-M1 — GENESIS Root Task
| Field | Value |
|---|---|
| Milestone | ASL-M1 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| GPG Tag | pending v0.1.1 |
| Tests | 46 passing / 0 failures |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Coverage | version · PdId · ARPi · DataTier · commissioning |
| Date | 2026-06-24 |

### ASL-M2 — ARPi-Broker PD
| Field | Value |
|---|---|
| Milestone | ASL-M2 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| Tests | 47 passing / 0 failures |
| Cumulative | 93 passing / 0 failures |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Coverage | broker · route · sequence · tier gate |
| Date | 2026-06-24 |

### ASL-M3 — Inverted-Admin + TrustGraph-Gate PD
| Field | Value |
|---|---|
| Milestone | ASL-M3 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| Tests | 63 passing / 0 failures |
| Cumulative | 156 passing / 0 failures |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Coverage | inverted-admin · dual-key · trust graph · cap tokens · trust score |
| Date | 2026-06-24 |

### ASL-M4 — DataTier-Enforcer PD
| Field | Value |
|---|---|
| Milestone | ASL-M4 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| Tests | 50 passing / 0 failures |
| Cumulative | 206 passing / 0 failures |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Coverage | flow · grant · audit · erasure · vault enforcement |
| Date | 2026-06-24 |

### ASL-M4.5 — SOMA-Identity PD (TriSec Point A)
| Field | Value |
|---|---|
| Milestone | ASL-M4.5 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| Tests | 55 passing / 0 failures |
| Cumulative | 261 passing / 0 failures |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Coverage | composite identity · threshold · binding · BindingMode |
| TriSec | Point A implemented — HW+kernel+OS+biometric → 32-byte hash |
| Date | 2026-06-24 |

### ASL-M5 — AXON-Bridge PD (Final Mandatory Sovereign PD)
| Field | Value |
|---|---|
| Milestone | ASL-M5 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| Tests | 52 passing / 0 failures |
| Cumulative | 313 passing / 0 failures |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Coverage | ABI contract · cap translation · @constant_time · AXON-STUB-001 |
| First .ax | ipc_bridge.ax + sovereign_identity.ax in ASL repo |
| Status | ALL SIX MANDATORY PDs COMPLETE |
| Date | 2026-06-24 |

### ASL-M6 — First Microkit Boot (GENESIS + ARPi-Broker on seL4)
| Field | Value |
|---|---|
| Milestone | ASL-M6 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| Board | qemu_virt_aarch64 debug |
| Boot | VERIFIED — seL4 kernel bootstrapped, two PDs active |
| Pattern | ASL v1.5 hybrid: C Microkit shim + Rust sovereignty staticlib |
| Key output | GENESIS ceremony + ARPi-Broker IPC READY on real seL4 |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Date | 2026-06-24 |

### ASL-M7 — Input + Storage + USB Driver PDs
| Field | Value |
|---|---|
| Milestone | ASL-M7 |
| Version | ASL v0.1.0 [seL4 15.0.0] |
| Board | qemu_virt_aarch64 debug |
| PDs booted | 5 — GENESIS · ARPi · Input · Storage · USB |
| Boot | VERIFIED — all 5 PDs active, interleaved serial output |
| TriSec | Point A ID-1 (HW-UID) ACTIVE on real seL4 |
| SOMA | Hardware identity sentinel verified — USB key fingerprint matched |
| Post Doctrine | P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓ |
| Date | 2026-06-25 |
