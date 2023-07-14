import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { FunctionData } from './function_data.js';
import { TxContext } from './tx_context.js';

/**
 * Transaction request.
 * @see cpp/src/aztec3/circuits/abis/tx_request.hpp.
 */
export class TxRequest {
  constructor(
    /**
     * Sender.
     */
    public origin: AztecAddress,
    /**
     * Function data representing the function to call.
     */
    public functionData: FunctionData,
    /**
     * Pedersen hash of function arguments.
     */
    public argsHash: Fr,
    /**
     * Transaction context.
     */
    public txContext: TxContext,
  ) {}

  static getFields(fields: FieldsOf<TxRequest>) {
    return [fields.origin, fields.functionData, fields.argsHash, fields.txContext] as const;
  }

  static from(fields: FieldsOf<TxRequest>): TxRequest {
    return new TxRequest(...TxRequest.getFields(fields));
  }

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    const fields = TxRequest.getFields(this);
    return serializeToBuffer([...fields]);
  }

  /**
   * Deserializes from a buffer or reader, corresponding to a write in cpp.
   * @param buffer - Buffer to read from.
   * @returns The deserialised TxRequest object.
   */
  static fromBuffer(buffer: Buffer | BufferReader): TxRequest {
    const reader = BufferReader.asReader(buffer);
    return new TxRequest(
      reader.readObject(AztecAddress),
      reader.readObject(FunctionData),
      reader.readFr(),
      reader.readObject(TxContext),
    );
  }
}
