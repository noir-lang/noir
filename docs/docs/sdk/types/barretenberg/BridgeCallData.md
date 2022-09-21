```ts
import { BitConfig } from './bit_config';
export declare class BridgeCallData {
    readonly bridgeAddressId: number;
    readonly inputAssetIdA: number;
    readonly outputAssetIdA: number;
    readonly inputAssetIdB?: number | undefined;
    readonly outputAssetIdB?: number | undefined;
    readonly auxData: number;
    static ZERO: BridgeCallData;
    static ENCODED_LENGTH_IN_BYTES: number;
    readonly bitConfig: BitConfig;
    constructor(bridgeAddressId: number, inputAssetIdA: number, outputAssetIdA: number, inputAssetIdB?: number | undefined, outputAssetIdB?: number | undefined, auxData?: number);
    static random(): BridgeCallData;
    static fromBigInt(val: bigint): BridgeCallData;
    static fromBuffer(buf: Buffer): BridgeCallData;
    static fromString(str: string): BridgeCallData;
    get firstInputVirtual(): boolean;
    get secondInputVirtual(): boolean;
    get firstOutputVirtual(): boolean;
    get secondOutputVirtual(): boolean;
    get secondInputInUse(): boolean;
    get secondOutputInUse(): boolean;
    get numInputAssets(): 1 | 2;
    get numOutputAssets(): 1 | 2;
    toBigInt(): bigint;
    toBuffer(): Buffer;
    toString(): string;
    equals(id: BridgeCallData): boolean;
}
```