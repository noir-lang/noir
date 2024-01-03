import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { computePrivateCallStackItemHash, computePublicCallStackItemHash } from '../abis/abis.js';
import { serializeToBuffer } from '../utils/serialize.js';
import { CallRequest, CallerContext } from './call_request.js';
import { FunctionData } from './function_data.js';
import { PrivateCircuitPublicInputs } from './private_circuit_public_inputs.js';
import { PublicCircuitPublicInputs } from './public_circuit_public_inputs.js';

/**
 * Call stack item on a private call.
 * @see cpp/src/aztec3/circuits/abis/call_stack_item.hpp.
 */
export class PrivateCallStackItem {
  constructor(
    /**
     * Address of the contract on which the function is invoked.
     */
    public contractAddress: AztecAddress,
    /**
     * Data identifying the function being called.
     */
    public functionData: FunctionData,
    /**
     * Public inputs to the private kernel circuit.
     */
    public publicInputs: PrivateCircuitPublicInputs,
    /**
     * Whether the current callstack item should be considered a public fn execution request.
     */
    public readonly isExecutionRequest: boolean,
  ) {
    if (isExecutionRequest) {
      throw new Error('boolean isExecutionRequest must be set to true for a PrivateCallStackItem object');
    }
  }

  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.functionData, this.publicInputs, this.isExecutionRequest);
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PrivateCallStackItem {
    const reader = BufferReader.asReader(buffer);
    return new PrivateCallStackItem(
      reader.readObject(AztecAddress),
      reader.readObject(FunctionData),
      reader.readObject(PrivateCircuitPublicInputs),
      reader.readBoolean(),
    );
  }

  /**
   * Returns a new instance of PrivateCallStackItem with zero contract address, function data and public inputs.
   * @returns A new instance of PrivateCallStackItem with zero contract address, function data and public inputs.
   */
  public static empty(): PrivateCallStackItem {
    return new PrivateCallStackItem(
      AztecAddress.ZERO,
      FunctionData.empty({ isPrivate: true }),
      PrivateCircuitPublicInputs.empty(),
      false,
    );
  }

  isEmpty() {
    return this.contractAddress.isZero() && this.functionData.isEmpty() && this.publicInputs.isEmpty();
  }

  /**
   * Computes this call stack item hash.
   * @returns Hash.
   */
  public hash() {
    return computePrivateCallStackItemHash(this);
  }

  /**
   * Creates a new CallRequest with values of the calling contract.
   * @returns A CallRequest instance with the contract address, caller context, and the hash of the call stack item.
   */
  public toCallRequest() {
    if (this.isEmpty()) {
      return CallRequest.empty();
    }

    const callContext = this.publicInputs.callContext;
    const callerContext = callContext.isDelegateCall
      ? new CallerContext(callContext.msgSender, callContext.storageContractAddress)
      : CallerContext.empty();
    // todo: populate side effect counters correctly
    return new CallRequest(this.hash(), callContext.msgSender, callerContext, Fr.ZERO, Fr.ZERO);
  }
}

/**
 * Call stack item on a public call.
 * @see cpp/src/aztec3/circuits/abis/call_stack_item.hpp.
 */
export class PublicCallStackItem {
  constructor(
    /**
     * Address of the contract on which the function is invoked.
     */
    public contractAddress: AztecAddress,
    /**
     * Data identifying the function being called.
     */
    public functionData: FunctionData,
    /**
     * Public inputs to the public kernel circuit.
     */
    public publicInputs: PublicCircuitPublicInputs,
    /**
     * Whether the current callstack item should be considered a public fn execution request.
     */
    public isExecutionRequest: boolean,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.functionData, this.publicInputs, this.isExecutionRequest);
  }

  /**
   * Returns a new instance of PublicCallStackItem with zero contract address, function data and public inputs.
   * @returns A new instance of PublicCallStackItem with zero contract address, function data and public inputs.
   */
  public static empty(): PublicCallStackItem {
    return new PublicCallStackItem(
      AztecAddress.ZERO,
      FunctionData.empty({ isPrivate: false }),
      PublicCircuitPublicInputs.empty(),
      false,
    );
  }

  isEmpty() {
    return this.contractAddress.isZero() && this.functionData.isEmpty() && this.publicInputs.isEmpty();
  }

  /**
   * Computes this call stack item hash.
   * @returns Hash.
   */
  public hash() {
    return computePublicCallStackItemHash(this);
  }

  /**
   * Creates a new CallRequest with values of the calling contract.
   * @returns A CallRequest instance with the contract address, caller context, and the hash of the call stack item.
   */
  public toCallRequest() {
    if (this.isEmpty()) {
      return CallRequest.empty();
    }

    const callContext = this.publicInputs.callContext;
    const callerContext = callContext.isDelegateCall
      ? new CallerContext(callContext.msgSender, callContext.storageContractAddress)
      : CallerContext.empty();
    // todo: populate side effect counters correctly
    return new CallRequest(this.hash(), callContext.msgSender, callerContext, Fr.ZERO, Fr.ZERO);
  }
}
