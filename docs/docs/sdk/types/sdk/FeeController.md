```ts
export declare class FeeController {
    readonly userId: GrumpkinAddress;
    private readonly userSigner;
    readonly proofTxs: Tx[];
    readonly fee: AssetValue;
    private readonly core;
    private feeProofOutputs;
    private txIds;
    constructor(userId: GrumpkinAddress, userSigner: Signer, proofTxs: Tx[], fee: AssetValue, core: CoreSdkInterface);
    createProof(): Promise<void>;
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
}
```