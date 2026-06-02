/* eslint-disable prettier/prettier */
import { decryptSecretFile, parseEncryptedSecretFile } from './secret-file.js';

export class BrowserFileSecretProvider {
  async readSecret({ file, pin }) {
    if (!file) {
      throw new Error('Select an encrypted secret JSON file.');
    }
    const encryptedFile = parseEncryptedSecretFile(await file.text());
    return decryptSecretFile(encryptedFile, pin);
  }
}

export class WebUsbSecretProvider {
  async readSecret({ filters = [] } = {}) {
    if (!globalThis.navigator?.usb) {
      throw new Error('WebUSB is not available in this browser.');
    }
    await globalThis.navigator.usb.requestDevice({ filters });
    throw new Error('Custom USB firmware protocol is required before WebUSB can provide a Noir secret.');
  }
}

export class FidoHsmSecretProvider {
  async readSecret() {
    throw new Error(
      'FIDO/WebAuthn does not expose a raw device_secret. Use a signature-verifying circuit, oracle flow, or custom hardware protocol.',
    );
  }
}
