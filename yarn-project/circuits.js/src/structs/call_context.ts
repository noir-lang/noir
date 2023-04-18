import { serializeToBuffer } from '../utils/serialize.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { FieldsOf } from '../utils/jsUtils.js';

/**
 * Call context.
 * @see abis/call_context.hpp
 */
export class CallContext {
  constructor(
    public msgSender: AztecAddress,
    public storageContractAddress: AztecAddress,
    public portalContractAddress: EthAddress,
    public isDelegateCall: boolean,
    public isStaticCall: boolean,
    public isContractDeployment: boolean,
  ) {}

  public static empty() {
    return new CallContext(AztecAddress.ZERO, AztecAddress.ZERO, EthAddress.ZERO, false, false, false);
  }

  static from(fields: FieldsOf<CallContext>): CallContext {
    return new CallContext(...CallContext.getFields(fields));
  }

  static getFields(fields: FieldsOf<CallContext>) {
    return [
      fields.msgSender,
      fields.storageContractAddress,
      fields.portalContractAddress,
      fields.isDelegateCall,
      fields.isStaticCall,
      fields.isContractDeployment,
    ] as const;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(...CallContext.getFields(this));
  }
}
