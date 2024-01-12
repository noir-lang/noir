import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

/**
 * The string encoding used for serializing BlockHeader objects.
 */
const STRING_ENCODING: BufferEncoding = 'hex';

/**
 * Information about the tree roots used for both public and private kernels.
 */
// TODO(#3937): Nuke this
export class BlockHeader {
  constructor(
    /**
     * Root of the note hash tree at the time of when this information was assembled.
     */
    public noteHashTreeRoot: Fr,
    /**
     * Root of the nullifier tree at the time of when this information was assembled.
     */
    public nullifierTreeRoot: Fr,
    /**
     * Root of the contract tree at the time of when this information was assembled.
     */
    public contractTreeRoot: Fr,
    /**
     * Root of the l1 to l2 message tree at the time of when this information was assembled.
     */
    public l1ToL2MessageTreeRoot: Fr,
    /**
     * Root of the state roots tree (archive) at the block prior to when this information was assembled.
     */
    public archiveRoot: Fr,
    /**
     * Root of the private kernel vk tree at the time of when this information was assembled.
     */
    public privateKernelVkTreeRoot: Fr, // TODO(#3441) future enhancement
    /**
     * Current public state tree hash.
     */
    public publicDataTreeRoot: Fr,
    /**
     * Previous globals hash, this value is used to recalculate the block hash.
     */
    public globalVariablesHash: Fr,
  ) {}

  static from(fields: FieldsOf<BlockHeader>) {
    return new BlockHeader(...BlockHeader.getFields(fields));
  }

  static random() {
    return new BlockHeader(
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

  static getFields(fields: FieldsOf<BlockHeader>) {
    return [
      fields.noteHashTreeRoot,
      fields.nullifierTreeRoot,
      fields.contractTreeRoot,
      fields.l1ToL2MessageTreeRoot,
      fields.archiveRoot,
      fields.privateKernelVkTreeRoot,
      fields.publicDataTreeRoot,
      fields.globalVariablesHash,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...BlockHeader.getFields(this));
  }

  toString() {
    // originally this was encoding as utf-8 (the default). This caused problems decoding back the data.
    return this.toBuffer().toString(STRING_ENCODING);
  }

  /**
   * Return the block header as an array of items in the order they are serialized in noir.
   * @returns Array of items in the order they are stored in the contract
   */
  toArray(): Fr[] {
    return [
      this.noteHashTreeRoot,
      this.nullifierTreeRoot,
      this.contractTreeRoot,
      this.l1ToL2MessageTreeRoot,
      this.archiveRoot, // TODO(#3441) Note private_kernel_vk_tree_root, is not included yet as
      // it is not present in noir,
      this.publicDataTreeRoot,
      this.globalVariablesHash,
    ];
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new BlockHeader(
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
    );
  }

  static fromString(str: string) {
    return BlockHeader.fromBuffer(Buffer.from(str, STRING_ENCODING));
  }

  isEmpty() {
    return (
      this.noteHashTreeRoot.isZero() &&
      this.nullifierTreeRoot.isZero() &&
      this.contractTreeRoot.isZero() &&
      this.l1ToL2MessageTreeRoot.isZero() &&
      this.archiveRoot.isZero() &&
      this.privateKernelVkTreeRoot.isZero() &&
      this.publicDataTreeRoot.isZero() &&
      this.globalVariablesHash.isZero()
    );
  }

  static empty() {
    return new BlockHeader(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }
}
