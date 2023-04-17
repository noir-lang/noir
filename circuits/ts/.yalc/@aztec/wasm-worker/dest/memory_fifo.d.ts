/**
 * A simple fifo queue. It can grow unbounded. It can have multiple producers and consumers.
 * Putting an item onto the queue always succeeds, unless either end() or cancel() has been called in which case
 * the item being pushed is simply discarded.
 */
export declare class MemoryFifo<T> {
    private waiting;
    private items;
    private flushing;
    /**
     * Length of queue.
     * @returns integer.
     */
    length(): number;
    /**
     * Returns next item within the queue, or blocks until and item has been put into the queue.
     * If given a timeout, the promise will reject if no item is received after `timeout` seconds.
     * If the queue is flushing, `null` is returned.
     * @param timeout - In seconds.
     * @returns Promise of result.
     */
    get(timeout?: number): Promise<T | null>;
    /**
     * Put an item onto back of the queue.
     * @param item - The item to enqueue.
     */
    put(item: T): void;
    /**
     * Once ended, no further items are added to queue. Consumers will consume remaining items within the queue.
     * The queue is not reusable after calling `end()`.
     * Any consumers waiting for an item receive null.
     */
    end(): void;
    /**
     * Once cancelled, all items are discarded from the queue, and no further items are added to the queue.
     * The queue is not reusable after calling `cancel()`.
     * Any consumers waiting for an item receive null.
     */
    cancel(): void;
    /**
     * Helper method that can be used to continously consume and process items on the queue.
     * @param handler - The item handler function.
     */
    process(handler: (item: T) => Promise<void>): Promise<void>;
}
//# sourceMappingURL=memory_fifo.d.ts.map