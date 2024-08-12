import { type DebugLogger } from '../log/logger.js';
import { BaseMemoryQueue } from './base_memory_queue.js';

/**
 * A simple fifo queue. It can grow unbounded. It can have multiple producers and consumers.
 * Putting an item onto the queue always succeeds, unless either end() or cancel() has been called in which case
 * the item being pushed is simply discarded.
 */
export class FifoMemoryQueue<T> extends BaseMemoryQueue<T> {
  private container = new FifoQueue<T>();

  constructor(log?: DebugLogger) {
    super(log);
  }

  protected override get items() {
    return this.container;
  }
}

class FifoQueue<T> {
  private items: T[] = [];

  public put(item: T): void {
    this.items.push(item);
  }

  public get(): T | undefined {
    return this.items.shift();
  }

  public get length(): number {
    return this.items.length;
  }

  public clear() {
    this.items = [];
  }
}
