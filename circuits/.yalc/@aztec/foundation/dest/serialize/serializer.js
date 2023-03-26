import { serializeBufferArrayToVector } from './index.js';
import { boolToByte, numToInt32BE, numToUInt32BE, serializeBigInt, serializeBufferToVector, serializeDate, } from './free_funcs.js';
// export type DeserializeFn<T> = (buf: Buffer, offset: number) => { elem: T; adv: number };
export class Serializer {
    constructor() {
        this.buf = [];
    }
    bool(bool) {
        this.buf.push(boolToByte(bool));
    }
    uInt32(num) {
        this.buf.push(numToUInt32BE(num));
    }
    int32(num) {
        this.buf.push(numToInt32BE(num));
    }
    bigInt(num) {
        this.buf.push(serializeBigInt(num));
    }
    /**
     * The given buffer is of variable length. Prefixes the buffer with its length.
     */
    vector(buf) {
        this.buf.push(serializeBufferToVector(buf));
    }
    /**
     * Directly serializes a buffer that maybe of fixed, or variable length.
     * It is assumed the corresponding deserialize function will handle variable length data, thus the length
     * does not need to be prefixed here.
     * If serializing a raw, variable length buffer, use vector().
     */
    buffer(buf) {
        this.buf.push(buf);
    }
    string(str) {
        this.vector(Buffer.from(str));
    }
    date(date) {
        this.buf.push(serializeDate(date));
    }
    getBuffer() {
        return Buffer.concat(this.buf);
    }
    serializeArray(arr) {
        this.buf.push(serializeBufferArrayToVector(arr.map((e) => e.toBuffer())));
    }
}
//# sourceMappingURL=data:application/json;base64,eyJ2ZXJzaW9uIjozLCJmaWxlIjoic2VyaWFsaXplci5qcyIsInNvdXJjZVJvb3QiOiIiLCJzb3VyY2VzIjpbIi4uLy4uL3NyYy9zZXJpYWxpemUvc2VyaWFsaXplci50cyJdLCJuYW1lcyI6W10sIm1hcHBpbmdzIjoiQUFBQSxPQUFPLEVBQUUsNEJBQTRCLEVBQUUsTUFBTSxZQUFZLENBQUM7QUFDMUQsT0FBTyxFQUNMLFVBQVUsRUFDVixZQUFZLEVBQ1osYUFBYSxFQUNiLGVBQWUsRUFDZix1QkFBdUIsRUFDdkIsYUFBYSxHQUNkLE1BQU0saUJBQWlCLENBQUM7QUFFekIsNEZBQTRGO0FBRTVGLE1BQU0sT0FBTyxVQUFVO0lBR3JCO1FBRlEsUUFBRyxHQUFhLEVBQUUsQ0FBQztJQUVaLENBQUM7SUFFVCxJQUFJLENBQUMsSUFBYTtRQUN2QixJQUFJLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxVQUFVLENBQUMsSUFBSSxDQUFDLENBQUMsQ0FBQztJQUNsQyxDQUFDO0lBRU0sTUFBTSxDQUFDLEdBQVc7UUFDdkIsSUFBSSxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsYUFBYSxDQUFDLEdBQUcsQ0FBQyxDQUFDLENBQUM7SUFDcEMsQ0FBQztJQUVNLEtBQUssQ0FBQyxHQUFXO1FBQ3RCLElBQUksQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLFlBQVksQ0FBQyxHQUFHLENBQUMsQ0FBQyxDQUFDO0lBQ25DLENBQUM7SUFFTSxNQUFNLENBQUMsR0FBVztRQUN2QixJQUFJLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyxlQUFlLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztJQUN0QyxDQUFDO0lBRUQ7O09BRUc7SUFDSSxNQUFNLENBQUMsR0FBVztRQUN2QixJQUFJLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyx1QkFBdUIsQ0FBQyxHQUFHLENBQUMsQ0FBQyxDQUFDO0lBQzlDLENBQUM7SUFFRDs7Ozs7T0FLRztJQUNJLE1BQU0sQ0FBQyxHQUFXO1FBQ3ZCLElBQUksQ0FBQyxHQUFHLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDO0lBQ3JCLENBQUM7SUFFTSxNQUFNLENBQUMsR0FBVztRQUN2QixJQUFJLENBQUMsTUFBTSxDQUFDLE1BQU0sQ0FBQyxJQUFJLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBQztJQUNoQyxDQUFDO0lBRU0sSUFBSSxDQUFDLElBQVU7UUFDcEIsSUFBSSxDQUFDLEdBQUcsQ0FBQyxJQUFJLENBQUMsYUFBYSxDQUFDLElBQUksQ0FBQyxDQUFDLENBQUM7SUFDckMsQ0FBQztJQUVNLFNBQVM7UUFDZCxPQUFPLE1BQU0sQ0FBQyxNQUFNLENBQUMsSUFBSSxDQUFDLEdBQUcsQ0FBQyxDQUFDO0lBQ2pDLENBQUM7SUFFTSxjQUFjLENBQUksR0FBUTtRQUMvQixJQUFJLENBQUMsR0FBRyxDQUFDLElBQUksQ0FBQyw0QkFBNEIsQ0FBQyxHQUFHLENBQUMsR0FBRyxDQUFDLENBQUMsQ0FBTSxFQUFFLEVBQUUsQ0FBQyxDQUFDLENBQUMsUUFBUSxFQUFFLENBQUMsQ0FBQyxDQUFDLENBQUM7SUFDakYsQ0FBQztDQUNGIn0=