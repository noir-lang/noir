import { Fr, Point } from '@aztec/foundation/fields';
import { updateInlineTestData } from '@aztec/foundation/testing';

import { PublicKeys } from './public_keys.js';

describe('PublicKeys', () => {
  it('computes public keys hash', () => {
    const keys = new PublicKeys(
      new Point(new Fr(1n), new Fr(2n), false),
      new Point(new Fr(3n), new Fr(4n), false),
      new Point(new Fr(5n), new Fr(6n), false),
      new Point(new Fr(7n), new Fr(8n), false),
    );

    const hash = keys.hash().toString();
    expect(hash).toMatchInlineSnapshot(`"0x146f68c0e0ba4067d61a3304bbfdec0797d5df1357db6c01247c48bfb345c7d7"`);

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData('noir-projects/aztec-nr/aztec/src/keys/public_keys.nr', 'expected_public_keys_hash', hash);
  });

  it('computes empty keys hash', () => {
    const keys = PublicKeys.empty();

    const hash = keys.hash().toString();
    expect(hash).toMatchInlineSnapshot(`"0x0000000000000000000000000000000000000000000000000000000000000000"`);

    // Run with AZTEC_GENERATE_TEST_DATA=1 to update noir test data
    updateInlineTestData('noir-projects/aztec-nr/aztec/src/keys/public_keys.nr', 'test_data_empty_hash', hash);
  });
});
