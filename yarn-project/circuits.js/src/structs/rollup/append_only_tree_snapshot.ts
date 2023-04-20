import { BufferReader, Fr } from '@aztec/foundation';
import { serializeToBuffer } from '../../utils/serialize.js';
import { UInt32 } from '../shared.js';

export class AppendOnlyTreeSnapshot {
  constructor(public root: Fr, public nextAvailableLeafIndex: UInt32) {}

  toBuffer() {
    return serializeToBuffer(this.root, this.nextAvailableLeafIndex);
  }

  static fromBuffer(buffer: Buffer | BufferReader): AppendOnlyTreeSnapshot {
    const reader = BufferReader.asReader(buffer);
    return new AppendOnlyTreeSnapshot(reader.readFr(), reader.readNumber());
  }

  static empty() {
    return new AppendOnlyTreeSnapshot(Fr.ZERO, 0);
  }
}
