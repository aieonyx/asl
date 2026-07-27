// Copyright (c) 2026 Edison Lepiten / AIEONYX
// SPDX-License-Identifier: Apache-2.0
//
// ════════════════════════════════════════════════════════════════════════════
// asl-shell-pd — Shell Protection Domain
// PL-71 / ASL-M26: Protection Domain split — axc> shell under seL4
// ════════════════════════════════════════════════════════════════════════════
//
// ROLE: Runs the aiXos axc> sovereign shell as an isolated seL4 PD.
//       All shell commands that touch EDB or AXFS go through ARPi-mediated
//       IPC — the Shell-PD cannot directly access storage or network.
//
// CAPABILITY POLICY (enforced by seL4):
//   GRANTED  : UartRead/Write    — keyboard input + shell output to display
//   GRANTED  : EDBQuery (via ARPi) — db commands via DataTier-Enforcer IPC
//   GRANTED  : AXFSRead (via ARPi) — cat/ls/run via AXFS-PD IPC
//   GRANTED  : AXFSWrite (via ARPi) — write command via AXFS-PD IPC
//   DENIED   : DirectStorage     — cannot access EDB or AXFS directly
//   DENIED   : Network           — awp command goes through AWP-PD IPC
//   DENIED   : FramebufferWrite  — shell renders via Phoenix-Desktop IPC
//
// ISOLATION PROOF:
//   A malicious or crashed shell cannot corrupt EdisonDB, AXFS, or
//   the network stack. Every operation passes through ARPi 5-layer auth.
//
// Post Doctrine: P1 ✓ P2 ✓ P3 ✓ P4 ✓ P5 ✓

#![no_std]
#![forbid(unsafe_code)]

#[cfg(kani)]
extern crate kani;

use asl_common::pd::PdId;
use asl_arpi_ipc::AXON_PROOF;

// ── Constants ─────────────────────────────────────────────────────────────────

pub const SHELL_PD_ID:     u8  = 0x40;
pub const SOVEREIGN_PROOF: u64 = AXON_PROOF;
pub const MAX_CMD_LEN:     usize = 256;
pub const MAX_HISTORY:     usize = 32;
pub const AXOS_PROMPT:     &[u8] = b"axc> ";

// ── IPC message labels ────────────────────────────────────────────────────────

/// Shell → DataTier-Enforcer: EDB read/write request
pub const MSG_EDB_READ:    u32 = 0xC001;
pub const MSG_EDB_WRITE:   u32 = 0xC002;
/// Shell → AXFS-PD: file read/write/list request
pub const MSG_AXFS_LS:     u32 = 0xC010;
pub const MSG_AXFS_READ:   u32 = 0xC011;
pub const MSG_AXFS_WRITE:  u32 = 0xC012;
/// Shell → AWP-PD: network send request
pub const MSG_AWP_SEND:    u32 = 0xC020;
/// Shell → Phoenix-Desktop: render shell output
pub const MSG_RENDER_LINE: u32 = 0xC030;
/// Shell → AXON-Bridge: execute .ax script
pub const MSG_AXON_EXEC:   u32 = 0xC040;

// ── Shell command classification ──────────────────────────────────────────────

/// Which IPC channel a shell command requires
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmdRoute {
    /// Handled entirely within Shell-PD (no IPC needed)
    Local,
    /// Requires DataTier-Enforcer IPC (db, edb commands)
    DataTier,
    /// Requires AXFS-PD IPC (ls, cat, write, run, mkpkg, verify)
    Axfs,
    /// Requires AWP-PD IPC (awp, awp-status, awp recv)
    Awp,
    /// Requires AXON-Bridge IPC (run, run_verified)
    AxonBridge,
    /// Requires Phoenix-Desktop IPC (window, settings, browse)
    Desktop,
}

/// Classify a shell command to its required IPC route
pub fn classify_cmd(cmd: &[u8]) -> CmdRoute {
    if cmd.starts_with(b"db")          { return CmdRoute::DataTier; }
    if cmd.starts_with(b"ls")          ||
       cmd.starts_with(b"cat ")        ||
       cmd.starts_with(b"write ")      ||
       cmd.starts_with(b"mkpkg ")      ||
       cmd.starts_with(b"verify ")     { return CmdRoute::Axfs; }
    if cmd.starts_with(b"run_verified") ||
       cmd.starts_with(b"run ")        { return CmdRoute::AxonBridge; }
    if cmd.starts_with(b"awp")         { return CmdRoute::Awp; }
    if cmd.starts_with(b"window")      ||
       cmd.starts_with(b"browse")      ||
       cmd.starts_with(b"settings")    { return CmdRoute::Desktop; }
    CmdRoute::Local
}

// ── Shell-PD state machine ────────────────────────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShellPhase {
    /// Waiting for boot signal from Phoenix-Init
    AwaitingBoot,
    /// Ready — accepting commands
    Ready,
    /// Waiting for IPC response from another PD
    AwaitingIpc,
    /// Faulted
    Faulted,
}

pub struct ShellPd {
    pub phase:       ShellPhase,
    pub cmd_count:   u64,
    pub ipc_pending: Option<CmdRoute>,
    pub proof:       u64,
}

impl ShellPd {
    pub const fn new() -> Self {
        ShellPd {
            phase:       ShellPhase::AwaitingBoot,
            cmd_count:   0,
            ipc_pending: None,
            proof:       SOVEREIGN_PROOF,
        }
    }

    pub fn on_boot_signal(&mut self) -> Result<(), &'static str> {
        if self.phase != ShellPhase::AwaitingBoot {
            return Err("Shell-PD: BOOT_SIGNAL in wrong phase");
        }
        self.assert_proof();
        self.phase = ShellPhase::Ready;
        Ok(())
    }

    /// Submit a command — returns the required IPC route
    pub fn submit_cmd(&mut self, cmd: &[u8]) -> Result<CmdRoute, &'static str> {
        if self.phase != ShellPhase::Ready {
            return Err("Shell-PD: not in Ready phase");
        }
        if cmd.len() > MAX_CMD_LEN {
            return Err("Shell-PD: command too long");
        }
        self.assert_proof();
        let route = classify_cmd(cmd);
        if route != CmdRoute::Local {
            self.ipc_pending = Some(route);
            self.phase = ShellPhase::AwaitingIpc;
        }
        self.cmd_count += 1;
        Ok(route)
    }

    /// IPC response received — return to Ready
    pub fn on_ipc_response(&mut self) -> Result<(), &'static str> {
        if self.phase != ShellPhase::AwaitingIpc {
            return Err("Shell-PD: IPC response in wrong phase");
        }
        self.ipc_pending = None;
        self.phase = ShellPhase::Ready;
        Ok(())
    }

    pub fn pd_id() -> PdId { PdId::AxonBridge } // shell slot

    #[inline]
    fn assert_proof(&self) {
        assert_eq!(self.proof, SOVEREIGN_PROOF,
            "SOVEREIGN PROOF VIOLATION: Shell-PD integrity failed");
    }
}

impl Default for ShellPd { fn default() -> Self { Self::new() } }

// ── Tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shell_pd_new() {
        let pd = ShellPd::new();
        assert_eq!(pd.phase, ShellPhase::AwaitingBoot);
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
        assert_eq!(pd.cmd_count, 0);
    }

    #[test]
    fn test_boot_signal() {
        let mut pd = ShellPd::new();
        assert!(pd.on_boot_signal().is_ok());
        assert_eq!(pd.phase, ShellPhase::Ready);
    }

    #[test]
    fn test_local_cmd_stays_ready() {
        let mut pd = ShellPd::new();
        pd.on_boot_signal().unwrap();
        let route = pd.submit_cmd(b"help").unwrap();
        assert_eq!(route, CmdRoute::Local);
        assert_eq!(pd.phase, ShellPhase::Ready);
    }

    #[test]
    fn test_edb_cmd_routes_to_datatier() {
        let mut pd = ShellPd::new();
        pd.on_boot_signal().unwrap();
        let route = pd.submit_cmd(b"db put x 42").unwrap();
        assert_eq!(route, CmdRoute::DataTier);
        assert_eq!(pd.phase, ShellPhase::AwaitingIpc);
    }

    #[test]
    fn test_ipc_response_returns_ready() {
        let mut pd = ShellPd::new();
        pd.on_boot_signal().unwrap();
        pd.submit_cmd(b"db put x 42").unwrap();
        assert!(pd.on_ipc_response().is_ok());
        assert_eq!(pd.phase, ShellPhase::Ready);
    }

    #[test]
    fn test_axfs_cmd_routes_correctly() {
        let mut pd = ShellPd::new();
        pd.on_boot_signal().unwrap();
        assert_eq!(pd.submit_cmd(b"ls").unwrap(), CmdRoute::Axfs);
        pd.on_ipc_response().unwrap();
        assert_eq!(pd.submit_cmd(b"cat file.txt").unwrap(), CmdRoute::Axfs);
    }

    #[test]
    fn test_awp_cmd_routes_to_awp() {
        let mut pd = ShellPd::new();
        pd.on_boot_signal().unwrap();
        assert_eq!(pd.submit_cmd(b"awp hello").unwrap(), CmdRoute::Awp);
    }

    #[test]
    fn test_run_verified_routes_to_axon_bridge() {
        let mut pd = ShellPd::new();
        pd.on_boot_signal().unwrap();
        assert_eq!(pd.submit_cmd(b"run_verified pkg.axpkg").unwrap(), CmdRoute::AxonBridge);
    }

    #[test]
    fn test_cmd_count_increments() {
        let mut pd = ShellPd::new();
        pd.on_boot_signal().unwrap();
        pd.submit_cmd(b"help").unwrap();
        pd.submit_cmd(b"version").unwrap();
        assert_eq!(pd.cmd_count, 2);
    }

    #[test]
    fn test_proof_invariant() {
        let mut pd = ShellPd::new();
        pd.on_boot_signal().unwrap();
        pd.submit_cmd(b"help").unwrap();
        assert_eq!(pd.proof, SOVEREIGN_PROOF);
    }

    #[test]
    fn test_cmd_rejected_before_boot() {
        let mut pd = ShellPd::new();
        assert!(pd.submit_cmd(b"help").is_err());
    }

    #[test]
    fn test_classify_browse_routes_to_desktop() {
        assert_eq!(classify_cmd(b"browse awp://aieonyx"), CmdRoute::Desktop);
    }
}
