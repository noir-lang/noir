import { createDebugLogger, DebugLogger } from '@aztec/log';
import { Buffer } from 'buffer';
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
  private memory!: WebAssembly.Memory;
  private heap!: Uint8Array;
  private instance?: WebAssembly.Instance;
  private mutexQ = new MemoryFifo<boolean>();
  private debug: DebugLogger;

  /**
   * Create a wasm module. Should be followed by await init();.
   * @param module - The module as a WebAssembly.Module or a Buffer.
   * @param importFn - Imports expected by the WASM.
   * @param loggerName - Optional, for debug logging.
   */
  constructor(
    private module: WebAssembly.Module | Buffer,
    private importFn: (module: WasmModule) => any,
    loggerName = 'wasm-worker',
  ) {
    this.debug = createDebugLogger(loggerName);
    this.mutexQ.put(true);
  }

  /**
   * Initialize this wasm module.
   * @param wasmImportEnv - Linked to a module called "env". Functions implementations referenced from e.g. C++.
   * @param initial - 20 pages by default. 20*2**16 \> 1mb stack size plus other overheads.
   * @param maximum - 8192 maximum by default. 512mb.
   */
  public async init(initial = 20, maximum = 8192) {
    this.debug(
      `initial mem: ${initial} pages, ${(initial * 2 ** 16) / (1024 * 1024)}mb. max mem: ${maximum} pages, ${
        (maximum * 2 ** 16) / (1024 * 1024)
      }mb`,
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
        random_get: (arr: number, length: number) => {
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
      const { instance } = await WebAssembly.instantiate(this.module, importObj);
      this.instance = instance;
    }
  }

  /**
   * The methods or objects exported by the WASM module.
   * @returns An indexable object.
   */
  public exports(): any {
    if (!this.instance) {
      throw new Error('WasmModule: not initialized!');
    }
    return this.instance.exports;
  }

  /**
   * Get the current logger.
   * @returns Logging function.
   */
  public getLogger() {
    return this.debug;
  }

  /**
   * Add a logger.
   * @param logger - Function to call when logging.
   */
  public addLogger(logger: DebugLogger) {
    const oldDebug = this.debug;
    this.debug = (...args: any[]) => {
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
  public call(name: string, ...args: any): number {
    if (!this.exports()[name]) {
      throw new Error(`WASM function ${name} not found.`);
    }
    try {
      // When returning values from the WASM, use >>> operator to convert
      // signed representation to unsigned representation.
      return this.exports()[name](...args) >>> 0;
    } catch (err: any) {
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
  public getRawMemory(): WebAssembly.Memory {
    return this.memory;
  }
  /**
   * Get the memory used by the WASM module, as a byte array.
   * @returns A Uint8Array view of the WASM module memory.
   */
  public getMemory(): Uint8Array {
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
  public memSize(): number {
    return this.getMemory().length;
  }

  /**
   * Get a slice of memory between two addresses.
   * @param start - The start address.
   * @param end - The end address.
   * @returns A Uint8Array view of memory.
   */
  public getMemorySlice(start: number, end: number) {
    return this.getMemory().slice(start, end);
  }

  /**
   * Write data into the heap.
   * @param arr - The data to write.
   * @param offset - The address to write data at.
   */
  public transferToHeap(arr: Uint8Array, offset: number) {
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
  public async acquire() {
    await this.mutexQ.get();
  }

  /**
   * Release the mutex, letting another promise call acquire().
   */
  public release() {
    if (this.mutexQ.length() !== 0) {
      throw new Error('Release called but not acquired.');
    }
    this.mutexQ.put(true);
  }
}
