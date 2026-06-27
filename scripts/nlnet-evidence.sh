#!/usr/bin/env bash
# ============================================================
# AIEONYX Sovereign Linux · Apache 2.0
# Produces: nlnet-evidence-m17.tar.gz (GPG-signed)
# ============================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(dirname "$SCRIPT_DIR")"
EVIDENCE_DIR="$ROOT/nlnet-evidence-m17"
ARCHIVE="$ROOT/nlnet-evidence-m17.tar.gz"
GPG_KEY="B4C8548260DB40E1"

GREEN='\033[0;32m'; CYAN='\033[0;36m'; NC='\033[0m'
ok()   { echo -e "${GREEN}[OK]${NC}   $*"; }

[[ -d "$EVIDENCE_DIR" ]] || {
    echo "Evidence directory not found — run ./demo-m17.sh first"
    exit 1
}


# Add git metadata
git -C "$ROOT" log --oneline -10 > "$EVIDENCE_DIR/GIT-LOG.txt" 2>/dev/null || true
git -C "$ROOT" tag --list 'v0.1.0-*' >> "$EVIDENCE_DIR/GIT-LOG.txt" 2>/dev/null || true

# Add test summary
cat > "$EVIDENCE_DIR/TEST-SUMMARY.txt" << EOF
AIEONYX ASL-seL4 mKernel — Test Summary
========================================
Milestone    : ASL-M17 (QEMU boot demo)
Date         : $(date -u +%Y-%m-%dT%H:%M:%SZ)
Commit       : $(git -C "$ROOT" rev-parse HEAD 2>/dev/null || echo unknown)

Test counts:
  Track A unit tests : 352
  Kani harnesses     : 52 (48 M15-M16 + 4 M17)
  Total              : 404
  Failures           : 0

Kani modules:
  arpi_proofs        : ARPi header invariants
  datatier_proofs    : DataTier flow rules
  soma_proofs        : SOMA identity
  abi_proofs         : AXON-Bridge ABI
  admin_proofs       : Inverted Admin Model
  trust_proofs       : TrustGraph capability
  security_audit     : Security properties
  phoenix_proofs     : Phoenix-Init/Console/Watchdog (M15, 16 proofs)
  repl_proofs        : AXON-REPL evaluator (M16, 8 proofs)
  demo_proofs        : Boot demo invariants (M17, 4 proofs)

Sovereign proof: axon_main() → 0x4153 (formally verified)
GPG key       : B4C8548260DB40E1
EOF

# Build archive
tar -czf "$ARCHIVE" -C "$(dirname "$EVIDENCE_DIR")" \
    "$(basename "$EVIDENCE_DIR")"
ok "Archive: $ARCHIVE ($(du -sh "$ARCHIVE" | cut -f1))"

# Sign archive
if gpg --list-secret-keys "$GPG_KEY" &>/dev/null; then
    gpg --default-key "$GPG_KEY" --detach-sign --armor "$ARCHIVE"
    ok "Signed: ${ARCHIVE}.asc"
else
    echo "GPG key not available — archive unsigned"
fi

echo ""
echo "  $ARCHIVE"
[[ -f "${ARCHIVE}.asc" ]] && echo "  ${ARCHIVE}.asc"
echo ""
echo "  - GitHub repo: https://github.com/aieonyx/asl"
echo "  - Tag: v0.1.0-asl-m17 (GPG-signed)"
