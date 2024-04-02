import { Fr, type NULLIFIER_TREE_HEIGHT, NullifierLeafPreimage } from '@aztec/circuits.js';

import { SiblingPath } from '../sibling_path/index.js';

/**
 * Nullifier membership witness.
 * @remarks When this represents membership witness of a low nullifier it can be used to perform a nullifier
 * non-inclusion proof by leveraging the "linked list structure" of leaves and proving that a lower nullifier
 * is pointing to a bigger next value than the nullifier we are trying to prove non-inclusion for.
 */
export class NullifierMembershipWitness {
  constructor(
    /**
     * The index of the nullifier in a tree.
     */
    public readonly index: bigint,
    /**
     * Preimage of the nullifier.
     */
    public readonly leafPreimage: NullifierLeafPreimage,
    /**
     * Sibling path to prove membership of the nullifier.
     */
    public readonly siblingPath: SiblingPath<typeof NULLIFIER_TREE_HEIGHT>,
  ) {}

  /**
   * Returns a field array representation of a nullifier witness.
   * @returns A field array representation of a nullifier witness.
   */
  public toFields(): Fr[] {
    return [new Fr(this.index), ...this.leafPreimage.toFields(), ...this.siblingPath.toFields()];
  }

  public toJSON() {
    return {
      index: '0x' + this.index.toString(16),
      leafPreimage: this.leafPreimage.toJSON(),
      siblingPath: this.siblingPath.toString(),
    };
  }

  static fromJSON(json: any): NullifierMembershipWitness {
    return new NullifierMembershipWitness(
      BigInt(json.index),
      NullifierLeafPreimage.fromJSON(json.leafPreimage),
      SiblingPath.fromString<typeof NULLIFIER_TREE_HEIGHT>(json.siblingPath),
    );
  }
}
