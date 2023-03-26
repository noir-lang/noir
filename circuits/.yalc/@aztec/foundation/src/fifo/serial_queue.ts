import { MemoryFifo } from './memory_fifo.js';

/**
 * A more specialised fifo queue that enqueues functions to execute. Enqueued functions are executed in serial.
 */
export class SerialQueue {
  private readonly queue = new MemoryFifo<() => Promise<void>>();
  private runningPromise!: Promise<void>;

  public start() {
    this.runningPromise = this.queue.process(fn => fn());
  }

  public length() {
    return this.queue.length();
  }

  public cancel() {
    this.queue.cancel();
    return this.runningPromise;
  }

  public end() {
    this.queue.end();
    return this.runningPromise;
  }

  /**
   * Enqueues fn for execution on the serial queue.
   * Returns the result of the function after execution.
   */
  public put<T>(fn: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      this.queue.put(async () => {
        try {
          const res = await fn();
          resolve(res);
        } catch (e) {
          reject(e);
        }
      });
    });
  }

  // Awaiting this ensures the queue is empty before resuming.
  public async syncPoint() {
    await this.put(async () => {});
  }
}
