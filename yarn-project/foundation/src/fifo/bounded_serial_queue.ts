import { Semaphore } from './semaphore.js';
import { SerialQueue } from './serial_queue.js';

/**
 * Leverages the unbounded SerialQueue and Semaphore to create a SerialQueue that will block when putting an item
 * if the queue size = maxQueueSize.
 */
export class BoundedSerialQueue {
  private readonly queue = new SerialQueue();
  private semaphore: Semaphore;

  constructor(maxQueueSize: number) {
    this.semaphore = new Semaphore(maxQueueSize);
  }

  public start() {
    this.queue.start();
  }

  public length() {
    return this.queue.length();
  }

  public cancel() {
    return this.queue.cancel();
  }

  public end() {
    return this.queue.end();
  }

  /**
   * The caller will block until fn is succesfully enqueued.
   * The fn itself is execute asyncronously and its result discarded.
   */
  public async put(fn: () => Promise<void>): Promise<void> {
    await this.semaphore.acquire();
    this.queue
      .put(async () => {
        try {
          await fn();
        } finally {
          this.semaphore.release();
        }
      })
      .catch(err => {
        console.error('BoundedSerialQueue handler exception:', err);
      });
  }

  /**
   * The caller will block until fn is successfully executed, and it's result returned.
   */
  public async exec<T>(fn: () => Promise<T>): Promise<T> {
    await this.semaphore.acquire();
    return this.queue.put(async () => {
      try {
        return await fn();
      } finally {
        this.semaphore.release();
      }
    });
  }

  // Awaiting this ensures the queue is empty before resuming.
  public async syncPoint() {
    await this.queue.syncPoint();
  }
}
