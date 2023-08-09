import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';

/**
 * Information about the tree roots used for both public and private kernels.
 */
export class ConstantHistoricBlockData {
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
    public readonly publicDataTreeRoot: Fr,
    /**
     * Previous globals hash, this value is used to recalculate the block hash.
     */
    public readonly prevGlobalVariablesHash: Fr,
  ) {}

  static from(fields: FieldsOf<ConstantHistoricBlockData>) {
    return new ConstantHistoricBlockData(...ConstantHistoricBlockData.getFields(fields));
  }

  static getFields(fields: FieldsOf<ConstantHistoricBlockData>) {
    return [
      fields.privateDataTreeRoot,
      fields.nullifierTreeRoot,
      fields.contractTreeRoot,
      fields.l1ToL2MessagesTreeRoot,
      fields.blocksTreeRoot,
      fields.privateKernelVkTreeRoot,
      fields.publicDataTreeRoot,
      fields.prevGlobalVariablesHash,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...ConstantHistoricBlockData.getFields(this));
  }

  toString() {
    return this.toBuffer().toString();
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ConstantHistoricBlockData(
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

  isEmpty() {
    return (
      this.privateDataTreeRoot.isZero() &&
      this.nullifierTreeRoot.isZero() &&
      this.contractTreeRoot.isZero() &&
      this.l1ToL2MessagesTreeRoot.isZero() &&
      this.blocksTreeRoot.isZero() &&
      this.privateKernelVkTreeRoot.isZero() &&
      this.publicDataTreeRoot.isZero() &&
      this.prevGlobalVariablesHash.isZero()
    );
  }

  static empty() {
    return new ConstantHistoricBlockData(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }
}
