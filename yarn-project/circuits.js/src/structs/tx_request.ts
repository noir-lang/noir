import { AztecAddress } from '@aztec/foundation/aztec-address';
import { keccak224 } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { FieldsOf, assertMemberLength } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { ARGS_LENGTH } from './constants.js';
import { FunctionData } from './function_data.js';
import { EcdsaSignature } from './shared.js';
import { TxContext } from './tx_context.js';

/**
 * Signed transaction request.
 * @see cpp/src/aztec3/circuits/abis/signed_tx_request.hpp.
 */
export class SignedTxRequest {
  constructor(
    /**
     * Transaction request.
     */
    public txRequest: TxRequest,
    /**
     * Signature.
     */
    public signature: EcdsaSignature,
  ) {}

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.txRequest, this.signature);
  }
}

/**
 * Transaction request.
 * @see cpp/src/aztec3/circuits/abis/tx_request.hpp.
 */
export class TxRequest {
  /**
   * Map from args index to list of fields. If an arg index is set here, then the corresponding arg
   * should be the hash of the unpacked args in this map. Used to bypass the MAX_ARGS limit.
   */
  private packedArgs: Map<number, Fr[]> = new Map();

  constructor(
    /**
     * Sender.
     */
    public from: AztecAddress,
    /**
     * Target.
     */
    public to: AztecAddress,
    /**
     * Function data representing the function to call.
     */
    public functionData: FunctionData,
    /**
     * Function arguments.
     */
    public args: Fr[],
    /**
     * Tx nonce.
     */
    public nonce: Fr,
    /**
     * Transaction context.
     */
    public txContext: TxContext,
    /**
     * Chain ID of the transaction. Here for replay protection.
     */
    public chainId: Fr,
  ) {
    assertMemberLength(this, 'args', ARGS_LENGTH);
  }

  setPackedArg(index: number, unpackedArgs: Fr[]) {
    // TODO: What hash flavor to use here? Who'll need to validate it?
    const hashed = Fr.fromBuffer(keccak224(Buffer.concat(unpackedArgs.map(fr => fr.toBuffer()))));
    this.args[index] = hashed;
    this.packedArgs.set(index, unpackedArgs);
  }

  getExpandedArgs(): Fr[] {
    const args = [];
    for (let i = 0; i < this.args.length; i++) {
      if (this.packedArgs.has(i)) {
        args.push(...this.packedArgs.get(i)!);
      } else {
        args.push(this.args[i]);
      }
    }
    return args;
  }

  static getFields(fields: FieldsOf<TxRequest>) {
    return [
      fields.from,
      fields.to,
      fields.functionData,
      fields.args,
      fields.nonce,
      fields.txContext,
      fields.chainId,
    ] as const;
  }

  static from(fields: FieldsOf<TxRequest>): TxRequest {
    return new TxRequest(...TxRequest.getFields(fields));
  }

  /**
   * Serialize as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(...TxRequest.getFields(this));
  }
}
