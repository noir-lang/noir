import { NodeDataStore, WasmModule, WebDataStore } from '@aztec/foundation/wasm';

import isNode from 'detect-node';
import { readFile } from 'fs/promises';

/**
 * Get the WASM binary.
 * @param path - Path to the WASM binary.
 * @returns The binary buffer.
 */
export async function fetchCode(path: string) {
  if (isNode) {
    return await readFile(path);
  } else {
    const res = await fetch(path);
    return Buffer.from(await res.arrayBuffer());
  }
}

/**
 * A low-level wrapper for an instance of a WASM.
 */
export abstract class WasmWrapper {
  protected store = isNode ? new NodeDataStore() : new WebDataStore();
  protected wasm!: WasmModule;

  abstract codePath: string;

  constructor(private loggerName?: string) {}

  /**
   * 30 pages by default. 30*2**16 \> 1mb stack size plus other overheads.
   * 8192 maximum by default. 512mb.
   * @param initial - Initial memory pages.
   * @param maximum - Max memory pages.
   * @returns The original instance of the wrapper.
   */
  public async init(initial = 30, maximum = 8192): Promise<WasmWrapper> {
    let wasm: WasmModule;
    this.wasm = wasm = new WasmModule(
      await fetchCode(this.codePath),
      module => ({
        /**
         * Log a string from wasm.
         * @param addr - The string address to log.
         */
        logstr(addr: number) {
          const str = wasm.getMemoryAsString(addr);
          const m = wasm.getMemory();
          const str2 = `${str} (mem: ${(m.length / (1024 * 1024)).toFixed(2)}MB)`;
          wasm.getLogger()(str2);
        },
        memory: module.getRawMemory(),
        ...this.getImportFns(module),
      }),
      this.loggerName,
    );
    await wasm.init(initial, maximum);
    return this;
  }

  /**
   * These are functions implementations for imports we've defined are needed.
   * The native C++ build defines these in a module called "env". We must implement TypeScript versions here.
   * @param module - The wasm module.
   * @returns An object of functions called from cpp that need to be answered by ts.
   */
  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  protected getImportFns(module: WasmModule): any {
    return {};
  }

  /**
   * Get a slice of memory between two addresses.
   * @param start - The start address.
   * @param end - The end address.
   * @returns A Uint8Array view of memory.
   */
  public getMemorySlice(start: number, end: number) {
    return this.wasm.getMemorySlice(start, end);
  }

  /**
   * Write data into the heap.
   * @param arr - The data to write.
   * @param offset - The address to write data at.
   */
  public writeMemory(offset: number, arr: Uint8Array) {
    this.wasm.writeMemory(offset, arr);
  }

  /**
   * Get memory as string.
   * @param offset - The address to get null-terminated string data from.
   * @returns JS string.
   */
  public getMemoryAsString(offset: number) {
    return this.wasm.getMemoryAsString(offset);
  }

  /**
   * Calls into the WebAssembly.
   * @param name - The method name.
   * @param args - The arguments to the method.
   * @returns The numeric integer or address result.
   */
  public call(name: string, ...args: any): number {
    return this.wasm.call(name, ...args);
  }
}
