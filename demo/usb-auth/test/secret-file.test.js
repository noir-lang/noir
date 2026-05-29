import { strict as assert } from 'node:assert';
import { webcrypto } from 'node:crypto';
import {
  createEncryptedSecretFile,
  decryptSecretFile,
  parseEncryptedSecretFile,
  serializeEncryptedSecretFile,
} from '../src/secret-file.js';

describe('encrypted secret files', () => {
  it('round trips a device secret with the correct PIN', async () => {
    const encrypted = await createEncryptedSecretFile('123456789', '123456', {
      cryptoProvider: webcrypto,
      iterations: 100000,
    });

    const decrypted = await decryptSecretFile(encrypted, '123456', { cryptoProvider: webcrypto });

    assert.equal(decrypted, '123456789');
  });

  it('rejects the wrong PIN', async () => {
    const encrypted = await createEncryptedSecretFile('123456789', '123456', {
      cryptoProvider: webcrypto,
      iterations: 100000,
    });

    await assert.rejects(() => decryptSecretFile(encrypted, '654321', { cryptoProvider: webcrypto }), /Unable to decrypt/);
  });

  it('serializes and parses the encrypted file format', async () => {
    const encrypted = await createEncryptedSecretFile('7', '123456', {
      cryptoProvider: webcrypto,
      iterations: 100000,
    });

    const parsed = parseEncryptedSecretFile(serializeEncryptedSecretFile(encrypted));

    assert.equal(parsed.version, 1);
    assert.equal(parsed.kdf, 'PBKDF2-HMAC-SHA256');
    assert.equal(parsed.cipher, 'AES-256-GCM');
  });
});
