import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { GasSettings } from '../gas_settings.js';
import { Header } from '../header.js';
import { TxContext } from '../tx_context.js';

/**
 * Data that is constant/not modified by neither of the kernels.
 */
export class CombinedConstantData {
  constructor(
    /**
     * Header of a block whose state is used during execution (not the block the transaction is included in).
     */
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

    /** Gas limits and max prices for this transaction as set by the sender. */
    public gasSettings: GasSettings,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.historicalHeader, this.txContext, this.gasSettings);
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
      reader.readObject(GasSettings),
    );
  }

  static empty() {
    return new CombinedConstantData(Header.empty(), TxContext.empty(), GasSettings.empty());
  }
}
