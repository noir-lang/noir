import { CircuitsWasm } from './circuits_wasm.js';

describe('basic barretenberg smoke test', () => {
  let wasm: CircuitsWasm;

  beforeAll(async () => {
    wasm = await CircuitsWasm.get();
  });

  it('should new malloc, transfer and slice mem', () => {
    const length = 1024;
    const ptr = wasm.call('bbmalloc', length);
    const buf = Buffer.alloc(length, 128);
    wasm.writeMemory(ptr, buf);
    const result = Buffer.from(wasm.getMemorySlice(ptr, ptr + length));
    wasm.call('bbfree', ptr);
    expect(result).toStrictEqual(buf);
  });
});
