import { Fr, Point } from '@aztec/foundation/fields';
import { updateInlineTestData } from '@aztec/foundation/testing';

import { PublicKeys } from '../types/public_keys.js';
import { computeAddress } from './derivation.js';

describe('🔑', () => {
  it('computing public keys hash matches Noir', () => {
    const masterNullifierPublicKey = new Point(new Fr(1), new Fr(2));
    const masterIncomingViewingPublicKey = new Point(new Fr(3), new Fr(4));
    const masterOutgoingViewingPublicKey = new Point(new Fr(5), new Fr(6));
    const masterTaggingPublicKey = new Point(new Fr(7), new Fr(8));

    const expected = Fr.fromString('0x2406c1c88b7afc13052335bb9af43fd35034b5ba0a9caab76eda2833cf8ec717');
    expect(
      new PublicKeys(
        masterNullifierPublicKey,
        masterIncomingViewingPublicKey,
        masterOutgoingViewingPublicKey,
        masterTaggingPublicKey,
      ).hash(),
    ).toEqual(expected);

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/aztec-nr/aztec/src/keys/public_keys.nr',
      'expected_public_keys_hash',
      expected.toString(),
    );
  });

  it('Address from partial matches Noir', () => {
    const publicKeysHash = new Fr(1n);
    const partialAddress = new Fr(2n);
    const address = computeAddress(publicKeysHash, partialAddress).toString();
    expect(address).toMatchSnapshot();

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData(
      'noir-projects/noir-protocol-circuits/crates/types/src/address/aztec_address.nr',
      'expected_computed_address_from_partial_and_pubkey',
      address.toString(),
    );
  });
});
