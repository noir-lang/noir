import { type FunctionAbi, FunctionType } from '@aztec/foundation/abi';
import { Fr } from '@aztec/foundation/fields';
import { setupCustomSnapshotSerializers } from '@aztec/foundation/testing';

import { AztecAddress, deriveKeys } from '../index.js';
import {
  computeContractAddressFromInstance,
  computeInitializationHash,
  computePartialAddress,
  computeSaltedInitializationHash,
} from './contract_address.js';

describe('ContractAddress', () => {
  setupCustomSnapshotSerializers(expect);
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
      deployer: AztecAddress.fromField(new Fr(4)),
    };
    const result = computeSaltedInitializationHash(mockInstance);
    expect(result).toMatchSnapshot();
  });

  it('computeInitializationHash', () => {
    const mockInitFn: FunctionAbi = {
      functionType: FunctionType.PRIVATE,
      isInitializer: false,
      isInternal: false,
      isStatic: false,
      name: 'fun',
      parameters: [{ name: 'param1', type: { kind: 'boolean' }, visibility: 'private' }],
      returnTypes: [],
    };
    const mockArgs: any[] = [true];
    const result = computeInitializationHash(mockInitFn, mockArgs);
    expect(result).toMatchSnapshot();
  });

  it('computeInitializationHash empty', () => {
    const result = computeInitializationHash(undefined, []);
    expect(result).toEqual(Fr.ZERO);
  });

  it('computeContractAddressFromInstance', () => {
    const secretKey = new Fr(2n);
    const salt = new Fr(3n);
    const contractClassId = new Fr(4n);
    const initializationHash = new Fr(5n);
    const deployer = AztecAddress.fromField(new Fr(7));
    const publicKeysHash = deriveKeys(secretKey).publicKeys.hash();

    const address = computeContractAddressFromInstance({
      publicKeysHash,
      salt,
      contractClassId,
      initializationHash,
      deployer,
      version: 1,
    }).toString();

    expect(address).toMatchSnapshot();
  });
});
