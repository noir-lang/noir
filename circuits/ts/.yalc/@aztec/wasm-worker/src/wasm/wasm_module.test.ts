import { WasmModule } from './wasm_module.js';
import { fileURLToPath } from 'url';
import { readFile } from 'fs/promises';
import { dirname } from 'path';

/**
 *
 */
async function fetchCode() {
  const __dirname = dirname(fileURLToPath(import.meta.url));
  return await readFile(`${__dirname}/../test/gcd.wasm`);
}

describe('barretenberg wasm', () => {
  let wasm!: WasmModule;

  beforeAll(async () => {
    wasm = new WasmModule(await fetchCode(), () => ({
      /*no imports*/
    }));
    await wasm.init();
  });

  it('should new malloc, transfer and slice mem', () => {
    expect(wasm.call('gcd', 1, 1)).toBe(1);
  });
});
