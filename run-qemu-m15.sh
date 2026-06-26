#!/usr/bin/env bash
# ============================================================
# ASL-M15 — QEMU Phoenix Lite ISO Boot Runner
# AIEONYX Sovereign Linux · Apache 2.0
# Usage:
#   ./run-qemu-m15.sh             # Interactive (default)
#   ./run-qemu-m15.sh --ci        # Non-interactive, timeout 30s
#   ./run-qemu-m15.sh --kernel    # Direct kernel+initrd (no ISO)
#   ./run-qemu-m15.sh --check     # Verify QEMU output, exit 0/1
# ============================================================
set -euo pipefail

BUILD_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/build"
ISO="$BUILD_DIR/phoenix-lite-v0.1.0.iso"
KERNEL="$BUILD_DIR/iso/boot/vmlinuz"
INITRD="$BUILD_DIR/initrd.gz"
QEMU_LOG="$BUILD_DIR/qemu-m15.log"

MODE="${1:-}"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'

info() { echo -e "${CYAN}[QEMU]${NC} $*"; }
ok()   { echo -e "${GREEN}[OK]${NC}  $*"; }
die()  { echo -e "${RED}[FAIL]${NC} $*" >&2; exit 1; }

# ── Common QEMU args ───────────────────────────────────────
QEMU_BASE=(
    qemu-system-aarch64
    -M    virt
    -cpu  cortex-a57
    -m    512M
    -smp  4
    -nographic
)

# ── ISO boot (GRUB) ────────────────────────────────────────
boot_iso() {
    info "ISO boot: $ISO"
    [[ -f "$ISO" ]] || die "ISO not found — run ./build-m15.sh first"
    "${QEMU_BASE[@]}" \
        -drive file="$ISO",format=raw,if=virtio,readonly=on \
        -no-reboot \
        "$@"
}

# ── Direct kernel boot (no GRUB, faster for CI) ───────────
boot_kernel() {
    info "Direct kernel boot: $KERNEL"
    [[ -f "$KERNEL" ]] || die "Kernel not found"
    [[ -f "$INITRD" ]] || die "initrd not found"
    "${QEMU_BASE[@]}" \
        -kernel "$KERNEL" \
        -initrd "$INITRD" \
        -append "console=ttyAMA0 sovereign=axon_main quiet" \
        -no-reboot \
        "$@"
}

case "$MODE" in

    --ci)
        info "CI mode — 30s timeout, capturing output"
        timeout 30 bash -c "$(declare -f boot_kernel); boot_kernel" \
            2>&1 | tee "$QEMU_LOG" || true
        ok "CI run complete → $QEMU_LOG"
        ;;

    --check)
        info "Checking QEMU log for sovereign boot markers…"
        [[ -f "$QEMU_LOG" ]] || die "No QEMU log — run with --ci first"
        PASS=0; FAIL=0

        check() {
            local pattern="$1" label="$2"
            if grep -q "$pattern" "$QEMU_LOG"; then
                echo -e "${GREEN}  PASS${NC}  $label"
                ((PASS++))
            else
                echo -e "${RED}  FAIL${NC}  $label"
                ((FAIL++))
            fi
        }

        check "AIEONYX"          "Banner: AIEONYX present"
        check "Phoenix Lite"     "Banner: Phoenix Lite"
        check "0x4153"           "Sovereign proof value"
        check "seL4"             "seL4 version mention"
        check "S4+i"             "S4+i Doctrine"
        check "GENESIS"          "GENESIS PD active"
        check "Track A"          "Track A confirmed"
        check "Track B"          "Track B confirmed"
        check "FIRST BOOT"       "First boot complete marker"
        check "axon_main"        "axon_main proof reference"

        echo ""
        echo -e "M15 QEMU checks: ${GREEN}$PASS PASS${NC}  ${RED}$FAIL FAIL${NC}"
        [[ $FAIL -eq 0 ]] && ok "All M15 boot markers verified" || die "QEMU boot check failed"
        ;;

    --kernel)
        info "Interactive kernel boot (Ctrl-A X to quit QEMU)"
        boot_kernel
        ;;

    "")
        info "Interactive ISO boot (Ctrl-A X to quit QEMU)"
        boot_iso
        ;;

    *)
        echo "Usage: $0 [--ci|--check|--kernel]"
        exit 1
        ;;
esac
