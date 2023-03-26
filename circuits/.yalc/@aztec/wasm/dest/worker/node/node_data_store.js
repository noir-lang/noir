import levelup from 'levelup';
import leveldown from 'leveldown';
import memdown from 'memdown';
/**
 * Cache for data used by wasm module.
 */
export class NodeDataStore {
    // eslint-disable-next-line
    constructor(path) {
        // Hack: Cast as any to work around packages "broken" with node16 resolution
        // See https://github.com/microsoft/TypeScript/issues/49160
        this.db = levelup(path ? leveldown(path) : memdown());
    }
    /**
     * Get a value from our DB.
     * @param key - The key to look up.
     * @returns The value.
     */
    async get(key) {
        return await this.db.get(key).catch(() => { });
    }
    /**
     * Set a value in our DB.
     * @param key - The key to update.
     * @param value - The value to set.
     */
    async set(key, value) {
        await this.db.put(key, value);
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoibm9kZV9kYXRhX3N0b3JlLmpzIiwic291cmNlUm9vdCI6IiIsInNvdXJjZXMiOlsiLi4vLi4vLi4vc3JjL3dvcmtlci9ub2RlL25vZGVfZGF0YV9zdG9yZS50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFDQSxPQUFPLE9BQW9CLE1BQU0sU0FBUyxDQUFDO0FBQzNDLE9BQU8sU0FBUyxNQUFNLFdBQVcsQ0FBQztBQUNsQyxPQUFPLE9BQU8sTUFBTSxTQUFTLENBQUM7QUFFOUI7O0dBRUc7QUFDSCxNQUFNLE9BQU8sYUFBYTtJQUd4QiwyQkFBMkI7SUFDM0IsWUFBWSxJQUFhO1FBQ3ZCLDRFQUE0RTtRQUM1RSwyREFBMkQ7UUFDM0QsSUFBSSxDQUFDLEVBQUUsR0FBRyxPQUFPLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBRSxTQUFpQixDQUFDLElBQUksQ0FBQyxDQUFDLENBQUMsQ0FBRSxPQUFlLEVBQUUsQ0FBQyxDQUFDO0lBQzFFLENBQUM7SUFFRDs7OztPQUlHO0lBQ0gsS0FBSyxDQUFDLEdBQUcsQ0FBQyxHQUFXO1FBQ25CLE9BQU8sTUFBTSxJQUFJLENBQUMsRUFBRSxDQUFDLEdBQUcsQ0FBQyxHQUFHLENBQUMsQ0FBQyxLQUFLLENBQUMsR0FBRyxFQUFFLEdBQUUsQ0FBQyxDQUFDLENBQUM7SUFDaEQsQ0FBQztJQUVEOzs7O09BSUc7SUFDSCxLQUFLLENBQUMsR0FBRyxDQUFDLEdBQVcsRUFBRSxLQUFhO1FBQ2xDLE1BQU0sSUFBSSxDQUFDLEVBQUUsQ0FBQyxHQUFHLENBQUMsR0FBRyxFQUFFLEtBQUssQ0FBQyxDQUFDO0lBQ2hDLENBQUM7Q0FDRiJ9