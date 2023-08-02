/**
 * Simple read/write interface for wasm modules.
 */
export interface DataStore {
  /**
   * Get a value from our DB.
   * @param key - The key to look up.
   * @returns The value.
   */
  get(key: string): Promise<Buffer | undefined>;

  /**
   * Set a value in our DB.
   * @param key - The key to update.
   * @param value - The value to set.
   * @returns Nothing.
   */
  set(key: string, value: Buffer): Promise<void>;
}
