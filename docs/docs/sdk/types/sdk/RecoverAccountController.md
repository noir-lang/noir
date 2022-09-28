```ts
export declare class RecoverAccountController extends DepositHandler {
    readonly recoveryPayload: RecoveryPayload;
    readonly deposit: AssetValue;
    readonly fee: AssetValue;
    readonly depositor: EthAddress;
    protected readonly core: CoreSdkInterface;
    private proofOutput;
    private txIds;
    private requireDeposit;
    constructor(recoveryPayload: RecoveryPayload, deposit: AssetValue, fee: AssetValue, depositor: EthAddress, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    createProof(): Promise<void>;
    exportProofTxs(): import("@aztec/barretenberg/rollup_provider").Tx[];
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
    private getProofOutputs;
}
```