import { FieldsOf } from '../index.js';
import { serializeToBuffer } from '../utils/serialize.js';
import {
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
    public contractAddress: AztecAddress,
    public functionData: FunctionData,
    public callContext: CallContext,
    public args: Fr[],
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.functionData, this.callContext, this.args);
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
    return new PublicCallStackItem(this.contractAddress, this.functionData, publicInputs);
  }
}
