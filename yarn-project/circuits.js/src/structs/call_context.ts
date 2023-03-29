import { serializeToBuffer } from '../utils/serialize.js';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { EthAddress } from '@aztec/foundation/eth-address';

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
  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer(): Buffer {
    return serializeToBuffer(
      this.msgSender,
      this.storageContractAddress,
      this.portalContractAddress,
      this.isDelegateCall,
      this.isStaticCall,
      this.isContractDeployment,
    );
  }
}
