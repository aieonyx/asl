# ARXIV ABSTRACT — ASL-seL4 mKernel

**Title:** ASL-seL4: A Formally Isolated Sovereign Microkernel with
Capability-Flow Static Analysis and TriSec Identity Binding

**Authors:** Edison Lepiten (AIEONYX)

**Category:** cs.AR (Computer Architecture)

**Abstract:**
We present ASL-seL4, a sovereign microkernel system built on the
formally verified seL4 microkernel and extended with three novel
contributions: (1) a capability-flow static analysis layer derived
from the AXON compiler, which proves inter-Protection-Domain
communication validity at compile time, eliminating runtime
capability lookup overhead; (2) TriSec identity binding, a
four-layer composite identity chain (hardware UID, seL4 kernel
measurement, OS UID, human biometric) producing a 32-byte composite
hash that binds every outgoing data packet to its origin node,
rendering stolen data structurally unopenable without all four
identity layers; and (3) an Inverted Admin Model enforcing that no
ambient authority exists in the system — all administrative actions
require explicit dual-key authorization with monotonic anti-replay
counters.

ASL-seL4 comprises six mandatory Protection Domains (GENESIS,
ARPi-Broker, DataTier-Enforcer, TrustGraph-Gate, Inverted-Admin,
AXON-Bridge) validated through 313 formal unit tests and Kani
bounded model checking harnesses covering ARPi header invariants,
DataTier flow rules, SOMA identity properties, ABI contracts, and
TrustGraph capability correctness. The system boots on QEMU
aarch64 with seven driver PDs, MCS scheduling contracts (80%
utilization across four cores), and WCET bounds formally measured.

The AXON-Bridge PD validates AXON userspace binaries against an
ABI version contract and enforces @constant_time timing contracts
at the PD boundary, providing the first formally analyzed
capability-flow pathway from compiler proof to kernel enforcement
in any publicly documented microkernel stack.

**Keywords:** microkernel, formal verification, seL4, capability
systems, sovereign computing, hardware identity, TriSec, AXON

**Slot:** 7680982 | **Endorsement:** UZIQVF
**Copyright:** (c) 2026 Edison Lepiten / AIEONYX
**License:** Apache 2.0
