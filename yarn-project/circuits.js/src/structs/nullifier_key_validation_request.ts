import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import {
  NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH,
  SCOPED_NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH,
} from '../constants.gen.js';

/**
 * Request for validating a nullifier key pair used in the app.
 */
export class NullifierKeyValidationRequest {
  constructor(
    /**
     * Public key of the nullifier key (Npk_m).
     */
    public readonly masterNullifierPublicKey: Point,
    /**
     * App-siloed nullifier secret key (nsk_app*).
     */
    public readonly appNullifierSecretKey: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.masterNullifierPublicKey, this.appNullifierSecretKey);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NullifierKeyValidationRequest(Point.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  toFields(): Fr[] {
    const fields = [this.masterNullifierPublicKey.toFields(), this.appNullifierSecretKey].flat();
    if (fields.length !== NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH) {
      throw new Error(
        `Invalid number of fields for NullifierKeyValidationRequest. Expected ${NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromFields(fields: Fr[] | FieldReader): NullifierKeyValidationRequest {
    const reader = FieldReader.asReader(fields);
    return new NullifierKeyValidationRequest(Point.fromFields(reader), reader.readField());
  }

  isEmpty() {
    return this.masterNullifierPublicKey.isZero() && this.appNullifierSecretKey.isZero();
  }

  static empty() {
    return new NullifierKeyValidationRequest(Point.ZERO, Fr.ZERO);
  }
}

/**
 * Request for validating a nullifier key pair used in the app.
 */
export class ScopedNullifierKeyValidationRequest {
  constructor(public readonly request: NullifierKeyValidationRequest, public readonly contractAddress: AztecAddress) {}

  toBuffer() {
    return serializeToBuffer(this.request, this.contractAddress);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ScopedNullifierKeyValidationRequest(
      NullifierKeyValidationRequest.fromBuffer(reader),
      AztecAddress.fromBuffer(reader),
    );
  }

  toFields(): Fr[] {
    const fields = [...this.request.toFields(), this.contractAddress];
    if (fields.length !== SCOPED_NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH) {
      throw new Error(
        `Invalid number of fields for ScopedNullifierKeyValidationRequest. Expected ${SCOPED_NULLIFIER_KEY_VALIDATION_REQUEST_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromFields(fields: Fr[] | FieldReader): ScopedNullifierKeyValidationRequest {
    const reader = FieldReader.asReader(fields);
    return new ScopedNullifierKeyValidationRequest(
      NullifierKeyValidationRequest.fromFields(reader),
      AztecAddress.fromFields(reader),
    );
  }

  isEmpty() {
    return this.request.isEmpty() && this.contractAddress.isZero();
  }

  static empty() {
    return new ScopedNullifierKeyValidationRequest(NullifierKeyValidationRequest.empty(), AztecAddress.ZERO);
  }
}
