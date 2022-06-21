```ts
class RecoveryPayload {
    trustedThirdPartyPublicKey: GrumpkinAddress;
    recoveryPublicKey: GrumpkinAddress;
    recoveryData: RecoveryData;
    constructor(trustedThirdPartyPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress, recoveryData: RecoveryData);
    static fromBuffer(data: Buffer): RecoveryPayload;
    static fromString(data: string): RecoveryPayload;
    toBuffer(): Buffer;
    toString(): string;
}
```