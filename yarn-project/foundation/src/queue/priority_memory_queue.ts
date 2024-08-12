import { BaseMemoryQueue } from './base_memory_queue.js';
import { PriorityQueue } from './priority_queue.js';

/**
 * A priority queue. It can grow unbounded. It can have multiple producers and consumers.
 * Putting an item onto the queue always succeeds, unless either end() or cancel() has been called in which case
 * the item being pushed is simply discarded.
 */
export class PriorityMemoryQueue<T> extends BaseMemoryQueue<T> {
  private container: PriorityQueue<T>;

  constructor(comparator: (a: T, b: T) => number) {
    super();
    this.container = new PriorityQueue(comparator);
  }

  protected override get items() {
    return this.container;
  }
}
