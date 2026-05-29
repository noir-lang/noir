import './style.css';
import { BrowserFileSecretProvider, FidoHsmSecretProvider, WebUsbSecretProvider } from './secret-providers.js';
import { createEncryptedSecretFile, serializeEncryptedSecretFile } from './secret-file.js';
import { randomField, userIdToField, computeCommitment } from './fields.js';
import { createAuthInputs, generateAndVerifyProof, proofToJson } from './proof.js';

const status = document.querySelector('#status');
const registerForm = document.querySelector('#register-form');
const proveForm = document.querySelector('#prove-form');
const registerOutput = document.querySelector('#register-output');
const proveOutput = document.querySelector('#prove-output');
const deviceOutput = document.querySelector('#device-output');
let circuitPromise;

registerForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  await withBusy('Generating file', async () => {
    const userId = document.querySelector('#register-user').value.trim();
    const pin = document.querySelector('#register-pin').value;
    const deviceSecret = randomField();
    const encryptedFile = await createEncryptedSecretFile(deviceSecret, pin, {
      deviceLabel: `USB ZK Auth: ${userId}`,
    });
    downloadSecretFile(encryptedFile, `usb-zk-secret-${safeName(userId)}.json`);
    const userIdHash = await userIdToField(userId);
    registerOutput.value = `Downloaded encrypted secret file.\nCommitment: ${computeCommitment(deviceSecret, userIdHash)}`;
  });
});

proveForm.addEventListener('submit', async (event) => {
  event.preventDefault();
  await withBusy('Generating proof', async () => {
    const userId = document.querySelector('#prove-user').value.trim();
    const pin = document.querySelector('#prove-pin').value;
    const file = document.querySelector('#secret-file').files[0];
    const provider = new BrowserFileSecretProvider();
    const deviceSecret = await provider.readSecret({ file, pin });
    const authInputs = await createAuthInputs({ deviceSecret, userId });
    const circuit = await getCircuit();
    const result = await generateAndVerifyProof(circuit, authInputs);
    proveOutput.value = JSON.stringify(proofToJson(result), null, 2);
  });
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

function getCircuit() {
  circuitPromise ??= fetch('/target/usb_auth.json').then((response) => {
    if (!response.ok) {
      throw new Error('Missing target/usb_auth.json. Run nargo compile in demo/usb-auth before proving.');
    }
    return response.json();
  });
  return circuitPromise;
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
