import { FileCrs, SRS_DEV_PATH } from '../crs/index.js';
import { BarretenbergWasm } from './barretenberg_wasm.js';

describe('basic barretenberg smoke test', () => {
  const wasm: BarretenbergWasm = new BarretenbergWasm();

  beforeAll(async () => {
    await wasm.init();
  });

  it('should new malloc, transfer and slice mem', () => {
    const length = 1024;
    const ptr = wasm.call('bbmalloc', length);
    const buf = Buffer.alloc(length, 128);
    wasm.writeMemory(ptr, buf);
    wasm.call('bbfree', ptr);
    const result = Buffer.from(wasm.getMemorySlice(ptr, ptr + length));
    expect(result).toStrictEqual(buf);
  });

  it('should use asyncify to do an async callback into js', async () => {
    const addr1 = await wasm.asyncCall('test_async_func', 1024 * 1024, 1);
    const addr2 = await wasm.asyncCall('test_async_func', 1024 * 1024 * 2, 2);
    expect(wasm.getMemorySlice(addr1, addr1 + 1024 * 1024).every(v => v === 1)).toBe(true);
    expect(wasm.getMemorySlice(addr2, addr2 + 1024 * 1024 * 2).every(v => v === 2)).toBe(true);
  });

  it('should correctly pass CRS data through env_load_verifier_crs', async () => {
    const crs = new FileCrs(0, SRS_DEV_PATH);
    await crs.init();
    const g2DataPtr = await wasm.asyncCall('test_env_load_verifier_crs');
    const g2Data = wasm.getMemorySlice(g2DataPtr, g2DataPtr + 128);
    expect(Buffer.from(g2Data)).toStrictEqual(crs.getG2Data());
    wasm.call('bbfree', g2DataPtr);
  });

  it('should correctly pass CRS data through env_load_prover_crs', async () => {
    const numPoints = 1024;
    const crs = new FileCrs(numPoints, SRS_DEV_PATH);
    await crs.init();
    const g1DataPtr = await wasm.asyncCall('test_env_load_prover_crs', numPoints);
    const g1Data = wasm.getMemorySlice(g1DataPtr, g1DataPtr + numPoints * 64);
    expect(Buffer.from(g1Data)).toStrictEqual(crs.getG1Data());
    wasm.call('bbfree', g1DataPtr);
  });
});
