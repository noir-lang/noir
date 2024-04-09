import { createDebugLogger } from '../log/index.js';
import { type WasmWorker } from './wasm_worker.js';

const log = createDebugLogger('bb:worker_pool');

/**
 * Type of a worker factory.
 * Used to customize WorkerPool worker construction.
 */
export type CreateWorker = (name: string, minMem: number, maxMem: number) => WasmWorker;
/**
 * Allocates a pool of WasmWorker's.
 * Worker 0 is allocated MAX_PAGES memory pages. This is because worker 0 will need to hold the proving key
 * (i.e. Has state), whereas the others are pure compute (they hold a little crs state).
 */
export class WorkerPool {
  // TODO(AD): Revisit what this means in aztec 3 context
  // --
  // Introduction of low mem prover work (polynomial cache) may actually increase mem usage when the backing store isn't
  // enabled. We were seeing intermittent failings related to memory in production for some users when limiting to
  // 6660 (416MB). It would be nice to understand why this is (the non determinism and/or the increased mem usage).
  // For now, increasing mem usage to 512MB. This maybe preferable to backing out the low mem work, but
  // ironically may break the chance of us using it in mobile.
  // We *could* enable the low memory backing store, but this needs a little bit of work to actually
  // read/write from indexeddb, performance testing, and actual further memory load testing.
  // At this point it's hard to know what our memory savings would be relative to just fully reverting the LMP.
  // public static MAX_PAGES = 6660;
  /**
   * The maximum number of memory pages to be used by the webassembly.
   */
  public static MAX_PAGES = 8192;
  /**
   * The workers in the pool.
   */
  private workers: WasmWorker[] = [];

  /**
   * Create an instance and initialize the workers.
   * @param createWorker - Worker factory.
   * @param poolSize - Pool size.
   * @returns An initialized WorkerPool.
   */
  static async new(createWorker: CreateWorker, poolSize: number) {
    const pool = new WorkerPool();
    await pool.init(createWorker, poolSize);
    return pool;
  }

  /**
   * Initialize the workers.
   * @param createWorker - Worker factory().
   * @param poolSize - Pool size.
   * @param maxMem - Max memory pages.
   */
  public async init(createWorker: CreateWorker, poolSize: number, maxMem = WorkerPool.MAX_PAGES) {
    log.debug(`creating ${poolSize} workers...`);
    const start = new Date().getTime();
    this.workers = await Promise.all(
      Array(poolSize)
        .fill(0)
        .map((_, i) => createWorker(`${i}`, i === 0 ? Math.min(WorkerPool.MAX_PAGES, maxMem) : 768, maxMem)),
    );

    log.debug(`created workers: ${new Date().getTime() - start}ms`);
  }

  /**
   * Tell all workers in the pool to stop processing.
   */
  public async destroy() {
    await Promise.all(this.workers.map(w => w.destroyWorker()));
  }
}
