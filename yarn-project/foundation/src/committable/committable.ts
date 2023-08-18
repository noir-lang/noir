/**
 * A class that allows for a value to be committed or rolled back.
 */
export class Committable<T> {
  private currentValue: T;
  private nextValue: T | undefined = undefined;

  constructor(initialValue: T) {
    this.currentValue = initialValue;
  }

  /**
   * Commits the uncommitted value.
   */
  public commit() {
    if (this.nextValue === undefined) {
      return;
    }
    this.currentValue = this.nextValue;
    this.nextValue = undefined;
  }

  /**
   * Rolls back the uncommitted value.
   */
  public rollback() {
    this.nextValue === undefined;
  }

  /**
   * Gets the current value.
   * @param includeUncommitted - Whether to include the uncommitted value.
   * @returns The current value if includeUncommitted is false, otherwise the uncommitted value.
   */
  public get(includeUncommitted: boolean = false): T {
    return includeUncommitted && this.nextValue ? this.nextValue : this.currentValue;
  }

  /**
   * Sets the next value to be committed to.
   * @param value - The new value to be set.
   */
  public set(value: T) {
    this.nextValue = value;
  }
}
