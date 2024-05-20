import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { GrumpkinPrivateKey } from '../../types/grumpkin_private_key.js';

export class NullifierKeyHint {
  constructor(public privateKey: GrumpkinPrivateKey, public requestIndex: number) {}

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new NullifierKeyHint(reader.readObject(GrumpkinPrivateKey), reader.readNumber());
  }

  toBuffer() {
    return serializeToBuffer(this.privateKey, this.requestIndex);
  }

  static empty() {
    return new NullifierKeyHint(GrumpkinPrivateKey.zero(), 0);
  }
}
