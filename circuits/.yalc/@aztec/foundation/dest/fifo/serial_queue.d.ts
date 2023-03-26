/**
 * A more specialised fifo queue that enqueues functions to execute. Enqueued functions are executed in serial.
 */
export declare class SerialQueue {
    private readonly queue;
    private runningPromise;
    start(): void;
    length(): number;
    cancel(): Promise<void>;
    end(): Promise<void>;
    /**
     * Enqueues fn for execution on the serial queue.
     * Returns the result of the function after execution.
     */
    put<T>(fn: () => Promise<T>): Promise<T>;
    syncPoint(): Promise<void>;
}
//# sourceMappingURL=serial_queue.d.ts.map