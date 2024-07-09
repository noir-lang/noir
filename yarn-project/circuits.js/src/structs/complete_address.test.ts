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
      '0x1b554ab89034d16274f0043eb2d4d1104f3d6e2b995a9b8f492dfcd881b8469522f7fcddfa3ce3e8f0cc8e82d7b94cdd740afa3e77f8e4a63ea78a239432dcab0471657de2b6216ade6c506d28fbc22ba8b8ed95c871ad9f3e3984e90d9723a7111223493147f6785514b1c195bb37a2589f22a6596d30bb2bb145fdc9ca8f1e273bbffd678edce8fe30e0deafc4f66d58357c06fd4a820285294b9746c3be9509115c96e962322ffed6522f57194627136b8d03ac7469109707f5e44190c4840c49773308a13d740a7f0d4f0e6163b02c5a408b6f965856b6a491002d073d5b00d3d81beb009873eb7116327cf47c612d5758ef083d4fda78e9b63980b2a7622f567d22d2b02fe1f4ad42db9d58a36afd1983e7e2909d1cab61cafedad6193a0a7c585381b10f4666044266a02405bf6e01fa564c8517d4ad5823493abd31de',
    );

    // Alternatively, a recipient could share the individual components with the sender
    const address = Fr.fromString('0x1b554ab89034d16274f0043eb2d4d1104f3d6e2b995a9b8f492dfcd881b84695');
    const npkM = Point.fromString(
      '0x22f7fcddfa3ce3e8f0cc8e82d7b94cdd740afa3e77f8e4a63ea78a239432dcab0471657de2b6216ade6c506d28fbc22ba8b8ed95c871ad9f3e3984e90d9723a7',
    );
    const ivpkM = Point.fromString(
      '0x111223493147f6785514b1c195bb37a2589f22a6596d30bb2bb145fdc9ca8f1e273bbffd678edce8fe30e0deafc4f66d58357c06fd4a820285294b9746c3be95',
    );
    const ovpkM = Point.fromString(
      '0x09115c96e962322ffed6522f57194627136b8d03ac7469109707f5e44190c4840c49773308a13d740a7f0d4f0e6163b02c5a408b6f965856b6a491002d073d5b',
    );
    const tpkM = Point.fromString(
      '0x00d3d81beb009873eb7116327cf47c612d5758ef083d4fda78e9b63980b2a7622f567d22d2b02fe1f4ad42db9d58a36afd1983e7e2909d1cab61cafedad6193a',
    );

    const partialAddress = Fr.fromString('0x0a7c585381b10f4666044266a02405bf6e01fa564c8517d4ad5823493abd31de');

    const completeAddressFromComponents = new CompleteAddress(
      address,
      new PublicKeys(npkM, ivpkM, ovpkM, tpkM),
      partialAddress,
    );
    // docs:end:instantiate-complete-address

    expect(completeAddressFromComponents.equals(completeAddressFromString)).toBe(true);
  });
});
