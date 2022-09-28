```ts
export declare class AztecSdk extends EventEmitter {
    private core;
    private blockchain;
    private provider;
    private feeCalculator;
    private txValueCalculator;
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
    getTxFees(assetId: number, { feeSignificantFigures }?: {
        feeSignificantFigures?: number | undefined;
    }): Promise<AssetValue[][]>;
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
    mint({ assetId, value }: AssetValue, account: EthAddress, options?: SendTxOptions): Promise<TxHash>;
    setSupportedAsset(assetAddress: EthAddress, assetGasLimit?: number, options?: SendTxOptions): Promise<TxHash>;
    getBridgeAddressId(address: EthAddress, gasLimit?: number): number;
    setSupportedBridge(bridgeAddress: EthAddress, bridgeGasLimit?: number, options?: SendTxOptions): Promise<TxHash>;
    processAsyncDefiInteraction(interactionNonce: number, options?: SendTxOptions): Promise<TxHash>;
    getDepositFees(assetId: number, options?: {
        feeSignificantFigures?: number;
    }): Promise<AssetValue[]>;
    getPendingDepositTxs(): Promise<import("@aztec/barretenberg/rollup_provider").DepositTx[]>;
    createDepositController(depositor: EthAddress, assetValue: AssetValue, fee: AssetValue, recipient: GrumpkinAddress, recipientSpendingKeyRequired?: boolean, provider?: EthereumProvider): DepositController;
    getWithdrawFees(assetId: number, options?: GetFeesOptions & {
        recipient?: EthAddress;
        assetValue?: AssetValue;
    }): Promise<AssetValue[]>;
    getMaxWithdrawValue(userId: GrumpkinAddress, assetId: number, options?: GetMaxTxValueOptions & {
        recipient?: EthAddress;
    }): Promise<{
        assetId: number;
        value: bigint;
        fee: {
            assetId: number;
            value: bigint;
        };
    }>;
    createWithdrawController(userId: GrumpkinAddress, userSigner: Signer, assetValue: AssetValue, fee: AssetValue, to: EthAddress): WithdrawController;
    getTransferFees(assetId: number, options?: GetFeesOptions & {
        assetValue?: AssetValue;
    }): Promise<AssetValue[]>;
    getMaxTransferValue(userId: GrumpkinAddress, assetId: number, options?: GetMaxTxValueOptions): Promise<{
        assetId: number;
        value: bigint;
        fee: {
            assetId: number;
            value: bigint;
        };
    }>;
    createTransferController(userId: GrumpkinAddress, userSigner: Signer, assetValue: AssetValue, fee: AssetValue, recipient: GrumpkinAddress, recipientSpendingKeyRequired?: boolean): TransferController;
    getDefiFees(bridgeCallData: BridgeCallData, options?: GetFeesOptions & {
        assetValue?: AssetValue;
    }): Promise<AssetValue[]>;
    getMaxDefiValue(userId: GrumpkinAddress, bridgeCallData: BridgeCallData, options?: Omit<GetMaxTxValueOptions, 'txSettlementTime'> & {
        txSettlementTime?: DefiSettlementTime;
    }): Promise<{
        assetId: number;
        value: bigint;
        fee: {
            assetId: number;
            value: bigint;
        };
    }>;
    createDefiController(userId: GrumpkinAddress, userSigner: Signer, bridgeCallData: BridgeCallData, assetValue: AssetValue, fee: AssetValue): DefiController;
    generateAccountRecoveryData(accountPublicKey: GrumpkinAddress, alias: string, trustedThirdPartyPublicKeys: GrumpkinAddress[]): Promise<RecoveryPayload[]>;
    getRegisterFees(assetId: number, options?: {
        feeSignificantFigures?: number;
    }): Promise<AssetValue[]>;
    createRegisterController(userId: GrumpkinAddress, alias: string, accountPrivateKey: Buffer, spendingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, deposit: AssetValue, fee: AssetValue, depositor?: EthAddress, provider?: EthereumProvider): RegisterController;
    getRecoverAccountFees(assetId: number, options?: {
        feeSignificantFigures?: number;
    }): Promise<AssetValue[]>;
    createRecoverAccountController(recoveryPayload: RecoveryPayload, deposit: AssetValue, fee: AssetValue, depositor?: EthAddress, provider?: EthereumProvider): RecoverAccountController;
    getAddSpendingKeyFees(assetId: number, options?: {
        feeSignificantFigures?: number;
    }): Promise<{
        value: bigint;
        assetId: number;
    }[]>;
    createAddSpendingKeyController(userId: GrumpkinAddress, userSigner: Signer, spendingPublicKey1: GrumpkinAddress, spendingPublicKey2: GrumpkinAddress | undefined, fee: AssetValue): AddSpendingKeyController;
    getMigrateAccountFees(assetId: number, options?: {
        feeSignificantFigures?: number;
    }): Promise<{
        value: bigint;
        assetId: number;
    }[]>;
    createMigrateAccountController(userId: GrumpkinAddress, userSigner: Signer, newAccountPrivateKey: Buffer, newSpendingPublicKey: GrumpkinAddress, recoveryPublicKey: GrumpkinAddress | undefined, deposit: AssetValue, fee: AssetValue, depositor?: EthAddress, provider?: EthereumProvider): MigrateAccountController;
    getProofTxsFees(assetId: number, proofTxs: Tx[], options?: GetFeesOptions): Promise<{
        value: bigint;
        assetId: number;
    }[]>;
    createFeeController(userId: GrumpkinAddress, userSigner: Signer, proofTxs: Tx[], fee: AssetValue): FeeController;
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
}
```
