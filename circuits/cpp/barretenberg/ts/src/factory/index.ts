import { BarretenbergApi, BarretenbergApiSync } from '../barretenberg_api/index.js';
import { BarretenbergBinder, BarretenbergBinderSync } from '../barretenberg_binder/index.js';
import { BarretenbergWasm, BarretenbergWasmWorker } from '../barretenberg_wasm/index.js';

/**
 * Returns a single threaded, synchronous, barretenberg api.
 * Can be used on the main thread to perform small light-weight requests like hashing etc.
 */
export async function newBarretenbergApiSync() {
  return new BarretenbergApiSync(new BarretenbergBinderSync(await BarretenbergWasm.new()));
}

export class BarretenbergApiAsync extends BarretenbergApi {
  constructor(private worker: any, private wasm: BarretenbergWasmWorker) {
    super(new BarretenbergBinder(wasm));
  }

  async getNumThreads() {
    return await this.wasm.getNumThreads();
  }

  async destroy() {
    await this.wasm.destroy();
    await this.worker.terminate();
  }
}

/**
 * Returns a multi threaded, asynchronous, barretenberg api.
 * It runs in a worker, and so can be used within the browser to execute long running, multi-threaded requests
 * like proof construction etc.
 */
export async function newBarretenbergApiAsync(threads?: number) {
  const { wasm, worker } = await BarretenbergWasm.newWorker(threads);
  return new BarretenbergApiAsync(worker, wasm);
}
