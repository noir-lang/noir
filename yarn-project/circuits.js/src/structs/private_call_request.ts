import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';

import { CallContext } from './call_context.js';
import { CallerContext } from './caller_context.js';
import { FunctionData } from './function_data.js';
import { type PrivateCallStackItem } from './index.js';

export class PrivateCallRequest {
  constructor(
    /**
     * The address of the contract being called.
     */
    public target: AztecAddress,
    /**
     * The call context of the call.
     */
    public callContext: CallContext,
    /**
     * The function data of the call.
     */
    public functionData: FunctionData,
    /**
     * The hash of the arguments of the call.
     */
    public argsHash: Fr,
    /**
     * The hash of the return values of the call.
     */
    public returnsHash: Fr,
    /**
     * The call context of the contract making the call.
     */
    public callerContext: CallerContext,
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
      this.target,
      this.callContext,
      this.functionData,
      this.argsHash,
      this.returnsHash,
      this.callerContext,
      this.startSideEffectCounter,
      this.endSideEffectCounter,
    ]);
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new PrivateCallRequest(
      reader.readObject(AztecAddress),
      reader.readObject(CallContext),
      reader.readObject(FunctionData),
      reader.readField(),
      reader.readField(),
      reader.readObject(CallerContext),
      reader.readU32(),
      reader.readU32(),
    );
  }

  toBuffer() {
    return serializeToBuffer(
      this.target,
      this.callContext,
      this.functionData,
      this.argsHash,
      this.returnsHash,
      this.callerContext,
      this.startSideEffectCounter,
      this.endSideEffectCounter,
    );
  }

  public static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateCallRequest(
      reader.readObject(AztecAddress),
      reader.readObject(CallContext),
      reader.readObject(FunctionData),
      Fr.fromBuffer(reader),
      Fr.fromBuffer(reader),
      reader.readObject(CallerContext),
      reader.readNumber(),
      reader.readNumber(),
    );
  }

  isEmpty() {
    return (
      this.target.isZero() &&
      this.callContext.isEmpty() &&
      this.functionData.isEmpty() &&
      this.argsHash.isZero() &&
      this.returnsHash.isZero() &&
      this.callerContext.isEmpty() &&
      this.startSideEffectCounter === 0 &&
      this.endSideEffectCounter === 0
    );
  }

  public static empty() {
    return new PrivateCallRequest(
      AztecAddress.ZERO,
      CallContext.empty(),
      FunctionData.empty(),
      Fr.ZERO,
      Fr.ZERO,
      CallerContext.empty(),
      0,
      0,
    );
  }

  equals(callRequest: PrivateCallRequest) {
    return (
      callRequest.target.equals(this.target) &&
      callRequest.callContext.equals(this.callContext) &&
      callRequest.functionData.equals(this.functionData) &&
      callRequest.argsHash.equals(this.argsHash) &&
      callRequest.returnsHash.equals(this.returnsHash) &&
      callRequest.callerContext.equals(this.callerContext) &&
      callRequest.startSideEffectCounter === this.startSideEffectCounter &&
      callRequest.endSideEffectCounter === this.endSideEffectCounter
    );
  }

  toString() {
    return `PrivateCallRequest(target: ${this.target}, callContext: ${this.callContext}, functionData: ${this.functionData}, argsHash: ${this.argsHash}, returnsHash: ${this.returnsHash}, callerContext: ${this.callerContext}, startSideEffectCounter: ${this.startSideEffectCounter}, endSideEffectCounter: ${this.endSideEffectCounter})`;
  }

  matchesStackItem(stackItem: PrivateCallStackItem) {
    return (
      stackItem.contractAddress.equals(this.target) &&
      stackItem.publicInputs.callContext.equals(this.callContext) &&
      stackItem.functionData.equals(this.functionData) &&
      stackItem.publicInputs.argsHash.equals(this.argsHash) &&
      stackItem.publicInputs.returnsHash.equals(this.returnsHash) &&
      stackItem.publicInputs.startSideEffectCounter.equals(new Fr(this.startSideEffectCounter)) &&
      stackItem.publicInputs.endSideEffectCounter.equals(new Fr(this.endSideEffectCounter))
    );
  }
}

export class ScopedPrivateCallRequest {
  constructor(public callRequest: PrivateCallRequest, public contractAddress: AztecAddress) {}

  toBuffer() {
    return serializeToBuffer(this.callRequest, this.contractAddress);
  }

  public static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ScopedPrivateCallRequest(reader.readObject(PrivateCallRequest), reader.readObject(AztecAddress));
  }

  isEmpty() {
    return this.callRequest.isEmpty() && this.contractAddress.isZero();
  }

  public static empty() {
    return new ScopedPrivateCallRequest(PrivateCallRequest.empty(), AztecAddress.ZERO);
  }

  equals(callRequest: ScopedPrivateCallRequest) {
    return callRequest.callRequest.equals(this.callRequest) && callRequest.contractAddress.equals(this.contractAddress);
  }

  toString() {
    return `ScopedPrivateCallRequest(callRequest: ${this.callRequest}, contractAddress: ${this.contractAddress})`;
  }
}
