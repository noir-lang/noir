import { AztecAddress } from '@aztec/foundation/aztec-address';
import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH } from '../constants.gen.js';
import { KeyValidationRequestAndGenerator } from './key_validation_request_and_generator.js';

/**
 * Request for validating keys used in the app.
 */
export class ScopedKeyValidationRequestAndGenerator {
  constructor(
    public readonly request: KeyValidationRequestAndGenerator,
    public readonly contractAddress: AztecAddress,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.request, this.contractAddress);
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ScopedKeyValidationRequestAndGenerator(
      KeyValidationRequestAndGenerator.fromBuffer(reader),
      AztecAddress.fromBuffer(reader),
    );
  }

  toFields(): Fr[] {
    const fields = [...this.request.toFields(), this.contractAddress];
    if (fields.length !== SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH) {
      throw new Error(
        `Invalid number of fields for ScopedKeyValidationRequestAndGenerator. Expected ${SCOPED_KEY_VALIDATION_REQUEST_AND_GENERATOR_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromFields(fields: Fr[] | FieldReader): ScopedKeyValidationRequestAndGenerator {
    const reader = FieldReader.asReader(fields);
    return new ScopedKeyValidationRequestAndGenerator(
      KeyValidationRequestAndGenerator.fromFields(reader),
      AztecAddress.fromFields(reader),
    );
  }

  isEmpty() {
    return this.request.isEmpty() && this.contractAddress.isZero();
  }

  static empty() {
    return new ScopedKeyValidationRequestAndGenerator(KeyValidationRequestAndGenerator.empty(), AztecAddress.ZERO);
  }
}
