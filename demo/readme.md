# Noir Demo — USB ZK Auth

A zero-knowledge authentication demo where a USB drive acts as a hardware key.
The browser generates a Noir proof bound to the drive's volume serial number; a
standalone native verifier checks the proof offline without any internet connection.

---

## Project Layout

```
demo/
├── usb-auth/          ← the only runnable demo (web + CLI + tests)
│   ├── src/
│   │   ├── main.nr            Noir circuit (hardware-bound ZK proof)
│   │   ├── circuit-artifact.js  compiled circuit (auto-generated)
│   │   ├── app.js             browser frontend
│   │   ├── cli.js             Node.js CLI (register / prove / verify)
│   │   ├── proof.js           proof generation & verification (WASM)
│   │   ├── fields.js          BN254 field helpers
│   │   ├── secret-file.js     AES-256-GCM encrypted secret file
│   │   └── secret-providers.js  browser / node / WebUSB / FIDO providers
│   ├── test/                  unit + proof tests (Mocha)
│   ├── scripts/               circuit build helper
│   ├── Nargo.toml             Noir package config
│   └── Prover.toml            example proof inputs
│
├── server/            ← empty (placeholder, no source files)
├── client/            ← empty (placeholder, no source files)
└── readme.md          ← this file
```

The native Rust verifier lives in the workspace at `tooling/usb-verifier-rs/`.

---

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | ≥ 20 | https://nodejs.org |
| Rust / Cargo | ≥ 1.89.0 | https://rustup.rs |
| yarn (workspace) | v4 | `corepack enable` |

Install JS dependencies from the **workspace root** once:

```powershell
yarn install
```

---

## 1 — Web Browser Demo

```powershell
cd demo\usb-auth
npm run dev
```

Open **http://127.0.0.1:5173** in your browser.

### What the demo does

1. **Register** — encrypts a random device secret with your PIN and saves it to the USB
   (via File System Access API) or downloads it as a JSON file.
2. **Prove** — decrypts the secret, runs the Noir circuit in WASM, produces a ZK proof
   bound to the USB volume serial number.
3. **Download USB Package** — after proving, download `proof.json` + `README.txt` +
   `verify-usb.bat` / `verify-usb.sh` scripts for fully-offline verification.

### Detecting the USB serial

Click **Auto-Detect** next to the serial field to trigger WebUSB device selection.
If WebUSB is unavailable, paste the serial manually.

On Windows, read it with:

```powershell
vol D:
# Volume Serial Number is 1234-ABCD  →  use 305441741 (hex 0x1234ABCD, no dash)
```

---

## 2 — Node.js CLI

All commands must be run from inside `demo\usb-auth\`:

```powershell
cd demo\usb-auth
```

### Register — create an encrypted secret file

```powershell
node src/cli.js register --out secret.json --pin mypin123 --user alice
```

Outputs the commitment hash and user-id hash to stdout (JSON).

### Prove — generate a ZK proof

```powershell
node src/cli.js prove `
  --secret secret.json `
  --pin    mypin123 `
  --user   alice `
  --out    proof.json
```

Saves `proof.json` and prints it to stdout. Add `--challenge <field>` to fix the
challenge value (defaults to random).

### Verify — structural check of a saved proof

```powershell
node src/cli.js verify --proof proof.json
```

Reads `proof.json` and echoes back `verified` + `nullifier`. This is a structural
check only — cryptographic re-verification requires the native Rust verifier or `bb`.

---

## 3 — Native Rust Verifier (`usb-verifier`)

### Build

```powershell
# From the workspace root (noir/)
cargo build -p usb-verifier --release
# Binary: target\release\usb-verifier.exe
```

### Verify a proof

```powershell
.\target\release\usb-verifier.exe `
  --proof  demo\usb-auth\proof.json `
  --serial 305441741 `
  --json
```

Exit code `0` = valid, `1` = invalid.

| Flag | Description |
|------|-------------|
| `--proof <FILE>` | Path to `proof.json` |
| `--serial <NUMBER>` | USB serial as a decimal integer (overrides auto-detect) |
| `--drive <LETTER>` | Drive letter (Windows) or mount point (macOS/Linux) for auto-detect |
| `--json` | Machine-readable JSON output |
| `--quiet` | No output; use exit code only |
| `--info` | Print embedded circuit identity and exit |

### Auto-detect the drive serial (Windows)

```powershell
.\target\release\usb-verifier.exe --proof proof.json --drive D --json
```

### Show embedded circuit identity

```powershell
.\target\release\usb-verifier.exe --info
# Circuit  : usb_auth
# Noir     : 0.33.0
# Inputs   : usb_serial, commitment, challenge, user_id_hash
# Bytecode : H4sIAAAAAAAA/62T...(204 chars)
```

The circuit bytecode is embedded at compile time — the USB only needs `proof.json`.

---

## 4 — Full End-to-End Flow

```powershell
cd demo\usb-auth

# Step 1: register
node src/cli.js register --out secret.json --pin mypin123 --user alice

# Step 2: prove (serial 0 = no hardware binding in CLI mode)
node src/cli.js prove --secret secret.json --pin mypin123 --user alice --out proof.json

# Step 3: verify with Rust verifier (serial must match what was used in step 2)
..\..\..\..\target\release\usb-verifier.exe --proof proof.json --serial 0 --json
```

Expected output:

```json
{
  "valid": true,
  "serial_match": true,
  "proof_verified": true,
  "nullifier": "...",
  "usb_serial_expected": "0",
  "usb_serial_actual": "0"
}
```

**Test a serial mismatch** (should fail):

```powershell
..\..\..\..\target\release\usb-verifier.exe --proof proof.json --serial 9999 --json
# "valid": false, "serial_match": false  →  exit 1
```

---

## 5 — Tests

```powershell
cd demo\usb-auth

# Unit tests (fields, secret-file, providers) — fast
npm test

# ZK proof generation + verification — ~1–2 s
npm run test:proof
```

Rust verifier tests (from workspace root):

```powershell
cargo test -p usb-verifier --bins --tests
```

---

## 6 — Regenerating the Circuit Artifact

If you edit `src/main.nr`, rebuild the JS artifact before testing:

```powershell
cd demo\usb-auth
npm run generate:circuit
```

This recompiles the Noir circuit via WASM and overwrites `src/circuit-artifact.js`.
Also update `src/circuit.rs` in `tooling/usb-verifier-rs/` if the bytecode changes.

---

## Circuit Architecture

```
main.nr inputs
  device_secret   (private)   long-term secret stored encrypted on USB
  usb_serial      (public)    volume serial — binds proof to this drive
  commitment      (public)    device_secret² + user_id_hash
  challenge       (public)    fresh random nonce per session
  user_id_hash    (public)    SHA-256(user_id) mod BN254

return value (public)
  nullifier = device_secret × challenge + user_id_hash + usb_serial
```

The verifier never sees `device_secret`. Replay attacks are blocked because each
proof uses a fresh `challenge`, and the `usb_serial` prevents using a proof on a
different drive.
