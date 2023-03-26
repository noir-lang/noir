/// <reference types="node" resolution-mode="require"/>
export declare class Serializer {
    private buf;
    constructor();
    bool(bool: boolean): void;
    uInt32(num: number): void;
    int32(num: number): void;
    bigInt(num: bigint): void;
    /**
     * The given buffer is of variable length. Prefixes the buffer with its length.
     */
    vector(buf: Buffer): void;
    /**
     * Directly serializes a buffer that maybe of fixed, or variable length.
     * It is assumed the corresponding deserialize function will handle variable length data, thus the length
     * does not need to be prefixed here.
     * If serializing a raw, variable length buffer, use vector().
     */
    buffer(buf: Buffer): void;
    string(str: string): void;
    date(date: Date): void;
    getBuffer(): Buffer;
    serializeArray<T>(arr: T[]): void;
}
//# sourceMappingURL=serializer.d.ts.map