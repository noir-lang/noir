import { BufferReader } from '@aztec/foundation/serialize';

import { computeVarArgsHash } from '../abis/abis.js';
import { CircuitsWasm, FieldsOf } from '../index.js';
import { serializeToBuffer } from '../utils/serialize.js';
import {
  AztecAddress,
  CallContext,
  Fr,
  FunctionData,
  PublicCallStackItem,
  PublicCircuitPublicInputs,
  Vector,
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
    public args: Fr[],
    /**
     * Optional side effect counter tracking position of this event in tx execution.
     */
    public sideEffectCounter?: number,
  ) {}

  /**
   * Serialize this as a buffer.
   * @returns The buffer.
   */
  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.functionData, this.callContext, new Vector(this.args));
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
      fields.functionData,
      fields.callContext,
      fields.args,
      fields.sideEffectCounter,
    ] as const;
  }

  /**
   * Creates a new PublicCallStackItem by populating with zeroes all fields related to result in the public circuit output.
   * @returns A PublicCallStackItem instance with the same contract address, function data, call context, and args.
   */
  async toPublicCallStackItem(): Promise<PublicCallStackItem> {
    const publicInputs = PublicCircuitPublicInputs.empty();
    publicInputs.callContext = this.callContext;
    publicInputs.argsHash = await this.getArgsHash();
    return new PublicCallStackItem(this.contractAddress, this.functionData, publicInputs, true);
  }

  /**
   * Returns the hash of the arguments for this request.
   * @returns Hash of the arguments for this request.
   */
  async getArgsHash() {
    return computeVarArgsHash(await CircuitsWasm.get(), this.args);
  }
}
