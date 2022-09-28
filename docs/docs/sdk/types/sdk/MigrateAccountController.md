```ts
export declare class MigrateAccountController extends DepositHandler {
    readonly userId: GrumpkinAddress;
    private readonly userSigner;
    readonly newAccountPrivateKey: Buffer;
    readonly newSpendingPublicKey: GrumpkinAddress;
    readonly recoveryPublicKey: GrumpkinAddress | undefined;
    readonly deposit: AssetValue;
    readonly fee: AssetValue;
    readonly depositor: EthAddress;
    protected readonly core: CoreSdkInterface;
    private proofOutput?;
    private txIds;
    private requireDeposit;
    constructor(userId: GrumpkinAddress, userSigner: Signer, newAccountPrivateKey: Buffer, newSpendingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, deposit: AssetValue, fee: AssetValue, depositor: EthAddress, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    createProof(): Promise<void>;
    exportProofTxs(): import("@aztec/barretenberg/rollup_provider").Tx[];
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
    private getProofOutputs;
}
```