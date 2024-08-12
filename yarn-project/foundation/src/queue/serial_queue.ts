import { FifoMemoryQueue } from './fifo_memory_queue.js';

/**
 * A more specialized fifo queue that enqueues functions to execute. Enqueued functions are executed in serial.
 */
export class SerialQueue {
  private readonly queue = new FifoMemoryQueue<() => Promise<void>>();
  private runningPromise!: Promise<void>;

  /**
   * Initializes the execution of enqueued functions in the serial queue.
   * Functions are executed in the order they were added to the queue, with each function
   * waiting for the completion of the previous one before starting its execution.
   * This method should be called once to start processing the queue.
   */
  public start() {
    this.runningPromise = this.queue.process(fn => fn());
  }

  /**
   * Returns the current number of enqueued functions in the serial queue.
   * This provides a way to check the size of the queue and monitor its progress.
   *
   * @returns The length of the serial queue as a number.
   */
  public length() {
    return this.queue.length();
  }

  /**
   * Cancels the processing of the remaining functions in the serial queue and resolves the running promise.
   * Any enqueued functions that have not yet been executed will be discarded. The queue can still accept new
   * functions after cancellation, but the previously enqueued functions will not be re-processed.
   *
   * @returns The running promise which resolves when the current executing function (if any) completes.
   */
  public cancel() {
    this.queue.cancel();
    return this.runningPromise;
  }

  /**
   * Signals the SerialQueue that it should finish processing its current task and stop accepting new tasks.
   * The returned Promise resolves when all enqueued tasks have completed execution.
   *
   * @returns A Promise that resolves when the queue is completely emptied and no new tasks are allowed.
   */
  public end() {
    this.queue.end();
    return this.runningPromise;
  }

  /**
   * Enqueues fn for execution on the serial queue.
   * Returns the result of the function after execution.
   * @param fn - The function to enqueue.
   * @returns A resolution promise. Rejects if the function does, or if the function could not be enqueued.
   */
  public put<T>(fn: () => Promise<T>): Promise<T> {
    return new Promise((resolve, reject) => {
      const accepted = this.queue.put(async () => {
        try {
          const res = await fn();
          resolve(res);
        } catch (e) {
          reject(e);
        }
      });
      if (!accepted) {
        reject(new Error('Could not enqueue function'));
      }
    });
  }

  /**
   * Awaiting this ensures the queue is empty before resuming.
   */
  public async syncPoint() {
    await this.put(async () => {});
  }
}
