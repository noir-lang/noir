import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { type CallContext } from './call_context.js';
import { CallRequest } from './call_request.js';
import { CallerContext } from './caller_context.js';
import { FunctionData } from './function_data.js';
import { PublicCallStackItemCompressed } from './public_call_stack_item_compressed.js';
import { PublicCircuitPublicInputs } from './public_circuit_public_inputs.js';

/**
 * Call stack item on a public call.
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

  static getFields(fields: FieldsOf<PublicCallStackItem>) {
    return [fields.contractAddress, fields.functionData, fields.publicInputs, fields.isExecutionRequest] as const;
  }

  toBuffer() {
    return serializeToBuffer(...PublicCallStackItem.getFields(this));
  }

  /**
   * Deserializes from a buffer or reader.
   * @param buffer - Buffer or reader to read from.
   * @returns The deserialized instance.
   */
  static fromBuffer(buffer: Buffer | BufferReader): PublicCallStackItem {
    const reader = BufferReader.asReader(buffer);
    return new PublicCallStackItem(
      reader.readObject(AztecAddress),
      reader.readObject(FunctionData),
      reader.readObject(PublicCircuitPublicInputs),
      reader.readBoolean(),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): PublicCallStackItem {
    const reader = FieldReader.asReader(fields);

    const contractAddress = AztecAddress.fromFields(reader);
    const functionData = FunctionData.fromFields(reader);
    const publicInputs = PublicCircuitPublicInputs.fromFields(reader);
    const isExecutionRequest = reader.readBoolean();

    return new PublicCallStackItem(contractAddress, functionData, publicInputs, isExecutionRequest);
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

  getCompressed(): PublicCallStackItemCompressed {
    let publicInputsToHash = this.publicInputs;
    if (this.isExecutionRequest) {
      // An execution request (such as an enqueued call from private) is hashed with
      // only the publicInput members present in a PublicCallRequest.
      // This allows us to check that the request (which is created/hashed before
      // side-effects and output info are unknown for public calls) matches the call
      // being processed by a kernel iteration.
      // WARNING: This subset of publicInputs that is set here must align with
      // `parse_public_call_stack_item_from_oracle` in enqueue_public_function_call.nr
      // and `PublicCallStackItem::as_execution_request()` in public_call_stack_item.ts
      const { callContext, argsHash } = this.publicInputs;
      publicInputsToHash = PublicCircuitPublicInputs.empty();
      publicInputsToHash.callContext = callContext;
      publicInputsToHash.argsHash = argsHash;
    }

    return new PublicCallStackItemCompressed(
      this.contractAddress,
      publicInputsToHash.callContext,
      this.functionData,
      publicInputsToHash.argsHash,
      publicInputsToHash.returnsHash,
      publicInputsToHash.revertCode,
      publicInputsToHash.startGasLeft,
      publicInputsToHash.endGasLeft,
    );
  }

  /**
   * Creates a new CallRequest with values of the calling contract.
   * @returns A CallRequest instance with the contract address, caller context, and the hash of the call stack item.
   */
  public toCallRequest(parentCallContext: CallContext) {
    if (this.isEmpty()) {
      return CallRequest.empty();
    }

    const currentCallContext = this.publicInputs.callContext;
    const callerContext = currentCallContext.isDelegateCall
      ? new CallerContext(
          parentCallContext.msgSender,
          parentCallContext.storageContractAddress,
          parentCallContext.isStaticCall,
        )
      : CallerContext.empty();
    // todo: populate side effect counters correctly

    const hash = this.getCompressed().hash();

    return new CallRequest(hash, parentCallContext.storageContractAddress, callerContext, Fr.ZERO, Fr.ZERO);
  }
}
