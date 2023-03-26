/**
 * Simple read/write interface for wasm modules.
 */
export interface DataStore {
  get(key: string): Promise<Buffer | undefined>;
  set(key: string, value: Buffer): Promise<void>;
}
