import levelup from 'levelup';
import memdown from 'memdown';
/**
 * Cache for data used by wasm module.
 * Stores in a LevelUp database.
 */
export class WebDataStore {
    constructor() {
        // TODO: The whole point of this is to reduce memory load in the browser.
        // Replace with leveljs so the data is stored in indexeddb and not in memory.
        // Hack: Cast as any to work around package "broken" with node16 resolution
        // See https://github.com/microsoft/TypeScript/issues/49160
        this.db = levelup(memdown());
    }
    /**
     * Lookup a key.
     * @param key - Key to lookup.
     * @returns The buffer.
     */
    async get(key) {
        return await this.db.get(key).catch(() => { });
    }
    /**
     * Alter a key.
     * @param key - Key to alter.
     * @param value - Buffer to store.
     */
    async set(key, value) {
        await this.db.put(key, value);
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoid2ViX2RhdGFfc3RvcmUuanMiLCJzb3VyY2VSb290IjoiIiwic291cmNlcyI6WyIuLi8uLi8uLi9zcmMvd29ya2VyL2Jyb3dzZXIvd2ViX2RhdGFfc3RvcmUudHMiXSwibmFtZXMiOltdLCJtYXBwaW5ncyI6IkFBQ0EsT0FBTyxPQUFvQixNQUFNLFNBQVMsQ0FBQztBQUMzQyxPQUFPLE9BQU8sTUFBTSxTQUFTLENBQUM7QUFFOUI7OztHQUdHO0FBQ0gsTUFBTSxPQUFPLFlBQVk7SUFHdkI7UUFDRSx5RUFBeUU7UUFDekUsNkVBQTZFO1FBQzdFLDJFQUEyRTtRQUMzRSwyREFBMkQ7UUFDM0QsSUFBSSxDQUFDLEVBQUUsR0FBRyxPQUFPLENBQUUsT0FBZSxFQUFFLENBQUMsQ0FBQztJQUN4QyxDQUFDO0lBRUQ7Ozs7T0FJRztJQUNILEtBQUssQ0FBQyxHQUFHLENBQUMsR0FBVztRQUNuQixPQUFPLE1BQU0sSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxDQUFDLENBQUMsS0FBSyxDQUFDLEdBQUcsRUFBRSxHQUFFLENBQUMsQ0FBQyxDQUFDO0lBQ2hELENBQUM7SUFFRDs7OztPQUlHO0lBQ0gsS0FBSyxDQUFDLEdBQUcsQ0FBQyxHQUFXLEVBQUUsS0FBYTtRQUNsQyxNQUFNLElBQUksQ0FBQyxFQUFFLENBQUMsR0FBRyxDQUFDLEdBQUcsRUFBRSxLQUFLLENBQUMsQ0FBQztJQUNoQyxDQUFDO0NBQ0YifQ==