/**
 * Represents a singleton value in the database.
 * Note: The singleton loses type info so it's recommended to serialize to buffer when storing it.
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
