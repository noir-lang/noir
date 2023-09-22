import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';

/**
 * The string encoding used for serialising HistoricBlockData objects.
 */
const STRING_ENCODING: BufferEncoding = 'hex';

/**
 * Information about the tree roots used for both public and private kernels.
 */
export class HistoricBlockData {
  constructor(
    /**
     * Root of the private data tree at the time of when this information was assembled.
     */
    public privateDataTreeRoot: Fr,
    /**
     * Root of the nullifier tree at the time of when this information was assembled.
     */
    public nullifierTreeRoot: Fr,
    /**
     * Root of the contract tree at the time of when this information was assembled.
     */
    public contractTreeRoot: Fr,
    /**
     * Root of the l1 to l2 messages tree at the time of when this information was assembled.
     */
    public l1ToL2MessagesTreeRoot: Fr,
    /**
     * Root of the historic blocks tree at the time of when this information was assembled.
     */
    public blocksTreeRoot: Fr,
    /**
     * Root of the private kernel vk tree at the time of when this information was assembled.
     */
    public privateKernelVkTreeRoot: Fr, // future enhancement
    /**
     * Current public state tree hash.
     */
    public publicDataTreeRoot: Fr,
    /**
     * Previous globals hash, this value is used to recalculate the block hash.
     */
    public globalVariablesHash: Fr,
  ) {}

  static from(fields: FieldsOf<HistoricBlockData>) {
    return new HistoricBlockData(...HistoricBlockData.getFields(fields));
  }

  static random() {
    return new HistoricBlockData(
      Fr.random(),
      Fr.random(),
      Fr.random(),
      Fr.random(),
      Fr.random(),
      Fr.random(),
      Fr.random(),
      Fr.random(),
    );
  }

  static getFields(fields: FieldsOf<HistoricBlockData>) {
    return [
      fields.privateDataTreeRoot,
      fields.nullifierTreeRoot,
      fields.contractTreeRoot,
      fields.l1ToL2MessagesTreeRoot,
      fields.blocksTreeRoot,
      fields.privateKernelVkTreeRoot,
      fields.publicDataTreeRoot,
      fields.globalVariablesHash,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...HistoricBlockData.getFields(this));
  }

  toString() {
    // originally this was encoding as utf-8 (the default). This caused problems decoding back the data.
    return this.toBuffer().toString(STRING_ENCODING);
  }

  /**
   * Return the historic block data as an array of items in the order they are serialised in noir.
   * @returns Array of items in the order they are stored in the contract
   */
  toArray(): Fr[] {
    return [
      this.privateDataTreeRoot,
      this.nullifierTreeRoot,
      this.contractTreeRoot,
      this.l1ToL2MessagesTreeRoot,
      this.blocksTreeRoot, // Note private_kernel_vk_tree_root, is not included yet as
      // it is not present in noir,
      this.publicDataTreeRoot,
      this.globalVariablesHash,
    ];
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new HistoricBlockData(
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
    );
  }

  static fromString(str: string) {
    return HistoricBlockData.fromBuffer(Buffer.from(str, STRING_ENCODING));
  }

  isEmpty() {
    return (
      this.privateDataTreeRoot.isZero() &&
      this.nullifierTreeRoot.isZero() &&
      this.contractTreeRoot.isZero() &&
      this.l1ToL2MessagesTreeRoot.isZero() &&
      this.blocksTreeRoot.isZero() &&
      this.privateKernelVkTreeRoot.isZero() &&
      this.publicDataTreeRoot.isZero() &&
      this.globalVariablesHash.isZero()
    );
  }

  static empty() {
    return new HistoricBlockData(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }
}
