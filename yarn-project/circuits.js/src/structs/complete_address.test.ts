import { AztecAddress } from '@aztec/foundation/aztec-address';
import { Fr, Point } from '@aztec/foundation/fields';

import { PublicKeys } from '../types/public_keys.js';
import { CompleteAddress } from './complete_address.js';

describe('CompleteAddress', () => {
  it('refuses to add an account with incorrect address for given partial address and pubkey', () => {
    expect(
      () =>
        new CompleteAddress(
          AztecAddress.random(),
          new PublicKeys(Point.random(), Point.random(), Point.random(), Point.random()),
          Fr.random(),
        ),
    ).toThrow(/cannot be derived/);
  });

  it('equals returns true when 2 instances are equal', () => {
    const address1 = CompleteAddress.random();
    const address2 = new CompleteAddress(address1.address, address1.publicKeys, address1.partialAddress);
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

  it('instantiates from string and individual components', () => {
    // docs:start:instantiate-complete-address
    // Typically a recipient would share their complete address with the sender
    const completeAddressFromString = CompleteAddress.fromString(
      '0x1de12596818ab6bc3584b943f791b206ff588d3c307358ab6918f59ed7d381bc02a9372135ce5b49b46102732fabd742c31642543396013dde5b460075864607264c605bc115c6cb92a4db0a6b893fd3777341078693d0af22e3ff53f4c2ee2a2fae73914fc50d325e2707a8e996f1ad498429f715f998225dc6bd2ede05aaee055ee137d28b634322e0ea98afc42dfc48833e8d2879c34d23d6d1d337069cca212af0f28b7865b339e202a0077fd3bd8dddc472d055945ad99c02dcccd28bb22bb3585fca3e5751c9913521a390458d63e4d9b292e4872582f3b13da214470c14083a4567cf4f1e92696e6c01923bc6a8b414159446268b12fe8669ce44f1f5196561aca6c654d2405a5653002cba5552b50b6ce1afc9515ed6682507abcb3010040d791aeb30138efc9c7d36b47684af2f26f686672448349f05934ae7bbbf',
    );

    // Alternatively, a recipient could share the individual components with the sender
    const address = Fr.fromString('0x1de12596818ab6bc3584b943f791b206ff588d3c307358ab6918f59ed7d381bc');
    const npkM = Point.fromString(
      '0x02a9372135ce5b49b46102732fabd742c31642543396013dde5b460075864607264c605bc115c6cb92a4db0a6b893fd3777341078693d0af22e3ff53f4c2ee2a',
    );
    const ivpkM = Point.fromString(
      '0x2fae73914fc50d325e2707a8e996f1ad498429f715f998225dc6bd2ede05aaee055ee137d28b634322e0ea98afc42dfc48833e8d2879c34d23d6d1d337069cca',
    );
    const ovpkM = Point.fromString(
      '0x212af0f28b7865b339e202a0077fd3bd8dddc472d055945ad99c02dcccd28bb22bb3585fca3e5751c9913521a390458d63e4d9b292e4872582f3b13da214470c',
    );
    const tpkM = Point.fromString(
      '0x14083a4567cf4f1e92696e6c01923bc6a8b414159446268b12fe8669ce44f1f5196561aca6c654d2405a5653002cba5552b50b6ce1afc9515ed6682507abcb30',
    );

    const partialAddress = Fr.fromString('0x10040d791aeb30138efc9c7d36b47684af2f26f686672448349f05934ae7bbbf');

    const completeAddressFromComponents = new CompleteAddress(
      address,
      new PublicKeys(npkM, ivpkM, ovpkM, tpkM),
      partialAddress,
    );
    // docs:end:instantiate-complete-address

    expect(completeAddressFromComponents.equals(completeAddressFromString)).toBe(true);
  });
});
