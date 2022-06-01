```ts
class AccountAliasId {
    aliasHash: AliasHash;
    accountNonce: number;
    static ZERO: AccountAliasId;
    constructor(aliasHash: AliasHash, accountNonce: number);
    static fromAlias(alias: string, accountNonce: number, blake2s: Blake2s): AccountAliasId;
    static random(): AccountAliasId;
    static fromBuffer(id: Buffer): AccountAliasId;
    toBuffer(): Buffer;
    toString(): string;
    equals(rhs: AccountAliasId): boolean;
}
```