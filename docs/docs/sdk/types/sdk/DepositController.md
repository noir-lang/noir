```ts
export declare class DepositController extends DepositHandler {
    readonly assetValue: AssetValue;
    readonly fee: AssetValue;
    readonly depositor: EthAddress;
    readonly recipient: GrumpkinAddress;
    readonly recipientSpendingKeyRequired: boolean;
    protected readonly core: CoreSdkInterface;
    private txIds;
    constructor(assetValue: AssetValue, fee: AssetValue, depositor: EthAddress, recipient: GrumpkinAddress, recipientSpendingKeyRequired: boolean, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    createProof(): Promise<void>;
    exportProofTxs(): import("@aztec/barretenberg/rollup_provider").Tx[];
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
}
```