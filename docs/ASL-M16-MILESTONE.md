# ASL-M16 — AxonScript REPL in Phoenix-Console

**AIEONYX Sovereign Linux** · Apache 2.0
**Milestone:** `v0.1.0-asl-m16`
**GPG:** `B4C8548260DB40E1`
**Builds on:** M15 — Phoenix Lite ISO first boot (commit 5d72788)
**NLNet:** August 1, 2026 — interactive sovereign shell evidence

---

## Goal

Wire the AxonScript REPL stub left in Phoenix-Console (M15) into a real
interactive sovereign shell running inside seL4 under QEMU aarch64.

After M16 the boot sequence ends with:

```
phoenix@aieonyx:~$ _
```

...and accepts AxonScript expressions evaluated by the AXON-Bridge PD,
proving the full sovereign stack is interactive end-to-end.

---

## Deliverables

| # | Deliverable | Path |
|---|-------------|------|
| D1 | AxonScript REPL PD | `pds/asl-axon-repl/` |
| D2 | Phoenix-Console upgrade (stub → live REPL) | `pds/asl-phoenix-console/` |
| D3 | REPL→AXON-Bridge IPC protocol | `asl-common/src/repl_ipc.rs` |
| D4 | `phoenix-lite.system` update (new IPC channels) | `phoenix-lite.system` |
| D5 | Kani harnesses — 8 new proofs | `asl-kani/src/repl_proofs.rs` |
| D6 | `build-m16.sh` | `build-m16.sh` |
| D7 | M16 milestone doc | `docs/ASL-M16-MILESTONE.md` |

---

## Architecture

```
Phoenix-Console PD
  │  reads line from serial (ttyAMA0)
  │  sends to AXON-Bridge via IPC (label 0x8000)
  ▼
AXON-Bridge PD
  │  validates AxonScript expression (sovereign syntax check)
  │  evaluates: literals, arithmetic, let bindings, sovereign() builtin
  │  returns result string via IPC reply
  ▼
Phoenix-Console PD
  │  prints result
  │  prints next prompt: phoenix@aieonyx:~$
  ▼
  (loop)
```

## Built-in REPL commands (M16 scope)

| Command | Result |
|---------|--------|
| `sovereign()` | prints `axon_main() → 0x4153` |
| `pd_count()` | prints `10` |
| `version()` | prints `ASL v1.0 [seL4 15.0.0]` |
| `let x = <expr>` | binds name in session scope |
| `<arithmetic>` | evaluates i64 expression |
| `help` | lists commands |
| `exit` | halts Phoenix-Console cleanly |

---

## Test Count Target

| Source | M15 | M16 delta | M16 total |
|--------|-----|-----------|-----------|
| Track A unit | 313 | +12 | 325 |
| Kani harnesses | 40 | +8 | 48 |
| **Total** | **352** | **+20** | **372** |

---

## Sovereign Proof Continuity

`axon_main() → 0x4153` must be the return value of `sovereign()` in the REPL.
Kani harness `proof_repl_sovereign_builtin` formally verifies this.

---

## Git Workflow

```bash
git add -A
git commit -S -m "feat(m16): AxonScript REPL wired into Phoenix-Console"
git tag -s v0.1.0-asl-m16 -m "ASL-M16 AxonScript REPL sovereign shell"
git push origin main --tags
```

---

*AIEONYX — S4+i · 3P Doctrine · Post Doctrine (5-check gate active)*
