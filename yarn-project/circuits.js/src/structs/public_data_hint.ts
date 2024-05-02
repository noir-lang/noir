import { Fr } from '@aztec/foundation/fields';
import { BufferReader, serializeToBuffer } from '@aztec/foundation/serialize';

import { PUBLIC_DATA_TREE_HEIGHT } from '../constants.gen.js';
import { MembershipWitness } from './membership_witness.js';
import { PublicDataTreeLeafPreimage } from './trees/index.js';

export class PublicDataHint {
  constructor(
    public leafSlot: Fr,
    public value: Fr,
    public overrideCounter: number,
    public membershipWitness: MembershipWitness<typeof PUBLIC_DATA_TREE_HEIGHT>,
    public leafPreimage: PublicDataTreeLeafPreimage,
  ) {}

  static empty() {
    return new PublicDataHint(
      Fr.ZERO,
      Fr.ZERO,
      0,
      MembershipWitness.empty(PUBLIC_DATA_TREE_HEIGHT),
      PublicDataTreeLeafPreimage.empty(),
    );
  }

  static fromBuffer(buffer: Buffer | BufferReader) {
    const reader = BufferReader.asReader(buffer);
    return new PublicDataHint(
      reader.readObject(Fr),
      reader.readObject(Fr),
      reader.readNumber(),
      MembershipWitness.fromBuffer(reader, PUBLIC_DATA_TREE_HEIGHT),
      reader.readObject(PublicDataTreeLeafPreimage),
    );
  }

  toBuffer() {
    return serializeToBuffer(
      this.leafSlot,
      this.value,
      this.overrideCounter,
      this.membershipWitness,
      this.leafPreimage,
    );
  }
}
