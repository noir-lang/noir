import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';

import { CallerContext } from './caller_context.js';

export class PrivateCallRequest {
  constructor(
    /**
     * The call stack item hash of the call.
     */
    public hash: Fr,
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
    return serializeToFields([this.hash, this.callerContext, this.startSideEffectCounter, this.endSideEffectCounter]);
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new PrivateCallRequest(
      reader.readField(),
      reader.readObject(CallerContext),
      reader.readU32(),
      reader.readU32(),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.hash, this.callerContext, this.startSideEffectCounter, this.endSideEffectCounter);
  }

  public static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PrivateCallRequest(
      Fr.fromBuffer(reader),
      reader.readObject(CallerContext),
      reader.readNumber(),
      reader.readNumber(),
    );
  }

  isEmpty() {
    return (
      this.hash.isZero() &&
      this.callerContext.isEmpty() &&
      this.startSideEffectCounter === 0 &&
      this.endSideEffectCounter === 0
    );
  }

  public static empty() {
    return new PrivateCallRequest(Fr.ZERO, CallerContext.empty(), 0, 0);
  }

  equals(callRequest: PrivateCallRequest) {
    return (
      callRequest.hash.equals(this.hash) &&
      callRequest.callerContext.equals(this.callerContext) &&
      callRequest.startSideEffectCounter === this.startSideEffectCounter &&
      callRequest.endSideEffectCounter === this.endSideEffectCounter
    );
  }

  toString() {
    return `PrivateCallRequest(hash: ${this.hash}, callerContext: ${this.callerContext}, startSideEffectCounter: ${this.startSideEffectCounter}, endSideEffectCounter: ${this.endSideEffectCounter})`;
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
