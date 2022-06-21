```ts
export interface AssetValue {
    assetId: bigint;
    amount: bigint;
}
export declare enum AztecAssetType {
    ETH = 0,
    ERC20 = 1,
    VIRTUAL = 2,
    NOT_USED = 3
}
export declare enum SolidityType {
    uint8 = 0,
    uint16 = 1,
    uint24 = 2,
    uint32 = 3,
    uint64 = 4,
    boolean = 5,
    string = 6,
    bytes = 7
}
export interface AztecAsset {
    id: bigint;
    assetType: AztecAssetType;
    erc20Address: string;
}
export interface AuxDataConfig {
    start: number;
    length: number;
    description: string;
    solidityType: SolidityType;
}
export interface BridgeDataFieldGetters {
    getInteractionPresentValue?(interactionNonce: bigint): Promise<AssetValue[]>;
    getAuxData?(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset): Promise<bigint[]>;
    auxDataConfig: AuxDataConfig[];
    getExpectedOutput(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset, auxData: bigint, inputValue: bigint): Promise<bigint[]>;
    getExpiration?(interactionNonce: bigint): Promise<bigint>;
    hasFinalised?(interactionNonce: bigint): Promise<Boolean>;
    getExpectedYield?(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset, auxData: bigint, inputValue: bigint): Promise<number[]>;
    getMarketSize?(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset, auxData: bigint): Promise<AssetValue[]>;
    getCurrentYield?(interactionNonce: bigint): Promise<number[]>;
    lPAuxData?(data: bigint[]): Promise<bigint[]>;
}
```