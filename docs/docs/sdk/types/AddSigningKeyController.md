```ts
class AddSigningKeyController {
    readonly userId: AccountId;
    private readonly userSigner;
    readonly signingPublicKey1: GrumpkinAddress;
    readonly signingPublicKey2: GrumpkinAddress | undefined;
    readonly fee: AssetValue;
    private readonly core;
    private proofOutput;
    private feeProofOutput?;
    private txId;
    constructor(userId: AccountId, userSigner: Signer, signingPublicKey1: GrumpkinAddress, signingPublicKey2: GrumpkinAddress | undefined, fee: AssetValue, core: CoreSdkInterface);
    createProof(): Promise<void>;
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
}
```