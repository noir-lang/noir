```ts
class AztecSdkUser {
    id: GrumpkinAddress;
    private sdk;
    constructor(id: GrumpkinAddress, sdk: AztecSdk);
    isSynching(): Promise<boolean>;
    awaitSynchronised(timeout?: number): Promise<void>;
    getSyncedToRollup(): Promise<number>;
    getSpendingKeys(): Promise<Buffer[]>;
    getBalance(assetId: number): Promise<{
        assetId: number;
        value: bigint;
    }>;
    getSpendableSum(assetId: number, spendingKeyRequired?: boolean, excludePendingNotes?: boolean): Promise<bigint>;
    getSpendableSums(spendingKeyRequired?: boolean, excludePendingNotes?: boolean): Promise<import("@aztec/barretenberg/asset").AssetValue[]>;
    getMaxSpendableValue(assetId: number, spendingKeyRequired?: boolean, excludePendingNotes?: boolean, numNotes?: number): Promise<bigint>;
    getTxs(): Promise<(import("..").UserAccountTx | import("..").UserDefiTx | import("..").UserDefiClaimTx | import("..").UserPaymentTx)[]>;
    getPaymentTxs(): Promise<import("..").UserPaymentTx[]>;
    getAccountTxs(): Promise<import("..").UserAccountTx[]>;
    getDefiTxs(): Promise<import("..").UserDefiTx[]>;
}
```
