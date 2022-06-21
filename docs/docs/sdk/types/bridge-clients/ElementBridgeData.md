```ts
export declare type BatchSwapStep = {
    poolId: string;
    assetInIndex: number;
    assetOutIndex: number;
    amount: string;
    userData: string;
};
export declare enum SwapType {
    SwapExactIn = 0,
    SwapExactOut = 1
}
export declare type FundManagement = {
    sender: string;
    recipient: string;
    fromInternalBalance: boolean;
    toInternalBalance: boolean;
};
export declare type ChainProperties = {
    eventBatchSize: number;
};
export declare class ElementBridgeData implements BridgeDataFieldGetters {
    private elementBridgeContract;
    private balancerContract;
    private rollupContract;
    private chainProperties;
    scalingFactor: bigint;
    private interactionBlockNumbers;
    private constructor();
    static create(provider: EthereumProvider, elementBridgeAddress: EthAddress, balancerAddress: EthAddress, rollupContractAddress: EthAddress, chainProperties?: ChainProperties): ElementBridgeData;
    private storeEventBlocks;
    private getCurrentBlock;
    private findDefiEventForNonce;
    getInteractionPresentValue(interactionNonce: bigint): Promise<AssetValue[]>;
    getCurrentYield(interactionNonce: bigint): Promise<number[]>;
    getAuxData(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset): Promise<bigint[]>;
    auxDataConfig: AuxDataConfig[];
    getExpectedOutput(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset, auxData: bigint, precision: bigint): Promise<bigint[]>;
    getExpectedYield(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset, auxData: bigint, precision: bigint): Promise<number[]>;
    getMarketSize(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset, auxData: bigint): Promise<AssetValue[]>;
    getExpiration(interactionNonce: bigint): Promise<bigint>;
    hasFinalised(interactionNonce: bigint): Promise<Boolean>;
}
```