import { BufferReader } from '@aztec/foundation/serialize';
import { Tuple } from '@aztec/foundation/serialize';
import { FieldsOf } from '../index.js';
import { serializeToBuffer } from '../utils/serialize.js';
import {
  ARGS_LENGTH,
  AztecAddress,
  CallContext,
  Fr,
  FunctionData,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
} from './index.js';

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
    public functionData: FunctionData,
    /**
     * Context of the public call.
     */
    public callContext: CallContext,
    /**
     * Function arguments.
     */
    public args: Tuple<Fr, typeof ARGS_LENGTH>,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.functionData, this.callContext, this.args);
  }

  /**
   * Deserialise this from a buffer.
   * @param buffer - The bufferable type from which to deserialise.
   * @returns The deserialised instance of PublicCallRequest.
   */
  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicCallRequest(
      new AztecAddress(reader.readBytes(32)),
      FunctionData.fromBuffer(reader),
      CallContext.fromBuffer(reader),
      reader.readArray<Fr, typeof ARGS_LENGTH>(ARGS_LENGTH, Fr),
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
    return [fields.contractAddress, fields.functionData, fields.callContext, fields.args] as const;
  }

  /**
   * Creates a new instance out of a PublicCallStackItem, dropping all fields related to execution result.
   * @param item - Input item to copy.
   * @returns A PublicCallRequest instance with the same contract address, function data, call context, and args.
   */
  static fromPublicCallStackItem(item: PublicCallStackItem): PublicCallRequest {
    return PublicCallRequest.from({
      contractAddress: item.contractAddress,
      functionData: item.functionData,
      callContext: item.publicInputs.callContext,
      args: item.publicInputs.args,
    });
  }

  /**
   * Creates a new PublicCallStackItem by populating with zeroes all fields related to result in the public circuit output.
   * @returns A PublicCallStackItem instance with the same contract address, function data, call context, and args.
   */
  toPublicCallStackItem(): PublicCallStackItem {
    const publicInputs = PublicCircuitPublicInputs.empty();
    publicInputs.callContext = this.callContext;
    publicInputs.args = this.args;
    return new PublicCallStackItem(this.contractAddress, this.functionData, publicInputs, true);
  }
}
