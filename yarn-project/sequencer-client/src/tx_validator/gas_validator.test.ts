import { type Tx, mockTx } from '@aztec/circuit-types';
import { AztecAddress, Fr } from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';
import { GasTokenContract } from '@aztec/noir-contracts.js';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { GasTxValidator, type PublicStateSource } from './gas_validator.js';

describe('GasTxValidator', () => {
  let validator: GasTxValidator;
  let publicStateSource: MockProxy<PublicStateSource>;
  let gasTokenAddress: AztecAddress;

  beforeEach(() => {
    gasTokenAddress = AztecAddress.random();
    publicStateSource = mock<PublicStateSource>({
      storageRead: mockFn().mockImplementation((_address: AztecAddress, _slot: Fr) => {
        return 0n;
      }),
    });

    validator = new GasTxValidator(publicStateSource, gasTokenAddress, true);
  });

  let tx: Tx;
  let payer: AztecAddress;
  let expectedBalanceSlot: Fr;

  beforeEach(() => {
    tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 1 });
    const teardownFn = tx.enqueuedPublicFunctionCalls.at(-1)!;
    payer = teardownFn.callContext.msgSender;
    expectedBalanceSlot = pedersenHash([GasTokenContract.storage.balances.slot, payer]);
  });

  it('allows fee paying txs if teardown caller has enough balance', async () => {
    publicStateSource.storageRead.mockImplementation((address, slot) => {
      if (address.equals(gasTokenAddress) && slot.equals(expectedBalanceSlot)) {
        return Promise.resolve(new Fr(1));
      } else {
        return Promise.resolve(Fr.ZERO);
      }
    });

    await expect(validator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });

  it('rejects txs if fee payer is out of balance', async () => {
    publicStateSource.storageRead.mockImplementation((address, slot) => {
      if (address.equals(gasTokenAddress) && slot.equals(expectedBalanceSlot)) {
        return Promise.resolve(Fr.ZERO);
      } else {
        return Promise.resolve(new Fr(1));
      }
    });
    await expect(validator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });

  it('rejects txs with no teardown call', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 0 });
    await expect(validator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });

  it('allows txs without a teardown call enqueued if fee not mandatory', async () => {
    const tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 0 });
    const lenientTxValidator = new GasTxValidator(publicStateSource, gasTokenAddress, false);
    await expect(lenientTxValidator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });
});
