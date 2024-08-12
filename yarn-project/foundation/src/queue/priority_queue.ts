/**
 * Priority queue implementation based on a custom comparator.
 */
export class PriorityQueue<T> {
  private items: T[];

  constructor(private comparator: (a: T, b: T) => number) {
    this.items = [];
  }

  public put(item: T): void {
    let i = 0;
    while (i < this.items.length && this.comparator(item, this.items[i]) >= 0) {
      i++;
    }
    this.items.splice(i, 0, item);
  }

  public get(): T | undefined {
    return this.items.shift();
  }

  public peek(): T | undefined {
    return this.items[0];
  }

  public clear() {
    this.items = [];
  }

  public get length(): number {
    return this.items.length;
  }
}
