import { FifoMemoryQueue } from './fifo_memory_queue.js';

/**
 * Allows the acquiring of up to `size` tokens before calls to acquire block, waiting for a call to release().
 */
export class Semaphore {
  private readonly queue = new FifoMemoryQueue<boolean>();

  constructor(size: number) {
    new Array(size).fill(true).map(() => this.queue.put(true));
  }

  /**
   * Acquires a token from the Semaphore, allowing access to a limited resource.
   * If no tokens are available, the call will block and wait until a token is released.
   * Use in conjunction with the release() method to manage access to resources with limited capacity.
   *
   * @returns A Promise that resolves when a token is acquired.
   */
  public async acquire() {
    await this.queue.get();
  }

  /**
   * Releases a token back into the semaphore, allowing another acquire call to proceed.
   * If there are any pending calls to acquire(), one of them will be unblocked and allowed to proceed.
   * This method should only be called by the holder of the acquired token to ensure proper functionality
   * and avoid unexpected behavior.
   */
  public release() {
    this.queue.put(true);
  }
}
