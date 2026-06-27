# ASL-M17 — QEMU aarch64 Full ISO Boot Demo

**AIEONYX Sovereign Linux** · Apache 2.0
**Milestone:** `v0.1.0-asl-m17`
**GPG:** `B4C8548260DB40E1`
**Builds on:** M16 — AxonScript REPL (commit e73b04a)

---

## Goal

Produce a clean, reproducible, GPG-signed QEMU aarch64 boot demo
that shows the full sovereign stack end-to-end:

```
seL4 boot → 10 PDs alive → Phoenix-Init → REPL → phoenix@aieonyx:~$
```

AIEONYX sovereign OS is real and running.

---

## Deliverables

| # | Deliverable | Path |
|---|-------------|------|
| D1 | Boot demo script (reproducible) | `demo-m17.sh` |
| D2 | Expected boot log (golden file) | `docs/boot-demo-m17.expected.log` |
| D3 | Log verification script | `scripts/verify-boot-log.sh` |
| D4 | CI workflow update (boot log gate) | `.github/workflows/asl-ci.yml` |
| D5 | Kani harnesses — 4 new proofs | `asl-kani/src/demo_proofs.rs` |
| D7 | M17 milestone doc | `docs/ASL-M17-MILESTONE.md` |

---

## Boot Demo Sequence (Expected Output)

```
╔══════════════════════════════════════════════════════╗
║         AIEONYX — Phoenix Lite v0.1.0                ║
║         ASL v1.0 [seL4 15.0.0]  S4+i Doctrine       ║
║         Sovereign Digital Civilization Stack          ║
╚══════════════════════════════════════════════════════╝

[GENESIS] Sovereign proof: axon_main() → 0x4153
[PHOENIX-INIT] Phase 1: Awakening
[PHOENIX-INIT] Phase 2: Sovereign Identity (SOMA)
[PHOENIX-INIT] Phase 3: Core Sovereign PDs
[PHOENIX-INIT] Phase 4: Hardware Discovery
[PHOENIX-INIT] Phase 5: Phoenix Console
[PHOENIX-INIT] Phase 6: Watchdog Armed
══════════════════════════════════════════════════════
  Phoenix Lite v0.1.0 — FIRST BOOT COMPLETE
  Track A ✓  Track B ✓  Track C ✓  M15 ✓  M16 ✓
  axon_main() → 0x4153 — proof anchored
══════════════════════════════════════════════════════

[CONSOLE] AxonScript REPL active
phoenix@aieonyx:~$ sovereign()
axon_main() -> 0x4153

phoenix@aieonyx:~$ pd_count()
10

phoenix@aieonyx:~$ version()
ASL v1.0 [seL4 15.0.0]

phoenix@aieonyx:~$ let x = 21 + 21
42

phoenix@aieonyx:~$
```

---


```
nlnet-evidence-m17/
├── boot-demo-m17.log          # Full QEMU boot log (raw)
├── boot-demo-m17.log.asc      # GPG signature
├── boot-demo-m17-clean.log    # Stripped ANSI, human-readable
├── VERIFICATION.txt           # Check results + sovereign proof
├── BUILD-REPRO.md             # Reproduction instructions
└── SHA256SUMS                 # Checksums of all files
```

---

## Verification Checks (verify-boot-log.sh)

| # | Check | Pattern |
|---|-------|---------|
| 1 | AIEONYX banner present | `AIEONYX` |
| 2 | Sovereign proof value | `0x4153` |
| 3 | seL4 version | `seL4 15.0.0` |
| 4 | S4+i Doctrine | `S4+i` |
| 5 | GENESIS PD active | `GENESIS` |
| 6 | First boot complete | `FIRST BOOT COMPLETE` |
| 7 | Track markers | `Track A` |
| 8 | REPL active | `AxonScript REPL` |
| 9 | REPL sovereign() | `axon_main() -> 0x4153` |
| 10 | Shell prompt | `phoenix@aieonyx` |
| 11 | pd_count result | `pd_count()` |
| 12 | REPL version | `version()` |

---

## Git Workflow

```bash
git add -A
git push origin main --tags
```

---

*AIEONYX — S4+i · 3P Doctrine · Post Doctrine (5-check gate active)*
