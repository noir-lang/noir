import { MemoryFifo } from './memory_fifo.js';

/**
 * Allows the acquiring of up to `size` tokens before calls to acquire block, waiting for a call to release().
 */
export class Semaphore {
  private readonly queue = new MemoryFifo<boolean>();

  constructor(size: number) {
    new Array(size).fill(true).map(() => this.queue.put(true));
  }

  public async acquire() {
    await this.queue.get();
  }

  public release() {
    this.queue.put(true);
  }
}
