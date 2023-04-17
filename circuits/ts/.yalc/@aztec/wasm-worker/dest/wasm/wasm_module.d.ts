/// <reference types="node" resolution-mode="require"/>
import { DebugLogger } from '@aztec/log';
import { Buffer } from 'buffer';
/**
 * WasmModule:
 *  Helper over a webassembly module.
 *  Assumes a few quirks.
 *  1) the module expects wasi_snapshot_preview1 with the methods from getEmptyWasiSdk
 *  2) of which the webassembly
 *  we instantiate only uses random_get (update this if more WASI sdk methods are needed).
 */
export declare class WasmModule {
    private module;
    private importFn;
    private memory;
    private heap;
    private instance?;
    private mutexQ;
    private debug;
    /**
     * Create a wasm module. Should be followed by await init();.
     * @param module - The module as a WebAssembly.Module or a Buffer.
     * @param importFn - Imports expected by the WASM.
     * @param loggerName - Optional, for debug logging.
     */
    constructor(module: WebAssembly.Module | Buffer, importFn: (module: WasmModule) => any, loggerName?: string);
    /**
     * Initialize this wasm module.
     * @param wasmImportEnv - Linked to a module called "env". Functions implementations referenced from e.g. C++.
     * @param initial - 20 pages by default. 20*2**16 \> 1mb stack size plus other overheads.
     * @param maximum - 8192 maximum by default. 512mb.
     */
    init(initial?: number, maximum?: number): Promise<void>;
    /**
     * The methods or objects exported by the WASM module.
     * @returns An indexable object.
     */
    exports(): any;
    /**
     * Get the current logger.
     * @returns Logging function.
     */
    getLogger(): DebugLogger;
    /**
     * Add a logger.
     * @param logger - Function to call when logging.
     */
    addLogger(logger: DebugLogger): void;
    /**
     * Calls into the WebAssembly.
     * @param name - The method name.
     * @param args - The arguments to the method.
     * @returns The numeric method result.
     */
    call(name: string, ...args: any): number;
    /**
     * Get the memory used by the WASM module.
     * @returns A WebAssembly memory object.
     */
    getRawMemory(): WebAssembly.Memory;
    /**
     * Get the memory used by the WASM module, as a byte array.
     * @returns A Uint8Array view of the WASM module memory.
     */
    getMemory(): Uint8Array;
    /**
     * The memory size in bytes.
     * @returns Number of bytes.
     */
    memSize(): number;
    /**
     * Get a slice of memory between two addresses.
     * @param start - The start address.
     * @param end - The end address.
     * @returns A Uint8Array view of memory.
     */
    getMemorySlice(start: number, end: number): Uint8Array;
    /**
     * Write data into the heap.
     * @param arr - The data to write.
     * @param offset - The address to write data at.
     */
    transferToHeap(arr: Uint8Array, offset: number): void;
    /**
     * When calling the wasm, sometimes a caller will require exclusive access over a series of calls.
     * E.g. When a result is written to address 0, one cannot have another caller writing to the same address via
     * transferToHeap before the result is read via sliceMemory.
     * Acquire() gets a single token from a fifo. The caller must call release() to add the token back.
     */
    acquire(): Promise<void>;
    /**
     * Release the mutex, letting another promise call acquire().
     */
    release(): void;
}
//# sourceMappingURL=wasm_module.d.ts.map