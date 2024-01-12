import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { NUM_FIELDS_PER_SHA256 } from '../../constants.gen.js';
import { AggregationObject } from '../aggregation_object.js';
import { PartialStateReference } from '../partial_state_reference.js';
import { RollupTypes } from '../shared.js';
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
    public aggregationObject: AggregationObject,
    /**
     * Data which is forwarded through the rollup circuits unchanged.
     */
    public constants: ConstantRollupData,
    /**
     * Partial state reference at the start of the rollup circuit.
     */
    public start: PartialStateReference,
    /**
     * Partial state reference at the end of the rollup circuit.
     */
    public end: PartialStateReference,
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
      reader.readObject(PartialStateReference),
      reader.readObject(PartialStateReference),
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
      this.aggregationObject,
      this.constants,

      this.start,
      this.end,

      this.calldataHash,
    );
  }
}
