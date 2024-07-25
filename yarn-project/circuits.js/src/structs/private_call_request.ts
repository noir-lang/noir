import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';

import { CallContext } from './call_context.js';
import { type PrivateCallStackItem } from './index.js';

export class PrivateCallRequest {
  constructor(
    /**
     * The address of the contract being called.
     */
    public contractAddress: AztecAddress,
    /**
     * The call context of the call.
     */
    public callContext: CallContext,
    /**
     * The hash of the arguments of the call.
     */
    public argsHash: Fr,
    /**
     * The hash of the return values of the call.
     */
    public returnsHash: Fr,
    /**
     * The start counter of the call.
     */
    public startSideEffectCounter: number,
    /**
     * The end counter of the call.
     */
    public endSideEffectCounter: number,
  ) {}

  toFields(): Fr[] {
    return serializeToFields([
      this.contractAddress,
      this.callContext,
      this.argsHash,
      this.returnsHash,
      this.startSideEffectCounter,
      this.endSideEffectCounter,
    ]);
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new PrivateCallRequest(
      reader.readObject(AztecAddress),
      reader.readObject(CallContext),
      reader.readField(),
      reader.readField(),
      reader.readU32(),
      reader.readU32(),
    );
  }

  toBuffer() {
    return serializeToBuffer(
      this.contractAddress,
      this.callContext,
      this.argsHash,
      this.returnsHash,
      this.startSideEffectCounter,
      this.endSideEffectCounter,
    );
  }

  public static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateCallRequest(
      reader.readObject(AztecAddress),
      reader.readObject(CallContext),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readNumber(),
      reader.readNumber(),
    );
  }

  isEmpty() {
    return (
      this.contractAddress.isZero() &&
      this.callContext.isEmpty() &&
      this.argsHash.isZero() &&
      this.returnsHash.isZero() &&
      this.startSideEffectCounter === 0 &&
      this.endSideEffectCounter === 0
    );
  }

  public static empty() {
    return new PrivateCallRequest(AztecAddress.ZERO, CallContext.empty(), Fr.ZERO, Fr.ZERO, 0, 0);
  }

  equals(callRequest: PrivateCallRequest) {
    return (
      callRequest.contractAddress.equals(this.contractAddress) &&
      callRequest.callContext.equals(this.callContext) &&
      callRequest.argsHash.equals(this.argsHash) &&
      callRequest.returnsHash.equals(this.returnsHash) &&
      callRequest.startSideEffectCounter === this.startSideEffectCounter &&
      callRequest.endSideEffectCounter === this.endSideEffectCounter
    );
  }

  toString() {
    return `PrivateCallRequest(target: ${this.contractAddress}, callContext: ${this.callContext}, argsHash: ${this.argsHash}, returnsHash: ${this.returnsHash}, startSideEffectCounter: ${this.startSideEffectCounter}, endSideEffectCounter: ${this.endSideEffectCounter})`;
  }

  matchesStackItem(stackItem: PrivateCallStackItem) {
    return (
      stackItem.contractAddress.equals(this.contractAddress) &&
      stackItem.publicInputs.callContext.equals(this.callContext) &&
      stackItem.publicInputs.argsHash.equals(this.argsHash) &&
      stackItem.publicInputs.returnsHash.equals(this.returnsHash) &&
      stackItem.publicInputs.startSideEffectCounter.equals(new Fr(this.startSideEffectCounter)) &&
      stackItem.publicInputs.endSideEffectCounter.equals(new Fr(this.endSideEffectCounter))
    );
  }
}
