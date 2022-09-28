```ts
class RegisterController extends DepositHandler {
    readonly userId: GrumpkinAddress;
    readonly alias: string;
    private readonly accountPrivateKey;
    readonly spendingPublicKey: GrumpkinAddress;
    readonly recoveryPublicKey: GrumpkinAddress | undefined;
    readonly deposit: AssetValue;
    readonly fee: AssetValue;
    readonly depositor: EthAddress;
    protected readonly core: CoreSdkInterface;
    private proofOutput?;
    private txIds;
    private requireDeposit;
    constructor(userId: GrumpkinAddress, alias: string, accountPrivateKey: Buffer, spendingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, deposit: AssetValue, fee: AssetValue, depositor: EthAddress, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    createProof(): Promise<void>;
    exportProofTxs(): import("@aztec/barretenberg/rollup_provider").Tx[];
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
    private getProofOutputs;
}
```
