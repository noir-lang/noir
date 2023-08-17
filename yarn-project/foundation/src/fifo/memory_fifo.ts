import { createDebugLogger } from '../log/index.js';

/**
 * A simple fifo queue. It can grow unbounded. It can have multiple producers and consumers.
 * Putting an item onto the queue always succeeds, unless either end() or cancel() has been called in which case
 * the item being pushed is simply discarded.
 */
export class MemoryFifo<T> {
  private waiting: ((item: T | null) => void)[] = [];
  private items: T[] = [];
  private flushing = false;

  constructor(private log = createDebugLogger('aztec:foundation:memory_fifo')) {}

  /**
   * Returns the current number of items in the queue.
   * The length represents the size of the queue at the time of invocation and may change as new items are added or consumed.
   *
   * @returns The number of items in the queue.
   */
  public length() {
    return this.items.length;
  }

  /**
   * Returns next item within the queue, or blocks until and item has been put into the queue.
   * If given a timeout, the promise will reject if no item is received after `timeout` seconds.
   * If the queue is flushing, `null` is returned.
   * @param timeout - The timeout in seconds.
   * @returns A result promise.
   */
  public get(timeout?: number): Promise<T | null> {
    if (this.items.length) {
      return Promise.resolve(this.items.shift()!);
    }

    if (this.items.length === 0 && this.flushing) {
      return Promise.resolve(null);
    }

    return new Promise<T | null>((resolve, reject) => {
      this.waiting.push(resolve);

      if (timeout) {
        setTimeout(() => {
          const index = this.waiting.findIndex(r => r === resolve);
          if (index > -1) {
            this.waiting.splice(index, 1);
            const err = new Error('Timeout getting item from queue.');
            reject(err);
          }
        }, timeout * 1000);
      }
    });
  }

  /**
   * Put an item onto back of the queue.
   * @param item - The item to enqueue.
   */
  public put(item: T) {
    if (this.flushing) {
      return;
    } else if (this.waiting.length) {
      this.waiting.shift()!(item);
    } else {
      this.items.push(item);
    }
  }

  /**
   * Once ended, no further items are added to queue. Consumers will consume remaining items within the queue.
   * The queue is not reusable after calling `end()`.
   * Any consumers waiting for an item receive null.
   */
  public end() {
    this.flushing = true;
    this.waiting.forEach(resolve => resolve(null));
  }

  /**
   * Once cancelled, all items are discarded from the queue, and no further items are added to the queue.
   * The queue is not reusable after calling `cancel()`.
   * Any consumers waiting for an item receive null.
   */
  public cancel() {
    this.flushing = true;
    this.items = [];
    this.waiting.forEach(resolve => resolve(null));
  }

  /**
   * Process items from the queue using a provided handler function.
   * The function iterates over items in the queue, invoking the handler for each item until the queue is empty or flushing.
   * If the handler throws an error, it will be caught and logged as 'Queue handler exception:', but the iteration will continue.
   * The process function returns a promise that resolves when there are no more items in the queue or the queue is flushing.
   *
   * @param handler - A function that takes an item of type T and returns a Promise<void> after processing the item.
   * @returns A Promise<void> that resolves when the queue is finished processing.
   */
  public async process(handler: (item: T) => Promise<void>) {
    try {
      while (true) {
        const item = await this.get();
        if (item === null) {
          break;
        }
        await handler(item);
      }
    } catch (err) {
      this.log.error('Queue handler exception:', err);
    }
  }
}
