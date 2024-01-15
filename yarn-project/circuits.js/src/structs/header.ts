import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { GlobalVariables } from './global_variables.js';
import { AppendOnlyTreeSnapshot } from './rollup/append_only_tree_snapshot.js';
import { StateReference } from './state_reference.js';

export const NUM_BYTES_PER_SHA256 = 32;

/** A header of an L2 block. */
export class Header {
  constructor(
    /** Snapshot of archive before the block is applied. */
    public lastArchive: AppendOnlyTreeSnapshot,
    /** Hash of the body of an L2 block. */
    public bodyHash: Buffer,
    /** State reference. */
    public state: StateReference,
    /** Global variables of an L2 block. */
    public globalVariables: GlobalVariables,
  ) {
    if (bodyHash.length !== 32) {
      throw new Error('Body hash buffer must be 32 bytes');
    }
  }

  toBuffer() {
    // Note: The order here must match the order in the HeaderDecoder solidity library.
    return serializeToBuffer(this.lastArchive, this.bodyHash, this.state, this.globalVariables);
  }

  static fromBuffer(buffer: Buffer | BufferReader): Header {
    const reader = BufferReader.asReader(buffer);
    return new Header(
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readBytes(NUM_BYTES_PER_SHA256),
      reader.readObject(StateReference),
      reader.readObject(GlobalVariables),
    );
  }
}
