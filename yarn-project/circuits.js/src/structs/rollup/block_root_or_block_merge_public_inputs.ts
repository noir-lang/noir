import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { GlobalVariables } from '../global_variables.js';
import { EthAddress } from '../index.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';

/**
 * Output of the block root and block merge rollup circuits.
 */
export class BlockRootOrBlockMergePublicInputs {
  constructor(
    /**
     * Archive tree immediately before this block range.
     */
    public previousArchive: AppendOnlyTreeSnapshot,
    /**
     * Archive tree after adding this block range.
     */
    public newArchive: AppendOnlyTreeSnapshot,
    /**
     * Identifier of the previous block before the range.
     */
    public previousBlockHash: Fr,
    /**
     * Identifier of the last block in the range.
     */
    public endBlockHash: Fr,
    /**
     * Global variables for the first block in the range.
     */
    public startGlobalVariables: GlobalVariables,
    /**
     * Global variables for the last block in the range.
     */
    public endGlobalVariables: GlobalVariables,
    /**
     * SHA256 hash of outhash. Used to make public inputs constant-sized (to then be unpacked on-chain).
     * Note: Truncated to 31 bytes to fit in Fr.
     */
    public outHash: Fr,
    /**
     * The summed `transaction_fee`s and recipients of the constituent blocks.
     */
    public fees: Tuple<FeeRecipient, 32>,
    /**
     * Root of the verification key tree.
     */
    public vkTreeRoot: Fr,
    /**
     * TODO(#7346): Temporarily added prover_id while we verify block-root proofs on L1
     */
    public proverId: Fr,
  ) {}

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized public inputs.
   */
  static fromBuffer(buffer: Buffer | BufferReader): BlockRootOrBlockMergePublicInputs {
    const reader = BufferReader.asReader(buffer);
    return new BlockRootOrBlockMergePublicInputs(
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readObject(GlobalVariables),
      reader.readObject(GlobalVariables),
      Fr.fromBuffer(reader),
      reader.readArray(32, FeeRecipient),
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
      this.previousArchive,
      this.newArchive,
      this.previousBlockHash,
      this.endBlockHash,
      this.startGlobalVariables,
      this.endGlobalVariables,
      this.outHash,
      this.fees,
      this.vkTreeRoot,
      this.proverId,
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
    return BlockRootOrBlockMergePublicInputs.fromBuffer(Buffer.from(str, 'hex'));
  }
}

export class FeeRecipient {
  constructor(public recipient: EthAddress, public value: Fr) {}

  static fromBuffer(buffer: Buffer | BufferReader): FeeRecipient {
    const reader = BufferReader.asReader(buffer);
    return new FeeRecipient(reader.readObject(EthAddress), Fr.fromBuffer(reader));
  }

  toBuffer() {
    return serializeToBuffer(this.recipient, this.value);
  }

  static getFields(fields: FieldsOf<FeeRecipient>) {
    return [fields.recipient, fields.value] as const;
  }

  toFields() {
    return serializeToFields(...FeeRecipient.getFields(this));
  }
}
