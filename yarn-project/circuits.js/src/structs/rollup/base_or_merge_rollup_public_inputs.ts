import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

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
     * Number of txs in this rollup.
     */
    public numTxs: number,
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

    /**
     * The summed `transaction_fee` of the constituent transactions.
     */
    public accumulatedFees: Fr,
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
      reader.readNumber(),
      reader.readObject(ConstantRollupData),
      reader.readObject(PartialStateReference),
      reader.readObject(PartialStateReference),
      //TODO check
      Fr.fromBuffer(reader),
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
      this.numTxs,
      this.constants,

      this.start,
      this.end,

      this.txsEffectsHash,
      this.outHash,

      this.accumulatedFees,
    );
  }

  /**
   * Serialize this as a hex string.
   * @returns - The hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes from a hex string.
   * @param str - A hex string to deserialize from.
   * @returns A new BaseOrMergeRollupPublicInputs instance.
   */
  static fromString(str: string) {
    return BaseOrMergeRollupPublicInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}
