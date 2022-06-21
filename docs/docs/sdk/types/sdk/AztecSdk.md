```ts
class AztecSdk extends EventEmitter {
    private core;
    private blockchain;
    private provider;
    constructor(core: CoreSdkInterface, blockchain: ClientEthereumBlockchain, provider: EthereumProvider);
    run(): Promise<void>;
    destroy(): Promise<void>;
    awaitSynchronised(timeout?: number): Promise<void>;
    isUserSynching(userId: GrumpkinAddress): Promise<boolean>;
    awaitUserSynchronised(userId: GrumpkinAddress, timeout?: number): Promise<void>;
    awaitSettlement(txId: TxId, timeout?: number): Promise<void>;
    awaitDefiDepositCompletion(txId: TxId, timeout?: number): Promise<void>;
    awaitDefiFinalisation(txId: TxId, timeout?: number): Promise<void>;
    awaitDefiSettlement(txId: TxId, timeout?: number): Promise<void>;
    awaitAllUserTxsSettled(timeout?: number): Promise<void>;
    awaitAllUserTxsClaimed(timeout?: number): Promise<void>;
    getLocalStatus(): Promise<import("../core_sdk").SdkStatus>;
    getRemoteStatus(): Promise<import("@aztec/barretenberg/rollup_provider").RollupProviderStatus>;
    isAccountRegistered(accountPublicKey: GrumpkinAddress, includePending?: boolean): Promise<boolean>;
    isAliasRegistered(alias: string, includePending?: boolean): Promise<boolean>;
    isAliasRegisteredToAccount(accountPublicKey: GrumpkinAddress, alias: string, includePending?: boolean): Promise<boolean>;
    getAccountPublicKey(alias: string): Promise<GrumpkinAddress | undefined>;
    getTxFees(assetId: number): Promise<AssetValue[][]>;
    userExists(accountPublicKey: GrumpkinAddress): Promise<boolean>;
    addUser(accountPrivateKey: Buffer, noSync?: boolean): Promise<AztecSdkUser>;
    removeUser(userId: GrumpkinAddress): Promise<void>;
    /**
     * Returns a AztecSdkUser for a locally resolved user.
     */
    getUser(userId: GrumpkinAddress): Promise<AztecSdkUser>;
    getUserSyncedToRollup(userId: GrumpkinAddress): Promise<number>;
    getUsers(): Promise<GrumpkinAddress[]>;
    getAccountKeySigningData(): Buffer;
    getSpendingKeySigningData(): Buffer;
    generateAccountKeyPair(account: EthAddress, provider?: EthereumProvider): Promise<{
        publicKey: GrumpkinAddress;
        privateKey: Buffer;
    }>;
    generateSpendingKeyPair(account: EthAddress, provider?: EthereumProvider): Promise<{
        publicKey: GrumpkinAddress;
        privateKey: Buffer;
    }>;
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
    mint({ assetId, value }: AssetValue, account: EthAddress, provider?: EthereumProvider): Promise<TxHash>;
    setSupportedAsset(assetAddress: EthAddress, assetGasLimit?: number, options?: SendTxOptions): Promise<TxHash>;
    getBridgeAddressId(address: EthAddress, gasLimit?: number): number;
    setSupportedBridge(bridgeAddress: EthAddress, bridgeGasLimit?: number, options?: SendTxOptions): Promise<TxHash>;
    processAsyncDefiInteraction(interactionNonce: number, options?: SendTxOptions): Promise<TxHash>;
    getDepositFees(assetId: number): Promise<AssetValue[]>;
    getPendingDepositTxs(): Promise<import("@aztec/barretenberg/rollup_provider").DepositTx[]>;
    createDepositController(depositor: EthAddress, value: AssetValue, fee: AssetValue, recipient: GrumpkinAddress, recipientSpendingKeyRequired?: boolean, feePayer?: FeePayer, provider?: EthereumProvider): DepositController;
    getWithdrawFees(assetId: number, recipient?: EthAddress): Promise<AssetValue[]>;
    createWithdrawController(userId: GrumpkinAddress, userSigner: Signer, value: AssetValue, fee: AssetValue, to: EthAddress): WithdrawController;
    getTransferFees(assetId: number): Promise<AssetValue[]>;
    createTransferController(userId: GrumpkinAddress, userSigner: Signer, value: AssetValue, fee: AssetValue, recipient: GrumpkinAddress, recipientSpendingKeyRequired?: boolean): TransferController;
    getDefiFees(bridgeId: BridgeId, userId?: GrumpkinAddress, depositValue?: AssetValue): Promise<{
        value: bigint;
        assetId: number;
    }[]>;
    createDefiController(userId: GrumpkinAddress, userSigner: Signer, bridgeId: BridgeId, value: AssetValue, fee: AssetValue): DefiController;
    generateAccountRecoveryData(accountPublicKey: GrumpkinAddress, alias: string, trustedThirdPartyPublicKeys: GrumpkinAddress[]): Promise<RecoveryPayload[]>;
    getRegisterFees({ assetId, value: depositValue }: AssetValue): Promise<AssetValue[]>;
    createRegisterController(userId: GrumpkinAddress, alias: string, accountPrivateKey: Buffer, spendingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, deposit: AssetValue, fee: AssetValue, depositor: EthAddress, feePayer?: FeePayer, provider?: EthereumProvider): RegisterController;
    getRecoverAccountFees(assetId: number): Promise<{
        value: bigint;
        assetId: number;
    }[]>;
    createRecoverAccountController(alias: string, recoveryPayload: RecoveryPayload, deposit: AssetValue, fee: AssetValue, depositor: EthAddress, provider?: EthereumProvider): RecoverAccountController;
    getAddSpendingKeyFees(assetId: number): Promise<AssetValue[]>;
    createAddSpendingKeyController(userId: GrumpkinAddress, userSigner: Signer, alias: string, spendingPublicKey1: GrumpkinAddress, spendingPublicKey2: GrumpkinAddress | undefined, fee: AssetValue): AddSpendingKeyController;
    getMigrateAccountFees(assetId: number): Promise<AssetValue[]>;
    createMigrateAccountController(userId: GrumpkinAddress, userSigner: Signer, alias: string, newAccountPrivateKey: Buffer, newSpendingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, fee: AssetValue): MigrateAccountController;
    depositFundsToContract({ assetId, value }: AssetValue, from: EthAddress, provider?: EthereumProvider): Promise<TxHash>;
    getUserPendingDeposit(assetId: number, account: EthAddress): Promise<bigint>;
    getUserPendingFunds(assetId: number, account: EthAddress): Promise<bigint>;
    isContract(address: EthAddress): Promise<boolean>;
    validateSignature(publicOwner: EthAddress, signature: Buffer, signingData: Buffer): boolean;
    getTransactionReceipt(txHash: TxHash, timeout?: number, interval?: number): Promise<Receipt>;
    flushRollup(userId: GrumpkinAddress, userSigner: Signer): Promise<void>;
    getSpendingKeys(userId: GrumpkinAddress): Promise<Buffer[]>;
    getPublicBalance(ethAddress: EthAddress, assetId: number): Promise<{
        assetId: number;
        value: bigint;
    }>;
    getBalances(userId: GrumpkinAddress): Promise<AssetValue[]>;
    getBalance(userId: GrumpkinAddress, assetId: number): Promise<{
        assetId: number;
        value: bigint;
    }>;
    getFormattedBalance(userId: GrumpkinAddress, assetId: number, symbol?: boolean, precision?: number): Promise<string>;
    getSpendableSum(userId: GrumpkinAddress, assetId: number, spendingKeyRequired?: boolean, excludePendingNotes?: boolean): Promise<bigint>;
    getSpendableSums(userId: GrumpkinAddress, spendingKeyRequired?: boolean, excludePendingNotes?: boolean): Promise<AssetValue[]>;
    getMaxSpendableValue(userId: GrumpkinAddress, assetId: number, spendingKeyRequired?: boolean, excludePendingNotes?: boolean, numNotes?: number): Promise<bigint>;
    getUserTxs(userId: GrumpkinAddress): Promise<(UserAccountTx | UserDefiTx | import("../user_tx").UserDefiClaimTx | UserPaymentTx)[]>;
    getPaymentTxs(userId: GrumpkinAddress): Promise<UserPaymentTx[]>;
    getAccountTxs(userId: GrumpkinAddress): Promise<UserAccountTx[]>;
    getDefiTxs(userId: GrumpkinAddress): Promise<UserDefiTx[]>;
    private getTransactionFees;
    private getAccountFee;
}
```
