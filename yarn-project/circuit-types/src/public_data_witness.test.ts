import { PUBLIC_DATA_TREE_HEIGHT, PublicDataTreeLeafPreimage } from '@aztec/circuits.js';
import { fr } from '@aztec/circuits.js/testing';
import { toBufferBE } from '@aztec/foundation/bigint-buffer';
import { randomInt } from '@aztec/foundation/crypto';

import { PublicDataWitness } from './public_data_witness.js';
import { SiblingPath } from './sibling_path/sibling_path.js';

describe('contract_artifact', () => {
  it('serializes and deserializes an instance', () => {
    const witness = makePublicDataWitness(randomInt(1000000));

    const deserialized = PublicDataWitness.fromBuffer(witness.toBuffer());
    expect(deserialized).toEqual(witness);
  });
});

/**
 * Factory function to create a PublicDataWitness based on given seed.
 * @param seed - A seed used to derive all parameters.
 * @returns A new instance of PublicDataWitness.
 */
function makePublicDataWitness(seed: number): PublicDataWitness {
  const leafPreimage = new PublicDataTreeLeafPreimage(fr(seed + 1), fr(seed + 2), fr(seed + 3), BigInt(seed + 4));
  const siblingPath = new SiblingPath(
    PUBLIC_DATA_TREE_HEIGHT,
    Array.from({ length: PUBLIC_DATA_TREE_HEIGHT }, (_, i) => toBufferBE(BigInt(seed + i + 6), 32)),
  );

  return new PublicDataWitness(BigInt(seed + 5), leafPreimage, siblingPath);
}
