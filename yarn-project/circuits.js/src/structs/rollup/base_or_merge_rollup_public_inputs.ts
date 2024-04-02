import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { AggregationObject } from '../aggregation_object.js';
import { PartialStateReference } from '../partial_state_reference.js';
import { type RollupTypes } from '../shared.js';
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
     * SHA256 hash of transactions effects. Used to make public inputs constant-sized (to then be unpacked on-chain).
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public txsEffectsHash: Fr,
    /**
     * SHA256 hash of outhash. Used to make public inputs constant-sized (to then be unpacked on-chain).
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public outHash: Fr,
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
      //TODO check
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
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

      this.txsEffectsHash,
      this.outHash,
    );
  }
}
