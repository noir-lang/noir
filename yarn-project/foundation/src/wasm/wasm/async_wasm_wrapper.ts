import { AsyncCallState, AsyncFnState, WasmModule, WasmWrapper } from '@aztec/foundation/wasm';

/**
 * A low-level wrapper for an instance of a WASM that uses AsyncCallState
 * to support awaitable calls from wasm to ts.
 */
export abstract class AsyncWasmWrapper extends WasmWrapper {
  protected asyncCallState = new AsyncCallState();

  /**
   * 20 pages by default. 20*2**16 \> 1mb stack size plus other overheads.
   * 8192 maximum by default. 512mb.
   * @param initial - Initial memory pages.
   * @param maximum - Max memory pages.
   * @returns The wrapper.
   */
  public async init(initial = 20, maximum = 8192): Promise<this> {
    await super.init(initial, maximum);
    this.asyncCallState.init(this.wasm);
    return this;
  }

  /**
   * These are functions implementations for imports we've defined are needed.
   * The native C++ build defines these in a module called "env". We must implement TypeScript versions here.
   * @param wasm - The wasm module.
   * @returns An object of functions called from cpp that need to be answered by ts.
   */
  protected getImportFns(wasm: WasmModule): any {
    return {
      /**
       * Read the data associated with the key located at keyAddr.
       * Malloc data within the WASM, copy the data into the WASM, and return the address to the caller.
       * The caller is responsible for taking ownership of (and freeing) the memory at the returned address.
       */
      // eslint-disable-next-line camelcase
      get_data: this.wrapAsyncImportFn(async (keyAddr: number, lengthOutAddr: number) => {
        const key = wasm.getMemoryAsString(keyAddr);
        wasm.getLogger()(`get_data: key: ${key}`);
        const data = await this.store.get(key);
        if (!data) {
          wasm.writeMemory(lengthOutAddr, numToUInt32LE(0));
          wasm.getLogger()(`get_data: no data found for: ${key}`);
          return 0;
        }
        const dataAddr = wasm.call('bbmalloc', data.length);
        wasm.writeMemory(lengthOutAddr, numToUInt32LE(data.length));
        wasm.writeMemory(dataAddr, data);
        wasm.getLogger()(`get_data: data at ${dataAddr} is ${data.length} bytes.`);
        return dataAddr;
      }),
      // eslint-disable-next-line camelcase
      set_data: this.wrapAsyncImportFn(async (keyAddr: number, dataAddr: number, dataLength: number) => {
        const key = wasm.getMemoryAsString(keyAddr);
        wasm.getLogger()(`set_data: key: ${key} addr: ${dataAddr} length: ${dataLength}`);
        await this.store.set(key, Buffer.from(wasm.getMemorySlice(dataAddr, dataAddr + dataLength)));
      }),
    };
  }

  /**
   * Wrap an async import funtion.
   * @param fn - The function.
   * @returns The AsyncCallState-adapted function.
   */
  protected wrapAsyncImportFn(fn: (...args: number[]) => Promise<number | void>) {
    return this.asyncCallState.wrapImportFn((state: AsyncFnState, ...args: number[]) => {
      if (!state.continuation) {
        return fn(...args);
      }
      return state.result;
    });
  }

  /**
   * Uses asyncify to enable async callbacks into js.
   * @see https://kripken.github.io/blog/wasm/2019/07/16/asyncify.html
   * @param name - The method name.
   * @param args - The arguments to the method.
   * @returns The numeric integer or address result.
   */
  public async asyncCall(name: string, ...args: any): Promise<number> {
    return await this.asyncCallState.call(name, ...args);
  }
}

/**
 * Convert a number to a little-endian uint32 buffer.
 * @param n - The number to convert.
 * @param bufferSize - Size of the returned buffer.
 * @returns Resulting buffer.
 * TODO: REFACTOR: Copied from bb serialize, move to a set of serialization fns here in foundation.
 */
function numToUInt32LE(n: number, bufferSize = 4) {
  const buf = Buffer.alloc(bufferSize);
  buf.writeUInt32LE(n, bufferSize - 4);
  return buf;
}
