import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { Header } from '../header.js';
import { TxContext } from '../tx_context.js';

/**
 * Data that is constant/not modified by neither of the kernels.
 */
export class CombinedConstantData {
  constructor(
    /**
     * Roots of the trees relevant for both kernel circuits.
     */
    public header: Header,
    /**
     * Context of the transaction.
     */
    public txContext: TxContext,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.header, this.txContext);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or buffer reader to read from.
   * @returns A new instance of CombinedConstantData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedConstantData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedConstantData(reader.readObject(Header), reader.readObject(TxContext));
  }

  static empty() {
    return new CombinedConstantData(Header.empty(), TxContext.empty());
  }
}
