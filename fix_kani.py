#!/usr/bin/env python3
"""
fix_kani.py — Clean up botched kani extern crate insertions.

For every #[no_std] Rust crate root:
1. Remove ALL existing `extern crate kani` lines (and their #[cfg(kani)] guards)
2. Find the last #![...] inner attribute
3. Insert exactly one clean block after it:

    #[cfg(kani)]
    extern crate kani;
"""

import sys
import re
from pathlib import Path

CRATE_ROOTS = [
    "asl-arpi/src/lib.rs",
    "asl-axon-bridge/src/lib.rs",
    "asl-common/src/lib.rs",
    "asl-datatier/src/lib.rs",
    "asl-firewall-cap/src/lib.rs",
    "asl-gpu-cap/src/lib.rs",
    "asl-inverted-admin/src/lib.rs",
    "asl-microkit/src/lib.rs",
    "asl-network-routing/src/lib.rs",
    "asl-power-mgmt/src/lib.rs",
    "asl-soma/src/lib.rs",
    "asl-touch-sensor/src/lib.rs",
    "asl-trustgraph/src/lib.rs",
    "asl-genesis/src/main.rs",
]

KANI_BLOCK = "#[cfg(kani)]\nextern crate kani;"

def fix_file(path: Path) -> str:
    if not path.exists():
        return f"SKIP (not found): {path}"

    original = path.read_text()
    lines = original.splitlines()

    # ── Step 1: strip ALL kani-related lines ──────────────────
    cleaned = []
    i = 0
    while i < len(lines):
        line = lines[i]
        # Remove bare extern crate kani (with or without cfg guard)
        if re.match(r'\s*extern crate kani\s*;', line):
            # Also remove the preceding #[cfg(kani)] if present
            if cleaned and re.match(r'\s*#\[cfg\(kani\)\]', cleaned[-1]):
                cleaned.pop()
            # And any blank line before the cfg guard
            while cleaned and cleaned[-1].strip() == '':
                cleaned.pop()
            i += 1
            continue
        if re.match(r'\s*#\[cfg\(kani\)\]', line):
            # Peek ahead — if next non-blank is extern crate kani, skip both
            j = i + 1
            while j < len(lines) and lines[j].strip() == '':
                j += 1
            if j < len(lines) and re.match(r'\s*extern crate kani\s*;', lines[j]):
                i = j + 1
                continue
        cleaned.append(line)
        i += 1

    # ── Step 2: find last #![...] inner attribute line ────────
    last_inner = -1
    for idx, line in enumerate(cleaned):
        if re.match(r'#!\[', line.strip()):
            last_inner = idx

    if last_inner == -1:
        return f"SKIP (no #![...] found): {path}"

    # ── Step 3: insert exactly one kani block after last inner attr ──
    insert_at = last_inner + 1
    # Skip any blank lines immediately after
    while insert_at < len(cleaned) and cleaned[insert_at].strip() == '':
        insert_at += 1

    cleaned.insert(insert_at, "")
    cleaned.insert(insert_at, KANI_BLOCK)

    result = "\n".join(cleaned) + "\n"
    path.write_text(result)
    return f"FIXED: {path}"


def main():
    asl_root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path.home() / "asl"
    print(f"ASL root: {asl_root}\n")
    for rel in CRATE_ROOTS:
        p = asl_root / rel
        print(fix_file(p))

    print("\n── Verification ──────────────────────────────────────")
    for rel in CRATE_ROOTS:
        p = asl_root / rel
        if not p.exists():
            continue
        content = p.read_text()
        count = content.count("extern crate kani")
        status = "✓" if count == 1 else f"✗ ({count} occurrences)"
        print(f"  {status}  {rel}")

if __name__ == "__main__":
    main()
