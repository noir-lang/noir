import { type Worker } from 'worker_threads';
import createDebug from 'debug';
import { Remote } from 'comlink';
import { getNumCpu, getRemoteBarretenbergWasm, getSharedMemoryAvailable } from '../helpers/index.js';
import { fetchCode } from '../fetch_code/index.js';
import { createThreadWorker } from '../barretenberg_wasm_thread/factory/node/index.js';
import { type BarretenbergWasmThreadWorker } from '../barretenberg_wasm_thread/index.js';
import { BarretenbergWasmBase } from '../barretenberg_wasm_base/index.js';

const debug = createDebug('bb.js:wasm');

export class BarretenbergWasmMain extends BarretenbergWasmBase {
  static MAX_THREADS = 32;
  private workers: Worker[] = [];
  private remoteWasms: BarretenbergWasmThreadWorker[] = [];
  private nextWorker = 0;
  private nextThreadId = 1;

  public getNumThreads() {
    return this.workers.length + 1;
  }

  /**
   * Init as main thread. Spawn child threads.
   */
  public async init(
    threads = Math.min(getNumCpu(), BarretenbergWasmMain.MAX_THREADS),
    logger: (msg: string) => void = debug,
    initial = 25,
    maximum = 2 ** 16,
  ) {
    this.logger = logger;

    const initialMb = (initial * 2 ** 16) / (1024 * 1024);
    const maxMb = (maximum * 2 ** 16) / (1024 * 1024);
    const shared = getSharedMemoryAvailable();

    if (!shared) {
      threads = 1;
    }

    this.logger(
      `initial mem: ${initial} pages, ${initialMb}MiB. ` +
        `max mem: ${maximum} pages, ${maxMb}MiB. ` +
        `threads: ${threads}, shared: ${shared}`,
    );

    this.memory = new WebAssembly.Memory({ initial, maximum, shared });

    const code = await fetchCode(shared);
    const { instance, module } = await WebAssembly.instantiate(code, this.getImportObj(this.memory));

    this.instance = instance;

    // Init all global/static data.
    this.call('_initialize');

    // Create worker threads. Create 1 less than requested, as main thread counts as a thread.
    if (threads > 1) {
      this.logger(`creating ${threads} worker threads...`);
      this.workers = await Promise.all(Array.from({ length: threads - 1 }).map(createThreadWorker));
      this.remoteWasms = await Promise.all(this.workers.map(getRemoteBarretenbergWasm<BarretenbergWasmThreadWorker>));
      await Promise.all(this.remoteWasms.map(w => w.initThread(module, this.memory)));
    }
    this.logger('init complete.');
  }

  /**
   * Called on main thread. Signals child threads to gracefully exit.
   */
  public async destroy() {
    await Promise.all(this.workers.map(w => w.terminate()));
  }

  protected getImportObj(memory: WebAssembly.Memory) {
    const baseImports = super.getImportObj(memory);

    /* eslint-disable camelcase */
    return {
      ...baseImports,
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
      env: {
        ...baseImports.env,
        env_hardware_concurrency: () => {
          // If there are no workers (we're already running as a worker, or the main thread requested no workers)
          // then we return 1, which should cause any algos using threading to just not create a thread.
          return this.remoteWasms.length + 1;
        },
      },
    };
    /* eslint-enable camelcase */
  }
}

export type BarretenbergWasmMainWorker = Remote<BarretenbergWasmMain>;
