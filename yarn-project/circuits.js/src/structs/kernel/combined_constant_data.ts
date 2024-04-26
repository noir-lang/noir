import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { GlobalVariables } from '../global_variables.js';
import { Header } from '../header.js';
import { TxContext } from '../tx_context.js';

/**
 * Data that is constant/not modified by neither of the kernels.
 */
export class CombinedConstantData {
  constructor(
    /** Header of a block whose state is used during execution (not the block the transaction is included in). */
    public historicalHeader: Header,
    /**
     * Context of the transaction.
     *
     * Note: `chainId` and `version` in txContext are not redundant to the values in
     * self.historical_header.global_variables because they can be different in case of a protocol upgrade. In such
     * a situation we could be using header from a block before the upgrade took place but be using the updated
     * protocol to execute and prove the transaction.
     */
    public txContext: TxContext,

    /** Present when output by a public kernel, empty otherwise. */
    public globalVariables: GlobalVariables,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.historicalHeader, this.txContext, this.globalVariables);
  }

  static from({ historicalHeader, txContext, globalVariables }: FieldsOf<CombinedConstantData>): CombinedConstantData {
    return new CombinedConstantData(historicalHeader, txContext, globalVariables);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer or buffer reader to read from.
   * @returns A new instance of CombinedConstantData.
   */
  static fromBuffer(buffer: Buffer | BufferReader): CombinedConstantData {
    const reader = BufferReader.asReader(buffer);
    return new CombinedConstantData(
      reader.readObject(Header),
      reader.readObject(TxContext),
      reader.readObject(GlobalVariables),
    );
  }

  static empty() {
    return new CombinedConstantData(Header.empty(), TxContext.empty(), GlobalVariables.empty());
  }
}
