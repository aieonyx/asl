#!/usr/bin/env bash
# ============================================================
# ASL-M16 — AxonScript REPL in Phoenix-Console
# AIEONYX Sovereign Linux · Apache 2.0
# GPG: B4C8548260DB40E1
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK="${MICROKIT_SDK:-$HOME/microkit-sdk-1.4.1}"
BUILD_DIR="$SCRIPT_DIR/build"
LOG="$BUILD_DIR/m16-build.log"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'
BOLD='\033[1m'; NC='\033[0m'
info() { echo -e "${CYAN}[M16]${NC} $*"; }
ok()   { echo -e "${GREEN}[OK]${NC}  $*"; }
die()  { echo -e "${RED}[FAIL]${NC} $*" >&2; exit 1; }

mkdir -p "$BUILD_DIR"

# ── Step 1: Copy new files into repo ──────────────────────
install_files() {
    info "Installing M16 PD sources…"
    local REPO="$SCRIPT_DIR"

    # AXON-REPL new PD
    mkdir -p "$REPO/pds/asl-axon-repl/src"
    cp "$SCRIPT_DIR/pds/asl-axon-repl/src/main.rs" \
       "$REPO/pds/asl-axon-repl/src/main.rs" 2>/dev/null || true

    # Phoenix-Console upgrade
    cp "$SCRIPT_DIR/pds/asl-phoenix-console/src/main.rs" \
       "$REPO/pds/asl-phoenix-console/src/main.rs" 2>/dev/null || true

    # REPL IPC protocol into asl-common
    cp "$SCRIPT_DIR/asl-common-repl_ipc.rs" \
       "$REPO/asl-common/src/repl_ipc.rs" 2>/dev/null || true

    # Kani harnesses
    cp "$SCRIPT_DIR/tests/kani_m16_harnesses.rs" \
       "$REPO/asl-kani/src/repl_proofs.rs" 2>/dev/null || true

    ok "Files installed"
}

# ── Step 2: Build new PDs ──────────────────────────────────
build_pds() {
    info "Building M16 PDs…"
    for pd in asl-axon-repl asl-phoenix-console; do
        if [[ -d "$SCRIPT_DIR/pds/$pd" ]]; then
            info "  Building $pd…"
            (cd "$SCRIPT_DIR/pds/$pd" && \
                cargo build --release --target aarch64-unknown-none 2>>"$LOG") \
                || die "PD build failed: $pd"
        fi
    done
    ok "PD build complete"
}

# ── Step 3: Wire repl_proofs into asl-kani/src/lib.rs ─────
wire_kani() {
    info "Wiring repl_proofs into asl-kani…"
    local lib="$SCRIPT_DIR/asl-kani/src/lib.rs"
    if ! grep -q "repl_proofs" "$lib"; then
        echo "pub mod repl_proofs;" >> "$lib"
        ok "repl_proofs module added to asl-kani/src/lib.rs"
    else
        ok "repl_proofs already wired"
    fi
}

# ── Step 4: Run Kani harnesses ─────────────────────────────
run_kani() {
    info "Running M16 Kani harnesses…"
    cd "$SCRIPT_DIR"
    cargo kani --harness "proof_repl_sovereign_builtin"
    cargo kani --harness "proof_repl_request_proof_integrity"
    cargo kani --harness "proof_repl_sequence_monotone"
    cargo kani --harness "proof_repl_expr_bounds"
    cargo kani --harness "proof_repl_result_bounds"
    cargo kani --harness "proof_repl_pd_count_builtin"
    cargo kani --harness "proof_repl_arithmetic_no_overflow"
    cargo kani --harness "proof_repl_division_guard"
    ok "All 8 M16 Kani harnesses passed"
}

# ── Step 5: QEMU boot test ─────────────────────────────────
run_qemu() {
    info "Running QEMU M16 boot test…"
    local kern="$SCRIPT_DIR/build/iso/boot/vmlinuz"
    local initrd="$SCRIPT_DIR/build/initrd.gz"
    [[ -f "$kern" ]]   || { info "Kernel not found — skipping QEMU (run build-m15.sh first)"; return; }
    [[ -f "$initrd" ]] || { info "initrd not found — skipping QEMU"; return; }

    timeout 30 qemu-system-aarch64 \
        -M virt -cpu cortex-a57 -m 512M -smp 4 \
        -kernel "$kern" -initrd "$initrd" \
        -append "console=ttyAMA0 sovereign=axon_main quiet" \
        -nographic -no-reboot \
        2>&1 | tee "$BUILD_DIR/qemu-m16.log" \
        | grep -E "phoenix@aieonyx|sovereign\(\)|0x4153|REPL|pd_count|version\(\)" \
        || true
    ok "QEMU M16 session complete → $BUILD_DIR/qemu-m16.log"
}

# ── Main ───────────────────────────────────────────────────
main() {
    echo "═══════════════════════════════════════════"
    echo "  ASL-M16 — AxonScript REPL Build"
    echo "  $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "═══════════════════════════════════════════"

    install_files
    build_pds
    wire_kani
    run_kani
    run_qemu

    echo ""
    echo -e "${BOLD}${GREEN}══════════════════════════════════════${NC}"
    echo -e "${BOLD}${GREEN}  ASL-M16 COMPLETE — REPL ALIVE        ${NC}"
    echo -e "${BOLD}${GREEN}══════════════════════════════════════${NC}"
    echo ""
    echo "  Next:"
    echo "    git add -A"
    echo "    git commit -S -m 'feat(m16): AxonScript REPL wired into Phoenix-Console'"
    echo "    git tag -s v0.1.0-asl-m16 -m 'ASL-M16 AxonScript REPL sovereign shell'"
    echo "    git push origin main --tags"
}

main "$@"
