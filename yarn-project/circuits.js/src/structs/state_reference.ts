import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { PartialStateReference } from './partial_state_reference.js';
import { AppendOnlyTreeSnapshot } from './rollup/append_only_tree_snapshot.js';

/**
 * Stores snapshots of all the trees but archive.
 */
export class StateReference {
  constructor(
    /** Snapshot of the l1 to l2 message tree. */
    public l1ToL2MessageTree: AppendOnlyTreeSnapshot,
    /** Reference to the rest of the state. */
    public partial: PartialStateReference,
  ) {}

  toBuffer() {
    // Note: The order here must match the order in the HeaderLib solidity library.
    return serializeToBuffer(this.l1ToL2MessageTree, this.partial);
  }

  toFieldArray(): Fr[] {
    return [...this.l1ToL2MessageTree.toFieldArray(), ...this.partial.toFieldArray()];
  }

  static fromBuffer(buffer: Buffer | BufferReader): StateReference {
    const reader = BufferReader.asReader(buffer);
    return new StateReference(reader.readObject(AppendOnlyTreeSnapshot), reader.readObject(PartialStateReference));
  }

  static empty(): StateReference {
    return new StateReference(AppendOnlyTreeSnapshot.empty(), PartialStateReference.empty());
  }

  isEmpty(): boolean {
    return this.l1ToL2MessageTree.isEmpty() && this.partial.isEmpty();
  }
}
