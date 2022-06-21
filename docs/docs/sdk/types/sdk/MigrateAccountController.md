```ts
class MigrateAccountController {
    readonly userId: GrumpkinAddress;
    private readonly userSigner;
    private readonly alias;
    readonly newAccountPrivateKey: Buffer;
    readonly newSpendingPublicKey: GrumpkinAddress;
    readonly recoveryPublicKey: GrumpkinAddress | undefined;
    readonly fee: AssetValue;
    private readonly core;
    private proofOutput;
    private feeProofOutput?;
    private txIds;
    constructor(userId: GrumpkinAddress, userSigner: Signer, alias: string, newAccountPrivateKey: Buffer, newSpendingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, fee: AssetValue, core: CoreSdkInterface);
    createProof(): Promise<void>;
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
}
```