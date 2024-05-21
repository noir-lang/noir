import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { KEY_VALIDATION_REQUEST_LENGTH, SCOPED_KEY_VALIDATION_REQUEST_LENGTH } from '../constants.gen.js';

/**
 * Request for validating keys used in the app.
 */
export class KeyValidationRequest {
  constructor(
    /**
     * Master public key (pk_m) corresponding to the same underlying secret as app secret key bellow.
     */
    public readonly masterPublicKey: Point,
    /**
     * App-siloed secret key (sk_app) corresponding to the same underlying secret as master public key above.
     */
    public readonly appSecretKey: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.masterPublicKey, this.appSecretKey);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new KeyValidationRequest(Point.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  toFields(): Fr[] {
    const fields = [this.masterPublicKey.toFields(), this.appSecretKey].flat();
    if (fields.length !== KEY_VALIDATION_REQUEST_LENGTH) {
      throw new Error(
        `Invalid number of fields for KeyValidationRequest. Expected ${KEY_VALIDATION_REQUEST_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromFields(fields: Fr[] | FieldReader): KeyValidationRequest {
    const reader = FieldReader.asReader(fields);
    return new KeyValidationRequest(Point.fromFields(reader), reader.readField());
  }

  isEmpty() {
    return this.masterPublicKey.isZero() && this.appSecretKey.isZero();
  }

  static empty() {
    return new KeyValidationRequest(Point.ZERO, Fr.ZERO);
  }
}

/**
 * Request for validating keys used in the app.
 */
export class ScopedKeyValidationRequest {
  constructor(public readonly request: KeyValidationRequest, public readonly contractAddress: AztecAddress) {}

  toBuffer() {
    return serializeToBuffer(this.request, this.contractAddress);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ScopedKeyValidationRequest(KeyValidationRequest.fromBuffer(reader), AztecAddress.fromBuffer(reader));
  }

  toFields(): Fr[] {
    const fields = [...this.request.toFields(), this.contractAddress];
    if (fields.length !== SCOPED_KEY_VALIDATION_REQUEST_LENGTH) {
      throw new Error(
        `Invalid number of fields for ScopedKeyValidationRequest. Expected ${SCOPED_KEY_VALIDATION_REQUEST_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromFields(fields: Fr[] | FieldReader): ScopedKeyValidationRequest {
    const reader = FieldReader.asReader(fields);
    return new ScopedKeyValidationRequest(KeyValidationRequest.fromFields(reader), AztecAddress.fromFields(reader));
  }

  isEmpty() {
    return this.request.isEmpty() && this.contractAddress.isZero();
  }

  static empty() {
    return new ScopedKeyValidationRequest(KeyValidationRequest.empty(), AztecAddress.ZERO);
  }
}
