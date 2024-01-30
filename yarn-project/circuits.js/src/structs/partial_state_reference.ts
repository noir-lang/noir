import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

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

  static empty(): PartialStateReference {
    return new PartialStateReference(
      AppendOnlyTreeSnapshot.empty(),
      AppendOnlyTreeSnapshot.empty(),
      AppendOnlyTreeSnapshot.empty(),
      AppendOnlyTreeSnapshot.empty(),
    );
  }

  toBuffer() {
    return serializeToBuffer(this.noteHashTree, this.nullifierTree, this.contractTree, this.publicDataTree);
  }

  toFieldArray() {
    return [
      ...this.noteHashTree.toFieldArray(),
      ...this.nullifierTree.toFieldArray(),
      ...this.contractTree.toFieldArray(),
      ...this.publicDataTree.toFieldArray(),
    ];
  }

  isEmpty(): boolean {
    return (
      this.noteHashTree.isEmpty() &&
      this.nullifierTree.isEmpty() &&
      this.contractTree.isEmpty() &&
      this.publicDataTree.isEmpty()
    );
  }
}
