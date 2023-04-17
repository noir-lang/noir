import { createDebugLogger } from '@aztec/log';
const debug = createDebugLogger('bb:worker_pool');
/**
 * Allocates a pool of WasmWorker's.
 * Worker 0 is allocated MAX_PAGES memory pages. This is because worker 0 will need to hold the proving key
 * (i.e. Has state), whereas the others are pure compute (they hold a little crs state).
 */
export class WorkerPool {
    constructor() {
        /**
         * The workers in the pool.
         */
        this.workers = [];
    }
    /**
     * Create an instance and initialize the workers.
     * @param createWorker - Worker factory.
     * @param poolSize - Pool size.
     * @returns An initialized WorkerPool.
     */
    static async new(createWorker, poolSize) {
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
    async init(createWorker, poolSize, maxMem = WorkerPool.MAX_PAGES) {
        debug(`creating ${poolSize} workers...`);
        const start = new Date().getTime();
        this.workers = await Promise.all(Array(poolSize)
            .fill(0)
            .map((_, i) => createWorker(`${i}`, i === 0 ? Math.min(WorkerPool.MAX_PAGES, maxMem) : 768, maxMem)));
        debug(`created workers: ${new Date().getTime() - start}ms`);
    }
    /**
     * Tell all workers in the pool to stop processing.
     */
    async destroy() {
        await Promise.all(this.workers.map(w => w.destroyWorker()));
    }
}
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
WorkerPool.MAX_PAGES = 8192;
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid29ya2VyX3Bvb2wuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi9zcmMvd29ya2VyL3dvcmtlcl9wb29sLnRzIl0sIm5hbWVzIjpbXSwibWFwcGluZ3MiOiJBQUFBLE9BQU8sRUFBRSxpQkFBaUIsRUFBRSxNQUFNLFlBQVksQ0FBQztBQUcvQyxNQUFNLEtBQUssR0FBRyxpQkFBaUIsQ0FBQyxnQkFBZ0IsQ0FBQyxDQUFDO0FBT2xEOzs7O0dBSUc7QUFDSCxNQUFNLE9BQU8sVUFBVTtJQUF2QjtRQWdCRTs7V0FFRztRQUNLLFlBQU8sR0FBaUIsRUFBRSxDQUFDO0lBc0NyQyxDQUFDO0lBcENDOzs7OztPQUtHO0lBQ0gsTUFBTSxDQUFDLEtBQUssQ0FBQyxHQUFHLENBQUMsWUFBMEIsRUFBRSxRQUFnQjtRQUMzRCxNQUFNLElBQUksR0FBRyxJQUFJLFVBQVUsRUFBRSxDQUFDO1FBQzlCLE1BQU0sSUFBSSxDQUFDLElBQUksQ0FBQyxZQUFZLEVBQUUsUUFBUSxDQUFDLENBQUM7UUFDeEMsT0FBTyxJQUFJLENBQUM7SUFDZCxDQUFDO0lBRUQ7Ozs7O09BS0c7SUFDSSxLQUFLLENBQUMsSUFBSSxDQUFDLFlBQTBCLEVBQUUsUUFBZ0IsRUFBRSxNQUFNLEdBQUcsVUFBVSxDQUFDLFNBQVM7UUFDM0YsS0FBSyxDQUFDLFlBQVksUUFBUSxhQUFhLENBQUMsQ0FBQztRQUN6QyxNQUFNLEtBQUssR0FBRyxJQUFJLElBQUksRUFBRSxDQUFDLE9BQU8sRUFBRSxDQUFDO1FBQ25DLElBQUksQ0FBQyxPQUFPLEdBQUcsTUFBTSxPQUFPLENBQUMsR0FBRyxDQUM5QixLQUFLLENBQUMsUUFBUSxDQUFDO2FBQ1osSUFBSSxDQUFDLENBQUMsQ0FBQzthQUNQLEdBQUcsQ0FBQyxDQUFDLENBQUMsRUFBRSxDQUFDLEVBQUUsRUFBRSxDQUFDLFlBQVksQ0FBQyxHQUFHLENBQUMsRUFBRSxFQUFFLENBQUMsS0FBSyxDQUFDLENBQUMsQ0FBQyxDQUFDLElBQUksQ0FBQyxHQUFHLENBQUMsVUFBVSxDQUFDLFNBQVMsRUFBRSxNQUFNLENBQUMsQ0FBQyxDQUFDLENBQUMsR0FBRyxFQUFFLE1BQU0sQ0FBQyxDQUFDLENBQ3ZHLENBQUM7UUFFRixLQUFLLENBQUMsb0JBQW9CLElBQUksSUFBSSxFQUFFLENBQUMsT0FBTyxFQUFFLEdBQUcsS0FBSyxJQUFJLENBQUMsQ0FBQztJQUM5RCxDQUFDO0lBRUQ7O09BRUc7SUFDSSxLQUFLLENBQUMsT0FBTztRQUNsQixNQUFNLE9BQU8sQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLE9BQU8sQ0FBQyxHQUFHLENBQUMsQ0FBQyxDQUFDLEVBQUUsQ0FBQyxDQUFDLENBQUMsYUFBYSxFQUFFLENBQUMsQ0FBQyxDQUFDO0lBQzlELENBQUM7O0FBdkRELHVEQUF1RDtBQUN2RCxLQUFLO0FBQ0wsc0hBQXNIO0FBQ3RILGdIQUFnSDtBQUNoSCxpSEFBaUg7QUFDakgscUdBQXFHO0FBQ3JHLDREQUE0RDtBQUM1RCxrR0FBa0c7QUFDbEcsMEZBQTBGO0FBQzFGLDZHQUE2RztBQUM3RyxrQ0FBa0M7QUFDbEM7O0dBRUc7QUFDVyxvQkFBUyxHQUFHLElBQUksQ0FBQyJ9