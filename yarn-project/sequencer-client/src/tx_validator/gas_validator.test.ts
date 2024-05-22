import { type Tx, mockTx } from '@aztec/circuit-types';
import { AztecAddress, Fr, GasSettings } from '@aztec/circuits.js';
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
      storageRead: mockFn().mockImplementation((_address: AztecAddress, _slot: Fr) => Fr.ZERO),
    });

    validator = new GasTxValidator(publicStateSource, gasTokenAddress);
  });

  let tx: Tx;
  let payer: AztecAddress;
  let expectedBalanceSlot: Fr;

  beforeEach(() => {
    tx = mockTx(1);
    tx.data.feePayer = AztecAddress.random();
    tx.data.constants.txContext.gasSettings = GasSettings.from({ ...GasSettings.empty(), inclusionFee: new Fr(1n) });
    payer = tx.data.feePayer;
    expectedBalanceSlot = pedersenHash([GasTokenContract.storage.balances.slot, payer]);

    expect(tx.data.constants.txContext.gasSettings.getFeeLimit()).toEqual(new Fr(1n));
  });

  it('allows fee paying txs if teardown caller has enough balance', async () => {
    publicStateSource.storageRead.mockImplementation((address, slot) => {
      if (address.equals(gasTokenAddress) && slot.equals(expectedBalanceSlot)) {
        return Promise.resolve(new Fr(1));
      }
      return Promise.resolve(Fr.ZERO);
    });

    await expect(validator.validateTxs([tx])).resolves.toEqual([[tx], []]);
  });

  it('rejects txs if fee payer is out of balance', async () => {
    await expect(validator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });

  it.skip('rejects txs with no fee payer', async () => {
    tx.data.feePayer = AztecAddress.ZERO;
    await expect(validator.validateTxs([tx])).resolves.toEqual([[], [tx]]);
  });
});
