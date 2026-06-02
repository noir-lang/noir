import { decryptSecretFile, parseEncryptedSecretFile } from './secret-file.js';
import { readFile, writeFile } from 'node:fs/promises';

export class NodePathSecretProvider {
  async readSecret({ path, pin }) {
    if (!path) {
      throw new Error('Missing --secret path.');
    }
    const encryptedFile = parseEncryptedSecretFile(await readFile(path, 'utf8'));
    return decryptSecretFile(encryptedFile, pin);
  }

  async writeSecret({ path, contents }) {
    if (!path) {
      throw new Error('Missing --out path.');
    }
    await writeFile(path, contents, 'utf8');
  }
}
