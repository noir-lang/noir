import { type Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { STATE_REFERENCE_LENGTH } from '../constants.gen.js';
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

  toFields(): Fr[] {
    const fields = [...this.l1ToL2MessageTree.toFields(), ...this.partial.toFields()];
    if (fields.length !== STATE_REFERENCE_LENGTH) {
      throw new Error(
        `Invalid number of fields for StateReference. Expected ${STATE_REFERENCE_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  static fromBuffer(buffer: Buffer | BufferReader): StateReference {
    const reader = BufferReader.asReader(buffer);
    return new StateReference(reader.readObject(AppendOnlyTreeSnapshot), reader.readObject(PartialStateReference));
  }

  static fromFields(fields: Fr[] | FieldReader): StateReference {
    const reader = FieldReader.asReader(fields);

    const l1ToL2MessageTree = AppendOnlyTreeSnapshot.fromFields(reader);
    const partial = PartialStateReference.fromFields(reader);

    return new StateReference(l1ToL2MessageTree, partial);
  }

  static empty(): StateReference {
    return new StateReference(AppendOnlyTreeSnapshot.zero(), PartialStateReference.empty());
  }

  isEmpty(): boolean {
    return this.l1ToL2MessageTree.isZero() && this.partial.isEmpty();
  }
}
