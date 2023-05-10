import { AztecAddress } from '@aztec/foundation/aztec-address';
import { serializeToBuffer } from '../utils/serialize.js';
import { FunctionData } from './function_data.js';
import { PrivateCircuitPublicInputs } from './private_circuit_public_inputs.js';
import { PublicCircuitPublicInputs } from './public_circuit_public_inputs.js';

/**
 * Call stack item on a private call.
 * @see cpp/src/aztec3/circuits/abis/call_stack_item.hpp.
 */
export class PrivateCallStackItem {
  constructor(
    public contractAddress: AztecAddress,
    public functionData: FunctionData,
    public publicInputs: PrivateCircuitPublicInputs,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.functionData, this.publicInputs);
  }

  public static empty() {
    return new PrivateCallStackItem(
      AztecAddress.ZERO,
      FunctionData.empty({ isPrivate: true }),
      PrivateCircuitPublicInputs.empty(),
    );
  }
}

/**
 * Call stack item on a public call.
 * @see cpp/src/aztec3/circuits/abis/call_stack_item.hpp.
 */
export class PublicCallStackItem {
  constructor(
    public contractAddress: AztecAddress,
    public functionData: FunctionData,
    public publicInputs: PublicCircuitPublicInputs,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.functionData, this.publicInputs);
  }

  public static empty() {
    return new PublicCallStackItem(
      AztecAddress.ZERO,
      FunctionData.empty({ isPrivate: false }),
      PublicCircuitPublicInputs.empty(),
    );
  }

  isEmpty() {
    return this.contractAddress.isZero() && this.functionData.isEmpty() && this.publicInputs.isEmpty();
  }
}
