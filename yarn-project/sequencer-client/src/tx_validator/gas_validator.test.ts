import { type Tx, mockTx } from '@aztec/circuit-types';
import { AztecAddress, Fr, FunctionSelector, GasSettings } from '@aztec/circuits.js';
import { pedersenHash } from '@aztec/foundation/crypto';
import { GasTokenContract } from '@aztec/noir-contracts.js';
import { GasTokenAddress } from '@aztec/protocol-contracts/gas-token';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { GasTxValidator, type PublicStateSource } from './gas_validator.js';
import { patchNonRevertibleFn, patchRevertibleFn } from './test_utils.js';

describe('GasTxValidator', () => {
  let validator: GasTxValidator;
  let publicStateSource: MockProxy<PublicStateSource>;
  let gasTokenAddress: AztecAddress;

  beforeEach(() => {
    gasTokenAddress = GasTokenAddress;
    publicStateSource = mock<PublicStateSource>({
      storageRead: mockFn().mockImplementation((_address: AztecAddress, _slot: Fr) => Fr.ZERO),
    });

    validator = new GasTxValidator(publicStateSource, gasTokenAddress);
  });

  let tx: Tx;
  let payer: AztecAddress;
  let expectedBalanceSlot: Fr;

  const TX_FEE = 100n;

  beforeEach(() => {
    tx = mockTx(1, { numberOfNonRevertiblePublicCallRequests: 2 });
    tx.data.feePayer = AztecAddress.random();
    tx.data.constants.txContext.gasSettings = GasSettings.from({
      ...GasSettings.empty(),
      inclusionFee: new Fr(TX_FEE),
    });
    payer = tx.data.feePayer;
    expectedBalanceSlot = pedersenHash([GasTokenContract.storage.balances.slot, payer]);

    expect(tx.data.constants.txContext.gasSettings.getFeeLimit()).toEqual(new Fr(TX_FEE));
  });

  const mockBalance = (balance: bigint) => {
    publicStateSource.storageRead.mockImplementation((address, slot) =>
      Promise.resolve(address.equals(gasTokenAddress) && slot.equals(expectedBalanceSlot) ? new Fr(balance) : Fr.ZERO),
    );
  };

  const expectValidateSuccess = async (tx: Tx) => {
    const result = await validator.validateTxs([tx]);
    expect(result[0].length).toEqual(1);
    expect(result).toEqual([[tx], []]);
  };

  const expectValidateFail = async (tx: Tx) => {
    const result = await validator.validateTxs([tx]);
    expect(result[1].length).toEqual(1);
    expect(result).toEqual([[], [tx]]);
  };

  it('allows fee paying txs if fee payer has enough balance', async () => {
    mockBalance(TX_FEE);
    await expectValidateSuccess(tx);
  });

  it('allows fee paying txs if fee payer claims enough balance during setup', async () => {
    mockBalance(TX_FEE - 1n);
    patchNonRevertibleFn(tx, 0, {
      address: GasTokenAddress,
      selector: FunctionSelector.fromSignature('_increase_public_balance((Field),Field)'),
      args: [payer, new Fr(1n)],
      msgSender: GasTokenAddress,
    });
    await expectValidateSuccess(tx);
  });

  it('rejects txs if fee payer has not enough balance', async () => {
    mockBalance(TX_FEE - 1n);
    await expectValidateFail(tx);
  });

  it('rejects txs if fee payer has zero balance', async () => {
    await expectValidateFail(tx);
  });

  it('rejects txs if fee payer claims balance outside setup', async () => {
    mockBalance(TX_FEE - 1n);
    patchRevertibleFn(tx, 0, {
      selector: FunctionSelector.fromSignature('_increase_public_balance((Field),Field)'),
      args: [payer, new Fr(1n)],
    });
    await expectValidateFail(tx);
  });

  it.skip('rejects txs with no fee payer', async () => {
    tx.data.feePayer = AztecAddress.ZERO;
    await expectValidateFail(tx);
  });
});
