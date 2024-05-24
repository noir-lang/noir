import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { GrumpkinPrivateKey } from '../../types/grumpkin_private_key.js';

export class KeyValidationHint {
  constructor(
    /** Master secret key used to derive sk_app and pk_m. */
    public skM: GrumpkinPrivateKey,
    /** Index of the request in the array of hints. */
    public requestIndex: number,
  ) {}

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new KeyValidationHint(reader.readObject(GrumpkinPrivateKey), reader.readNumber());
  }

  toBuffer() {
    return serializeToBuffer(this.skM, this.requestIndex);
  }

  static empty() {
    return new KeyValidationHint(GrumpkinPrivateKey.zero(), 0);
  }
}
