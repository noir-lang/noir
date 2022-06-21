```ts
class SchnorrSignature {
    private buffer;
    static SIZE: number;
    constructor(buffer: Buffer);
    static isSignature(signature: string): boolean;
    static fromString(signature: string): SchnorrSignature;
    static randomSignature(): SchnorrSignature;
    s(): Buffer;
    e(): Buffer;
    toBuffer(): Buffer;
    toString(): string;
}
```