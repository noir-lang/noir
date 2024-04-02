import { type Proxify } from '../transport/index.js';
import { type WasmModule } from '../wasm/wasm_module.js';

/**
 * Represents either a WASM web worker, or node.js worker.
 */
export type WasmWorker = Proxify<WasmModule> & { destroyWorker(): void };
