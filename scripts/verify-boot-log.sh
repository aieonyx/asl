#!/usr/bin/env bash
# ============================================================
# ASL-M17 — Boot Log Verification Script
# AIEONYX Sovereign Linux · Apache 2.0
# Usage: ./verify-boot-log.sh <logfile>
#        ./verify-boot-log.sh  (uses default evidence path)
# ============================================================
set -euo pipefail

LOG="${1:-$(dirname "$0")/../nlnet-evidence-m17/boot-demo-m17-clean.log}"

RED='\033[0;31m'; GREEN='\033[0;32m'; CYAN='\033[0;36m'
BOLD='\033[1m'; NC='\033[0m'

[[ -f "$LOG" ]] || { echo -e "${RED}[FAIL]${NC} Log not found: $LOG"; exit 1; }

echo -e "${BOLD}ASL-M17 Boot Log Verification${NC}"
echo -e "Log: $LOG"
echo ""

pass=0; fail=0

check() {
    local pattern="$1" label="$2"
    if grep -q "$pattern" "$LOG"; then
        echo -e "${GREEN}  PASS${NC}  $label"
        ((pass++)) || true
    else
        echo -e "${RED}  FAIL${NC}  $label"
        ((fail++)) || true
    fi
}

# ── Sovereign OS markers ───────────────────────────────────
check "AIEONYX"                "1. AIEONYX banner"
check "0x4153"                 "2. Sovereign proof 0x4153"
check "seL4 15.0.0"           "3. seL4 version 15.0.0"
check "S4+i"                  "4. S4+i Doctrine"
check "GENESIS"               "5. GENESIS PD active"
check "FIRST BOOT COMPLETE"   "6. First boot complete marker"
check "Track A"               "7. Track A confirmed"
check "AxonScript REPL"       "8. REPL active"
check "axon_main() -> 0x4153" "9. REPL sovereign() = 0x4153"
check "phoenix@aieonyx"       "10. Shell prompt"
check "pd_count()"            "11. pd_count() builtin"
check "version()"             "12. version() builtin"

echo ""
echo -e "${BOLD}Result: ${GREEN}$pass PASS${NC}  ${RED}$fail FAIL${NC} / $((pass+fail)) checks${NC}"

if [[ $fail -eq 0 ]]; then
    echo -e "${GREEN}${BOLD}VERIFIED — Phoenix Lite sovereign boot confirmed${NC}"
    exit 0
else
    echo -e "${RED}${BOLD}FAILED — $fail checks did not pass${NC}"
    exit 1
fi
