```ts
class RegisterController {
    readonly userId: GrumpkinAddress;
    readonly alias: string;
    private readonly accountPrivateKey;
    readonly spendingPublicKey: GrumpkinAddress;
    readonly recoveryPublicKey: GrumpkinAddress | undefined;
    readonly depositValue: AssetValue;
    readonly fee: AssetValue;
    readonly depositor: EthAddress;
    readonly feePayer: FeePayer | undefined;
    private readonly core;
    private depositController?;
    private proofOutput?;
    private txIds;
    constructor(userId: GrumpkinAddress, alias: string, accountPrivateKey: Buffer, spendingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, depositValue: AssetValue, fee: AssetValue, depositor: EthAddress, feePayer: FeePayer | undefined, core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
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
