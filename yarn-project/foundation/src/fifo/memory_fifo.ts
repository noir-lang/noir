/**
 * A simple fifo queue. It can grow unbounded. It can have multiple producers and consumers.
 * Putting an item onto the queue always succeeds, unless either end() or cancel() has been called in which case
 * the item being pushed is simply discarded.
 */
export class MemoryFifo<T> {
  private waiting: ((item: T | null) => void)[] = [];
  private items: T[] = [];
  private flushing = false;

  public length() {
    return this.items.length;
  }

  /**
   * Returns next item within the queue, or blocks until and item has been put into the queue.
   * If given a timeout, the promise will reject if no item is received after `timeout` seconds.
   * If the queue is flushing, `null` is returned.
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
   * Helper method that can be used to continously consume and process items on the queue.
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
      console.error('Queue handler exception:', err);
    }
  }
}
