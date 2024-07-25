import { mockTx } from '@aztec/circuit-types';
import { type AztecAddress, Fr, type FunctionSelector } from '@aztec/circuits.js';
import { makeAztecAddress, makeSelector } from '@aztec/circuits.js/testing';
import { type ContractDataSource } from '@aztec/types/contracts';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { PhasesTxValidator } from './phases_validator.js';
import { patchNonRevertibleFn } from './test_utils.js';

describe('PhasesTxValidator', () => {
  let contractDataSource: MockProxy<ContractDataSource>;
  let txValidator: PhasesTxValidator;
  let allowedContractClass: Fr;
  let allowedContract: AztecAddress;
  let allowedSetupSelector1: FunctionSelector;
  let allowedSetupSelector2: FunctionSelector;

  beforeEach(() => {
    allowedContractClass = Fr.random();
    allowedContract = makeAztecAddress();
    allowedSetupSelector1 = makeSelector(1);
    allowedSetupSelector2 = makeSelector(2);

    contractDataSource = mock<ContractDataSource>({
      getContract: mockFn().mockImplementation(() => {
        return {
          contractClassId: Fr.random(),
        };
      }),
    });

    txValidator = new PhasesTxValidator(contractDataSource, [
      {
        classId: allowedContractClass,
        selector: allowedSetupSelector1,
      },
      {
        address: allowedContract,
        selector: allowedSetupSelector1,
      },
      {
        classId: allowedContractClass,
        selector: allowedSetupSelector2,
      },
      {
        address: allowedContract,
        selector: allowedSetupSelector2,
      },
    ]);
  });

  it('allows setup functions on the contracts allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 1 });
    patchNonRevertibleFn(tx, 0, { address: allowedContract, selector: allowedSetupSelector1 });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });

  it('allows setup functions on the contracts class allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 1 });
    const { address } = patchNonRevertibleFn(tx, 0, { selector: allowedSetupSelector1 });

    contractDataSource.getContract.mockImplementationOnce(contractAddress => {
      if (address.equals(contractAddress)) {
        return Promise.resolve({
          contractClassId: allowedContractClass,
        } as any);
      } else {
        return Promise.resolve(undefined);
      }
    });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });

  it('rejects txs with setup functions not on the allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 2 });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });

  it('rejects setup functions not on the contracts class list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 1 });
    // good selector, bad contract class
    const { address } = patchNonRevertibleFn(tx, 0, { selector: allowedSetupSelector1 });
    contractDataSource.getContract.mockImplementationOnce(contractAddress => {
      if (address.equals(contractAddress)) {
        return Promise.resolve({
          contractClassId: Fr.random(),
        } as any);
      } else {
        return Promise.resolve(undefined);
      }
    });
    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });

  it('allows multiple setup functions on the allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 2 });
    patchNonRevertibleFn(tx, 0, { address: allowedContract, selector: allowedSetupSelector1 });
    patchNonRevertibleFn(tx, 1, { address: allowedContract, selector: allowedSetupSelector2 });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });

  it('rejects if one setup functions is not on the allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 2 });
    patchNonRevertibleFn(tx, 0, { address: allowedContract, selector: allowedSetupSelector1 });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });
});
