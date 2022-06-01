```ts
class AliasHash {
    private buffer;
    constructor(buffer: Buffer);
    static random(): AliasHash;
    static fromAlias(alias: string, blake2s: Blake2s): AliasHash;
    static fromString(hash: string): AliasHash;
    toBuffer(): Buffer;
    toBuffer32(): Buffer;
    toString(): string;
    equals(rhs: AliasHash): boolean;
}
```