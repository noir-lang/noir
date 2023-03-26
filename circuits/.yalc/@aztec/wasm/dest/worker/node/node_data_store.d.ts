/// <reference types="node" resolution-mode="require"/>
import { DataStore } from '../data_store.js';
/**
 * Cache for data used by wasm module.
 */
export declare class NodeDataStore implements DataStore {
    private db;
    constructor(path?: string);
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
     */
    set(key: string, value: Buffer): Promise<void>;
}
//# sourceMappingURL=node_data_store.d.ts.map