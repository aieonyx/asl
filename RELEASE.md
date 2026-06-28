# ASL v1.0 — AIEONYX Sovereign Layer
## Phoenix Desktop Release

**Release date:** 2026-06-28  
**GPG key:** B4C8548260DB40E1 (Edison Lepiten / AIEONYX Project Root Key v1)  
**Repository:** github.com/aieonyx/asl  
**License:** Apache-2.0

---

## What is ASL?

The AIEONYX Sovereign Layer (ASL) is a capability-based microkernel stack
built on seL4, designed to give individuals and communities full sovereignty
over their computing environment. ASL v1.0 delivers the complete Desktop
profile — the foundation of aixOs/Phoenix, the AIEONYX sovereign desktop OS.

ASL is not a fork. It is a new sovereign stack built from first principles
on the formally verified seL4 microkernel.

---

## v1.0 delivers: Phoenix Desktop profile

**Four sovereign nodes boot and link into a coherent desktop:**

| Node | Role | PDs |
|------|------|-----|
| PRIMARY | Phoenix OS — all mandatory PDs | 6 |
| BASTION | Security node — ARPi + TrustGraph + SOMA | 3 |
| DATASTORE | EdisonDB WAL+MVCC + AES-256-GCM | 2 |
| RENDERER | HANIEL 1280×720 ARGB8888 sovereign render | 2 |

**13 Protection Domains. 3 inter-node ARPi channels. Star topology.**  
**`axon_main() → 0x4153` verified on all 4 nodes.**

---

## Milestones delivered (M1–M24)

| Tag | Milestone | Key deliverable |
|-----|-----------|-----------------|
| v0.1.0-asl-m1  | GENESIS PD | First sovereign PD — 46 tests |
| v0.1.0-asl-m2  | ARPi-Broker PD | Sovereign IPC routing — 47 tests |
| v0.1.0-asl-m3  | Inverted Admin + TrustGraph | 63 tests |
| v0.1.0-asl-m4  | DataTier-Enforcer | Three-tier data — 50 tests |
| v0.1.0-asl-m5  | AXON-Bridge PD | Compiler bridge — 52 tests |
| v0.1.0-asl-m6  | First seL4 boot | GENESIS + ARPi on live seL4 |
| v0.1.0-asl-m7  | Driver PDs | Input + Storage + USB |
| v0.1.0-asl-m8  | Network PD | Aegis mesh + AWP stub |
| v0.1.0-asl-m9  | AXON userspace | axon_main() → 0x4153 on live seL4 |
| v0.1.0-asl-m10 | MCS scheduler | 7 PDs / 4 cores / 80% utilisation |
| v0.1.0-asl-m11 | HANIEL first pixel | 1280×720 ARGB8888 sovereign pixel |
| v0.1.0-asl-m12 | EdisonDB PD | WAL+MVCC + GDPR Art.17 |
| v0.1.0-asl-m13 | Onyxia Browser PD | awp:// + ✦ indicator |
| v0.1.0-asl-m14 | Kani verification | 52 formal harnesses |
| v0.1.0-asl-m15 | Phoenix Lite ISO | First boot — 10 PDs |
| v0.1.0-asl-m16 | AxonScript REPL | Sovereign evaluator PD |
| v0.1.0-asl-m17 | QEMU boot demo | 12/12 sovereign checks PASS |
| v0.1.0-asl-m18 | DataTier encryption | AES-256-GCM — AUDIT-001 resolved |
| v0.1.0-asl-m19 | HANIEL PD | WebKitGTK replaced for AWP URLs |
| v0.1.0-asl-m20 | AWP Protocol | Five-layer stack live in seL4 |
| v0.1.0-asl-m21 | ARPi live IPC | Full 5-layer sovereign auth |
| v0.1.0-asl-m22 | AXON migration | sovereign_arpi.ax → seL4 aarch64 ELF |
| v0.1.0-asl-m23 | Multi-node boot | Phoenix Desktop — SOVEREIGN READY |
| v0.1.0-asl-m24 | ASL v1.0 release | This release |

---

## Test suite

| Crate | Tests |
|-------|-------|
| asl-common (m1_sovereignty) | 46 |
| asl-arpi (m2_broker) | 47 |
| asl-inverted-admin + asl-trustgraph | 63 |
| asl-axon-bridge (m5_bridge) | 52 |
| asl-crypto-bridge | 10 |
| asl-datatier | 10 |
| asl-haniel | 57 |
| asl-awp | 55 |
| asl-arpi-ipc | 53 |
| asl-axon-migration | 28 |
| asl-multinode | 38 |
| Kani formal harnesses (M14) | 52 |
| Prior milestones (M1–M14) | 284 |
| **Total** | **655+** |
| **Failures** | **0** |

---

## Formal verification

52 Kani harnesses across all critical PDs.  
Properties proven: sovereignty invariants, capability isolation,
nonce monotonicity, buffer bounds, authentication correctness.

---

## Sovereign stack summary

```
seL4 15.0.0 / Microkit SDK 1.4.1
AXONYX compiler v0.66 (1606+ tests, P45–P66)
ARPi protocol v1.0 (AIEONYX-SPEC-ARPi-v1.0)
AWP protocol v1.0 (five-layer sovereign network)
HANIEL render surface (1280×720 ARGB8888)
EdisonDB (WAL+MVCC, AES-256-GCM, GDPR Art.17)
axon_main() → 0x4153 (sovereign proof, all nodes)
```

---

## Novel CS contributions

TERM-001–TERM-058 registered across Onyxia, EdisonDB, AXON, HANIEL, ASL.  
See AIEONYX CS Contributions Registry.

---

## What comes next

**ASL v2.0 — Mobile profile** (future)
- Touch-Input PD
- Cellular PD
- ARM Mali GPU driver
- Adaptive render surface

**ASL v3.0 — IoT profile** (future)
- Headless profile
- Sensor-Hub PD
- Real-time scheduler
- Minimal footprint

The ASL Core (6 mandatory PDs) is invariant across all profiles.

---

## GPG verification

All milestone tags are GPG-signed with key B4C8548260DB40E1.

```bash
git tag -v v0.1.0-asl-m24
gpg --verify
```

---

*Built by Edison Lepiten — AIEONYX Sovereign Founder*  
*Prague, Czech Republic — 2026*  
*"Sovereignty is not a feature. It is the foundation."*
