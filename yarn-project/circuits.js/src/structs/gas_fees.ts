import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer, serializeToFields } from '@aztec/foundation/serialize';
import { type FieldsOf } from '@aztec/foundation/types';

import { inspect } from 'util';

/** Gas prices for each dimension. */
export class GasFees {
  constructor(public readonly feePerDaGas: Fr, public readonly feePerL1Gas: Fr, public readonly feePerL2Gas: Fr) {}

  static from(fields: FieldsOf<GasFees>) {
    return new GasFees(fields.feePerDaGas, fields.feePerL1Gas, fields.feePerL2Gas);
  }

  static random() {
    return new GasFees(Fr.random(), Fr.random(), Fr.random());
  }

  static empty() {
    return new GasFees(Fr.ZERO, Fr.ZERO, Fr.ZERO);
  }

  isEmpty() {
    return this.feePerDaGas.isZero() && this.feePerL1Gas.isZero() && this.feePerL2Gas.isZero();
  }

  static fromBuffer(buffer: Buffer | BufferReader): GasFees {
    const reader = BufferReader.asReader(buffer);
    return new GasFees(reader.readObject(Fr), reader.readObject(Fr), reader.readObject(Fr));
  }

  toBuffer() {
    return serializeToBuffer(this.feePerDaGas, this.feePerL1Gas, this.feePerL2Gas);
  }

  static fromFields(fields: Fr[] | FieldReader) {
    const reader = FieldReader.asReader(fields);
    return new GasFees(reader.readField(), reader.readField(), reader.readField());
  }

  toFields() {
    return serializeToFields(this.feePerDaGas, this.feePerL1Gas, this.feePerL2Gas);
  }

  static fromJSON(obj: any) {
    return new GasFees(Fr.fromString(obj.feePerDaGas), Fr.fromString(obj.feePerL1Gas), Fr.fromString(obj.feePerL2Gas));
  }

  toJSON() {
    return {
      feePerDaGas: this.feePerDaGas.toString(),
      feePerL1Gas: this.feePerL1Gas.toString(),
      feePerL2Gas: this.feePerL2Gas.toString(),
    };
  }

  [inspect.custom]() {
    return `GasFees { feePerDaGas=${this.feePerDaGas} feePerL1Gas=${this.feePerL1Gas} feePerL2Gas=${this.feePerL2Gas} }`;
  }
}
