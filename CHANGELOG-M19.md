# ASL M19 — HANIEL PD Sovereign Render Surface

**Tag:** v0.1.0-asl-m19
**Date:** 2026-06-28

## What was built

### asl-haniel — new Protection Domain crate

The HANIEL Protection Domain is the sovereign renderer for the AIEONYX
Sovereign Layer. It replaces WebKitGTK as the renderer for AWP URLs inside
the Onyxia Browser PD.

**URL routing policy:**

| Scheme    | Route                    |
|-----------|--------------------------|
| `awp://`  | HANIEL PD (this crate)   |
| `https://`| WebKitGTK legacy fallback|
| `http://` | BLOCKED — cleartext      |
| other     | BLOCKED                  |

**Capability policy (enforced by seL4):**

| Capability     | Status  |
|----------------|---------|
| DisplaySurface | GRANTED |
| FontRead       | GRANTED |
| Network        | DENIED  |
| StorageWrite   | DENIED  |

The renderer has zero network access (NetworkNone). It cannot exfiltrate
data even if compromised — enforced at the seL4 capability level.

**Render surface:** 1280×720 ARGB8888 (sovereign standard)

The `RenderSurface` struct provides:
- Flat pixel buffer, row-major, top-left origin
- `put_pixel` / `get_pixel` with silent OOB drop
- `clear` — fill entire surface
- `commit` — increment frame counter, reset render budget
- `spend_budget` — enforce per-frame render budget (1000 units)

### Sovereign proof

`axon_main() → 0x4153` remains invariant. Verified in `sovereign_proof_invariant`
Kani harness.

## Formal verification

Eight Kani harnesses in `asl-kani/src/haniel_proofs.rs`:

| Harness | Property proven |
|---------|----------------|
| `sovereign_proof_invariant` | AXON_PROOF == 0x4153 always |
| `awp_always_routes_haniel` | any `awp://` URL → Haniel |
| `http_always_blocked` | any `http://` URL → Block |
| `network_cap_always_denied` | Network cap never granted |
| `surface_dimensions_invariant` | surface always 1280×720 |
| `pixel_oob_safe` | OOB writes never panic |
| `budget_monotone_decrease` | spend_budget always decreases or errors |
| `frame_count_monotone` | commit() always increments |

## Test counts

| Suite | Tests | Status |
|-------|-------|--------|
| asl-haniel unit | 36 | PASS |
| asl-haniel integration (m19_haniel) | 30 | PASS |
| Kani harnesses | 8 | PASS |
| **M19 total** | **74** | **PASS** |
| Prior (M1–M18) | 431 | PASS |
| **Workspace total** | **505** | **0 failures** |

## Milestone chain

| Milestone | What |
|-----------|------|
| M15 | Phoenix Lite ISO first boot |
| M16 | AxonScript REPL PD |
| M17 | QEMU aarch64 boot demo |
| M18 | DataTier-Enforcer AES-256-GCM (AUDIT-001 resolved) |
| **M19** | **HANIEL PD — WebKitGTK replaced for AWP URLs** |
| M20 | AWP protocol live inside seL4 |
