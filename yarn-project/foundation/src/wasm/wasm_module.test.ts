import { readFile } from 'fs/promises';
import { dirname } from 'path';

import { fileURLToPath } from '../url/index.js';
import { WasmModule } from './wasm_module.js';

/**
 * Fetch a simple WASM.
 */
async function fetchCode() {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  return await readFile(`${__dirname}/fixtures/gcd.wasm`);
}

describe('simple wasm', () => {
  let wasm!: WasmModule;

  beforeAll(async () => {
    wasm = new WasmModule(await fetchCode(), () => ({
      /*no imports*/
    }));
    await wasm.init(1, 1, /* No init method */ null);
  });

  it('should call gcd with correct result', () => {
    expect(wasm.call('gcd', 12312, 12123)).toBe(27);
  });
});
