import { BufferReader, Fr } from '@aztec/foundation';
import { serializeToBuffer } from '../../utils/serialize.js';
import { UInt32 } from '../shared.js';

export class AppendOnlyTreeSnapshot {
  /**
   * Constructs a new append only tree snapshot.
   * @param root - Root of the append only tree.
   * @param nextAvailableLeafIndex - Index of the next available leaf in the append only tree.
   * Note: We include the next available leaf index in the snapshot so that the snapshot can be used to verify that
   *       the insertion was performed at the correct place. If we only verified tree root then it could happen that
   *       some leaves would get overwritten and the tree root check would still pass.
   *       TLDR: We need to store the next available leaf index to ensure that the "append only" property was
   *             preserved when verifying state transitions.
   */
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
