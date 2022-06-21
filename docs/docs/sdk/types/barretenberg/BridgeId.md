```ts
import { BitConfig } from './bit_config';
export declare class BridgeId {
    readonly addressId: number;
    readonly inputAssetIdA: number;
    readonly outputAssetIdA: number;
    readonly inputAssetIdB?: number | undefined;
    readonly outputAssetIdB?: number | undefined;
    readonly auxData: number;
    static ZERO: BridgeId;
    static ENCODED_LENGTH_IN_BYTES: number;
    readonly bitConfig: BitConfig;
    constructor(addressId: number, inputAssetIdA: number, outputAssetIdA: number, inputAssetIdB?: number | undefined, outputAssetIdB?: number | undefined, auxData?: number);
    static random(): BridgeId;
    static fromBigInt(val: bigint): BridgeId;
    static fromBuffer(buf: Buffer): BridgeId;
    static fromString(str: string): BridgeId;
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
    equals(id: BridgeId): boolean;
}
```