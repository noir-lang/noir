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
    /**
     * Address of the account which represents the entity who invoked the call.
     */
    public msgSender: AztecAddress,
    /**
     * The contract address against which all state changes will be stored. Not called `contractAddress` because during
     * delegate call the contract whose code is being executed may be different from the contract whose state is being
     * modified.
     */
    public storageContractAddress: AztecAddress,
    /**
     * Address of the portal contract to the storage contract.
     */
    public portalContractAddress: EthAddress,
    /**
     * Determines whether the call is a delegate call (see Ethereum's delegate call opcode for more information).
     */
    public isDelegateCall: boolean,
    /**
     * Determines whether the call is modifying state.
     */
    public isStaticCall: boolean,
    /**
     * Determines whether the call is a contract deployment.
     */
    public isContractDeployment: boolean,
  ) {}

  /**
   * Returns a new instance of CallContext with zero msg sender, storage contract address and portal contract address.
   * @returns A new instance of CallContext with zero msg sender, storage contract address and portal contract address.
   */
  public static empty(): CallContext {
    return new CallContext(AztecAddress.ZERO, AztecAddress.ZERO, EthAddress.ZERO, false, false, false);
  }

  isEmpty() {
    return this.msgSender.isZero() && this.storageContractAddress.isZero() && this.portalContractAddress.isZero();
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
