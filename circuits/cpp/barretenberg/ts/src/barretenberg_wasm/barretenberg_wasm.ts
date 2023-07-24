import { type Worker } from 'worker_threads';
import { EventEmitter } from 'events';
import createDebug from 'debug';
import { Remote, proxy } from 'comlink';
import { randomBytes } from '../random/index.js';
import {
  fetchCode,
  getNumCpu,
  createWorker,
  getRemoteBarretenbergWasm,
  threadLogger,
  killSelf,
} from 'dynamic/barretenberg_wasm';

const debug = createDebug('bb.js:wasm');

EventEmitter.defaultMaxListeners = 30;

export class BarretenbergWasm {
  static MAX_THREADS = 32;
  private memStore: { [key: string]: Uint8Array } = {};
  private memory!: WebAssembly.Memory;
  private instance!: WebAssembly.Instance;
  private workers: Worker[] = [];
  private remoteWasms: BarretenbergWasmWorker[] = [];
  private nextWorker = 0;
  private nextThreadId = 1;
  private isThread = false;
  private logger: (msg: string) => void = debug;

  public static async new() {
    const barretenberg = new BarretenbergWasm();
    await barretenberg.init(1);
    return barretenberg;
  }

  /**
   * Construct and initialise BarretenbergWasm within a Worker. Return both the worker and the wasm proxy.
   * Used when running in the browser, because we can't block the main thread.
   */
  public static async newWorker(threads?: number) {
    const worker = createWorker();
    const wasm = getRemoteBarretenbergWasm(worker);
    await wasm.init(threads, proxy(debug));
    return { worker, wasm };
  }

  public getNumThreads() {
    return this.workers.length + 1;
  }

  /**
   * Init as main thread. Spawn child threads.
   */
  public async init(
    threads = Math.min(getNumCpu(), BarretenbergWasm.MAX_THREADS),
    logger: (msg: string) => void = debug,
    initial = 25,
    maximum = 2 ** 16,
  ) {
    this.logger = logger;

    const initialMb = (initial * 2 ** 16) / (1024 * 1024);
    const maxMb = (maximum * 2 ** 16) / (1024 * 1024);
    this.logger(
      `initial mem: ${initial} pages, ${initialMb}MiB. ` +
        `max mem: ${maximum} pages, ${maxMb}MiB. ` +
        `threads: ${threads}`,
    );

    this.memory = new WebAssembly.Memory({ initial, maximum, shared: threads > 1 });

    // Annoyingly the wasm declares if it's memory is shared or not. So now we need two wasms if we want to be
    // able to fallback on "non shared memory" situations.
    const code = await fetchCode(threads > 1);
    const { instance, module } = await WebAssembly.instantiate(code, this.getImportObj(this.memory));

    this.instance = instance;

    // Init all global/static data.
    this.call('_initialize');

    // Create worker threads. Create 1 less than requested, as main thread counts as a thread.
    this.logger('creating worker threads...');
    this.workers = (await Promise.all(Array.from({ length: threads - 1 }).map(createWorker))) as any;
    this.remoteWasms = await Promise.all(this.workers.map(getRemoteBarretenbergWasm as any));
    await Promise.all(this.remoteWasms.map(w => w.initThread(module, this.memory)));
    this.logger('init complete.');
  }

  /**
   * Init as worker thread.
   */
  public async initThread(module: WebAssembly.Module, memory: WebAssembly.Memory) {
    this.isThread = true;
    this.logger = threadLogger() || this.logger;
    this.memory = memory;
    this.instance = await WebAssembly.instantiate(module, this.getImportObj(this.memory));
  }

  /**
   * Called on main thread. Signals child threads to gracefully exit.
   */
  public async destroy() {
    await Promise.all(this.workers.map(w => w.terminate()));
  }

  private getImportObj(memory: WebAssembly.Memory) {
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
      wasi: {
        'thread-spawn': (arg: number) => {
          arg = arg >>> 0;
          const id = this.nextThreadId++;
          const worker = this.nextWorker++ % this.remoteWasms.length;
          // this.logger(`spawning thread ${id} on worker ${worker} with arg ${arg >>> 0}`);
          this.remoteWasms[worker].call('wasi_thread_start', id, arg).catch(this.logger);
          // this.remoteWasms[worker].postMessage({ msg: 'thread', data: { id, arg } });
          return id;
        },
      },

      // These are functions implementations for imports we've defined are needed.
      // The native C++ build defines these in a module called "env". We must implement TypeScript versions here.
      env: {
        env_hardware_concurrency: () => {
          // If there are no workers (we're already running as a worker, or the main thread requested no workers)
          // then we return 1, which should cause any algos using threading to just not create a thread.
          return this.remoteWasms.length + 1;
        },
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
          this.memStore[key] = this.getMemorySlice(dataAddr, dataAddr + dataLength).slice();
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
      if (this.isThread) {
        killSelf();
      } else {
        throw err;
      }
    }
  }

  public memSize() {
    return this.getMemory().length;
  }

  public getMemorySlice(start: number, end?: number) {
    return this.getMemory().subarray(start, end);
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

export type BarretenbergWasmWorker = Remote<BarretenbergWasm>;
