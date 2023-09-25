import { Fr } from '@aztec/foundation/fields';
import { BufferReader } from '@aztec/foundation/serialize';

import { serializeToBuffer } from '../../utils/serialize.js';
import { STRING_ENCODING, UInt32 } from '../shared.js';

/**
 * Snapshot of an append only tree.
 *
 * Used in circuits to verify that tree insertions are performed correctly.
 */
export class AppendOnlyTreeSnapshot {
  constructor(
    /**
     * Root of the append only tree when taking the snapshot.
     */
    public root: Fr,
    /**
     * Index of the next available leaf in the append only tree.
     *
     * Note: We include the next available leaf index in the snapshot so that the snapshot can be used to verify that
     *       the insertion was performed at the correct place. If we only verified tree root then it could happen that
     *       some leaves would get overwritten and the tree root check would still pass.
     *       TLDR: We need to store the next available leaf index to ensure that the "append only" property was
     *             preserved when verifying state transitions.
     */
    public nextAvailableLeafIndex: UInt32,
  ) {}

  toBuffer() {
    return serializeToBuffer(this.root, this.nextAvailableLeafIndex);
  }

  toString(): string {
    return this.toBuffer().toString(STRING_ENCODING);
  }

  static fromBuffer(buffer: Buffer | BufferReader): AppendOnlyTreeSnapshot {
    const reader = BufferReader.asReader(buffer);
    return new AppendOnlyTreeSnapshot(reader.readFr(), reader.readNumber());
  }

  static fromString(str: string): AppendOnlyTreeSnapshot {
    return AppendOnlyTreeSnapshot.fromBuffer(Buffer.from(str, STRING_ENCODING));
  }

  static empty() {
    return new AppendOnlyTreeSnapshot(Fr.ZERO, 0);
  }
}
