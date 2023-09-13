import { type Worker } from 'worker_threads';
import { BarretenbergWasm, BarretenbergWasmWorker } from './index.js';

describe('barretenberg wasm', () => {
  let worker!: Worker;
  let wasm!: BarretenbergWasmWorker;

  beforeAll(async () => {
    ({ wasm, worker } = await BarretenbergWasm.new(2));
  }, 20000);

  afterAll(async () => {
    await wasm.destroy();
    await worker.terminate();
  });

  it('should new malloc, transfer and slice mem', async () => {
    const length = 1024;
    const ptr = await wasm.call('bbmalloc', length);
    const buf = Buffer.alloc(length, 128);
    await wasm.writeMemory(ptr, buf);
    const result = Buffer.from(await wasm.getMemorySlice(ptr, ptr + length));
    await wasm.call('bbfree', ptr);
    expect(result).toStrictEqual(buf);
  });

  it('test abort', async () => {
    await expect(() => wasm.call('test_abort')).rejects.toThrow();
  });

  it('test c/c++ stdout/stderr', async () => {
    // We're checking we don't crash, but you can manually confirm you see log lines handled by logstr.
    await wasm.call('test_stdout_stderr');
  });

  it('should new malloc, transfer and slice mem', async () => {
    const length = 1024;
    const ptr = await wasm.call('bbmalloc', length);
    const buf = Buffer.alloc(length, 128);
    await wasm.writeMemory(ptr, buf);
    const result = Buffer.from(await wasm.getMemorySlice(ptr, ptr + length));
    await wasm.call('bbfree', ptr);
    expect(result).toStrictEqual(buf);
  });
});
