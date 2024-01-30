import { ABIParameterVisibility, FunctionAbi, FunctionType } from '@aztec/foundation/abi';
import { Fr, Point } from '@aztec/foundation/fields';
import { ContractInstance } from '@aztec/types/contracts';

import { EthAddress, PublicKey } from '../index.js';
import {
  computeContractAddressFromInstance,
  computeContractAddressFromPartial,
  computeInitializationHash,
  computePartialAddress,
  computePublicKeysHash,
  computeSaltedInitializationHash,
} from './contract_address.js';

describe('ContractAddress', () => {
  it('computeContractAddressFromInstance', () => {
    const mockInstance: ContractInstance = {
      version: 1,
      contractClassId: new Fr(1),
      initializationHash: new Fr(2),
      portalContractAddress: EthAddress.fromField(new Fr(3)),
      publicKeysHash: new Fr(4),
      salt: new Fr(5),
    };
    const result = computeContractAddressFromInstance(mockInstance);
    expect(result).toMatchSnapshot();
  });

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

  it('computeContractAddressFromPartial', () => {
    const mockArgs = {
      publicKeyHash: new Fr(1),
      partialAddress: new Fr(2),
    };
    const result = computeContractAddressFromPartial(mockArgs);
    expect(result).toMatchSnapshot();
  });

  it('computePublicKeysHash', () => {
    const mockPublicKey: PublicKey = new Point(new Fr(1), new Fr(2));
    const result = computePublicKeysHash(mockPublicKey);
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
});
