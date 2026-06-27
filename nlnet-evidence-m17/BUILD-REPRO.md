# Phoenix Lite Boot Demo — Reproduction Instructions

## Prerequisites

```bash
# Ubuntu/Debian
sudo apt install qemu-system-arm gpg

# Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add aarch64-unknown-none
```

## Reproduce

```bash
git clone https://github.com/aieonyx/asl.git
cd asl
git checkout v0.1.0-asl-m17

# Build ISO (requires microkit-sdk-1.4.1)
export MICROKIT_SDK=~/microkit-sdk-1.4.1
chmod +x build-m15.sh
./build-m15.sh

# Run boot demo
chmod +x demo-m17.sh
./demo-m17.sh

# Verify output
./scripts/verify-boot-log.sh nlnet-evidence-m17/boot-demo-m17-clean.log
```

## Expected result

All 12 verification checks pass.
Final line: `phoenix@aieonyx:~$`
Sovereign proof: `axon_main() → 0x4153`

## GPG verification

```bash
gpg --verify nlnet-evidence-m17/boot-demo-m17.log.asc \
             nlnet-evidence-m17/boot-demo-m17.log
```

Key fingerprint: B4C8548260DB40E1
