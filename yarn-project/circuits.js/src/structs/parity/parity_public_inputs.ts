import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';
import { FieldsOf } from '@aztec/foundation/types';

import { AggregationObject } from '../aggregation_object.js';

export class ParityPublicInputs {
  constructor(
    /** Aggregated proof of all the parity circuit iterations. */
    public aggregationObject: AggregationObject,
    /** Root of the SHA256 tree. */
    public shaRoot: Buffer,
    /** Root of the converted tree. */
    public convertedRoot: Fr,
  ) {
    if (shaRoot.length !== 32) {
      throw new Error(`shaRoot buffer must be 32 bytes. Got ${shaRoot.length} bytes`);
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
    return new ParityPublicInputs(reader.readObject(AggregationObject), reader.readBytes(32), reader.readObject(Fr));
  }
}
