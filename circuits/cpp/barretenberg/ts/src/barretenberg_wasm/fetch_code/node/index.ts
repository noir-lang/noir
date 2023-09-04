import { readFile } from 'fs/promises';
import { dirname } from 'path';
import { fileURLToPath } from 'url';

// eslint-disable-next-line @typescript-eslint/no-unused-vars
export async function fetchCode(multithreaded: boolean) {
  const path = dirname(fileURLToPath(import.meta.url)) + '/../../../barretenberg-threads.wasm';
  return await readFile(path);
}
