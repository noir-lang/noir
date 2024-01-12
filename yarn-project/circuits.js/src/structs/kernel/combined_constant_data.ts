import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { TxContext } from '../tx_context.js';
import { BlockHeader } from './block_header.js';

/**
 * Data that is constant/not modified by neither of the kernels.
 */
export class CombinedConstantData {
  constructor(
    /**
     * Roots of the trees relevant for both kernel circuits.
     */
    public blockHeader: BlockHeader,
    /**
     * Context of the transaction.
     */
    public txContext: TxContext,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.blockHeader, this.txContext);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or buffer reader to read from.
   * @returns A new instance of CombinedConstantData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedConstantData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedConstantData(reader.readObject(BlockHeader), reader.readObject(TxContext));
  }

  static empty() {
    return new CombinedConstantData(BlockHeader.empty(), TxContext.empty());
  }
}
