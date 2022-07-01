```ts
class AddSpendingKeyController {
    readonly userId: GrumpkinAddress;
    private readonly userSigner;
    readonly spendingPublicKey1: GrumpkinAddress;
    readonly spendingPublicKey2: GrumpkinAddress | undefined;
    readonly fee: AssetValue;
    private readonly core;
    private proofOutput;
    private feeProofOutput?;
    private txIds;
    constructor(userId: GrumpkinAddress, userSigner: Signer, spendingPublicKey1: GrumpkinAddress, spendingPublicKey2: GrumpkinAddress | undefined, fee: AssetValue, core: CoreSdkInterface);
    createProof(): Promise<void>;
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
}
```