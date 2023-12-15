import { proxy } from 'comlink';
import { BarretenbergApi, BarretenbergApiSync } from '../barretenberg_api/index.js';
import { createMainWorker } from '../barretenberg_wasm/barretenberg_wasm_main/factory/node/index.js';
import { BarretenbergWasmMain, BarretenbergWasmMainWorker } from '../barretenberg_wasm/barretenberg_wasm_main/index.js';
import { getRemoteBarretenbergWasm } from '../barretenberg_wasm/helpers/index.js';
import { BarretenbergWasmWorker, fetchModuleAndThreads } from '../barretenberg_wasm/index.js';
import createDebug from 'debug';

const debug = createDebug('bb.js:wasm');

export type BackendOptions = {
  threads?: number;
  memory?: { initial?: number; maximum?: number };
};

/**
 * The main class library consumers interact with.
 * It extends the generated api, and provides a static constructor "new" to compose components.
 */
export class Barretenberg extends BarretenbergApi {
  private constructor(private worker: any, wasm: BarretenbergWasmWorker) {
    super(wasm);
  }

  /**
   * Constructs an instance of Barretenberg.
   * Launches it within a worker. This is necessary as it blocks waiting on child threads to complete,
   * and blocking the main thread in the browser is not allowed.
   * It threads > 1 (defaults to hardware availability), child threads will be created on their own workers.
   */
  static async new({ threads: desiredThreads, memory }: BackendOptions = {}) {
    const worker = createMainWorker();
    const wasm = getRemoteBarretenbergWasm<BarretenbergWasmMainWorker>(worker);
    const { module, threads } = await fetchModuleAndThreads(desiredThreads);
    await wasm.init(module, threads, proxy(debug), memory?.initial, memory?.maximum);
    return new Barretenberg(worker, wasm);
  }

  async getNumThreads() {
    return await this.wasm.getNumThreads();
  }

  async destroy() {
    await this.wasm.destroy();
    await this.worker.terminate();
  }
}

let barretenbergSyncSingleton: BarretenbergSync;
let barretenbergSyncSingletonPromise: Promise<BarretenbergSync>;

export class BarretenbergSync extends BarretenbergApiSync {
  private constructor(wasm: BarretenbergWasmMain) {
    super(wasm);
  }

  static async new() {
    const wasm = new BarretenbergWasmMain();
    const { module, threads } = await fetchModuleAndThreads(1);
    await wasm.init(module, threads);
    return new BarretenbergSync(wasm);
  }

  static initSingleton() {
    if (!barretenbergSyncSingletonPromise) {
      barretenbergSyncSingletonPromise = BarretenbergSync.new().then(s => (barretenbergSyncSingleton = s));
    }
    return barretenbergSyncSingletonPromise;
  }

  static getSingleton() {
    if (!barretenbergSyncSingleton) {
      throw new Error('First call BarretenbergSync.initSingleton() on @aztec/bb.js module.');
    }
    return barretenbergSyncSingleton;
  }

  getWasm() {
    return this.wasm;
  }
}

// If we're in ESM environment, use top level await. CJS users need to call it manually.
// Need to ignore for cjs build.
// eslint-disable-next-line @typescript-eslint/ban-ts-comment
// @ts-ignore
await BarretenbergSync.initSingleton(); // POSTPROCESS ESM ONLY
