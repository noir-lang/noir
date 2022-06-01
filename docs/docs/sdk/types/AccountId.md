```ts
class AccountId {
    publicKey: GrumpkinAddress;
    accountNonce: number;
    constructor(publicKey: GrumpkinAddress, accountNonce: number);
    static fromBuffer(id: Buffer): AccountId;
    static fromString(idStr: string): AccountId;
    static random(): AccountId;
    equals(rhs: AccountId): boolean;
    toBuffer(): Buffer;
    toString(): string;
    toShortString(): string;
}
```