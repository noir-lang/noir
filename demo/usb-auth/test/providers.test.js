import { strict as assert } from 'node:assert';
import { mkdtemp, readFile, rm, writeFile } from 'node:fs/promises';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { webcrypto } from 'node:crypto';
import { createEncryptedSecretFile, serializeEncryptedSecretFile } from '../src/secret-file.js';
import { NodePathSecretProvider } from '../src/node-providers.js';
import { FidoHsmSecretProvider, WebUsbSecretProvider } from '../src/secret-providers.js';

describe('secret providers', () => {
  it('reads an encrypted secret from a local path', async () => {
    const tempDir = await mkdtemp(join(tmpdir(), 'usb-auth-'));
    const path = join(tempDir, 'secret.json');
    try {
      const encrypted = await createEncryptedSecretFile('42', '123456', {
        cryptoProvider: webcrypto,
        iterations: 100000,
      });
      await writeFile(path, serializeEncryptedSecretFile(encrypted), 'utf8');

      const secret = await new NodePathSecretProvider().readSecret({ path, pin: '123456' });

      assert.equal(secret, '42');
      assert.match(await readFile(path, 'utf8'), /PBKDF2-HMAC-SHA256/);
    } finally {
      await rm(tempDir, { recursive: true, force: true });
    }
  });

  it('reports WebUSB as unavailable when the browser API is absent', async () => {
    await assert.rejects(() => new WebUsbSecretProvider().readSecret(), /WebUSB is not available/);
  });

  it('explains the FIDO/HSM integration boundary', async () => {
    await assert.rejects(() => new FidoHsmSecretProvider().readSecret(), /does not expose a raw device_secret/);
  });
});
