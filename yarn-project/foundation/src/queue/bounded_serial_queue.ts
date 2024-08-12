import { createDebugLogger } from '../log/index.js';
import { Semaphore } from './semaphore.js';
import { SerialQueue } from './serial_queue.js';

/**
 * Leverages the unbounded SerialQueue and Semaphore to create a SerialQueue that will block when putting an item
 * if the queue size = maxQueueSize.
 */
export class BoundedSerialQueue {
  private readonly queue = new SerialQueue();
  private semaphore: Semaphore;

  constructor(maxQueueSize: number, private log = createDebugLogger('aztec:foundation:bounded_serial_queue')) {
    this.semaphore = new Semaphore(maxQueueSize);
  }

  /**
   * Initializes the underlying SerialQueue instance, allowing items to be processed from the queue.
   * The start method should be called before using the BoundedSerialQueue to ensure proper functionality.
   */
  public start() {
    this.queue.start();
  }

  /**
   * Returns the current number of items in the queue.
   * This is useful for monitoring the size of BoundedSerialQueue and understanding its utilization.
   *
   * @returns The length of the queue as an integer value.
   */
  public length() {
    return this.queue.length();
  }

  /**
   * Cancels the current operation in the SerialQueue, if any, and clears the queue.
   * Any pending tasks in the queue will not be executed, and the queue will be emptied.
   * This method is useful for cleaning up resources and stopping ongoing processes when they are no longer needed.
   * @returns A promise, resolved once cancelled.
   */
  public cancel() {
    return this.queue.cancel();
  }

  /**
   * Ends the queue processing gracefully, preventing new items from being added.
   * The currently executing item, if any, will complete and remaining queued items
   * will be processed in order. Once all items have been processed, the queue becomes
   * permanently unusable.
   *
   * @returns A promise that resolves when all items in the queue have been processed.
   */
  public end() {
    return this.queue.end();
  }

  /**
   * The caller will block until fn is successfully enqueued.
   * The fn itself is execute asynchronously and its result discarded.
   * TODO(AD) do we need this if we have exec()?
   * @param fn - The function to call once unblocked.
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
        this.log.error('BoundedSerialQueue handler exception:', err);
      });
  }

  /**
   * The caller will block until fn is successfully executed, and it's result returned.
   * @param fn - The function.
   * @returns A promise that resolves with the result once executed.
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

  /**
   * Awaiting this ensures the queue is empty before resuming.
   */
  public async syncPoint() {
    await this.queue.syncPoint();
  }
}
