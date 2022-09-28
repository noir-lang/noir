```ts
export declare class DepositHandler {
    readonly assetValue: AssetValue;
    readonly fee: AssetValue;
    readonly depositor: EthAddress;
    readonly recipient: GrumpkinAddress;
    readonly recipientSpendingKeyRequired: boolean;
    protected readonly core: CoreSdkInterface;
    private readonly blockchain;
    private readonly provider;
    protected readonly publicInput: AssetValue;
    private depositProofOutput?;
    private pendingFundsStatus;
    constructor(assetValue: AssetValue, fee: AssetValue, depositor: EthAddress, recipient: GrumpkinAddress, recipientSpendingKeyRequired: boolean, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    getPendingFunds(): Promise<bigint>;
    getRequiredFunds(): Promise<bigint>;
    getPublicAllowance(): Promise<bigint>;
    hasPermitSupport(): boolean;
    approve(permitDeadline?: bigint): Promise<TxHash | undefined>;
    awaitApprove(timeout?: number, interval?: number): Promise<void>;
    depositFundsToContract(permitDeadline?: bigint): Promise<TxHash | undefined>;
    depositFundsToContractWithNonStandardPermit(permitDeadline?: bigint): Promise<TxHash>;
    awaitDepositFundsToContract(timeout?: number, interval?: number): Promise<true | undefined>;
    createProof(txRefNo?: number): Promise<void>;
    getProofOutput(): ProofOutput;
    getProofHash(): Buffer;
    isProofApproved(): Promise<boolean>;
    approveProof(): Promise<TxHash | undefined>;
    awaitApproveProof(timeout?: number, interval?: number): Promise<true | undefined>;
    getSigningData(): Buffer;
    sign(): Promise<void>;
    isSignatureValid(): boolean;
    private getPendingFundsStatus;
    private createPermitArgs;
    private createPermitArgsNonStandard;
    private getContractChainId;
    private sendTransactionAndCheckOnchainData;
    private awaitTransaction;
}
```