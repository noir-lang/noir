import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { inspect } from 'util';

import { computeVarArgsHash } from '../hash/hash.js';
import { CallContext } from './call_context.js';
import { CallRequest } from './call_request.js';
import { CallerContext } from './caller_context.js';
import { FunctionData, FunctionSelector } from './index.js';
import { PublicCallStackItem } from './public_call_stack_item.js';
import { PublicCircuitPublicInputs } from './public_circuit_public_inputs.js';
import { Vector } from './shared.js';

/**
 * Represents a request to call a public function from a private function. Serialization is
 * equivalent to a public call stack item, but without the result fields.
 */
export class PublicCallRequest {
  constructor(
    /**
     *Address of the contract on which the function is invoked.
     */
    public contractAddress: AztecAddress,
    /**
     * Data identifying the function being called.
     */
    public functionSelector: FunctionSelector,
    /**
     * Context of the public call.
     * TODO(#3417): Check if all fields of CallContext are actually needed.
     */
    public callContext: CallContext,
    /**
     * Context of the public call.
     * TODO(#3417): Check if all fields of CallContext are actually needed.
     */
    public parentCallContext: CallContext,
    /**
     * The start side effect counter for this call request.
     */
    public sideEffectCounter: number,
    /**
     * Function arguments.
     */
    public args: Fr[],
  ) {}

  getSize() {
    return this.isEmpty() ? 0 : this.toBuffer().length;
  }

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(
      this.contractAddress,
      this.functionSelector,
      this.callContext,
      this.parentCallContext,
      this.sideEffectCounter,
      new Vector(this.args),
    );
  }

  /**
   * Deserialize this from a buffer.
   * @param buffer - The bufferable type from which to deserialize.
   * @returns The deserialized instance of PublicCallRequest.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicCallRequest(
      new AztecAddress(reader.readBytes(32)),
      FunctionSelector.fromBuffer(reader),
      CallContext.fromBuffer(reader),
      CallContext.fromBuffer(reader),
      reader.readNumber(),
      reader.readVector(Fr),
    );
  }

  /**
   * Create PublicCallRequest from a fields dictionary.
   * @param fields - The dictionary.
   * @returns A PublicCallRequest object.
   */
  static from(fields: FieldsOf<PublicCallRequest>): PublicCallRequest {
    return new PublicCallRequest(...PublicCallRequest.getFields(fields));
  }

  /**
   * Serialize into a field array. Low-level utility.
   * @param fields - Object with fields.
   * @returns The array.
   */
  static getFields(fields: FieldsOf<PublicCallRequest>) {
    return [
      fields.contractAddress,
      fields.functionSelector,
      fields.callContext,
      fields.parentCallContext,
      fields.sideEffectCounter,
      fields.args,
    ] as const;
  }

  /**
   * Creates a new PublicCallStackItem by populating with zeroes all fields related to result in the public circuit output.
   * @returns A PublicCallStackItem instance with the same contract address, function data, call context, and args.
   */
  toPublicCallStackItem() {
    const publicInputs = PublicCircuitPublicInputs.empty();
    publicInputs.callContext = this.callContext;
    publicInputs.argsHash = this.getArgsHash();
    return new PublicCallStackItem(
      this.contractAddress,
      new FunctionData(this.functionSelector, false),
      publicInputs,
      true,
    );
  }

  /**
   * Creates a new CallRequest with values of the calling contract.
   * @returns A CallRequest instance with the contract address, caller context, and the hash of the call stack item.
   */
  toCallRequest() {
    const item = this.toPublicCallStackItem();
    const callerContext = this.callContext.isDelegateCall
      ? new CallerContext(
          this.parentCallContext.msgSender,
          this.parentCallContext.storageContractAddress,
          this.parentCallContext.isStaticCall,
        )
      : CallerContext.empty();
    return new CallRequest(
      item.getCompressed().hash(),
      this.parentCallContext.storageContractAddress,
      callerContext,
      new Fr(this.sideEffectCounter),
      Fr.ZERO,
    );
  }

  /**
   * Returns the hash of the arguments for this request.
   * @returns Hash of the arguments for this request.
   */
  getArgsHash() {
    return computeVarArgsHash(this.args);
  }

  static empty() {
    return new PublicCallRequest(
      AztecAddress.ZERO,
      FunctionSelector.empty(),
      CallContext.empty(),
      CallContext.empty(),
      0,
      [],
    );
  }

  isEmpty(): boolean {
    return (
      this.contractAddress.isZero() &&
      this.functionSelector.isEmpty() &&
      this.callContext.isEmpty() &&
      this.parentCallContext.isEmpty() &&
      this.sideEffectCounter == 0 &&
      this.args.length === 0
    );
  }

  [inspect.custom]() {
    return `PublicCallRequest {
      contractAddress: ${this.contractAddress}
      functionSelector: ${this.functionSelector}
      callContext: ${this.callContext}
      parentCallContext: ${this.parentCallContext}
      sideEffectCounter: ${this.sideEffectCounter}
      args: ${this.args}
    }`;
  }
}
