import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';

import { CompleteAddress } from './complete_address.js';

describe('CompleteAddress', () => {
  it('refuses to add an account with incorrect address for given partial address and pubkey', async () => {
    await expect(CompleteAddress.create(AztecAddress.random(), Point.random(), Fr.random())).rejects.toThrowError(
      /cannot be derived/,
    );
  });

  it('equals returns true when 2 instances are equal', async () => {
    const address1 = await CompleteAddress.random();
    const address2 = await CompleteAddress.create(address1.address, address1.publicKey, address1.partialAddress);
    expect(address1.equals(address2)).toBe(true);
  });

  it('equals returns true when 2 instances are not equal', async () => {
    const address1 = await CompleteAddress.random();
    const address2 = await CompleteAddress.random();
    expect(address1.equals(address2)).toBe(false);
  });

  it('serializes / deserializes correctly', async () => {
    const expectedAddress = await CompleteAddress.random();
    const address = CompleteAddress.fromBuffer(expectedAddress.toBuffer());
    expect(address.equals(expectedAddress)).toBe(true);
  });
});
