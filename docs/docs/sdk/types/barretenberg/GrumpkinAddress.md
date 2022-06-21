```ts
class GrumpkinAddress {
    private buffer;
    static SIZE: number;
    static ZERO: GrumpkinAddress;
    constructor(buffer: Buffer);
    static isAddress(address: string): boolean;
    static fromString(address: string): GrumpkinAddress;
    /**
     * NOT a valid address! Do not use in proofs.
     */
    static random(): GrumpkinAddress;
    /**
     * A valid address (is a point on the curve).
     */
    static one(): GrumpkinAddress;
    equals(rhs: GrumpkinAddress): boolean;
    toBuffer(): Buffer;
    x(): Buffer;
    y(): Buffer;
    toString(): string;
    toShortString(): string;
}
```