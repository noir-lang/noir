import { BufferReader, Fr } from '@aztec/foundation';
import { serializeToBuffer } from '../../utils/serialize.js';
import { RollupTypes } from '../shared.js';
import { AggregationObject } from '../aggregation_object.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import { ConstantBaseRollupData } from './base_rollup.js';

/**
 * Output of the base rollup circuit
 */

export class BaseOrMergeRollupPublicInputs {
  constructor(
    public rollupType: RollupTypes,
    public rollupSubTreeHeight: Fr,
    public endAggregationObject: AggregationObject,
    public constants: ConstantBaseRollupData,

    public startPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,
    public endPrivateDataTreeSnapshot: AppendOnlyTreeSnapshot,

    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,

    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    public endContractTreeSnapshot: AppendOnlyTreeSnapshot,

    public startPublicDataTreeTreeSnapshot: AppendOnlyTreeSnapshot,
    public endPublicDataTreeTreeSnapshot: AppendOnlyTreeSnapshot,

    // Hashes (sha256), to make public inputs constant-sized (to then be unpacked on-chain). Length 2 for high and low
    public calldataHash: [Fr, Fr],
  ) {}

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param bufferReader - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): BaseOrMergeRollupPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new BaseOrMergeRollupPublicInputs(
      reader.readNumber(),
      reader.readFr(),
      reader.readObject(AggregationObject),
      reader.readObject(ConstantBaseRollupData),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readArray(2, Fr) as [Fr, Fr],
    );
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.rollupType,
      this.rollupSubTreeHeight,
      this.endAggregationObject,
      this.constants,

      this.startPrivateDataTreeSnapshot,
      this.endPrivateDataTreeSnapshot,

      this.startNullifierTreeSnapshot,
      this.endNullifierTreeSnapshot,

      this.startContractTreeSnapshot,
      this.endContractTreeSnapshot,

      this.startPublicDataTreeTreeSnapshot,
      this.endPublicDataTreeTreeSnapshot,

      this.calldataHash,
    );
  }
}
