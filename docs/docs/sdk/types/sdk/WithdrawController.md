```ts
class WithdrawController {
    readonly userId: GrumpkinAddress;
    private readonly userSigner;
    readonly assetValue: AssetValue;
    readonly fee: AssetValue;
    readonly recipient: EthAddress;
    private readonly core;
    private proofOutput;
    private feeProofOutput?;
    private txIds;
    constructor(userId: GrumpkinAddress, userSigner: Signer, assetValue: AssetValue, fee: AssetValue, recipient: EthAddress, core: CoreSdkInterface);
    createProof(): Promise<void>;
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
}
```