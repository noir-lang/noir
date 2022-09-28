```ts
export declare class DefiController {
    readonly userId: GrumpkinAddress;
    private readonly userSigner;
    readonly bridgeCallData: BridgeCallData;
    readonly assetValue: AssetValue;
    readonly fee: AssetValue;
    private readonly core;
    private readonly requireFeePayingTx;
    private proofOutput?;
    private jsProofOutputs;
    private feeProofOutputs;
    private txIds;
    constructor(userId: GrumpkinAddress, userSigner: Signer, bridgeCallData: BridgeCallData, assetValue: AssetValue, fee: AssetValue, core: CoreSdkInterface);
    createProof(): Promise<void>;
    exportProofTxs(): import("@aztec/barretenberg/rollup_provider").Tx[];
    send(): Promise<TxId>;
    awaitDefiDepositCompletion(timeout?: number): Promise<void>;
    awaitDefiFinalisation(timeout?: number): Promise<void>;
    awaitSettlement(timeout?: number): Promise<void>;
    getInteractionNonce(): Promise<number | undefined>;
    private getDefiTxId;
}
```