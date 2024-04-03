import { mockTx, mockTxForRollup } from '@aztec/circuit-types';
import {
  type AztecAddress,
  CallContext,
  EthAddress,
  Fr,
  FunctionData,
  FunctionSelector,
  type GlobalVariables,
  PublicCallRequest,
} from '@aztec/circuits.js';
import { makeAztecAddress, makeGlobalVariables } from '@aztec/circuits.js/testing';
import { pedersenHash } from '@aztec/foundation/crypto';
import { getCanonicalGasTokenAddress } from '@aztec/protocol-contracts/gas-token';
import { type ContractDataSource } from '@aztec/types/contracts';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { type NullifierSource, type PublicStateSource, TxValidator } from './tx_validator.js';

describe('TxValidator', () => {
  let validator: TxValidator;
  let globalVariables: GlobalVariables;
  let nullifierSource: MockProxy<NullifierSource>;
  let publicStateSource: MockProxy<PublicStateSource>;
  let contractDataSource: MockProxy<ContractDataSource>;
  let allowedFPCClass: Fr;
  let allowedFPC: AztecAddress;
  let gasPortalAddress: EthAddress;
  let gasTokenAddress: AztecAddress;

  beforeEach(() => {
    gasPortalAddress = EthAddress.random();
    gasTokenAddress = getCanonicalGasTokenAddress(gasPortalAddress);
    allowedFPCClass = Fr.random();
    allowedFPC = makeAztecAddress(100);

    nullifierSource = mock<NullifierSource>({
      getNullifierIndex: mockFn().mockImplementation(() => {
        return Promise.resolve(undefined);
      }),
    });
    publicStateSource = mock<PublicStateSource>({
      storageRead: mockFn().mockImplementation((contractAddress: AztecAddress, _slot: Fr) => {
        if (contractAddress.equals(gasTokenAddress)) {
          return Promise.resolve(new Fr(1));
        } else {
          return Promise.reject(Fr.ZERO);
        }
      }),
    });
    contractDataSource = mock<ContractDataSource>({
      getContract: mockFn().mockImplementation(() => {
        return Promise.resolve({
          contractClassId: allowedFPCClass,
        });
      }),
    });

    globalVariables = makeGlobalVariables();
    validator = new TxValidator(nullifierSource, publicStateSource, contractDataSource, globalVariables, {
      allowedFeePaymentContractClasses: [allowedFPCClass],
      allowedFeePaymentContractInstances: [allowedFPC],
      gasPortalAddress,
    });
  });

  describe('inspects tx metadata', () => {
    it('allows only transactions for the right chain', async () => {
      const goodTx = nonFeePayingTx();
      const badTx = nonFeePayingTx();
      badTx.data.constants.txContext.chainId = Fr.random();

      await expect(validator.validateTxs([goodTx, badTx])).resolves.toEqual([[goodTx], [badTx]]);
    });
  });

  describe('inspects tx nullifiers', () => {
    it('rejects duplicates in non revertible data', async () => {
      const badTx = nonFeePayingTx();
      badTx.data.forRollup!.end.newNullifiers[1] = badTx.data.forRollup!.end.newNullifiers[0];
      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });

    it('rejects duplicates in revertible data', async () => {
      const badTx = nonFeePayingTx();
      badTx.data.forRollup!.end.newNullifiers[1] = badTx.data.forRollup!.end.newNullifiers[0];
      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });

    it('rejects duplicates across phases', async () => {
      const badTx = nativeFeePayingTx(makeAztecAddress());
      badTx.data.forPublic!.end.newNullifiers[0] = badTx.data.forPublic!.endNonRevertibleData.newNullifiers[0];
      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });

    it('rejects duplicates across txs', async () => {
      const firstTx = nonFeePayingTx();
      const secondTx = nonFeePayingTx();
      secondTx.data.forRollup!.end.newNullifiers[0] = firstTx.data.forRollup!.end.newNullifiers[0];
      await expect(validator.validateTxs([firstTx, secondTx])).resolves.toEqual([[firstTx], [secondTx]]);
    });

    it('rejects duplicates against history', async () => {
      const badTx = nonFeePayingTx();
      nullifierSource.getNullifierIndex.mockReturnValueOnce(Promise.resolve(1n));
      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });
  });

  describe('inspects how fee is paid', () => {
    it('allows native gas', async () => {
      const tx = nativeFeePayingTx(makeAztecAddress());
      // check that the whitelist on contract address won't shadow this check
      contractDataSource.getContract.mockImplementationOnce(() => {
        return Promise.resolve({ contractClassId: Fr.random() } as any);
      });
      await expect(validator.validateTxs([tx])).resolves.toEqual([[tx], []]);
    });

    it('allows correct contract class', async () => {
      const fpc = makeAztecAddress();
      const tx = fxFeePayingTx(fpc);

      contractDataSource.getContract.mockImplementationOnce(address => {
        if (fpc.equals(address)) {
          return Promise.resolve({ contractClassId: allowedFPCClass } as any);
        } else {
          return Promise.resolve({ contractClassId: Fr.random() });
        }
      });

      await expect(validator.validateTxs([tx])).resolves.toEqual([[tx], []]);
    });

    it('allows correct contract', async () => {
      const tx = fxFeePayingTx(allowedFPC);
      // check that the whitelist on contract address works and won't get shadowed by the class whitelist
      contractDataSource.getContract.mockImplementationOnce(() => {
        return Promise.resolve({ contractClassId: Fr.random() } as any);
      });
      await expect(validator.validateTxs([tx])).resolves.toEqual([[tx], []]);
    });

    it('rejects incorrect contract and class', async () => {
      const fpc = makeAztecAddress();
      const tx = fxFeePayingTx(fpc);

      contractDataSource.getContract.mockImplementationOnce(() => {
        return Promise.resolve({ contractClassId: Fr.random() } as any);
      });

      await expect(validator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
    });
  });

  describe('inspects tx gas', () => {
    it('allows native fee paying txs', async () => {
      const sender = makeAztecAddress();
      const expectedBalanceSlot = pedersenHash([new Fr(1), sender]);
      const tx = nativeFeePayingTx(sender);

      publicStateSource.storageRead.mockImplementation((address, slot) => {
        if (address.equals(gasTokenAddress) && slot.equals(expectedBalanceSlot)) {
          return Promise.resolve(new Fr(1));
        } else {
          return Promise.resolve(Fr.ZERO);
        }
      });

      await expect(validator.validateTxs([tx])).resolves.toEqual([[tx], []]);
    });

    it('rejects native fee paying txs if out of balance', async () => {
      const sender = makeAztecAddress();
      const expectedBalanceSlot = pedersenHash([new Fr(1), sender]);
      const tx = nativeFeePayingTx(sender);

      publicStateSource.storageRead.mockImplementation((address, slot) => {
        if (address.equals(gasTokenAddress) && slot.equals(expectedBalanceSlot)) {
          return Promise.resolve(Fr.ZERO);
        } else {
          return Promise.resolve(new Fr(1));
        }
      });

      await expect(validator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
    });

    it('allows txs paying through a fee payment contract', async () => {
      const fpcAddress = makeAztecAddress();
      const expectedBalanceSlot = pedersenHash([new Fr(1), fpcAddress]);
      const tx = fxFeePayingTx(fpcAddress);

      publicStateSource.storageRead.mockImplementation((address, slot) => {
        if (address.equals(gasTokenAddress) && slot.equals(expectedBalanceSlot)) {
          return Promise.resolve(new Fr(1));
        } else {
          return Promise.resolve(Fr.ZERO);
        }
      });

      await expect(validator.validateTxs([tx])).resolves.toEqual([[tx], []]);
    });

    it('rejects txs paying through a fee payment contract out of balance', async () => {
      const fpcAddress = makeAztecAddress();
      const expectedBalanceSlot = pedersenHash([new Fr(1), fpcAddress]);
      const tx = nativeFeePayingTx(fpcAddress);

      publicStateSource.storageRead.mockImplementation((address, slot) => {
        if (address.equals(gasTokenAddress) && slot.equals(expectedBalanceSlot)) {
          return Promise.resolve(Fr.ZERO);
        } else {
          return Promise.resolve(new Fr(1));
        }
      });

      await expect(validator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
    });
  });

  describe('inspects tx max block number', () => {
    it('rejects tx with lower max block number', async () => {
      const badTx = maxBlockNumberTx(globalVariables.blockNumber.sub(new Fr(1)));

      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });

    it('allows tx with larger max block number', async () => {
      const goodTx = maxBlockNumberTx(globalVariables.blockNumber.add(new Fr(1)));

      await expect(validator.validateTxs([goodTx])).resolves.toEqual([[goodTx], []]);
    });

    it('allows tx with equal max block number', async () => {
      const goodTx = maxBlockNumberTx(globalVariables.blockNumber);

      await expect(validator.validateTxs([goodTx])).resolves.toEqual([[goodTx], []]);
    });

    it('allows tx with unset max block number', async () => {
      const goodTx = nonFeePayingTx();

      await expect(validator.validateTxs([goodTx])).resolves.toEqual([[goodTx], []]);
    });
  });

  // get unique txs that are also stable across test runs
  let txSeed = 1;

  function mockValidTx(forRollup = true, numberOfNonRevertiblePublicCallRequests = 0) {
    const tx = forRollup
      ? mockTxForRollup(txSeed++)
      : mockTx(txSeed++, { numberOfNonRevertiblePublicCallRequests, numberOfRevertiblePublicCallRequests: 0 });
    tx.data.constants.txContext.chainId = globalVariables.chainId;
    tx.data.constants.txContext.version = globalVariables.version;
    return tx;
  }

  /** Creates a mock tx for the current chain */
  function nonFeePayingTx() {
    return mockValidTx();
  }

  /** Create a tx that pays for its cost natively */
  function nativeFeePayingTx(feePayer: AztecAddress) {
    const tx = mockValidTx(false, 1);
    const gasTokenAddress = getCanonicalGasTokenAddress(gasPortalAddress);
    const signature = FunctionSelector.random();

    const feeExecutionFn = new PublicCallRequest(
      gasTokenAddress,
      new FunctionData(signature, false),
      new CallContext(feePayer, gasTokenAddress, gasPortalAddress, signature, false, false, 1),
      CallContext.empty(),
      [],
    );

    tx.data.forPublic!.endNonRevertibleData.publicCallStack[0] = feeExecutionFn.toCallRequest();
    tx.enqueuedPublicFunctionCalls[0] = feeExecutionFn;

    return tx;
  }

  /** Create a tx that uses fee abstraction to pay for its cost */
  function fxFeePayingTx(feePaymentContract: AztecAddress) {
    const tx = mockValidTx(false, 2);

    // the contract calls itself. Both functions are internal
    const feeSetupSelector = FunctionSelector.random();
    const feeSetupFn = new PublicCallRequest(
      feePaymentContract,
      new FunctionData(feeSetupSelector, true),
      new CallContext(feePaymentContract, feePaymentContract, EthAddress.ZERO, feeSetupSelector, false, false, 1),
      CallContext.empty(),
      [],
    );
    tx.data.forPublic!.endNonRevertibleData.publicCallStack[0] = feeSetupFn.toCallRequest();
    tx.enqueuedPublicFunctionCalls[0] = feeSetupFn;

    const feeExecutionSelector = FunctionSelector.random();
    const feeExecutionFn = new PublicCallRequest(
      feePaymentContract,
      new FunctionData(feeExecutionSelector, true),
      new CallContext(feePaymentContract, feePaymentContract, EthAddress.ZERO, feeExecutionSelector, false, false, 2),
      CallContext.empty(),
      [],
    );
    tx.data.forPublic!.endNonRevertibleData.publicCallStack[1] = feeExecutionFn.toCallRequest();
    tx.enqueuedPublicFunctionCalls[1] = feeExecutionFn;

    return tx;
  }

  /** Create a tx that constraints its max block number */
  function maxBlockNumberTx(maxBlockNumber: Fr) {
    const tx = nonFeePayingTx();

    tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.isSome = true;
    tx.data.forRollup!.rollupValidationRequests.maxBlockNumber.value = maxBlockNumber;

    return tx;
  }
});
