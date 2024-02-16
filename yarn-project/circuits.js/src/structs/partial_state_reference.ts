import { Fr } from '@aztec/foundation/fields';
import { BufferReader, FieldReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { PARTIAL_STATE_REFERENCE_LENGTH } from '../constants.gen.js';
import { AppendOnlyTreeSnapshot } from './rollup/append_only_tree_snapshot.js';

/**
 * Stores snapshots of trees which are commonly needed by base or merge rollup circuits.
 */
export class PartialStateReference {
  constructor(
    /** Snapshot of the note hash tree. */
    public readonly noteHashTree: AppendOnlyTreeSnapshot,
    /** Snapshot of the nullifier tree. */
    public readonly nullifierTree: AppendOnlyTreeSnapshot,
    /** Snapshot of the contract tree. */
    public readonly contractTree: AppendOnlyTreeSnapshot,
    /** Snapshot of the public data tree. */
    public readonly publicDataTree: AppendOnlyTreeSnapshot,
  ) {}

  static fromBuffer(buffer: Buffer | BufferReader): PartialStateReference {
    const reader = BufferReader.asReader(buffer);
    return new PartialStateReference(
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
      reader.readObject(AppendOnlyTreeSnapshot),
    );
  }

  static fromFields(fields: Fr[] | FieldReader): PartialStateReference {
    const reader = FieldReader.asReader(fields);

    const noteHashTree = AppendOnlyTreeSnapshot.fromFields(reader);
    const nullifierTree = AppendOnlyTreeSnapshot.fromFields(reader);
    const contractTree = AppendOnlyTreeSnapshot.fromFields(reader);
    const publicDataTree = AppendOnlyTreeSnapshot.fromFields(reader);

    return new PartialStateReference(noteHashTree, nullifierTree, contractTree, publicDataTree);
  }

  static empty(): PartialStateReference {
    return new PartialStateReference(
      AppendOnlyTreeSnapshot.zero(),
      AppendOnlyTreeSnapshot.zero(),
      AppendOnlyTreeSnapshot.zero(),
      AppendOnlyTreeSnapshot.zero(),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.noteHashTree, this.nullifierTree, this.contractTree, this.publicDataTree);
  }

  toFields() {
    const fields = [
      ...this.noteHashTree.toFields(),
      ...this.nullifierTree.toFields(),
      ...this.contractTree.toFields(),
      ...this.publicDataTree.toFields(),
    ];
    if (fields.length !== PARTIAL_STATE_REFERENCE_LENGTH) {
      throw new Error(
        `Invalid number of fields for PartialStateReference. Expected ${PARTIAL_STATE_REFERENCE_LENGTH}, got ${fields.length}`,
      );
    }
    return fields;
  }

  isEmpty(): boolean {
    return (
      this.noteHashTree.isZero() &&
      this.nullifierTree.isZero() &&
      this.contractTree.isZero() &&
      this.publicDataTree.isZero()
    );
  }
}
