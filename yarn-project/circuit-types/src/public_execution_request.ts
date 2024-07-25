import { CallContext, type PublicCallRequest, Vector } from '@aztec/circuits.js';
import { computeVarArgsHash } from '@aztec/circuits.js/hash';
import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { inspect } from 'util';

/**
 * The execution request of a public function.
 */
export class PublicExecutionRequest {
  constructor(
    public contractAddress: AztecAddress,
    /**
     * Context of the public call.
     * TODO(#3417): Check if all fields of CallContext are actually needed.
     */
    public callContext: CallContext,
    /**
     * Function arguments.
     */
    public args: Fr[],
  ) {}

  getSize() {
    return this.isEmpty() ? 0 : this.toBuffer().length;
  }

  toBuffer() {
    return serializeToBuffer(this.contractAddress, this.callContext, new Vector(this.args));
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicExecutionRequest(
      new AztecAddress(reader.readBytes(32)),
      CallContext.fromBuffer(reader),
      reader.readVector(Fr),
    );
  }

  static from(fields: FieldsOf<PublicExecutionRequest>): PublicExecutionRequest {
    return new PublicExecutionRequest(...PublicExecutionRequest.getFields(fields));
  }

  static getFields(fields: FieldsOf<PublicExecutionRequest>) {
    return [fields.contractAddress, fields.callContext, fields.args] as const;
  }

  static empty() {
    return new PublicExecutionRequest(AztecAddress.ZERO, CallContext.empty(), []);
  }

  isEmpty(): boolean {
    return this.contractAddress.isZero() && this.callContext.isEmpty() && this.args.length === 0;
  }

  isForCallRequest(callRequest: PublicCallRequest) {
    return (
      this.contractAddress.equals(callRequest.item.contractAddress) &&
      this.callContext.equals(callRequest.item.callContext) &&
      computeVarArgsHash(this.args).equals(callRequest.item.argsHash)
    );
  }

  [inspect.custom]() {
    return `PublicExecutionRequest {
      contractAddress: ${this.contractAddress}
      callContext: ${this.callContext}
      args: ${this.args}
    }`;
  }
}
