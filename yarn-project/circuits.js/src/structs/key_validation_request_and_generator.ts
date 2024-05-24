import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH } from '../constants.gen.js';
import { KeyValidationRequest } from './key_validation_request.js';

/**
 * Request for validating keys used in the app and a generator.
 */
export class KeyValidationRequestAndGenerator {
  constructor(
    /** The key validation request. */
    public readonly request: KeyValidationRequest,
    /**
     * The generator index which can be used along with sk_m to derive the sk_app stored in the request.
     * Note: This generator constrains that a correct key type gets validated in the kernel.
     */
    public readonly skAppGenerator: Fr,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.request, this.skAppGenerator);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new KeyValidationRequestAndGenerator(reader.readObject(KeyValidationRequest), Fr.fromBuffer(reader));
  }

  toFields(): Fr[] {
    const fields = [...this.request.toFields(), this.skAppGenerator];
    if (fields.length !== KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH) {
      throw new Error(
        `Invalid number of fields for KeyValidationRequestAndGenerator. Expected ${KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromFields(fields: Fr[] | FieldReader): KeyValidationRequestAndGenerator {
    const reader = FieldReader.asReader(fields);
    return new KeyValidationRequestAndGenerator(KeyValidationRequest.fromFields(reader), reader.readField());
  }

  isEmpty() {
    return this.request.isEmpty() && this.skAppGenerator.isZero();
  }

  static empty() {
    return new KeyValidationRequestAndGenerator(KeyValidationRequest.empty(), Fr.ZERO);
  }
}
