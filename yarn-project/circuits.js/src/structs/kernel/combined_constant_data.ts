import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { TxContext } from '../tx_context.js';

/**
 * Encapsulates the roots of all the trees relevant for the kernel circuits.
 */
export class PrivateHistoricTreeRoots {
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
  ) {}

  static from(fields: FieldsOf<PrivateHistoricTreeRoots>): PrivateHistoricTreeRoots {
    return new PrivateHistoricTreeRoots(...PrivateHistoricTreeRoots.getFields(fields));
  }

  static getFields(fields: FieldsOf<PrivateHistoricTreeRoots>) {
    return [
      fields.privateDataTreeRoot,
      fields.nullifierTreeRoot,
      fields.contractTreeRoot,
      fields.l1ToL2MessagesTreeRoot,
      fields.blocksTreeRoot,
      fields.privateKernelVkTreeRoot,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...PrivateHistoricTreeRoots.getFields(this));
  }

  toString() {
    return this.toBuffer().toString();
  }

  isEmpty() {
    return (
      this.privateDataTreeRoot.isZero() &&
      this.nullifierTreeRoot.isZero() &&
      this.contractTreeRoot.isZero() &&
      this.l1ToL2MessagesTreeRoot.isZero() &&
      this.blocksTreeRoot.isZero() &&
      this.privateKernelVkTreeRoot.isZero()
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or reader to read from.
   * @returns A new instance of PrivateHistoricTreeRoots.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateHistoricTreeRoots {
    const reader = BufferReader.asReader(buffer);
    return new PrivateHistoricTreeRoots(
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
    );
  }

  static fromString(str: string): PrivateHistoricTreeRoots {
    return PrivateHistoricTreeRoots.fromBuffer(Buffer.from(str, 'hex'));
  }

  static empty() {
    return new PrivateHistoricTreeRoots(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }
}

/**
 * Information about the tree roots used for both public and private kernels.
 */
export class CombinedHistoricTreeRoots {
  constructor(
    /**
     * Root of the trees relevant for kernel circuits.
     */
    public readonly privateHistoricTreeRoots: PrivateHistoricTreeRoots,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.privateHistoricTreeRoots);
  }

  toString() {
    return this.toBuffer().toString();
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new CombinedHistoricTreeRoots(reader.readObject(PrivateHistoricTreeRoots));
  }

  isEmpty() {
    return this.privateHistoricTreeRoots.isEmpty();
  }

  static empty() {
    return new CombinedHistoricTreeRoots(PrivateHistoricTreeRoots.empty());
  }
}

/**
 * Data that is constant/not modified by neither of the kernels.
 */
export class CombinedConstantData {
  constructor(
    /**
     * Roots of the trees relevant for both kernel circuits.
     */
    public historicTreeRoots: CombinedHistoricTreeRoots,
    /**
     * Context of the transaction.
     */
    public txContext: TxContext,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.historicTreeRoots, this.txContext);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or buffer reader to read from.
   * @returns A new instance of CombinedConstantData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedConstantData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedConstantData(reader.readObject(CombinedHistoricTreeRoots), reader.readObject(TxContext));
  }

  static empty() {
    return new CombinedConstantData(CombinedHistoricTreeRoots.empty(), TxContext.empty());
  }
}
