import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';
import { BufferReader } from '@aztec/foundation/serialize';

import { FieldsOf } from '../utils/jsUtils.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { Fr, FunctionSelector } from './index.js';

/**
 * Call context.
 * @see abis/call_context.hpp
 */
export class CallContext {
  /**
   * Address of the portal contract to the storage contract.
   */
  public portalContractAddress: EthAddress;
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
     * Union type is a kludge until C++ has an eth address type.
     */
    portalContractAddress: EthAddress | Fr,
    /**
     * Function selector of the function being called.
     */
    public functionSelector: FunctionSelector,
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
  ) {
    this.portalContractAddress =
      portalContractAddress instanceof EthAddress ? portalContractAddress : EthAddress.fromField(portalContractAddress);
  }

  /**
   * Returns a new instance of CallContext with zero msg sender, storage contract address and portal contract address.
   * @returns A new instance of CallContext with zero msg sender, storage contract address and portal contract address.
   */
  public static empty(): CallContext {
    return new CallContext(
      AztecAddress.ZERO,
      AztecAddress.ZERO,
      Fr.ZERO,
      FunctionSelector.empty(),
      false,
      false,
      false,
    );
  }

  isEmpty() {
    return (
      this.msgSender.isZero() &&
      this.storageContractAddress.isZero() &&
      this.portalContractAddress.isZero() &&
      this.functionSelector.isEmpty()
    );
  }

  static from(fields: FieldsOf<CallContext>): CallContext {
    return new CallContext(...CallContext.getFields(fields));
  }

  static getFields(fields: FieldsOf<CallContext>) {
    return [
      fields.msgSender,
      fields.storageContractAddress,
      fields.portalContractAddress,
      fields.functionSelector,
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

  /**
   * Deserialise this from a buffer.
   * @param buffer - The bufferable type from which to deserialise.
   * @returns The deserialised instance of PublicCallRequest.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new CallContext(
      new AztecAddress(reader.readBytes(32)),
      new AztecAddress(reader.readBytes(32)),
      new EthAddress(reader.readBytes(32)),
      FunctionSelector.fromBuffer(reader.readBytes(4)),
      reader.readBoolean(),
      reader.readBoolean(),
      reader.readBoolean(),
    );
  }
}
