import { WasmWorker } from './wasm_worker.js';
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
export declare class WorkerPool {
    /**
     * The maximum number of memory pages to be used by the webassembly.
     */
    static MAX_PAGES: number;
    /**
     * The workers in the pool.
     */
    private workers;
    /**
     * Create an instance and initialize the workers.
     * @param createWorker - Worker factory.
     * @param poolSize - Pool size.
     * @returns An initialized WorkerPool.
     */
    static new(createWorker: CreateWorker, poolSize: number): Promise<WorkerPool>;
    /**
     * Initialize the workers.
     * @param createWorker - Worker factory().
     * @param poolSize - Pool size.
     * @param maxMem - Max memory pages.
     */
    init(createWorker: CreateWorker, poolSize: number, maxMem?: number): Promise<void>;
    /**
     * Tell all workers in the pool to stop processing.
     */
    destroy(): Promise<void>;
}
//# sourceMappingURL=worker_pool.d.ts.map