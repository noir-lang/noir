```ts
class DefiController {
    readonly userId: AccountId;
    private readonly userSigner;
    readonly bridgeId: BridgeId;
    readonly depositValue: AssetValue;
    readonly fee: AssetValue;
    private readonly core;
    private proofOutput?;
    private jsProofOutput?;
    private feeProofOutput?;
    private txId?;
    constructor(userId: AccountId, userSigner: Signer, bridgeId: BridgeId, depositValue: AssetValue, fee: AssetValue, core: CoreSdkInterface);
    createProof(): Promise<void>;
    send(): Promise<TxId>;
    awaitDefiDepositCompletion(timeout?: number): Promise<void>;
    awaitDefiFinalisation(timeout?: number): Promise<void>;
    awaitSettlement(timeout?: number): Promise<void>;
    getInteractionNonce(): Promise<number | undefined>;
}
```