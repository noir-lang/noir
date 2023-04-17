import { createDebugLogger } from '@aztec/log';
import { MemoryFifo } from '../memory_fifo.js';
import { getEmptyWasiSdk } from './empty_wasi_sdk.js';
import { randomBytes } from 'crypto';
/**
 * WasmModule:
 *  Helper over a webassembly module.
 *  Assumes a few quirks.
 *  1) the module expects wasi_snapshot_preview1 with the methods from getEmptyWasiSdk
 *  2) of which the webassembly
 *  we instantiate only uses random_get (update this if more WASI sdk methods are needed).
 */
export class WasmModule {
    /**
     * Create a wasm module. Should be followed by await init();.
     * @param module - The module as a WebAssembly.Module or a Buffer.
     * @param importFn - Imports expected by the WASM.
     * @param loggerName - Optional, for debug logging.
     */
    constructor(module, importFn, loggerName = 'wasm-worker') {
        this.module = module;
        this.importFn = importFn;
        this.mutexQ = new MemoryFifo();
        this.debug = createDebugLogger(loggerName);
        this.mutexQ.put(true);
    }
    /**
     * Initialize this wasm module.
     * @param wasmImportEnv - Linked to a module called "env". Functions implementations referenced from e.g. C++.
     * @param initial - 20 pages by default. 20*2**16 \> 1mb stack size plus other overheads.
     * @param maximum - 8192 maximum by default. 512mb.
     */
    async init(initial = 20, maximum = 8192) {
        this.debug(`initial mem: ${initial} pages, ${(initial * 2 ** 16) / (1024 * 1024)}mb. max mem: ${maximum} pages, ${(maximum * 2 ** 16) / (1024 * 1024)}mb`);
        this.memory = new WebAssembly.Memory({ initial, maximum });
        // Create a view over the memory buffer.
        // We do this once here, as webkit *seems* bugged out and actually shows this as new memory,
        // thus displaying double. It's only worse if we create views on demand. I haven't established yet if
        // the bug is also exasperating the termination on mobile due to "excessive memory usage". It could be
        // that the OS is actually getting an incorrect reading in the same way the memory profiler does...
        // The view will have to be recreated if the memory is grown. See getMemory().
        this.heap = new Uint8Array(this.memory.buffer);
        // We support the wasi 12 SDK, but only implement random_get
        /* eslint-disable camelcase */
        const importObj = {
            wasi_snapshot_preview1: {
                ...getEmptyWasiSdk(this.debug),
                random_get: (arr, length) => {
                    arr = arr >>> 0;
                    const heap = this.getMemory();
                    const randomData = randomBytes(length);
                    for (let i = arr; i < arr + length; ++i) {
                        heap[i] = randomData[i - arr];
                    }
                },
            },
            env: this.importFn(this),
        };
        if (this.module instanceof WebAssembly.Module) {
            this.instance = await WebAssembly.instantiate(this.module, importObj);
        }
        else {
            const { instance } = await WebAssembly.instantiate(this.module, importObj);
            this.instance = instance;
        }
    }
    /**
     * The methods or objects exported by the WASM module.
     * @returns An indexable object.
     */
    exports() {
        if (!this.instance) {
            throw new Error('WasmModule: not initialized!');
        }
        return this.instance.exports;
    }
    /**
     * Get the current logger.
     * @returns Logging function.
     */
    getLogger() {
        return this.debug;
    }
    /**
     * Add a logger.
     * @param logger - Function to call when logging.
     */
    addLogger(logger) {
        const oldDebug = this.debug;
        this.debug = (...args) => {
            logger(...args);
            oldDebug(...args);
        };
    }
    /**
     * Calls into the WebAssembly.
     * @param name - The method name.
     * @param args - The arguments to the method.
     * @returns The numeric method result.
     */
    call(name, ...args) {
        if (!this.exports()[name]) {
            throw new Error(`WASM function ${name} not found.`);
        }
        try {
            // When returning values from the WASM, use >>> operator to convert
            // signed representation to unsigned representation.
            return this.exports()[name](...args) >>> 0;
        }
        catch (err) {
            const message = `WASM function ${name} aborted, error: ${err}`;
            this.debug(message);
            this.debug(err.stack);
            throw new Error(message);
        }
    }
    /**
     * Get the memory used by the WASM module.
     * @returns A WebAssembly memory object.
     */
    getRawMemory() {
        return this.memory;
    }
    /**
     * Get the memory used by the WASM module, as a byte array.
     * @returns A Uint8Array view of the WASM module memory.
     */
    getMemory() {
        // If the memory is grown, our view over it will be lost. Recreate the view.
        if (this.heap.length === 0) {
            this.heap = new Uint8Array(this.memory.buffer);
        }
        return this.heap;
    }
    /**
     * The memory size in bytes.
     * @returns Number of bytes.
     */
    memSize() {
        return this.getMemory().length;
    }
    /**
     * Get a slice of memory between two addresses.
     * @param start - The start address.
     * @param end - The end address.
     * @returns A Uint8Array view of memory.
     */
    getMemorySlice(start, end) {
        return this.getMemory().slice(start, end);
    }
    /**
     * Write data into the heap.
     * @param arr - The data to write.
     * @param offset - The address to write data at.
     */
    transferToHeap(arr, offset) {
        const mem = this.getMemory();
        for (let i = 0; i < arr.length; i++) {
            mem[i + offset] = arr[i];
        }
    }
    /**
     * When calling the wasm, sometimes a caller will require exclusive access over a series of calls.
     * E.g. When a result is written to address 0, one cannot have another caller writing to the same address via
     * transferToHeap before the result is read via sliceMemory.
     * Acquire() gets a single token from a fifo. The caller must call release() to add the token back.
     */
    async acquire() {
        await this.mutexQ.get();
    }
    /**
     * Release the mutex, letting another promise call acquire().
     */
    release() {
        if (this.mutexQ.length() !== 0) {
            throw new Error('Release called but not acquired.');
        }
        this.mutexQ.put(true);
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid2FzbV9tb2R1bGUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvd2FzbS93YXNtX21vZHVsZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsaUJBQWlCLEVBQWUsTUFBTSxZQUFZLENBQUM7QUFFNUQsT0FBTyxFQUFFLFVBQVUsRUFBRSxNQUFNLG1CQUFtQixDQUFDO0FBQy9DLE9BQU8sRUFBRSxlQUFlLEVBQUUsTUFBTSxxQkFBcUIsQ0FBQztBQUN0RCxPQUFPLEVBQUUsV0FBVyxFQUFFLE1BQU0sUUFBUSxDQUFDO0FBRXJDOzs7Ozs7O0dBT0c7QUFDSCxNQUFNLE9BQU8sVUFBVTtJQU9yQjs7Ozs7T0FLRztJQUNILFlBQ1UsTUFBbUMsRUFDbkMsUUFBcUMsRUFDN0MsVUFBVSxHQUFHLGFBQWE7UUFGbEIsV0FBTSxHQUFOLE1BQU0sQ0FBNkI7UUFDbkMsYUFBUSxHQUFSLFFBQVEsQ0FBNkI7UUFYdkMsV0FBTSxHQUFHLElBQUksVUFBVSxFQUFXLENBQUM7UUFjekMsSUFBSSxDQUFDLEtBQUssR0FBRyxpQkFBaUIsQ0FBQyxVQUFVLENBQUMsQ0FBQztRQUMzQyxJQUFJLENBQUMsTUFBTSxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsQ0FBQztJQUN4QixDQUFDO0lBRUQ7Ozs7O09BS0c7SUFDSSxLQUFLLENBQUMsSUFBSSxDQUFDLE9BQU8sR0FBRyxFQUFFLEVBQUUsT0FBTyxHQUFHLElBQUk7UUFDNUMsSUFBSSxDQUFDLEtBQUssQ0FDUixnQkFBZ0IsT0FBTyxXQUFXLENBQUMsT0FBTyxHQUFHLENBQUMsSUFBSSxFQUFFLENBQUMsR0FBRyxDQUFDLElBQUksR0FBRyxJQUFJLENBQUMsZ0JBQWdCLE9BQU8sV0FDMUYsQ0FBQyxPQUFPLEdBQUcsQ0FBQyxJQUFJLEVBQUUsQ0FBQyxHQUFHLENBQUMsSUFBSSxHQUFHLElBQUksQ0FDcEMsSUFBSSxDQUNMLENBQUM7UUFDRixJQUFJLENBQUMsTUFBTSxHQUFHLElBQUksV0FBVyxDQUFDLE1BQU0sQ0FBQyxFQUFFLE9BQU8sRUFBRSxPQUFPLEVBQUUsQ0FBQyxDQUFDO1FBQzNELHdDQUF3QztRQUN4Qyw0RkFBNEY7UUFDNUYscUdBQXFHO1FBQ3JHLHNHQUFzRztRQUN0RyxtR0FBbUc7UUFDbkcsOEVBQThFO1FBQzlFLElBQUksQ0FBQyxJQUFJLEdBQUcsSUFBSSxVQUFVLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxNQUFNLENBQUMsQ0FBQztRQUUvQyw0REFBNEQ7UUFDNUQsOEJBQThCO1FBQzlCLE1BQU0sU0FBUyxHQUFHO1lBQ2hCLHNCQUFzQixFQUFFO2dCQUN0QixHQUFHLGVBQWUsQ0FBQyxJQUFJLENBQUMsS0FBSyxDQUFDO2dCQUM5QixVQUFVLEVBQUUsQ0FBQyxHQUFXLEVBQUUsTUFBYyxFQUFFLEVBQUU7b0JBQzFDLEdBQUcsR0FBRyxHQUFHLEtBQUssQ0FBQyxDQUFDO29CQUNoQixNQUFNLElBQUksR0FBRyxJQUFJLENBQUMsU0FBUyxFQUFFLENBQUM7b0JBQzlCLE1BQU0sVUFBVSxHQUFHLFdBQVcsQ0FBQyxNQUFNLENBQUMsQ0FBQztvQkFDdkMsS0FBSyxJQUFJLENBQUMsR0FBRyxHQUFHLEVBQUUsQ0FBQyxHQUFHLEdBQUcsR0FBRyxNQUFNLEVBQUUsRUFBRSxDQUFDLEVBQUU7d0JBQ3ZDLElBQUksQ0FBQyxDQUFDLENBQUMsR0FBRyxVQUFVLENBQUMsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxDQUFDO3FCQUMvQjtnQkFDSCxDQUFDO2FBQ0Y7WUFDRCxHQUFHLEVBQUUsSUFBSSxDQUFDLFFBQVEsQ0FBQyxJQUFJLENBQUM7U0FDekIsQ0FBQztRQUVGLElBQUksSUFBSSxDQUFDLE1BQU0sWUFBWSxXQUFXLENBQUMsTUFBTSxFQUFFO1lBQzdDLElBQUksQ0FBQyxRQUFRLEdBQUcsTUFBTSxXQUFXLENBQUMsV0FBVyxDQUFDLElBQUksQ0FBQyxNQUFNLEVBQUUsU0FBUyxDQUFDLENBQUM7U0FDdkU7YUFBTTtZQUNMLE1BQU0sRUFBRSxRQUFRLEVBQUUsR0FBRyxNQUFNLFdBQVcsQ0FBQyxXQUFXLENBQUMsSUFBSSxDQUFDLE1BQU0sRUFBRSxTQUFTLENBQUMsQ0FBQztZQUMzRSxJQUFJLENBQUMsUUFBUSxHQUFHLFFBQVEsQ0FBQztTQUMxQjtJQUNILENBQUM7SUFFRDs7O09BR0c7SUFDSSxPQUFPO1FBQ1osSUFBSSxDQUFDLElBQUksQ0FBQyxRQUFRLEVBQUU7WUFDbEIsTUFBTSxJQUFJLEtBQUssQ0FBQyw4QkFBOEIsQ0FBQyxDQUFDO1NBQ2pEO1FBQ0QsT0FBTyxJQUFJLENBQUMsUUFBUSxDQUFDLE9BQU8sQ0FBQztJQUMvQixDQUFDO0lBRUQ7OztPQUdHO0lBQ0ksU0FBUztRQUNkLE9BQU8sSUFBSSxDQUFDLEtBQUssQ0FBQztJQUNwQixDQUFDO0lBRUQ7OztPQUdHO0lBQ0ksU0FBUyxDQUFDLE1BQW1CO1FBQ2xDLE1BQU0sUUFBUSxHQUFHLElBQUksQ0FBQyxLQUFLLENBQUM7UUFDNUIsSUFBSSxDQUFDLEtBQUssR0FBRyxDQUFDLEdBQUcsSUFBVyxFQUFFLEVBQUU7WUFDOUIsTUFBTSxDQUFDLEdBQUcsSUFBSSxDQUFDLENBQUM7WUFDaEIsUUFBUSxDQUFDLEdBQUcsSUFBSSxDQUFDLENBQUM7UUFDcEIsQ0FBQyxDQUFDO0lBQ0osQ0FBQztJQUVEOzs7OztPQUtHO0lBQ0ksSUFBSSxDQUFDLElBQVksRUFBRSxHQUFHLElBQVM7UUFDcEMsSUFBSSxDQUFDLElBQUksQ0FBQyxPQUFPLEVBQUUsQ0FBQyxJQUFJLENBQUMsRUFBRTtZQUN6QixNQUFNLElBQUksS0FBSyxDQUFDLGlCQUFpQixJQUFJLGFBQWEsQ0FBQyxDQUFDO1NBQ3JEO1FBQ0QsSUFBSTtZQUNGLG1FQUFtRTtZQUNuRSxvREFBb0Q7WUFDcEQsT0FBTyxJQUFJLENBQUMsT0FBTyxFQUFFLENBQUMsSUFBSSxDQUFDLENBQUMsR0FBRyxJQUFJLENBQUMsS0FBSyxDQUFDLENBQUM7U0FDNUM7UUFBQyxPQUFPLEdBQVEsRUFBRTtZQUNqQixNQUFNLE9BQU8sR0FBRyxpQkFBaUIsSUFBSSxvQkFBb0IsR0FBRyxFQUFFLENBQUM7WUFDL0QsSUFBSSxDQUFDLEtBQUssQ0FBQyxPQUFPLENBQUMsQ0FBQztZQUNwQixJQUFJLENBQUMsS0FBSyxDQUFDLEdBQUcsQ0FBQyxLQUFLLENBQUMsQ0FBQztZQUN0QixNQUFNLElBQUksS0FBSyxDQUFDLE9BQU8sQ0FBQyxDQUFDO1NBQzFCO0lBQ0gsQ0FBQztJQUNEOzs7T0FHRztJQUNJLFlBQVk7UUFDakIsT0FBTyxJQUFJLENBQUMsTUFBTSxDQUFDO0lBQ3JCLENBQUM7SUFDRDs7O09BR0c7SUFDSSxTQUFTO1FBQ2QsNEVBQTRFO1FBQzVFLElBQUksSUFBSSxDQUFDLElBQUksQ0FBQyxNQUFNLEtBQUssQ0FBQyxFQUFFO1lBQzFCLElBQUksQ0FBQyxJQUFJLEdBQUcsSUFBSSxVQUFVLENBQUMsSUFBSSxDQUFDLE1BQU0sQ0FBQyxNQUFNLENBQUMsQ0FBQztTQUNoRDtRQUNELE9BQU8sSUFBSSxDQUFDLElBQUksQ0FBQztJQUNuQixDQUFDO0lBRUQ7OztPQUdHO0lBQ0ksT0FBTztRQUNaLE9BQU8sSUFBSSxDQUFDLFNBQVMsRUFBRSxDQUFDLE1BQU0sQ0FBQztJQUNqQyxDQUFDO0lBRUQ7Ozs7O09BS0c7SUFDSSxjQUFjLENBQUMsS0FBYSxFQUFFLEdBQVc7UUFDOUMsT0FBTyxJQUFJLENBQUMsU0FBUyxFQUFFLENBQUMsS0FBSyxDQUFDLEtBQUssRUFBRSxHQUFHLENBQUMsQ0FBQztJQUM1QyxDQUFDO0lBRUQ7Ozs7T0FJRztJQUNJLGNBQWMsQ0FBQyxHQUFlLEVBQUUsTUFBYztRQUNuRCxNQUFNLEdBQUcsR0FBRyxJQUFJLENBQUMsU0FBUyxFQUFFLENBQUM7UUFDN0IsS0FBSyxJQUFJLENBQUMsR0FBRyxDQUFDLEVBQUUsQ0FBQyxHQUFHLEdBQUcsQ0FBQyxNQUFNLEVBQUUsQ0FBQyxFQUFFLEVBQUU7WUFDbkMsR0FBRyxDQUFDLENBQUMsR0FBRyxNQUFNLENBQUMsR0FBRyxHQUFHLENBQUMsQ0FBQyxDQUFDLENBQUM7U0FDMUI7SUFDSCxDQUFDO0lBRUQ7Ozs7O09BS0c7SUFDSSxLQUFLLENBQUMsT0FBTztRQUNsQixNQUFNLElBQUksQ0FBQyxNQUFNLENBQUMsR0FBRyxFQUFFLENBQUM7SUFDMUIsQ0FBQztJQUVEOztPQUVHO0lBQ0ksT0FBTztRQUNaLElBQUksSUFBSSxDQUFDLE1BQU0sQ0FBQyxNQUFNLEVBQUUsS0FBSyxDQUFDLEVBQUU7WUFDOUIsTUFBTSxJQUFJLEtBQUssQ0FBQyxrQ0FBa0MsQ0FBQyxDQUFDO1NBQ3JEO1FBQ0QsSUFBSSxDQUFDLE1BQU0sQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLENBQUM7SUFDeEIsQ0FBQztDQUNGIn0=