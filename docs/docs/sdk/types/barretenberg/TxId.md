```ts
class TxId {
    private buffer;
    constructor(buffer: Buffer);
    static deserialize(buffer: Buffer, offset: number): {
        elem: TxId;
        adv: number;
    };
    static fromString(hash: string): TxId;
    static random(): TxId;
    equals(rhs: TxId): boolean;
    toBuffer(): Buffer;
    toString(): string;
    toDepositSigningData(): Buffer;
}
```