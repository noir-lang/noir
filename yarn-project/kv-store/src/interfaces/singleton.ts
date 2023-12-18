/**
 * Represents a singleton value in the database.
 */
export interface AztecSingleton<T> {
  /**
   * Gets the value.
   */
  get(): T | undefined;

  /**
   * Sets the value.
   * @param val - The new value
   */
  set(val: T): Promise<boolean>;

  /**
   * Deletes the value.
   */
  delete(): Promise<boolean>;
}
