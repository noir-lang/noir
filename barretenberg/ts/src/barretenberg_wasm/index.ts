import { proxy } from 'comlink';
import createDebug from 'debug';
import { createMainWorker } from './barretenberg_wasm_main/factory/node/index.js';
import { getRemoteBarretenbergWasm } from './helpers/node/index.js';
import { BarretenbergWasmMain, BarretenbergWasmMainWorker } from './barretenberg_wasm_main/index.js';

const debug = createDebug('bb.js:wasm');

export class BarretenbergWasm extends BarretenbergWasmMain {
  /**
   * Construct and initialise BarretenbergWasm within a Worker. Return both the worker and the wasm proxy.
   * Used when running in the browser, because we can't block the main thread.
   */
  public static async new(threads?: number) {
    const worker = createMainWorker();
    const wasm = getRemoteBarretenbergWasm<BarretenbergWasmMainWorker>(worker);
    await wasm.init(threads, proxy(debug));
    return { worker, wasm };
  }
}

export type BarretenbergWasmWorker = BarretenbergWasmMainWorker;
