const SECRET_FILE_VERSION = 1;
const KDF_NAME = 'PBKDF2-HMAC-SHA256';
const CIPHER_NAME = 'AES-256-GCM';
export const DEFAULT_PBKDF2_ITERATIONS = 600000;

const encoder = new TextEncoder();
const decoder = new TextDecoder();

export async function createEncryptedSecretFile(deviceSecret, pin, options = {}) {
  requirePin(pin);
  const cryptoProvider = options.cryptoProvider ?? globalThis.crypto;
  const iterations = options.iterations ?? DEFAULT_PBKDF2_ITERATIONS;
  const salt = randomBytes(16, cryptoProvider);
  const nonce = randomBytes(12, cryptoProvider);
  const key = await deriveAesKey(pin, salt, iterations, cryptoProvider);
  const plaintext = encoder.encode(JSON.stringify({ device_secret: String(deviceSecret) }));
  const ciphertext = await cryptoProvider.subtle.encrypt({ name: 'AES-GCM', iv: nonce }, key, plaintext);

  return {
    version: SECRET_FILE_VERSION,
    cipher: CIPHER_NAME,
    kdf: KDF_NAME,
    iterations,
    salt: bytesToBase64(salt),
    nonce: bytesToBase64(nonce),
    ciphertext: bytesToBase64(new Uint8Array(ciphertext)),
    deviceLabel: options.deviceLabel ?? 'USB ZK Auth Device',
  };
}

export async function decryptSecretFile(encryptedFile, pin, options = {}) {
  requirePin(pin);
  validateEncryptedSecretFile(encryptedFile);
  const cryptoProvider = options.cryptoProvider ?? globalThis.crypto;
  const salt = base64ToBytes(encryptedFile.salt);
  const nonce = base64ToBytes(encryptedFile.nonce);
  const ciphertext = base64ToBytes(encryptedFile.ciphertext);
  const key = await deriveAesKey(pin, salt, encryptedFile.iterations, cryptoProvider);

  let plaintext;
  try {
    plaintext = await cryptoProvider.subtle.decrypt({ name: 'AES-GCM', iv: nonce }, key, ciphertext);
  } catch (_error) {
    throw new Error('Unable to decrypt secret file. Check the PIN and file contents.');
  }

  const payload = JSON.parse(decoder.decode(plaintext));
  if (!payload.device_secret) {
    throw new Error('Secret file payload is missing device_secret.');
  }
  return String(payload.device_secret);
}

export function serializeEncryptedSecretFile(encryptedFile) {
  return `${JSON.stringify(encryptedFile, null, 2)}\n`;
}

export function parseEncryptedSecretFile(text) {
  const parsed = JSON.parse(text);
  validateEncryptedSecretFile(parsed);
  return parsed;
}

function validateEncryptedSecretFile(encryptedFile) {
  if (!encryptedFile || typeof encryptedFile !== 'object') {
    throw new Error('Secret file must be a JSON object.');
  }
  if (encryptedFile.version !== SECRET_FILE_VERSION) {
    throw new Error(`Unsupported secret file version: ${encryptedFile.version}`);
  }
  if (encryptedFile.kdf !== KDF_NAME || encryptedFile.cipher !== CIPHER_NAME) {
    throw new Error('Unsupported secret file crypto parameters.');
  }
  if (!Number.isInteger(encryptedFile.iterations) || encryptedFile.iterations < 100000) {
    throw new Error('Secret file KDF iterations are too low or invalid.');
  }
  for (const key of ['salt', 'nonce', 'ciphertext']) {
    if (typeof encryptedFile[key] !== 'string' || encryptedFile[key].length === 0) {
      throw new Error(`Secret file is missing ${key}.`);
    }
  }
}

async function deriveAesKey(pin, salt, iterations, cryptoProvider) {
  const baseKey = await cryptoProvider.subtle.importKey('raw', encoder.encode(pin), 'PBKDF2', false, ['deriveKey']);
  return cryptoProvider.subtle.deriveKey(
    {
      name: 'PBKDF2',
      hash: 'SHA-256',
      salt,
      iterations,
    },
    baseKey,
    { name: 'AES-GCM', length: 256 },
    false,
    ['encrypt', 'decrypt'],
  );
}

function requirePin(pin) {
  if (typeof pin !== 'string' || pin.length < 6) {
    throw new Error('PIN must be at least 6 characters.');
  }
}

function randomBytes(length, cryptoProvider) {
  const bytes = new Uint8Array(length);
  cryptoProvider.getRandomValues(bytes);
  return bytes;
}

function bytesToBase64(bytes) {
  if (typeof Buffer !== 'undefined') {
    return Buffer.from(bytes).toString('base64');
  }
  let binary = '';
  for (const byte of bytes) {
    binary += String.fromCharCode(byte);
  }
  return btoa(binary);
}

function base64ToBytes(value) {
  if (typeof Buffer !== 'undefined') {
    return new Uint8Array(Buffer.from(value, 'base64'));
  }
  const binary = atob(value);
  const bytes = new Uint8Array(binary.length);
  for (let index = 0; index < binary.length; index += 1) {
    bytes[index] = binary.charCodeAt(index);
  }
  return bytes;
}
