import { type AnyTx, mockTx, mockTxForRollup } from '@aztec/circuit-types';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { DoubleSpendTxValidator, type NullifierSource } from './double_spend_validator.js';

describe('DoubleSpendTxValidator', () => {
  let txValidator: DoubleSpendTxValidator<AnyTx>;
  let nullifierSource: MockProxy<NullifierSource>;

  beforeEach(() => {
    nullifierSource = mock<NullifierSource>({
      getNullifierIndex: mockFn().mockImplementation(() => {
        return Promise.resolve(undefined);
      }),
    });
    txValidator = new DoubleSpendTxValidator(nullifierSource);
  });

  it('rejects duplicates in non revertible data', async () => {
    const badTx = mockTxForRollup();
    badTx.data.forRollup!.end.nullifiers[1] = badTx.data.forRollup!.end.nullifiers[0];
    await expect(txValidator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
  });

  it('rejects duplicates in revertible data', async () => {
    const badTx = mockTxForRollup();
    badTx.data.forRollup!.end.nullifiers[1] = badTx.data.forRollup!.end.nullifiers[0];
    await expect(txValidator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
  });

  it('rejects duplicates across phases', async () => {
    const badTx = mockTx(1, {
      numberOfNonRevertiblePublicCallRequests: 1,
      numberOfRevertiblePublicCallRequests: 1,
    });
    badTx.data.forPublic!.end.nullifiers[0] = badTx.data.forPublic!.endNonRevertibleData.nullifiers[0];
    await expect(txValidator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
  });

  it('rejects duplicates across txs', async () => {
    const firstTx = mockTxForRollup(1);
    const secondTx = mockTxForRollup(2);
    secondTx.data.forRollup!.end.nullifiers[0] = firstTx.data.forRollup!.end.nullifiers[0];
    await expect(txValidator.validateTxs([firstTx, secondTx])).resolves.toEqual([[firstTx], [secondTx]]);
  });

  it('rejects duplicates against history', async () => {
    const badTx = mockTx();
    nullifierSource.getNullifierIndex.mockReturnValueOnce(Promise.resolve(1n));
    await expect(txValidator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
  });
});
