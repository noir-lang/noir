import { Fr, type PUBLIC_DATA_TREE_HEIGHT, type PublicDataTreeLeafPreimage } from '@aztec/circuits.js';

import { type SiblingPath } from '../sibling_path/index.js';

/**
 * Public data witness.
 * @remarks This allows to prove either:
 * - That a slot in the public data tree is empty (0 value) if it falls within the range of the leaf.
 * - The current value of a slot in the public data tree if it matches exactly the slot of the leaf.
 */
export class PublicDataWitness {
  constructor(
    /**
     * The index of the leaf in the public data tree.
     */
    public readonly index: bigint,
    /**
     * Preimage of a low leaf. All the slots in the range of the leaf are empty, and the current value of the
     * leaf slot is stored in the leaf.
     */
    public readonly leafPreimage: PublicDataTreeLeafPreimage,
    /**
     * Sibling path to prove membership of the leaf.
     */
    public readonly siblingPath: SiblingPath<typeof PUBLIC_DATA_TREE_HEIGHT>,
  ) {}

  /**
   * Returns a field array representation of a public data witness.
   * @returns A field array representation of a public data witness.
   */
  public toFields(): Fr[] {
    return [
      new Fr(this.index),
      new Fr(this.leafPreimage.slot),
      new Fr(this.leafPreimage.value),
      new Fr(this.leafPreimage.nextIndex),
      new Fr(this.leafPreimage.nextSlot),
      ...this.siblingPath.toFields(),
    ];
  }
}
