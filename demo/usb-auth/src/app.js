import './style.css';
import { BrowserFileSecretProvider, FidoHsmSecretProvider, WebUsbSecretProvider } from './secret-providers.js';
import { createEncryptedSecretFile, serializeEncryptedSecretFile } from './secret-file.js';
import { randomField, userIdToField, computeCommitment } from './fields.js';

// proof.js pulls in @aztec/bb.js (141 MB WASM package). Import it lazily so
// the dev server never analyzes it at startup — only loaded when the user
// actually clicks "Generate proof".
async function loadProof() {
  const [{ createAuthInputs, generateAndVerifyProof, proofToJson }, { default: circuit }] = await Promise.all([
    import('./proof.js'),
    import('./circuit-artifact.js'),
  ]);
  return { createAuthInputs, generateAndVerifyProof, proofToJson, circuit };
}

const status = document.querySelector('#status');
const registerForm = document.querySelector('#register-form');
const proveForm = document.querySelector('#prove-form');
const registerOutput = document.querySelector('#register-output');
const proveOutput = document.querySelector('#prove-output');
const deviceOutput = document.querySelector('#device-output');
const proofActions = document.querySelector('#proof-actions');

let lastProofJson = null;

registerForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  await withBusy('Generating file', async () => {
    const userId = document.querySelector('#register-user').value.trim();
    const pin = document.querySelector('#register-pin').value;
    const usbSerial = document.querySelector('#register-usb-serial').value || '0';
    const deviceSecret = randomField();
    const encryptedFile = await createEncryptedSecretFile(deviceSecret, pin, {
      deviceLabel: `USB ZK Auth: ${userId}`,
      usbSerial,
    });
    
    // Try File System Access API if supported
    try {
      if (window.showDirectoryPicker) {
        const handle = await window.showDirectoryPicker();
        const fileHandle = await handle.getFileHandle(`usb-zk-secret-${safeName(userId)}.json`, { create: true });
        const writable = await fileHandle.createWritable();
        await writable.write(serializeEncryptedSecretFile(encryptedFile));
        await writable.close();
        registerOutput.value = `Saved encrypted secret to USB.\nSerial: ${usbSerial}`;
      } else {
        downloadSecretFile(encryptedFile, `usb-zk-secret-${safeName(userId)}.json`);
        registerOutput.value = `Downloaded encrypted secret file.\nSerial: ${usbSerial}`;
      }
    } catch (e) {
      console.warn('FS Access failed, falling back to download', e);
      downloadSecretFile(encryptedFile, `usb-zk-secret-${safeName(userId)}.json`);
      registerOutput.value = `Downloaded encrypted secret file.\nSerial: ${usbSerial}`;
    }

    const userIdHash = await userIdToField(userId);
    registerOutput.value += `\nCommitment: ${computeCommitment(deviceSecret, userIdHash)}`;
  });
});

proveForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  await withBusy('Loading ZK backend…', async () => {
    const { createAuthInputs, generateAndVerifyProof, proofToJson, circuit } = await loadProof();
    status.textContent = 'Generating proof';
    const userId = document.querySelector('#prove-user').value.trim();
    const pin = document.querySelector('#prove-pin').value;
    const usbSerial = document.querySelector('#usb-serial').value || '0';
    const file = document.querySelector('#secret-file').files[0];
    const provider = new BrowserFileSecretProvider();
    const deviceSecret = await provider.readSecret({ file, pin });
    const authInputs = await createAuthInputs({ deviceSecret, userId, usbSerial });
    const result = await generateAndVerifyProof(circuit, authInputs);
    lastProofJson = proofToJson(result);
    proveOutput.value = JSON.stringify(lastProofJson, null, 2);
    proofActions.hidden = false;
  });
});

const detectUsb = async () => {
  try {
    if (!navigator.usb) throw new Error('WebUSB not supported');
    const device = await navigator.usb.requestDevice({ filters: [] });
    return device.serialNumber || 'UNKNOWN-SERIAL';
  } catch (e) {
    return '1234-ABCD'; // Placeholder for demo
  }
};

document.querySelector('#detect-usb').addEventListener('click', async () => {
  const serial = await detectUsb();
  document.querySelector('#usb-serial').value = serial;
});

document.querySelector('#register-detect-usb').addEventListener('click', async () => {
  const serial = await detectUsb();
  document.querySelector('#register-usb-serial').value = serial;
});

document.querySelector('#webusb-button').addEventListener('click', async () => {
  await withBusy('Checking WebUSB', async () => {
    try {
      await new WebUsbSecretProvider().readSecret();
    } catch (error) {
      deviceOutput.value = error.message;
    }
  });
});

document.querySelector('#download-proof').addEventListener('click', () => {
  if (!lastProofJson) return;
  downloadBlob(JSON.stringify(lastProofJson, null, 2), 'proof.json', 'application/json');
});

document.querySelector('#download-usb-package').addEventListener('click', () => {
  if (!lastProofJson) return;
  downloadBlob(JSON.stringify(lastProofJson, null, 2), 'proof.json', 'application/json');
  downloadBlob(usbPackageReadme(lastProofJson), 'README.txt', 'text/plain');
  downloadBlob(usbVerifyBat(), 'verify-usb.bat', 'text/plain');
  downloadBlob(usbVerifySh(), 'verify-usb.sh', 'text/plain');
});

document.querySelector('#fido-button').addEventListener('click', async () => {
  await withBusy('Checking FIDO/HSM', async () => {
    try {
      await new FidoHsmSecretProvider().readSecret();
    } catch (error) {
      deviceOutput.value = error.message;
    }
  });
});

async function withBusy(label, action) {
  setDisabled(true);
  status.textContent = label;
  try {
    await action();
    status.textContent = 'Ready';
  } catch (error) {
    status.textContent = 'Error';
    const target = document.activeElement?.closest('form') === proveForm ? proveOutput : registerOutput;
    target.value = error.message;
  } finally {
    setDisabled(false);
  }
}

function setDisabled(disabled) {
  for (const button of document.querySelectorAll('button')) {
    button.disabled = disabled;
  }
}

function downloadSecretFile(encryptedFile, filename) {
  const blob = new Blob([serializeEncryptedSecretFile(encryptedFile)], { type: 'application/json' });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function safeName(value) {
  return value.toLowerCase().replace(/[^a-z0-9_-]+/g, '-').replace(/^-|-$/g, '') || 'user';
}

function downloadBlob(content, filename, mimeType) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  link.click();
  URL.revokeObjectURL(url);
}

function usbPackageReadme(proofJson) {
  const serial = proofJson?.publicInputs?.usb_serial ?? '(unknown)';
  const nullifier = proofJson?.nullifier ?? '(unknown)';
  return [
    'USB ZK Auth — Portable Verifier Package',
    '========================================',
    '',
    'This package contains a zero-knowledge proof bound to your USB hardware serial.',
    '',
    `USB Serial (embedded) : ${serial}`,
    `Nullifier             : ${nullifier}`,
    `Proof verified        : ${proofJson?.verified ? 'YES' : 'NO'}`,
    '',
    'To verify this proof offline:',
    '  Windows : verify-usb.bat',
    '  Linux/Mac: chmod +x verify-usb.sh && ./verify-usb.sh',
    '',
    'Both scripts use usb-verifier (download from GitHub Releases or build from source):',
    '  https://github.com/noir-lang/noir/releases',
    '',
    'Build from source:',
    '  cargo build -p usb-verifier --release',
    '  # Binary at: target/release/usb-verifier',
  ].join('\n');
}

function usbVerifyBat() {
  return [
    '@echo off',
    'REM Verify a USB ZK proof bound to this drive\'s hardware serial.',
    'REM Usage: verify-usb.bat [DRIVE_LETTER]',
    'SET DRIVE=%1',
    'IF "%DRIVE%"=="" SET DRIVE=D',
    'usb-verifier --proof proof.json --drive %DRIVE% --json',
    'IF %ERRORLEVEL% EQU 0 (',
    '  echo PROOF VALID',
    ') ELSE (',
    '  echo PROOF INVALID',
    ')',
  ].join('\r\n');
}

function usbVerifySh() {
  return [
    '#!/bin/sh',
    '# Verify a USB ZK proof bound to this drive\'s hardware serial.',
    '# Usage: ./verify-usb.sh [/mount/point]',
    'MOUNT="${1:-/media/$USER/USB}"',
    './usb-verifier --proof proof.json --drive "$MOUNT" --json',
    'STATUS=$?',
    'if [ $STATUS -eq 0 ]; then',
    '  echo "PROOF VALID"',
    'else',
    '  echo "PROOF INVALID"',
    'fi',
    'exit $STATUS',
  ].join('\n');
}
