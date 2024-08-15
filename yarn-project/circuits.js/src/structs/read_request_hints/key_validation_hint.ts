import { GrumpkinScalar } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

export class KeyValidationHint {
  constructor(
    /** Master secret key used to derive sk_app and pk_m. */
    public skM: GrumpkinScalar,
    /** Index of the request in the array of hints. */
    public requestIndex: number,
  ) {}

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new KeyValidationHint(reader.readObject(GrumpkinScalar), reader.readNumber());
  }

  toBuffer() {
    return serializeToBuffer(this.skM, this.requestIndex);
  }

  static nada(keyValidationRequestLen: number) {
    return new KeyValidationHint(GrumpkinScalar.zero(), keyValidationRequestLen);
  }
}
