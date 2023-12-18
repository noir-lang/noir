import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { NUM_FIELDS_PER_SHA256 } from '../../constants.gen.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { AggregationObject } from '../aggregation_object.js';
import { RollupTypes } from '../shared.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import { ConstantRollupData } from './base_rollup.js';

/**
 * Output of the base and merge rollup circuits.
 */
export class BaseOrMergeRollupPublicInputs {
  constructor(
    /**
     * Specifies from which type of rollup circuit these inputs are from.
     */
    public rollupType: RollupTypes,
    /**
     * Rollup sub tree height.
     * Note 1: Base rollup circuit always have a sub tree height of 0.
     * Note 2: With each merge, the sub tree height increases by 1.
     */
    public rollupSubtreeHeight: Fr,
    /**
     * Native aggregation state at the end of the rollup circuit.
     */
    public endAggregationObject: AggregationObject,
    /**
     * Data which is forwarded through the rollup circuits unchanged.
     */
    public constants: ConstantRollupData,

    /**
     * Snapshot of the note hash tree at the start of the rollup circuit.
     */
    public startNoteHashTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the note hash tree at the end of the rollup circuit.
     */
    public endNoteHashTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the nullifier tree at the start of the rollup circuit.
     */
    public startNullifierTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the nullifier tree at the end of the rollup circuit.
     */
    public endNullifierTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the contract tree at the start of the rollup circuit.
     */
    public startContractTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the contract tree at the end of the rollup circuit.
     */
    public endContractTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * Snapshot of the public data tree at the start of the rollup circuit.
     */
    public startPublicDataTreeSnapshot: AppendOnlyTreeSnapshot,
    /**
     * Snapshot of the public data tree at the end of the rollup circuit.
     */
    public endPublicDataTreeSnapshot: AppendOnlyTreeSnapshot,

    /**
     * SHA256 hashes of calldata. Used to make public inputs constant-sized (to then be unpacked on-chain).
     * Note: Length 2 for high and low.
     */
    public calldataHash: [Fr, Fr],
  ) {}

  /**
   * Deserializes from a buffer or reader.
   * Note: Corresponds to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized public inputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): BaseOrMergeRollupPublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new BaseOrMergeRollupPublicInputs(
      reader.readNumber(),
      Fr.fromBuffer(reader),
      reader.readObject(AggregationObject),
      reader.readObject(ConstantRollupData),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readArray(NUM_FIELDS_PER_SHA256, Fr) as [Fr, Fr],
    );
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.rollupType,
      this.rollupSubtreeHeight,
      this.endAggregationObject,
      this.constants,

      this.startNoteHashTreeSnapshot,
      this.endNoteHashTreeSnapshot,

      this.startNullifierTreeSnapshot,
      this.endNullifierTreeSnapshot,

      this.startContractTreeSnapshot,
      this.endContractTreeSnapshot,

      this.startPublicDataTreeSnapshot,
      this.endPublicDataTreeSnapshot,

      this.calldataHash,
    );
  }
}
