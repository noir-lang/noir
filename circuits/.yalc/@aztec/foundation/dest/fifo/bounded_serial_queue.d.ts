/**
 * Leverages the unbounded SerialQueue and Semaphore to create a SerialQueue that will block when putting an item
 * if the queue size = maxQueueSize.
 */
export declare class BoundedSerialQueue {
    private readonly queue;
    private semaphore;
    constructor(maxQueueSize: number);
    start(): void;
    length(): number;
    cancel(): Promise<void>;
    end(): Promise<void>;
    /**
     * The caller will block until fn is succesfully enqueued.
     * The fn itself is execute asyncronously and its result discarded.
     */
    put(fn: () => Promise<void>): Promise<void>;
    /**
     * The caller will block until fn is successfully executed, and it's result returned.
     */
    exec<T>(fn: () => Promise<T>): Promise<T>;
    syncPoint(): Promise<void>;
}
//# sourceMappingURL=bounded_serial_queue.d.ts.map