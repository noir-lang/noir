```ts
class AztecSdk extends EventEmitter {
    private core;
    private blockchain;
    private provider;
    constructor(core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    run(): Promise<void>;
    destroy(): Promise<void>;
    awaitSynchronised(): Promise<void>;
    isUserSynching(userId: AccountId): Promise<boolean>;
    awaitUserSynchronised(userId: AccountId): Promise<void>;
    awaitSettlement(txId: TxId, timeout?: number): Promise<void>;
    awaitDefiDepositCompletion(txId: TxId, timeout?: number): Promise<void>;
    awaitDefiFinalisation(txId: TxId, timeout?: number): Promise<void>;
    awaitDefiSettlement(txId: TxId, timeout?: number): Promise<void>;
    getLocalStatus(): Promise<import("../core_sdk").SdkStatus>;
    getRemoteStatus(): Promise<import("@aztec/barretenberg/rollup_provider").RollupProviderStatus>;
    getTxFees(assetId: number): Promise<AssetValue[][]>;
    getLatestAccountNonce(publicKey: GrumpkinAddress): Promise<number>;
    getRemoteLatestAccountNonce(publicKey: GrumpkinAddress): Promise<number>;
    getLatestAliasNonce(alias: string): Promise<number>;
    getRemoteLatestAliasNonce(alias: string): Promise<number>;
    getAccountId(alias: string, accountNonce?: number): Promise<AccountId | undefined>;
    getRemoteAccountId(alias: string, accountNonce?: number): Promise<AccountId | undefined>;
    isAliasAvailable(alias: string): Promise<boolean>;
    isRemoteAliasAvailable(alias: string): Promise<boolean>;
    userExists(userId: AccountId): Promise<boolean>;
    addUser(privateKey: Buffer, accountNonce?: number, noSync?: boolean): Promise<AztecSdkUser>;
    removeUser(userId: AccountId): Promise<void>;
    /**
     * Returns a AztecSdkUser for a locally resolved user.
     */
    getUser(userId: AccountId): Promise<AztecSdkUser>;
    getUserData(userId: AccountId): Promise<import("../user").UserData>;
    getUsersData(): Promise<import("../user").UserData[]>;
    createSchnorrSigner(privateKey: Buffer): Promise<SchnorrSigner>;
    derivePublicKey(privateKey: Buffer): Promise<GrumpkinAddress>;
    getAssetIdByAddress(address: EthAddress, gasLimit?: number): number;
    getAssetIdBySymbol(symbol: string, gasLimit?: number): number;
    fromBaseUnits({ assetId, value }: AssetValue, symbol?: boolean, precision?: number): string;
    toBaseUnits(assetId: number, value: string): {
        assetId: number;
        value: bigint;
    };
    getAssetInfo(assetId: number): import("@aztec/barretenberg/blockchain").BlockchainAsset;
    isFeePayingAsset(assetId: number): Promise<boolean>;
    isVirtualAsset(assetId: number): boolean;
    mint(assetId: number, value: bigint, account: EthAddress, provider?: EthereumProvider): Promise<TxHash>;
    setSupportedAsset(assetAddress: EthAddress, assetGasLimit?: number, options?: SendTxOptions): Promise<TxHash>;
    getBridgeAddressId(address: EthAddress, gasLimit?: number): number;
    setSupportedBridge(bridgeAddress: EthAddress, bridgeGasLimit?: number, options?: SendTxOptions): Promise<TxHash>;
    processAsyncDefiInteraction(interactionNonce: number, options?: SendTxOptions): Promise<TxHash>;
    private getTransactionFees;
    getDepositFees(assetId: number): Promise<AssetValue[]>;
    createDepositController(userId: AccountId, userSigner: Signer, value: AssetValue, fee: AssetValue, from: EthAddress, to?: AccountId, provider?: EthereumProvider): DepositController;
    getWithdrawFees(assetId: number, recipient?: EthAddress): Promise<AssetValue[]>;
    createWithdrawController(userId: AccountId, userSigner: Signer, value: AssetValue, fee: AssetValue, to: EthAddress): WithdrawController;
    getTransferFees(assetId: number): Promise<AssetValue[]>;
    createTransferController(userId: AccountId, userSigner: Signer, value: AssetValue, fee: AssetValue, to: AccountId): TransferController;
    getDefiFees(bridgeId: BridgeId, userId?: AccountId, depositValue?: AssetValue): Promise<{
        value: bigint;
        assetId: number;
    }[]>;
    createDefiController(userId: AccountId, userSigner: Signer, bridgeId: BridgeId, value: AssetValue, fee: AssetValue): DefiController;
    generateAccountRecoveryData(alias: string, publicKey: GrumpkinAddress, trustedThirdPartyPublicKeys: GrumpkinAddress[], accountNonce?: number): Promise<RecoveryPayload[]>;
    getRegisterFees({ assetId, value: depositValue }: AssetValue): Promise<AssetValue[]>;
    createRegisterController(userId: AccountId, userSigner: Signer, alias: string, signingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, deposit: AssetValue, fee: AssetValue, depositor: EthAddress, provider?: EthereumProvider): RegisterController;
    getRecoverAccountFees(assetId: number): Promise<AssetValue[]>;
    createRecoverAccountController(recoveryPayload: RecoveryPayload, fee: AssetValue): RecoverAccountController;
    getAddSigningKeyFees(assetId: number): Promise<AssetValue[]>;
    createAddSigningKeyController(userId: AccountId, userSigner: Signer, signingPublicKey1: GrumpkinAddress, signingPublicKey2: GrumpkinAddress | undefined, fee: AssetValue): AddSigningKeyController;
    getMigrateAccountFees(assetId: number): Promise<AssetValue[]>;
    createMigrateAccountController(userId: AccountId, userSigner: Signer, newSigningPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, newAccountPrivateKey: Buffer | undefined, fee: AssetValue): MigrateAccountController;
    depositFundsToContract({ assetId, value }: AssetValue, from: EthAddress, provider?: EthereumProvider): Promise<TxHash>;
    getUserPendingDeposit(assetId: number, account: EthAddress): Promise<bigint>;
    getUserPendingFunds(assetId: number, account: EthAddress): Promise<bigint>;
    isContract(address: EthAddress): Promise<boolean>;
    validateSignature(publicOwner: EthAddress, signature: Buffer, signingData: Buffer): boolean;
    getTransactionReceipt(txHash: TxHash, interval?: number, timeout?: number): Promise<Receipt>;
    flushRollup(userId: AccountId, userSigner: Signer): Promise<void>;
    getSigningKeys(userId: AccountId): Promise<Buffer[]>;
    getPublicBalance(assetId: number, ethAddress: EthAddress): Promise<bigint>;
    getPublicBalanceAv(assetId: number, ethAddress: EthAddress): Promise<{
        assetId: number;
        value: bigint;
    }>;
    getBalances(userId: AccountId): Promise<AssetValue[]>;
    getBalance(assetId: number, userId: AccountId): Promise<bigint>;
    getBalanceAv(assetId: number, userId: AccountId): Promise<{
        assetId: number;
        value: bigint;
    }>;
    getFormattedBalance(assetId: number, userId: AccountId, symbol?: boolean, precision?: number): Promise<string>;
    getSpendableSum(assetId: number, userId: AccountId, excludePendingNotes?: boolean): Promise<bigint>;
    getSpendableSums(userId: AccountId, excludePendingNotes?: boolean): Promise<AssetValue[]>;
    getMaxSpendableValue(assetId: number, userId: AccountId, numNotes?: number, excludePendingNotes?: boolean): Promise<bigint>;
    getUserTxs(userId: AccountId): Promise<(UserAccountTx | UserDefiTx | import("../user_tx").UserDefiClaimTx | UserPaymentTx)[]>;
    getPaymentTxs(userId: AccountId): Promise<UserPaymentTx[]>;
    getAccountTxs(userId: AccountId): Promise<UserAccountTx[]>;
    getDefiTxs(userId: AccountId): Promise<UserDefiTx[]>;
    getRemoteUnsettledAccountTxs(): Promise<import("@aztec/barretenberg/rollup_provider").AccountTx[]>;
    getRemoteUnsettledPaymentTxs(): Promise<import("@aztec/barretenberg/rollup_provider").JoinSplitTx[]>;
    private getAccountFee;
}
```