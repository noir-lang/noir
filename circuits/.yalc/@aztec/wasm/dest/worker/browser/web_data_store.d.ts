/// <reference types="node" resolution-mode="require"/>
import { DataStore } from '../data_store.js';
/**
 * Cache for data used by wasm module.
 * Stores in a LevelUp database.
 */
export declare class WebDataStore implements DataStore {
    private db;
    constructor();
    /**
     * Lookup a key.
     * @param key - Key to lookup.
     * @returns The buffer.
     */
    get(key: string): Promise<Buffer | undefined>;
    /**
     * Alter a key.
     * @param key - Key to alter.
     * @param value - Buffer to store.
     */
    set(key: string, value: Buffer): Promise<void>;
}
//# sourceMappingURL=web_data_store.d.ts.map