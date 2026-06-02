import { useState, useCallback, useRef } from 'react';
import { BrowserFileSecretProvider, FidoHsmSecretProvider, WebUsbSecretProvider } from './secret-providers.js';
import { createEncryptedSecretFile, serializeEncryptedSecretFile } from './secret-file.js';
import { randomField, userIdToField, computeCommitment } from './fields.js';

// Lazy-load proof.js (pulls in 141 MB @aztec/bb.js) only on first prove click.
async function loadProof() {
  const [{ createAuthInputs, generateAndVerifyProof, proofToJson }, { default: circuit }] =
    await Promise.all([import('./proof.js'), import('./circuit-artifact.js')]);
  return { createAuthInputs, generateAndVerifyProof, proofToJson, circuit };
}

/** Scan a FileSystemDirectoryHandle for usb-zk-secret-*.json files. */
async function scanDirForSecrets(dirHandle) {
  const found = [];
  for await (const [name, handle] of dirHandle.entries()) {
    if (handle.kind === 'file' && name.startsWith('usb-zk-secret-') && name.endsWith('.json')) {
      found.push({ name, handle });
    }
  }
  return found;
}

async function detectUsbSerial() {
  try {
    if (!navigator.usb) throw new Error('WebUSB not supported');
    const device = await navigator.usb.requestDevice({ filters: [] });
    return device.serialNumber || '';
  } catch {
    return '';
  }
}

function downloadBlob(content, filename, mimeType) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  Object.assign(document.createElement('a'), { href: url, download: filename }).click();
  URL.revokeObjectURL(url);
}

function safeName(v) {
  return v.toLowerCase().replace(/[^a-z0-9_-]+/g, '-').replace(/^-|-$/g, '') || 'user';
}

function buildReadme(proof) {
  const serial = proof?.publicInputs?.usb_serial ?? '(unknown)';
  const nullifier = proof?.nullifier ?? '(unknown)';
  return [
    'USB ZK Auth — Portable Verifier Package',
    '========================================',
    '',
    `USB Serial (embedded) : ${serial}`,
    `Nullifier             : ${nullifier}`,
    `Proof verified        : ${proof?.verified ? 'YES' : 'NO'}`,
    '',
    'Verify offline:',
    '  Windows  : verify-usb.bat',
    '  Linux/Mac: chmod +x verify-usb.sh && ./verify-usb.sh',
    '',
    'Build the verifier from source:',
    '  cargo build -p usb-verifier --release',
  ].join('\n');
}

// ── Sub-components ──────────────────────────────────────────────────────────

function StatusBadge({ status }) {
  const isError = status === 'Error';
  const isReady = status === 'Ready';
  return (
    <div
      className="status-badge"
      style={
        isError
          ? { background: '#fee2e2', color: '#b91c1c', borderColor: '#fca5a5' }
          : isReady
            ? {}
            : { background: 'rgba(255,255,255,.14)', color: '#fde68a' }
      }
    >
      {status}
    </div>
  );
}

function FieldGroup({ label, hint, children }) {
  return (
    <div className="field-group">
      <label>
        {label}
        {hint && <span className="label-hint"> {hint}</span>}
      </label>
      {children}
    </div>
  );
}

function Output({ value }) {
  if (!value) return null;
  return <output>{value}</output>;
}

const CAN_SCAN = typeof window !== 'undefined' && 'showDirectoryPicker' in window;

/**
 * Device scan panel.
 *
 * Chrome / Edge: "Select Device" opens a directory picker. The app scans for
 * usb-zk-secret-*.json files and tries to auto-detect the volume serial via
 * WebUSB so the serial field is filled automatically.
 *
 * Firefox / Safari: falls back to a plain file-input for manual selection.
 *
 * onScanResult({ name, handle?, file? }) — called when a file is ready.
 * onSerialDetected(serial: string)       — called when a serial is found.
 */
function DeviceScanPanel({ busy, onScanResult, onSerialDetected }) {
  const [scanState, setScanState] = useState('idle'); // idle | scanning | found | empty | error
  const [files, setFiles]         = useState([]);
  const [selectedIdx, setSelectedIdx] = useState(0);
  const [dirName, setDirName]     = useState('');
  const dirHandleRef = useRef(null);

  // ── Directory scan (Chrome / Edge) ──────────────────────────────────────
  const scan = useCallback(async () => {
    setScanState('scanning');
    try {
      const dir = await window.showDirectoryPicker({ mode: 'read' });
      dirHandleRef.current = dir;
      setDirName(dir.name);

      // Scan for secret files and detect serial in parallel.
      const [found, serial] = await Promise.all([
        scanDirForSecrets(dir),
        detectUsbSerial(),
      ]);

      if (serial) onSerialDetected(serial);

      setFiles(found);
      setSelectedIdx(0);
      setScanState(found.length > 0 ? 'found' : 'empty');
      onScanResult(found.length > 0 ? found[0] : null);
    } catch (err) {
      setScanState(err.name === 'AbortError' ? 'idle' : 'error');
      onScanResult(null);
    }
  }, [onScanResult, onSerialDetected]);

  const rescan = useCallback(async () => {
    if (!dirHandleRef.current) { scan(); return; }
    setScanState('scanning');
    const found = await scanDirForSecrets(dirHandleRef.current);
    setFiles(found);
    setSelectedIdx(0);
    setScanState(found.length > 0 ? 'found' : 'empty');
    onScanResult(found.length > 0 ? found[0] : null);
  }, [scan, onScanResult]);

  const select = (idx) => {
    setSelectedIdx(idx);
    onScanResult(files[idx]);
  };

  // ── File-input fallback (Firefox / Safari) ───────────────────────────────
  if (!CAN_SCAN) {
    return (
      <div className="device-scan">
        <div className="scan-status fallback-notice">
          <span>ℹ</span>
          <span>Auto-scan unavailable in this browser — select the file manually.</span>
        </div>
        <input
          type="file"
          accept="application/json,.json"
          disabled={busy}
          className="fallback-file-input"
          onChange={(e) => {
            const file = e.target.files?.[0] ?? null;
            onScanResult(file ? { name: file.name, file } : null);
          }}
        />
      </div>
    );
  }

  // ── Scan UI (Chrome / Edge) ───────────────────────────────────────────────
  return (
    <div className="device-scan">
      {scanState === 'idle' && (
        <button type="button" className="btn-scan" disabled={busy} onClick={scan}>
          <span className="scan-icon">🔌</span> Select Device
        </button>
      )}

      {scanState === 'scanning' && (
        <div className="scan-status scanning">
          <span className="spin">⟳</span> Scanning <code>{dirName || '…'}</code>
        </div>
      )}

      {scanState === 'found' && (
        <div className="scan-result">
          <div className="scan-status found">
            <span>✓</span>
            <span>
              Found {files.length} secret file{files.length !== 1 ? 's' : ''} on <code>{dirName}</code>
            </span>
            <button type="button" className="btn-rescan" disabled={busy} onClick={rescan}>
              Re-scan
            </button>
          </div>
          {files.length > 1 ? (
            <ul className="file-list">
              {files.map((f, i) => (
                <li key={f.name}>
                  <label className={`file-option ${i === selectedIdx ? 'selected' : ''}`}>
                    <input
                      type="radio"
                      name="secret-file-choice"
                      checked={i === selectedIdx}
                      onChange={() => select(i)}
                      disabled={busy}
                    />
                    {f.name}
                  </label>
                </li>
              ))}
            </ul>
          ) : (
            <div className="file-single">
              <span className="file-icon">🔑</span>
              <span className="file-name">{files[0].name}</span>
            </div>
          )}
        </div>
      )}

      {scanState === 'empty' && (
        <div className="scan-result">
          <div className="scan-status empty">
            <span>⚠</span>
            <span>
              No secret files found on <code>{dirName}</code>. Register one first (Step 1).
            </span>
            <button type="button" className="btn-rescan" disabled={busy} onClick={rescan}>
              Re-scan
            </button>
          </div>
        </div>
      )}

      {scanState === 'error' && (
        <div className="scan-result">
          <div className="scan-status scan-error">
            <span>✕</span>
            <span>Could not access the device. Try again.</span>
            <button type="button" className="btn-rescan" disabled={busy} onClick={scan}>
              Retry
            </button>
          </div>
        </div>
      )}
    </div>
  );
}

// ── Main App ────────────────────────────────────────────────────────────────

export default function App() {
  const [status, setStatus]           = useState('Ready');
  const [busy, setBusy]               = useState(false);
  const [registerOut, setRegisterOut] = useState('');
  const [proveOut, setProveOut]       = useState('');
  const [deviceOut, setDeviceOut]     = useState('');
  const [proofJson, setProofJson]     = useState(null);

  // Prove form: the selected FileSystemFileHandle from the device scan
  const selectedSecretRef = useRef(null); // { name, handle }

  // Form field refs (avoids re-renders on every keystroke)
  const regUser   = useRef(); const regSerial = useRef(); const regPin = useRef();
  const prvUser   = useRef(); const prvSerial = useRef(); const prvPin = useRef();

  const withBusy = useCallback(async (label, action, onError) => {
    setBusy(true);
    setStatus(label);
    try {
      await action();
      setStatus('Ready');
    } catch (err) {
      setStatus('Error');
      onError(err.message);
    } finally {
      setBusy(false);
    }
  }, []);

  // ── Register ──────────────────────────────────────────────────────────────
  const handleRegister = useCallback(
    async (e) => {
      e.preventDefault();
      await withBusy(
        'Generating file…',
        async () => {
          const userId    = regUser.current.value.trim();
          const pin       = regPin.current.value;
          const usbSerial = regSerial.current.value || '0';
          const secret    = randomField();
          const encrypted = await createEncryptedSecretFile(secret, pin, {
            deviceLabel: `USB ZK Auth: ${userId}`,
            usbSerial,
          });

          let saved = false;
          try {
            if (window.showDirectoryPicker) {
              const dir      = await window.showDirectoryPicker();
              const fh       = await dir.getFileHandle(`usb-zk-secret-${safeName(userId)}.json`, { create: true });
              const writable = await fh.createWritable();
              await writable.write(serializeEncryptedSecretFile(encrypted));
              await writable.close();
              saved = true;
            }
          } catch { /* fall through */ }

          if (!saved) {
            downloadBlob(
              serializeEncryptedSecretFile(encrypted),
              `usb-zk-secret-${safeName(userId)}.json`,
              'application/json',
            );
          }

          const userIdHash = await userIdToField(userId);
          setRegisterOut(
            `${saved ? 'Saved to USB.' : 'Downloaded secret file.'}\n` +
            `Serial: ${usbSerial}\n` +
            `Commitment: ${computeCommitment(secret, userIdHash)}`,
          );
        },
        setRegisterOut,
      );
    },
    [withBusy],
  );

  // ── Prove ─────────────────────────────────────────────────────────────────
  const handleProve = useCallback(
    async (e) => {
      e.preventDefault();
      if (!selectedSecretRef.current) {
        setProveOut('No secret file selected. Scan your device first.');
        return;
      }
      await withBusy(
        'Loading ZK backend…',
        async () => {
          const { createAuthInputs, generateAndVerifyProof, proofToJson, circuit } = await loadProof();
          setStatus('Generating proof…');
          const userId    = prvUser.current.value.trim();
          const pin       = prvPin.current.value;
          const usbSerial = prvSerial.current.value || '0';

          // Resolve to a File object — either from a FileSystemFileHandle (scan)
          // or a plain File (file-input fallback).
          const entry  = selectedSecretRef.current;
          const file   = entry.handle ? await entry.handle.getFile() : entry.file;
          const secret = await new BrowserFileSecretProvider().readSecret({ file, pin });
          const inputs = await createAuthInputs({ deviceSecret: secret, userId, usbSerial });
          const result = await generateAndVerifyProof(circuit, inputs);
          const pj     = proofToJson(result);
          setProofJson(pj);
          setProveOut(JSON.stringify(pj, null, 2));
        },
        setProveOut,
      );
    },
    [withBusy],
  );

  // ── Download actions ──────────────────────────────────────────────────────
  const downloadProof = () =>
    proofJson && downloadBlob(JSON.stringify(proofJson, null, 2), 'proof.json', 'application/json');

  const downloadPackage = () => {
    if (!proofJson) return;
    downloadBlob(JSON.stringify(proofJson, null, 2), 'proof.json', 'application/json');
    downloadBlob(buildReadme(proofJson), 'README.txt', 'text/plain');
    downloadBlob(
      '@echo off\nSET DRIVE=%1\nIF "%DRIVE%"=="" SET DRIVE=D\nusb-verifier --proof proof.json --drive %DRIVE% --json',
      'verify-usb.bat',
      'text/plain',
    );
    downloadBlob(
      '#!/bin/sh\nMOUNT="${1:-/media/$USER/USB}"\n./usb-verifier --proof proof.json --drive "$MOUNT" --json',
      'verify-usb.sh',
      'text/plain',
    );
  };

  // ── Serial auto-detect ────────────────────────────────────────────────────
  const autoDetectReg  = async () => { const s = await detectUsbSerial(); if (s) regSerial.current.value = s; };
  const autoDetectPrv  = async () => { const s = await detectUsbSerial(); if (s) prvSerial.current.value = s; };

  const checkWebUsb = () =>
    withBusy('Checking WebUSB…', async () => {
      try { await new WebUsbSecretProvider().readSecret(); }
      catch (err) { setDeviceOut(err.message); }
    }, setDeviceOut);

  const checkFido = () =>
    withBusy('Checking FIDO/HSM…', async () => {
      try { await new FidoHsmSecretProvider().readSecret(); }
      catch (err) { setDeviceOut(err.message); }
    }, setDeviceOut);

  // ── Render ────────────────────────────────────────────────────────────────
  return (
    <>
      <header className="site-header">
        <div className="header-inner">
          <div className="brand">
            <span className="brand-icon">🔐</span>
            <div>
              <h1>USB ZK Auth</h1>
              <p className="brand-sub">Hardware-bound zero-knowledge proofs — fully offline</p>
            </div>
          </div>
          <StatusBadge status={status} />
        </div>
      </header>

      <main className="shell">
        <div className="grid">

          {/* ── Register ─────────────────────────────────────────────── */}
          <form className="card" onSubmit={handleRegister}>
            <div className="card-header">
              <span className="step-num">1</span>
              <h2>Register Secret File</h2>
            </div>

            <FieldGroup label="User ID">
              <input ref={regUser} type="text" name="username" defaultValue="demo-user"
                autoComplete="username" placeholder="alice" disabled={busy} />
            </FieldGroup>

            <FieldGroup label="USB Serial" hint="(Volume ID)">
              <div className="input-row">
                <input ref={regSerial} type="text" placeholder="e.g. 305441741" disabled={busy} />
                <button type="button" className="btn-secondary" disabled={busy} onClick={autoDetectReg}>
                  Auto-Detect
                </button>
              </div>
              <p className="field-hint">
                Windows: run <code>vol D:</code> — paste the number without the dash.
              </p>
            </FieldGroup>

            <FieldGroup label="PIN" hint="(min 6 chars)">
              <input ref={regPin} type="password" name="new-password"
                minLength={6} autoComplete="new-password" placeholder="••••••••" disabled={busy} />
            </FieldGroup>

            <button type="submit" className="btn-primary" disabled={busy}>
              Generate encrypted USB file
            </button>
            <Output value={registerOut} />
          </form>

          {/* ── Prove ────────────────────────────────────────────────── */}
          <form className="card" onSubmit={handleProve}>
            <div className="card-header">
              <span className="step-num">2</span>
              <h2>Prove From USB File</h2>
            </div>

            <FieldGroup label="User ID">
              <input ref={prvUser} type="text" name="username" defaultValue="demo-user"
                autoComplete="username" placeholder="alice" disabled={busy} />
            </FieldGroup>

            <FieldGroup label="Device" hint="(select your USB drive — serial &amp; secret file auto-detected)">
              <DeviceScanPanel
                busy={busy}
                onScanResult={(entry) => { selectedSecretRef.current = entry; }}
                onSerialDetected={(s) => { if (prvSerial.current) prvSerial.current.value = s; }}
              />
            </FieldGroup>

            <FieldGroup label="USB Serial (Volume ID)" hint="(auto-filled on scan, or enter manually)">
              <div className="input-row">
                <input ref={prvSerial} type="text" placeholder="e.g. 305441741" disabled={busy} />
                <button type="button" className="btn-secondary" disabled={busy} onClick={autoDetectPrv}>
                  Detect
                </button>
              </div>
              <p className="field-hint">Must match the serial used during registration.</p>
            </FieldGroup>

            <FieldGroup label="PIN">
              <input ref={prvPin} type="password" name="current-password"
                minLength={6} autoComplete="current-password" placeholder="••••••••" disabled={busy} />
            </FieldGroup>

            <button type="submit" className="btn-primary" disabled={busy}>
              Generate &amp; verify proof
            </button>

            {proofJson && (
              <div className="download-bar">
                <button type="button" className="btn-ghost" onClick={downloadProof}>
                  ⬇ proof.json
                </button>
                <button type="button" className="btn-ghost" onClick={downloadPackage}>
                  ⬇ USB Package (4 files)
                </button>
              </div>
            )}

            <Output value={proveOut} />
          </form>

          {/* ── How it works ─────────────────────────────────────────── */}
          <section className="card card-info">
            <div className="card-header">
              <span className="step-num info-icon">ℹ</span>
              <h2>How it works</h2>
            </div>
            <ol className="flow-list">
              <li>The USB holds an <strong>AES-256-GCM encrypted</strong> device secret.</li>
              <li>Click <em>Select Device</em> — the browser scans the drive for <code>usb-zk-secret-*.json</code> and auto-fills the volume serial.</li>
              <li>Noir generates a fresh ZK proof binding the secret to the <strong>USB serial</strong>.</li>
              <li>The verifier checks the proof &amp; serial — it never sees the device secret.</li>
              <li>Download the <strong>USB Package</strong> to verify offline with the Rust tool.</li>
            </ol>
            <div className="divider" />
            <p className="verify-hint">
              Verify offline:<br />
              <code>usb-verifier --proof proof.json --drive D --json</code>
            </p>
          </section>

          {/* ── Device Diagnostics ───────────────────────────────────── */}
          <section className="card card-debug">
            <div className="card-header">
              <span className="step-num debug-icon">⚙</span>
              <h2>Device Diagnostics</h2>
            </div>
            <p className="field-hint" style={{ marginBottom: 14 }}>
              Test WebUSB and FIDO/HSM integration boundaries.
            </p>
            <div className="action-row">
              <button type="button" className="btn-secondary" disabled={busy} onClick={checkWebUsb}>
                Try WebUSB
              </button>
              <button type="button" className="btn-secondary" disabled={busy} onClick={checkFido}>
                Check FIDO / HSM
              </button>
            </div>
            <Output value={deviceOut} />
          </section>

        </div>
      </main>
    </>
  );
}
