import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';
import { FieldsOf } from '../../utils/jsUtils.js';
import { serializeToBuffer } from '../../utils/serialize.js';
import { TxContext } from '../tx_context.js';

export class PrivateHistoricTreeRoots {
  constructor(
    public privateDataTreeRoot: Fr,
    public nullifierTreeRoot: Fr,
    public contractTreeRoot: Fr,
    public l1ToL2MessagesTreeRoot: Fr,
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
      fields.privateKernelVkTreeRoot,
    ] as const;
  }

  toBuffer() {
    return serializeToBuffer(...PrivateHistoricTreeRoots.getFields(this));
  }

  isEmpty() {
    return (
      this.privateDataTreeRoot.isZero() &&
      this.nullifierTreeRoot.isZero() &&
      this.contractTreeRoot.isZero() &&
      this.l1ToL2MessagesTreeRoot.isZero() &&
      this.privateKernelVkTreeRoot.isZero()
    );
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateHistoricTreeRoots {
    const reader = BufferReader.asReader(buffer);
    return new PrivateHistoricTreeRoots(
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
      reader.readFr(),
    );
  }

  static empty() {
    return new PrivateHistoricTreeRoots(Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }
}

export class CombinedHistoricTreeRoots {
  constructor(public readonly privateHistoricTreeRoots: PrivateHistoricTreeRoots) {}

  toBuffer() {
    return serializeToBuffer(this.privateHistoricTreeRoots);
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

export class CombinedConstantData {
  constructor(public historicTreeRoots: CombinedHistoricTreeRoots, public txContext: TxContext) {}

  toBuffer() {
    return serializeToBuffer(this.historicTreeRoots, this.txContext);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedConstantData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedConstantData(reader.readObject(CombinedHistoricTreeRoots), reader.readObject(TxContext));
  }

  static empty() {
    return new CombinedConstantData(CombinedHistoricTreeRoots.empty(), TxContext.empty());
  }
}
