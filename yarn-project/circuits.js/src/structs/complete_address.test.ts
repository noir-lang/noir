import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';

import { CompleteAddress } from './complete_address.js';

describe('CompleteAddress', () => {
  // TODO(#5834): re-enable or remove this test
  it.skip('refuses to add an account with incorrect address for given partial address and pubkey', () => {
    expect(() => CompleteAddress.create(AztecAddress.random(), Point.random(), Fr.random())).toThrow(
      /cannot be derived/,
    );
  });

  it('equals returns true when 2 instances are equal', () => {
    const address1 = CompleteAddress.random();
    const address2 = CompleteAddress.create(address1.address, address1.publicKey, address1.partialAddress);
    expect(address1.equals(address2)).toBe(true);
  });

  it('equals returns true when 2 instances are not equal', () => {
    const address1 = CompleteAddress.random();
    const address2 = CompleteAddress.random();
    expect(address1.equals(address2)).toBe(false);
  });

  it('serializes / deserializes correctly', () => {
    const expectedAddress = CompleteAddress.random();
    const address = CompleteAddress.fromBuffer(expectedAddress.toBuffer());
    expect(address.equals(expectedAddress)).toBe(true);
  });
});
