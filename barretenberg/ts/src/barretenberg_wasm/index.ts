import { proxy } from 'comlink';
import createDebug from 'debug';
import { createMainWorker } from './barretenberg_wasm_main/factory/node/index.js';
import { getRemoteBarretenbergWasm, getSharedMemoryAvailable } from './helpers/node/index.js';
import { BarretenbergWasmMain, BarretenbergWasmMainWorker } from './barretenberg_wasm_main/index.js';
import { fetchCode } from './fetch_code/index.js';

const debug = createDebug('bb.js:wasm');

export async function fetchModuleAndThreads(desiredThreads?: number) {
  const shared = getSharedMemoryAvailable();
  const threads = shared ? desiredThreads : 1;
  const code = await fetchCode(shared);
  const module = await WebAssembly.compile(code);
  return { module, threads };
}

export class BarretenbergWasm extends BarretenbergWasmMain {
  /**
   * Construct and initialize BarretenbergWasm within a Worker. Return both the worker and the wasm proxy.
   * Used when running in the browser, because we can't block the main thread.
   */
  public static async new(desiredThreads?: number) {
    const worker = createMainWorker();
    const wasm = getRemoteBarretenbergWasm<BarretenbergWasmMainWorker>(worker);
    const { module, threads } = await fetchModuleAndThreads(desiredThreads);
    await wasm.init(module, threads, proxy(debug));
    return { worker, wasm };
  }
}

export type BarretenbergWasmWorker = BarretenbergWasmMainWorker;
