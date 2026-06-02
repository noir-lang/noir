# Plan: USB Portable Verifier (Cross-Platform & Web-Provisioned)

## Objective
Enable a seamless flow where users visit a web portal to provision a standard USB drive with a ZK proof, its artifacts, and a standalone native verifier. The verifier works **completely offline**, binds to the **hardware serial** for replay protection, and supports both GUI and CLI modes.

## Implementation Phases

### Phase 1: Advanced Web Provisioning
Enhance `demo/usb-auth` to be the "Provisioning Station".
- **Task 1.1**: Implement **WebUSB Serial Auto-Detection** to pre-fill the `usb_serial` input for the Noir circuit.
- **Task 1.2**: Implement **File System Access API** for "One-Click Provisioning" (direct write to USB folder).
- **Task 1.3**: Provide a robust **Manual Fallback** (ZIP download) for unsupported browsers.

### Phase 2: Hardware Binding & Secure Communication
- **Task 2.1**: Update `main.nr` to bind to `usb_serial` (Public Input).
- **Task 2.2**: **Encrypted Communication (MVP Alternatives)**:
  - Since standard USBs can't do mTLS handshakes, explore a **Local Encrypted File Protocol**:
    - The verifier writes a random encrypted challenge to a temporary file on the USB.
    - The ZK proof must "solve" or be tied to a hash of this local interaction. (Note: This might still require a challenge-response, but can be done entirely locally/offline).
  - Alternatively, use a **Noise Protocol** inspired handshake if the "USB" is actually a more capable device later.

### Phase 3: Native Multi-Platform Verifier (Rust)
A standalone tool written in Rust for performance and easy cross-compilation.
- **Task 3.1**: Create `tooling/usb-verifier-rs`.
- **Task 3.2**: Support:
  - **GUI Mode**: Simple "Verify" button for humans.
  - **CLI Mode**: `--json` or `--quiet` flags for machine integration.
  - **Julia Integration**: Ensure CLI output is easily consumable by Julia (e.g., standard JSON).
- **Task 3.3**: Cross-compile targets:
  - `x86_64-pc-windows-msvc` (.exe)
  - `x86_64-apple-darwin` / `aarch64-apple-darwin` (macOS)
  - `x86_64-unknown-linux-gnu` (Linux)

### Phase 4: Full Offline Integration
- **Task 4.1**: Bundle all circuit bytecode into the Rust binary so the USB only needs to hold the `proof.json`.
- **Task 4.2**: Ensure the web demo provides the correct OS-specific binary during provisioning.

## Implementation Status

### Phase 1 — Web Provisioning
- [x] **Task 1.1**: WebUSB Serial Auto-Detection (`demo/usb-auth/src/app.js`)
- [x] **Task 1.2**: File System Access API directory picker (`demo/usb-auth/src/app.js`)
- [x] **Task 1.3**: USB Package download (proof.json + README.txt + verify-usb.bat/sh) (`demo/usb-auth/src/app.js`)

### Phase 2 — Hardware Binding
- [x] **Task 2.1**: `usb_serial` as public input in `demo/usb-auth/src/main.nr`
- [ ] **Task 2.2**: Encrypted challenge protocol (not yet implemented; local file binding is the current MVP)

### Phase 3 — Native Rust Verifier
- [x] **Task 3.1**: `tooling/usb-verifier-rs` created (Rust binary, workspace member)
- [x] **Task 3.2**: CLI mode (`--json`, `--quiet`), `--info` for circuit identity, serial binding check
- [x] **Task 3.3**: Cross-compile targets supported (pure Rust, no C deps); platform serial detection on Windows/macOS/Linux

### Phase 4 — Full Offline Integration
- [x] **Task 4.1**: Circuit bytecode embedded in Rust binary (`src/circuit.rs` — `CIRCUIT.bytecode`)
- [ ] **Task 4.2**: OS-specific binary links during web provisioning (requires GitHub Releases hosting)

## TODO List
- [x] Research File System Access API "Directory Picker" permissions.
- [x] Implement Windows/macOS/Linux serial detection in Rust.
- [x] Refactor `main.nr` to accept hardware serial as an input.
- [x] Build the "Provisioning UI" with WebUSB auto-detect.
- [ ] Wire `bb verify` subprocess for cryptographic re-verification in the Rust binary.
- [ ] Add GitHub Releases links for OS-specific binary download in provisioning UI.

---

## The Grilling (Round 4)

1. **"Integration" Target**: You mentioned the `.exe` should be machine-readable for integration. What specific systems or languages (e.g., Python, C#, Node) will be calling this `.exe`? I can optimize the CLI output format (JSON/Protobuf/Exit Codes) for them.
2. **WebUSB Limitations**: WebUSB can read serial numbers from the **device descriptor**, but the File System Access API sees the **Volume Serial**. These can be different. Which one should we standardize on for the ZK proof? (I suggest Volume Serial as it's easier to read from the native `.exe` side).
3. **Encryption Goal**: Since we are offline and using standard USBs, "encryption" usually refers to protecting the **Privacy** of the user or the **Integrity** of the proof. Which is your priority?
