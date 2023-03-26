/// <reference types="node" resolution-mode="require"/>
export type DeserializeFn<T> = (buf: Buffer, offset: number) => {
    elem: T;
    adv: number;
};
export declare class Deserializer {
    private buf;
    private offset;
    constructor(buf: Buffer, offset?: number);
    bool(): boolean;
    uInt32(): number;
    int32(): number;
    bigInt(width?: number): bigint;
    vector(): Buffer;
    buffer(width: number): Buffer;
    string(): string;
    date(): Date;
    deserializeArray<T>(fn: DeserializeFn<T>): T[];
    exec<T>(fn: DeserializeFn<T>): T;
    getOffset(): number;
}
//# sourceMappingURL=deserializer.d.ts.map