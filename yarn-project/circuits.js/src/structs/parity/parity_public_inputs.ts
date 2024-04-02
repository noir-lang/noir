import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { AggregationObject } from '../aggregation_object.js';

export class ParityPublicInputs {
  constructor(
    /** Aggregated proof of all the parity circuit iterations. */
    public aggregationObject: AggregationObject,
    /** Root of the SHA256 tree. */
    public shaRoot: Fr,
    /** Root of the converted tree. */
    public convertedRoot: Fr,
  ) {
    if (shaRoot.toBuffer()[0] != 0) {
      throw new Error(`shaRoot buffer must be 31 bytes. Got 32 bytes`);
    }
  }

  toBuffer() {
    return serializeToBuffer(...ParityPublicInputs.getFields(this));
  }

  static from(fields: FieldsOf<ParityPublicInputs>): ParityPublicInputs {
    return new ParityPublicInputs(...ParityPublicInputs.getFields(fields));
  }

  static getFields(fields: FieldsOf<ParityPublicInputs>) {
    return [fields.aggregationObject, fields.shaRoot, fields.convertedRoot] as const;
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new ParityPublicInputs(reader.readObject(AggregationObject), reader.readObject(Fr), reader.readObject(Fr));
  }
}
