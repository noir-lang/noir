import { WasmWorker } from '../wasm_worker.js';
/**
 * Instantiate a web worker.
 * @param url - The URL.
 * @param initialMem - Initial memory pages.
 * @param maxMem - Maximum memory pages.
 * @returns The worker.
 */
export declare function createWebWorker(url: string, initialMem?: number, maxMem?: number): Promise<WasmWorker>;
//# sourceMappingURL=web_worker.d.ts.map