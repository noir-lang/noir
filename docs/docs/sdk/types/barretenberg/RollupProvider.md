```ts
interface RollupProvider extends BlockSource {
    sendTxs(txs: Tx[]): Promise<TxId[]>;
    getStatus(): Promise<RollupProviderStatus>;
    getTxFees(assetId: number): Promise<AssetValue[][]>;
    getDefiFees(bridgeId: BridgeId): Promise<AssetValue[]>;
    getPendingTxs(): Promise<PendingTx[]>;
    getPendingNoteNullifiers(): Promise<Buffer[]>;
    getPendingDepositTxs(): Promise<DepositTx[]>;
    clientLog(msg: any): Promise<void>;
    getInitialWorldState(): Promise<InitialWorldState>;
    isAccountRegistered(accountPublicKey: GrumpkinAddress): Promise<boolean>;
    isAliasRegistered(alias: string): Promise<boolean>;
    isAliasRegisteredToAccount(accountPublicKey: GrumpkinAddress, alias: string): Promise<boolean>;
}
```