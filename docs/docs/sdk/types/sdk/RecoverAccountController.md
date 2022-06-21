```ts
class RecoverAccountController {
    readonly alias: string;
    readonly recoveryPayload: RecoveryPayload;
    readonly depositValue: AssetValue;
    readonly fee: AssetValue;
    readonly depositor: EthAddress;
    private readonly core;
    private depositController?;
    private proofOutput;
    private txIds;
    constructor(alias: string, recoveryPayload: RecoveryPayload, depositValue: AssetValue, fee: AssetValue, depositor: EthAddress, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    getPendingFunds(): Promise<bigint>;
    getRequiredFunds(): Promise<bigint>;
    getPublicAllowance(): Promise<bigint>;
    approve(): Promise<import("@aztec/barretenberg/blockchain").TxHash | undefined>;
    awaitApprove(timeout?: number, interval?: number): Promise<void>;
    depositFundsToContract(permitDeadline?: bigint): Promise<import("@aztec/barretenberg/blockchain").TxHash | undefined>;
    depositFundsToContractWithNonStandardPermit(permitDeadline: bigint): Promise<import("@aztec/barretenberg/blockchain").TxHash | undefined>;
    awaitDepositFundsToContract(timeout?: number, interval?: number): Promise<true | undefined>;
    createProof(): Promise<void>;
    getProofHash(): Buffer | undefined;
    getSigningData(): Buffer | undefined;
    isProofApproved(): Promise<boolean>;
    approveProof(): Promise<import("@aztec/barretenberg/blockchain").TxHash | undefined>;
    awaitApproveProof(timeout?: number, interval?: number): Promise<true | undefined>;
    sign(): Promise<void>;
    isSignatureValid(): boolean;
    send(): Promise<TxId>;
    awaitSettlement(timeout?: number): Promise<void>;
}
```