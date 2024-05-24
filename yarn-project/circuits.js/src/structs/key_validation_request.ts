import { Fr, Point } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { KEY_VALIDATION_REQUEST_LENGTH } from '../constants.gen.js';

/**
 * Request for validating keys used in the app.
 */
export class KeyValidationRequest {
  constructor(
    /** Master public key corresponding to the same underlying secret as app secret key below. */
    public readonly pkM: Point,
    /** App-siloed secret key corresponding to the same underlying secret as master public key above. */
    public readonly skApp: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.pkM, this.skApp);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new KeyValidationRequest(Point.fromBuffer(reader), Fr.fromBuffer(reader));
  }

  toFields(): Fr[] {
    const fields = [this.pkM.toFields(), this.skApp].flat();
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
    return this.pkM.isZero() && this.skApp.isZero();
  }

  static empty() {
    return new KeyValidationRequest(Point.ZERO, Fr.ZERO);
  }
}
