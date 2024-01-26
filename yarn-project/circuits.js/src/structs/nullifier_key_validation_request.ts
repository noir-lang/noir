import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, GrumpkinScalar, Point } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { GrumpkinPrivateKey } from '../types/grumpkin_private_key.js';

/**
 * Request for validating a nullifier key pair used in the app.
 */
export class NullifierKeyValidationRequest {
  constructor(
    /**
     * Public key of the nullifier key.
     */
    public readonly publicKey: Point,
    /**
     * Secret key of the nullifier key.
     */
    public readonly secretKey: GrumpkinPrivateKey,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.publicKey, this.secretKey);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NullifierKeyValidationRequest(Point.fromBuffer(reader), GrumpkinScalar.fromBuffer(reader));
  }

  toFields(): Fr[] {
    return [this.publicKey.toFields(), this.secretKey.high, this.secretKey.low].flat();
  }

  static fromFields(fields: Fr[] | FieldReader): NullifierKeyValidationRequest {
    const reader = FieldReader.asReader(fields);
    return new NullifierKeyValidationRequest(Point.fromFields(reader), reader.readFq());
  }

  isEmpty() {
    return this.publicKey.isZero() && this.secretKey.isZero();
  }

  static empty() {
    return new NullifierKeyValidationRequest(Point.ZERO, GrumpkinScalar.ZERO);
  }
}

/**
 * Request for validating a nullifier key pair used in the app.
 */
export class NullifierKeyValidationRequestContext {
  constructor(
    /**
     * Public key of the nullifier key.
     */
    public readonly publicKey: Point,
    /**
     * Secret key of the nullifier key.
     */
    public readonly secretKey: GrumpkinPrivateKey,
    /**
     * The storage contract address the nullifier key is for.
     */
    public readonly contractAddress: AztecAddress,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.publicKey, this.secretKey, this.contractAddress);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NullifierKeyValidationRequestContext(
      Point.fromBuffer(reader),
      GrumpkinScalar.fromBuffer(reader),
      AztecAddress.fromBuffer(reader),
    );
  }

  toFields(): Fr[] {
    return [this.publicKey.toFields(), this.secretKey.high, this.secretKey.low, this.contractAddress].flat();
  }

  static fromFields(fields: Fr[] | FieldReader): NullifierKeyValidationRequestContext {
    const reader = FieldReader.asReader(fields);
    return new NullifierKeyValidationRequestContext(
      Point.fromFields(reader),
      reader.readFq(),
      AztecAddress.fromFields(reader),
    );
  }

  isEmpty() {
    return this.publicKey.isZero() && this.secretKey.isZero() && this.contractAddress.isZero();
  }

  static empty() {
    return new NullifierKeyValidationRequestContext(Point.ZERO, GrumpkinScalar.ZERO, AztecAddress.ZERO);
  }
}
