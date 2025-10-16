# SamHan-FGT-Parser (Rust GUI)

**Cross‑platform, GUI‑only FortiGate config parser** built in Rust with `egui/eframe`.
Runs on macOS, Linux, and Windows. Offline, no server.

> Unofficial. Not affiliated with Fortinet. Trademarks belong to their owners.

## Features
- Open/paste `.conf` (FortiGate CLI backup)
- Original tokenizer+parser → **JSON/YAML** output
- **Search filter** on keys & values
- **CSV exports**: Addresses (v4), Policies (v4), **generic export by path** (e.g., `firewall.service.custom`)
- MIT license

## Build
```bash
cargo build --release
./target/release/SamHan-fgt-parser   # or .exe on Windows
```

## CI (GitHub Actions)
Artifacts for macOS, Linux, Windows in `.github/workflows/build.yml`.

## Trademarks
**Fortinet®, FortiGate®, FortiOS®** are trademarks of Fortinet, Inc. This project is independent and not affiliated with Fortinet.
