import { makeTuple } from '@aztec/foundation/array';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, type Tuple, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import {
  ARCHIVE_HEIGHT,
  MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  PUBLIC_DATA_TREE_HEIGHT,
} from '../../constants.gen.js';
import { ClientIvcProof } from '../client_ivc_proof.js';
import { GlobalVariables } from '../global_variables.js';
import { KernelData } from '../kernel/kernel_data.js';
import { MembershipWitness } from '../membership_witness.js';
import { PartialStateReference } from '../partial_state_reference.js';
import { PublicDataHint } from '../public_data_hint.js';
import { type UInt32 } from '../shared.js';
import { PublicDataTreeLeaf, PublicDataTreeLeafPreimage } from '../trees/index.js';
import { AppendOnlyTreeSnapshot } from './append_only_tree_snapshot.js';
import { StateDiffHints } from './state_diff_hints.js';

/**
 * Data which is forwarded through the base rollup circuits unchanged.
 */
export class ConstantRollupData {
  constructor(
    /** Archive tree snapshot at the very beginning of the entire rollup. */
    public lastArchive: AppendOnlyTreeSnapshot,

    /**
     * Root of the verification key tree.
     */
    public vkTreeRoot: Fr,
    /**
     * Global variables for the block
     */
    public globalVariables: GlobalVariables,
  ) {}

  static from(fields: FieldsOf<ConstantRollupData>): ConstantRollupData {
    return new ConstantRollupData(...ConstantRollupData.getFields(fields));
  }

  static fromBuffer(buffer: Buffer | BufferReader): ConstantRollupData {
    const reader = BufferReader.asReader(buffer);
    return new ConstantRollupData(
      reader.readObject(AppendOnlyTreeSnapshot),
      Fr.fromBuffer(reader),
      reader.readObject(GlobalVariables),
    );
  }

  static getFields(fields: FieldsOf<ConstantRollupData>) {
    return [fields.lastArchive, fields.vkTreeRoot, fields.globalVariables] as const;
  }

  static empty() {
    return new ConstantRollupData(AppendOnlyTreeSnapshot.zero(), Fr.ZERO, GlobalVariables.empty());
  }

  toBuffer() {
    return serializeToBuffer(...ConstantRollupData.getFields(this));
  }
}

/**
 * Inputs to the base rollup circuit.
 */
export class BaseRollupInputs {
  constructor(
    /** Data of the 2 kernels that preceded this base rollup circuit. */
    public kernelData: KernelData,
    /** Partial state reference at the start of the rollup. */
    public start: PartialStateReference,
    /** Hints used while proving state diff validity. */
    public stateDiffHints: StateDiffHints,
    /** Public data read hint for accessing the balance of the fee payer. */
    public feePayerGasTokenBalanceReadHint: PublicDataHint,

    /**
     * The public data writes to be inserted in the tree, sorted high slot to low slot.
     */
    public sortedPublicDataWrites: Tuple<PublicDataTreeLeaf, typeof MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,

    /**
     * The indexes of the sorted public data writes to the original ones.
     */
    public sortedPublicDataWritesIndexes: Tuple<UInt32, typeof MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX>,
    /**
     * The public data writes which need to be updated to perform the batch insertion of the new public data writes.
     * See `StandardIndexedTree.batchInsert` function for more details.
     */
    public lowPublicDataWritesPreimages: Tuple<
      PublicDataTreeLeafPreimage,
      typeof MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,
    /**
     * Membership witnesses for the nullifiers which need to be updated to perform the batch insertion of the new
     * nullifiers.
     */
    public lowPublicDataWritesMembershipWitnesses: Tuple<
      MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
      typeof MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX
    >,

    /**
     * Membership witnesses of blocks referred by each of the 2 kernels.
     */
    public archiveRootMembershipWitness: MembershipWitness<typeof ARCHIVE_HEIGHT>,
    /**
     * Data which is not modified by the base rollup circuit.
     */
    public constants: ConstantRollupData,
  ) {}

  static from(fields: FieldsOf<BaseRollupInputs>): BaseRollupInputs {
    return new BaseRollupInputs(...BaseRollupInputs.getFields(fields));
  }

  static getFields(fields: FieldsOf<BaseRollupInputs>) {
    return [
      fields.kernelData,
      fields.start,
      fields.stateDiffHints,
      fields.feePayerGasTokenBalanceReadHint,
      fields.sortedPublicDataWrites,
      fields.sortedPublicDataWritesIndexes,
      fields.lowPublicDataWritesPreimages,
      fields.lowPublicDataWritesMembershipWitnesses,
      fields.archiveRootMembershipWitness,
      fields.constants,
    ] as const;
  }

  /**
   * Serializes the inputs to a buffer.
   * @returns The inputs serialized to a buffer.
   */
  toBuffer() {
    return serializeToBuffer(...BaseRollupInputs.getFields(this));
  }

  /**
   * Serializes the inputs to a hex string.
   * @returns The instance serialized to a hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes the inputs from a buffer.
   * @param buffer - The buffer to deserialize from.
   * @returns A new BaseRollupInputs instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): BaseRollupInputs {
    const reader = BufferReader.asReader(buffer);
    return new BaseRollupInputs(
      reader.readObject(KernelData),
      reader.readObject(PartialStateReference),
      reader.readObject(StateDiffHints),
      reader.readObject(PublicDataHint),
      reader.readArray(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataTreeLeaf),
      reader.readNumbers(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX),
      reader.readArray(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataTreeLeafPreimage),
      reader.readArray(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, {
        fromBuffer: buffer => MembershipWitness.fromBuffer(buffer, PUBLIC_DATA_TREE_HEIGHT),
      }),
      MembershipWitness.fromBuffer(reader, ARCHIVE_HEIGHT),
      reader.readObject(ConstantRollupData),
    );
  }

  /**
   * Deserializes the inputs from a hex string.
   * @param str - A hex string to deserialize from.
   * @returns A new BaseRollupInputs instance.
   */
  static fromString(str: string) {
    return BaseRollupInputs.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new BaseRollupInputs(
      KernelData.empty(),
      PartialStateReference.empty(),
      StateDiffHints.empty(),
      PublicDataHint.empty(),
      makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataTreeLeaf.empty),
      makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, () => 0),
      makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataTreeLeafPreimage.empty),
      makeTuple(MAX_TOTAL_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, () => MembershipWitness.empty(PUBLIC_DATA_TREE_HEIGHT)),
      MembershipWitness.empty(ARCHIVE_HEIGHT),
      ConstantRollupData.empty(),
    );
  }
}

export class TubeInputs {
  constructor(public clientIVCData: ClientIvcProof) {}

  static from(fields: FieldsOf<TubeInputs>): TubeInputs {
    return new TubeInputs(...TubeInputs.getFields(fields));
  }

  static getFields(fields: FieldsOf<TubeInputs>) {
    return [fields.clientIVCData] as const;
  }

  /**
   * Serializes the inputs to a buffer.
   * @returns The inputs serialized to a buffer.
   */
  toBuffer() {
    return serializeToBuffer(...TubeInputs.getFields(this));
  }

  /**
   * Serializes the inputs to a hex string.
   * @returns The instance serialized to a hex string.
   */
  toString() {
    return this.toBuffer().toString('hex');
  }

  /**
   * Deserializes the inputs from a buffer.
   * @param buffer - The buffer to deserialize from.
   * @returns A new TubeInputs instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TubeInputs {
    const reader = BufferReader.asReader(buffer);
    return new TubeInputs(reader.readObject(ClientIvcProof));
  }

  isEmpty(): boolean {
    return this.clientIVCData.isEmpty();
  }
  /**
   * Deserializes the inputs from a hex string.
   * @param str - A hex string to deserialize from.
   * @returns A new TubeInputs instance.
   */
  static fromString(str: string) {
    return TubeInputs.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new TubeInputs(ClientIvcProof.empty());
  }
}
