import { createDebugLogger } from "@aztec/foundation";
import { Buffer } from "buffer";
import { MemoryFifo } from "../memory_fifo.js";
import { getEmptyWasiSdk } from "./empty_wasi_sdk.js";
import { randomBytes } from "crypto";
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
  constructor(module, importFn, loggerName = "wasm") {
    this.module = module;
    this.importFn = importFn;
    this.mutexQ = new MemoryFifo();
    this.debug = createDebugLogger(loggerName);
    this.mutexQ.put(true);
  }
  /**
   * Return the wasm source.
   * @returns The source.
   */
  getModule() {
    return this.module;
  }
  /**
   * Initialize this wasm module.
   * @param wasmImportEnv - Linked to a module called "env". Functions implementations referenced from e.g. C++.
   * @param initial - 20 pages by default. 20*2**16 \> 1mb stack size plus other overheads.
   * @param maximum - 8192 maximum by default. 512mb.
   */
  async init(initial = 20, maximum = 8192) {
    this.debug(
      `initial mem: ${initial} pages, ${
        (initial * 2 ** 16) / (1024 * 1024)
      }mb. max mem: ${maximum} pages, ${(maximum * 2 ** 16) / (1024 * 1024)}mb`
    );
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
    } else {
      const { instance } = await WebAssembly.instantiate(
        this.module,
        importObj
      );
      this.instance = instance;
    }
  }
  /**
   * The methods or objects exported by the WASM module.
   * @returns An indexable object.
   */
  exports() {
    if (!this.instance) {
      throw new Error("WasmModule: not initialized!");
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
    } catch (err) {
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
   * @param offset - The address to write data at.
   * @param arr - The data to write.
   */
  writeMemory(offset, arr) {
    const mem = this.getMemory();
    for (let i = 0; i < arr.length; i++) {
      mem[i + offset] = arr[i];
    }
  }
  /**
   * Read WASM memory as a JS string.
   * @param addr - The memory address.
   * @returns A JS string.
   */
  getMemoryAsString(addr) {
    addr = addr >>> 0;
    const m = this.getMemory();
    let i = addr;
    for (; m[i] !== 0; ++i);
    return Buffer.from(m.slice(addr, i)).toString("ascii");
  }
  /**
   * When calling the wasm, sometimes a caller will require exclusive access over a series of calls.
   * E.g. When a result is written to address 0, one cannot have another caller writing to the same address via
   * writeMemory before the result is read via sliceMemory.
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
      throw new Error("Release called but not acquired.");
    }
    this.mutexQ.put(true);
  }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid2FzbV9tb2R1bGUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvd2FzbS93YXNtX21vZHVsZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsaUJBQWlCLEVBQWUsTUFBTSxZQUFZLENBQUM7QUFDNUQsT0FBTyxFQUFFLE1BQU0sRUFBRSxNQUFNLFFBQVEsQ0FBQztBQUNoQyxPQUFPLEVBQUUsVUFBVSxFQUFFLE1BQU0sbUJBQW1CLENBQUM7QUFDL0MsT0FBTyxFQUFFLGVBQWUsRUFBRSxNQUFNLHFCQUFxQixDQUFDO0FBQ3RELE9BQU8sRUFBRSxXQUFXLEVBQUUsTUFBTSxRQUFRLENBQUM7QUFFckM7Ozs7Ozs7R0FPRztBQUNILE1BQU0sT0FBTyxVQUFVO0lBT3JCOzs7OztPQUtHO0lBQ0gsWUFDVSxNQUFtQyxFQUNuQyxRQUFxQyxFQUM3QyxVQUFVLEdBQUcsTUFBTTtRQUZYLFdBQU0sR0FBTixNQUFNLENBQTZCO1FBQ25DLGFBQVEsR0FBUixRQUFRLENBQTZCO1FBWHZDLFdBQU0sR0FBRyxJQUFJLFVBQVUsRUFBVyxDQUFDO1FBY3pDLElBQUksQ0FBQyxLQUFLLEdBQUcsaUJBQWlCLENBQUMsVUFBVSxDQUFDLENBQUM7UUFDM0MsSUFBSSxDQUFDLE1BQU0sQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLENBQUM7SUFDeEIsQ0FBQztJQUVEOzs7T0FHRztJQUNJLFNBQVM7UUFDZCxPQUFPLElBQUksQ0FBQyxNQUFNLENBQUM7SUFDckIsQ0FBQztJQUNEOzs7OztPQUtHO0lBQ0ksS0FBSyxDQUFDLElBQUksQ0FBQyxPQUFPLEdBQUcsRUFBRSxFQUFFLE9BQU8sR0FBRyxJQUFJO1FBQzVDLElBQUksQ0FBQyxLQUFLLENBQ1IsZ0JBQWdCLE9BQU8sV0FBVyxDQUFDLE9BQU8sR0FBRyxDQUFDLElBQUksRUFBRSxDQUFDLEdBQUcsQ0FBQyxJQUFJLEdBQUcsSUFBSSxDQUFDLGdCQUFnQixPQUFPLFdBQzFGLENB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
