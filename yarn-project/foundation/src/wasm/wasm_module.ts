import { Buffer } from 'buffer';

import { randomBytes } from '../crypto/index.js';
import { type LogFn, createDebugOnlyLogger } from '../log/index.js';
import { FifoMemoryQueue } from '../queue/index.js';
import { getEmptyWasiSdk } from './empty_wasi_sdk.js';

/**
 * The base shape of a WASM module providing low level memory and synchronous call access.
 * Note that the Aztec codebase used to support asyncify but has fully moved away,
 * using web workers if needed.
 */
export interface IWasmModule {
  /**
   * Low level call function, returns result as number.
   * @param name - The function name.
   * @param args - The function args.
   * @returns The value as an integer (could be pointer).
   */
  call(name: string, ...args: any): number;

  /**
   * Get a slice of memory between two addresses.
   * @param start - The start address.
   * @param end - The end address.
   * @returns A Uint8Array view of memory.
   */
  getMemorySlice(start: number, end: number): Uint8Array;

  /**
   * Write data into the heap.
   * @param offset - The address to write data at.
   * @param arr - The data to write.
   */
  writeMemory(offset: number, arr: Uint8Array): void;
}

/**
 * WasmModule:
 *  Helper over a webassembly module.
 *  Assumes a few quirks.
 *  1) the module expects wasi_snapshot_preview1 with the methods from getEmptyWasiSdk
 *  2) of which the webassembly
 *  we instantiate only uses random_get (update this if more WASI sdk methods are needed).
 */
export class WasmModule implements IWasmModule {
  private memory!: WebAssembly.Memory;
  private heap!: Uint8Array;
  private instance?: WebAssembly.Instance;
  private mutexQ = new FifoMemoryQueue<boolean>();
  private debug: LogFn;

  /**
   * Create a wasm module. Should be followed by await init();.
   * @param module - The module as a WebAssembly.Module or a Buffer.
   * @param importFn - Imports expected by the WASM.
   * @param loggerName - Optional, for debug logging.
   */
  constructor(
    private module: WebAssembly.Module | Buffer,
    private importFn: (module: WasmModule) => any,
    loggerName = 'aztec:wasm',
  ) {
    this.debug = createDebugOnlyLogger(loggerName);
    this.mutexQ.put(true);
  }

  /**
   * Return the wasm source.
   * @returns The source.
   */
  public getModule(): WebAssembly.Module | Buffer {
    return this.module;
  }
  /**
   * Initialize this wasm module.
   * @param wasmImportEnv - Linked to a module called "env". Functions implementations referenced from e.g. C++.
   * @param initial - 30 pages by default. 30*2**16 \> 1mb stack size plus other overheads.
   * @param initMethod - Defaults to calling '_initialize'.
   * @param maximum - 8192 maximum by default. 512mb.
   */
  public async init(initial = 30, maximum = 8192, initMethod: string | null = '_initialize') {
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

    // Init all global/static data.
    if (initMethod) {
      this.call(initMethod);
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
  public addLogger(logger: LogFn) {
    const oldDebug = this.debug;
    this.debug = (msg: string) => {
      logger(msg);
      oldDebug(msg);
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
      const message = `WASM function ${name} aborted, error: ${err}\n${err.stack}`;
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
  public getMemorySlice(start: number, end: number): Uint8Array {
    return this.getMemory().slice(start, end);
  }

  /**
   * Write data into the heap.
   * @param offset - The address to write data at.
   * @param arr - The data to write.
   */
  public writeMemory(offset: number, arr: Uint8Array) {
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
  public getMemoryAsString(addr: number) {
    addr = addr >>> 0;
    const m = this.getMemory();
    let i = addr;
    while (m[i] !== 0) {
      ++i;
    }
    return Buffer.from(m.slice(addr, i)).toString('ascii');
  }

  /**
   * When calling the wasm, sometimes a caller will require exclusive access over a series of calls.
   * E.g. When a result is written to address 0, one cannot have another caller writing to the same address via
   * writeMemory before the result is read via sliceMemory.
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
