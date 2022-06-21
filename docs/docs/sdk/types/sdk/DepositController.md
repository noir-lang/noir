```ts
class DepositController {
    readonly assetValue: AssetValue;
    readonly fee: AssetValue;
    readonly depositor: EthAddress;
    readonly recipient: GrumpkinAddress;
    readonly recipientSpendingKeyRequired: boolean;
    readonly feePayer: FeePayer | undefined;
    private readonly core;
    private readonly blockchain;
    private readonly provider;
    private readonly publicInput;
    private proofOutput?;
    private feeProofOutput?;
    private txIds;
    private pendingFundsStatus;
    constructor(assetValue: AssetValue, fee: AssetValue, depositor: EthAddress, recipient: GrumpkinAddress, recipientSpendingKeyRequired: boolean, feePayer: FeePayer | undefined, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    getPendingFunds(): Promise<bigint>;
    getRequiredFunds(): Promise<bigint>;
    getPublicAllowance(): Promise<bigint>;
    hasPermitSupport(): boolean;
    approve(): Promise<TxHash | undefined>;
    awaitApprove(timeout?: number, interval?: number): Promise<true | undefined>;
    depositFundsToContract(permitDeadline?: bigint): Promise<TxHash | undefined>;
    depositFundsToContractWithNonStandardPermit(permitDeadline?: bigint): Promise<TxHash | undefined>;
    awaitDepositFundsToContract(timeout?: number, interval?: number): Promise<true | undefined>;
    createProof(txRefNo?: number): Promise<void>;
    getProofHash(): Buffer;
    isProofApproved(): Promise<boolean>;
    approveProof(): Promise<TxHash | undefined>;
    awaitApproveProof(timeout?: number, interval?: number): Promise<true | undefined>;
    getSigningData(): Buffer;
    sign(): Promise<void>;
    isSignatureValid(): boolean;
    getProofs(): ProofOutput[];
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
    private getPendingFundsStatus;
    private createPermitArgs;
    private createPermitArgsNonStandard;
    private getContractChainId;
    private sendTransactionAndCheckOnchainData;
    private awaitTransaction;
}
```