import { type Tx, mockTx } from '@aztec/circuit-types';
import { type AztecAddress, Fr, type FunctionSelector } from '@aztec/circuits.js';
import { makeAztecAddress, makeSelector } from '@aztec/circuits.js/testing';
import { type ContractDataSource } from '@aztec/types/contracts';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { PhasesTxValidator } from './phases_validator.js';

describe('PhasesTxValidator', () => {
  let contractDataSource: MockProxy<ContractDataSource>;
  let txValidator: PhasesTxValidator;
  let allowedContractClass: Fr;
  let allowedContract: AztecAddress;
  let allowedSetupSelector1: FunctionSelector;
  let allowedSetupSelector2: FunctionSelector;
  let allowedTeardownSelector: FunctionSelector;

  beforeEach(() => {
    allowedContractClass = Fr.random();
    allowedContract = makeAztecAddress();
    allowedSetupSelector1 = makeSelector(1);
    allowedSetupSelector2 = makeSelector(2);
    allowedTeardownSelector = makeSelector(3);

    contractDataSource = mock<ContractDataSource>({
      getContract: mockFn().mockImplementation(() => {
        return {
          contractClassId: Fr.random(),
        };
      }),
    });

    txValidator = new PhasesTxValidator(
      contractDataSource,
      [
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
      ],
      [
        {
          classId: allowedContractClass,
          selector: allowedTeardownSelector,
        },
        {
          address: allowedContract,
          selector: allowedTeardownSelector,
        },
      ],
    );
  });

  it('allows teardown functions on the contracts allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 1 });
    patchNonRevertibleFn(tx, 0, { address: allowedContract, selector: allowedTeardownSelector });
    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });

  it('allows teardown functions on the contracts class allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 1 });
    const { address } = patchNonRevertibleFn(tx, 0, { selector: allowedTeardownSelector });
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

  it('rejects teardown functions not on the contracts class list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 1 });
    // good selector, bad contract class
    const { address } = patchNonRevertibleFn(tx, 0, { selector: allowedTeardownSelector });
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

  it('rejects teardown functions not on the selector allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 1 });
    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });

  it('allows setup functions on the contracts allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 2 });
    patchNonRevertibleFn(tx, 0, { address: allowedContract, selector: allowedSetupSelector1 });
    patchNonRevertibleFn(tx, 1, { address: allowedContract, selector: allowedTeardownSelector });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });

  it('allows setup functions on the contracts class allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 2 });
    const { address } = patchNonRevertibleFn(tx, 0, { selector: allowedSetupSelector1 });
    patchNonRevertibleFn(tx, 1, { address: allowedContract, selector: allowedTeardownSelector });

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
    // only patch teardown
    patchNonRevertibleFn(tx, 1, { address: allowedContract, selector: allowedTeardownSelector });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });

  it('rejects setup functions not on the contracts class list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 2 });
    // good selector, bad contract class
    const { address } = patchNonRevertibleFn(tx, 0, { selector: allowedSetupSelector1 });
    patchNonRevertibleFn(tx, 1, { address: allowedContract, selector: allowedTeardownSelector });
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
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 3 });
    patchNonRevertibleFn(tx, 0, { address: allowedContract, selector: allowedSetupSelector1 });
    patchNonRevertibleFn(tx, 1, { address: allowedContract, selector: allowedSetupSelector2 });
    patchNonRevertibleFn(tx, 2, { address: allowedContract, selector: allowedTeardownSelector });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });

  it('rejects if one setup functions is not on the allow list', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 3 });
    patchNonRevertibleFn(tx, 0, { address: allowedContract, selector: allowedSetupSelector1 });
    // don't patch index 1
    patchNonRevertibleFn(tx, 2, { address: allowedContract, selector: allowedTeardownSelector });

    await expect(txValidator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });

  function patchNonRevertibleFn(
    tx: Tx,
    index: number,
    { address, selector }: { address?: AztecAddress; selector: FunctionSelector },
  ): { address: AztecAddress; selector: FunctionSelector } {
    const fn = tx.enqueuedPublicFunctionCalls.at(-1 * index - 1)!;
    fn.contractAddress = address ?? fn.contractAddress;
    fn.functionData.selector = selector;
    tx.data.forPublic!.endNonRevertibleData.publicCallStack[index] = fn.toCallRequest();

    return {
      address: fn.contractAddress,
      selector: fn.functionData.selector,
    };
  }
});
