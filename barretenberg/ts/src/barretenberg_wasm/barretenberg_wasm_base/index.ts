import createDebug from 'debug';
import { randomBytes } from '../../random/index.js';
import { killSelf } from '../helpers/index.js';

const debug = createDebug('bb.js:wasm');

/**
 * Base implementation of BarretenbergWasm.
 * Contains code that is common to the "main thread" implementation and the "child thread" implementation.
 */
export class BarretenbergWasmBase {
  protected memStore: { [key: string]: Uint8Array } = {};
  protected memory!: WebAssembly.Memory;
  protected instance!: WebAssembly.Instance;
  protected logger: (msg: string) => void = debug;

  protected getImportObj(memory: WebAssembly.Memory) {
    /* eslint-disable camelcase */
    const importObj = {
      // We need to implement a part of the wasi api:
      // https://github.com/WebAssembly/WASI/blob/main/phases/snapshot/docs.md
      // We literally only need to support random_get, everything else is noop implementated in barretenberg.wasm.
      wasi_snapshot_preview1: {
        random_get: (out: any, length: number) => {
          out = out >>> 0;
          const randomData = randomBytes(length);
          const mem = this.getMemory();
          mem.set(randomData, out);
        },
        clock_time_get: (a1: number, a2: number, out: number) => {
          out = out >>> 0;
          const ts = BigInt(new Date().getTime()) * 1000000n;
          const view = new DataView(this.getMemory().buffer);
          view.setBigUint64(out, ts, true);
        },
        proc_exit: () => {
          this.logger('PANIC: proc_exit was called. This is maybe caused by "joining" with unstable wasi pthreads.');
          this.logger(new Error().stack!);
          killSelf();
        },
      },

      // These are functions implementations for imports we've defined are needed.
      // The native C++ build defines these in a module called "env". We must implement TypeScript versions here.
      env: {
        /**
         * The 'info' call we use for logging in C++, calls this under the hood.
         * The native code will just print to std:err (to avoid std::cout which is used for IPC).
         * Here we just emit the log line for the client to decide what to do with.
         */
        logstr: (addr: number) => {
          const str = this.stringFromAddress(addr);
          const m = this.getMemory();
          const str2 = `${str} (mem: ${(m.length / (1024 * 1024)).toFixed(2)}MiB)`;
          this.logger(str2);
          if (str2.startsWith('WARNING:')) {
            this.logger(new Error().stack!);
          }
        },

        get_data: (keyAddr: number, outBufAddr: number) => {
          const key = this.stringFromAddress(keyAddr);
          outBufAddr = outBufAddr >>> 0;
          const data = this.memStore[key];
          if (!data) {
            this.logger(`get_data miss ${key}`);
            return;
          }
          // this.logger(`get_data hit ${key} size: ${data.length} dest: ${outBufAddr}`);
          // this.logger(Buffer.from(data.slice(0, 64)).toString('hex'));
          this.writeMemory(outBufAddr, data);
        },

        set_data: (keyAddr: number, dataAddr: number, dataLength: number) => {
          const key = this.stringFromAddress(keyAddr);
          dataAddr = dataAddr >>> 0;
          this.memStore[key] = this.getMemorySlice(dataAddr, dataAddr + dataLength);
          // this.logger(`set_data: ${key} length: ${dataLength}`);
        },

        memory,
      },
    };
    /* eslint-enable camelcase */

    return importObj;
  }

  public exports(): any {
    return this.instance.exports;
  }

  /**
   * When returning values from the WASM, use >>> operator to convert signed representation to unsigned representation.
   */
  public call(name: string, ...args: any) {
    if (!this.exports()[name]) {
      throw new Error(`WASM function ${name} not found.`);
    }
    try {
      return this.exports()[name](...args) >>> 0;
    } catch (err: any) {
      const message = `WASM function ${name} aborted, error: ${err}`;
      this.logger(message);
      this.logger(err.stack);
      throw err;
    }
  }

  public memSize() {
    return this.getMemory().length;
  }

  /**
   * Returns a copy of the data, not a view.
   */
  public getMemorySlice(start: number, end: number) {
    return this.getMemory().subarray(start, end).slice();
  }

  public writeMemory(offset: number, arr: Uint8Array) {
    const mem = this.getMemory();
    mem.set(arr, offset);
  }

  // PRIVATE METHODS

  private getMemory() {
    return new Uint8Array(this.memory.buffer);
  }

  private stringFromAddress(addr: number) {
    addr = addr >>> 0;
    const m = this.getMemory();
    let i = addr;
    for (; m[i] !== 0; ++i);
    const textDecoder = new TextDecoder('ascii');
    return textDecoder.decode(m.slice(addr, i));
  }
}
