import { ABIParameterVisibility, FunctionAbi, FunctionType } from '@aztec/foundation/abi';
import { Fr, Point } from '@aztec/foundation/fields';

import { EthAddress } from '../index.js';
import {
  computeContractAddressFromInstance,
  computeContractAddressFromPartial,
  computeInitializationHash,
  computePartialAddress,
  computePublicKeysHash,
  computeSaltedInitializationHash,
} from './contract_address.js';

describe('ContractAddress', () => {
  it('computePartialAddress', () => {
    const mockInstance = {
      contractClassId: new Fr(1),
      saltedInitializationHash: new Fr(2),
    };
    const result = computePartialAddress(mockInstance);
    expect(result).toMatchSnapshot();
  });

  it('computeSaltedInitializationHash', () => {
    const mockInstance = {
      initializationHash: new Fr(1),
      salt: new Fr(2),
      portalContractAddress: EthAddress.fromField(new Fr(3)),
    };
    const result = computeSaltedInitializationHash(mockInstance);
    expect(result).toMatchSnapshot();
  });

  it('computeInitializationHash', () => {
    const mockInitFn: FunctionAbi = {
      functionType: FunctionType.SECRET,
      isInternal: false,
      name: 'fun',
      parameters: [{ name: 'param1', type: { kind: 'boolean' }, visibility: ABIParameterVisibility.SECRET }],
      returnTypes: [],
    };
    const mockArgs: any[] = [true];
    const result = computeInitializationHash(mockInitFn, mockArgs);
    expect(result).toMatchSnapshot();
  });

  it('computeContractAddressFromInstance', () => {
    const publicKey = new Point(new Fr(1n), new Fr(2n));
    const salt = new Fr(3n);
    const contractClassId = new Fr(4n);
    const initializationHash = new Fr(5n);
    const portalContractAddress = EthAddress.fromField(new Fr(6n));

    const address = computeContractAddressFromInstance({
      publicKeysHash: computePublicKeysHash(publicKey),
      salt,
      contractClassId,
      initializationHash,
      portalContractAddress,
      version: 1,
    }).toString();

    expect(address).toMatchSnapshot();

    // Value used in "compute_address" test in aztec_address.nr
    // console.log("address", address);
  });

  it('Public key hash matches Noir', () => {
    const publicKey = new Point(new Fr(1n), new Fr(2n));
    const hash = computePublicKeysHash(publicKey).toString();
    expect(hash).toMatchSnapshot();

    // Value used in "compute_public_keys_hash" test in public_keys_hash.nr
    // console.log("hash", hash);
  });

  it('Address from partial matches Noir', () => {
    const publicKey = new Point(new Fr(1n), new Fr(2n));
    const partialAddress = new Fr(3n);
    const address = computeContractAddressFromPartial({ publicKey, partialAddress }).toString();
    expect(address).toMatchSnapshot();

    // Value used in "compute_address_from_partial_and_pubkey" test in aztec_address.nr
    // console.log("address", address);
  });
});
