```ts
import { AssetValue, AuxDataConfig, AztecAsset, BridgeDataFieldGetters } from "../bridge-data";
import { EthereumProvider } from "@aztec/barretenberg/blockchain";
import { EthAddress } from "@aztec/barretenberg/address";
export declare class LidoBridgeData implements BridgeDataFieldGetters {
    private wstETHContract;
    private lidoOracleContract;
    private curvePoolContract;
    scalingFactor: bigint;
    private constructor();
    static create(provider: EthereumProvider, wstEthAddress: EthAddress, lidoOracleAddress: EthAddress, curvePoolAddress: EthAddress): LidoBridgeData;
    auxDataConfig: AuxDataConfig[];
    getInteractionPresentValue(interactionNonce: bigint): Promise<AssetValue[]>;
    getAuxData(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset): Promise<bigint[]>;
    getExpectedOutput(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset, auxData: bigint, inputValue: bigint): Promise<bigint[]>;
    getExpectedYield(inputAssetA: AztecAsset, inputAssetB: AztecAsset, outputAssetA: AztecAsset, outputAssetB: AztecAsset, auxData: bigint, precision: bigint): Promise<number[]>;
    getMarketS
```