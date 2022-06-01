```ts
class DepositController {
    readonly userId: AccountId;
    private readonly userSigner;
    readonly assetValue: AssetValue;
    readonly fee: AssetValue;
    readonly from: EthAddress;
    readonly to: AccountId;
    private readonly core;
    private readonly blockchain;
    private readonly provider;
    private readonly publicInput;
    private readonly requireFeePayingTx;
    private proofOutput?;
    private feeProofOutput?;
    private txIds?;
    private txHash?;
    constructor(userId: AccountId, userSigner: Signer, assetValue: AssetValue, fee: AssetValue, from: EthAddress, to: AccountId, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    getPendingFunds(): Promise<bigint>;
    getRequiredFunds(): Promise<bigint>;
    getPublicAllowance(): Promise<bigint>;
    approve(): Promise<TxHash>;
    depositFundsToContract(): Promise<TxHash>;
    depositFundsToContractWithPermit(deadline: bigint): Promise<TxHash>;
    depositFundsToContractWithNonStandardPermit(deadline: bigint): Promise<TxHash>;
    depositFundsToContractWithProofApproval(): Promise<TxHash>;
    depositFundsToContractWithPermitAndProofApproval(deadline: bigint): Promise<TxHash>;
    depositFundsToContractWithNonStandardPermitAndProofApproval(deadline: bigint): Promise<TxHash>;
    awaitDepositFundsToContract(): Promise<void>;
    createProof(txRefNo?: number): Promise<void>;
    getSigningData(): Buffer;
    getTxId(): TxId;
    isProofApproved(): Promise<boolean>;
    approveProof(): Promise<TxHash>;
    sign(): Promise<void>;
    isSignatureValid(): boolean;
    getProofs(): ProofOutput[];
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
    private createPermitArgs;
    private createPermitArgsNonStandard;
}
```