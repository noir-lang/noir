```ts
export declare class RecoveryData {
    accountPublicKey: GrumpkinAddress;
    signature: SchnorrSignature;
    constructor(accountPublicKey: GrumpkinAddress, signature: SchnorrSignature);
    static fromBuffer(data: Buffer): RecoveryData;
    static fromString(data: string): RecoveryData;
    toBuffer(): Buffer;
    toString(): string;
}
```