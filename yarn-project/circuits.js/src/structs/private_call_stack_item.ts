import { AztecAddress } from '@aztec/foundation/aztec-address';
import { pedersenHash } from '@aztec/foundation/crypto';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import { GeneratorIndex } from '../constants.gen.js';
import { CallRequest, CallerContext } from './call_request.js';
import { FunctionData } from './function_data.js';
import { PrivateCircuitPublicInputs } from './private_circuit_public_inputs.js';

/**
 * Call stack item on a private call.
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
  ) {}

  static getFields(fields: FieldsOf<PrivateCallStackItem>) {
    return [fields.contractAddress, fields.functionData, fields.publicInputs] as const;
  }

  toBuffer() {
    return serializeToBuffer(...PrivateCallStackItem.getFields(this));
  }

  toFields(): Fr[] {
    return serializeToFields(...PrivateCallStackItem.getFields(this));
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
    );
  }

  static fromFields(fields: Fr[] | FieldReader): PrivateCallStackItem {
    const reader = FieldReader.asReader(fields);
    return new PrivateCallStackItem(
      AztecAddress.fromFields(reader),
      FunctionData.fromFields(reader),
      PrivateCircuitPublicInputs.fromFields(reader),
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
    );
  }

  isEmpty() {
    return this.contractAddress.isZero() && this.functionData.isEmpty() && this.publicInputs.isEmpty();
  }

  /**
   * Computes this call stack item hash.
   * @returns Hash.
   */
  public hash(): Fr {
    return Fr.fromBuffer(
      pedersenHash(
        this.toFields().map(field => field.toBuffer()),
        GeneratorIndex.CALL_STACK_ITEM,
      ),
    );
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
