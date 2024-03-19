import { mockTx as baseMockTx } from '@aztec/circuit-types';
import { Fr, GlobalVariables } from '@aztec/circuits.js';
import { makeGlobalVariables } from '@aztec/circuits.js/testing';

import { MockProxy, mock, mockFn } from 'jest-mock-extended';

import { NullifierSource, TxValidator } from './tx_validator.js';

describe('TxValidator', () => {
  let validator: TxValidator;
  let globalVariables: GlobalVariables;
  let nullifierSource: MockProxy<NullifierSource>;

  beforeEach(() => {
    nullifierSource = mock<NullifierSource>({
      getNullifierIndex: mockFn().mockImplementation(() => {
        return Promise.resolve(undefined);
      }),
    });
    globalVariables = makeGlobalVariables();
    validator = new TxValidator(nullifierSource, globalVariables);
  });

  describe('inspects tx metadata', () => {
    it('allows only transactions for the right chain', async () => {
      const goodTx = mockTx();
      const badTx = mockTx();
      badTx.data.constants.txContext.chainId = Fr.random();

      await expect(validator.validateTxs([goodTx, badTx])).resolves.toEqual([[goodTx], [badTx]]);
    });
  });

  describe('inspects tx nullifiers', () => {
    it('rejects duplicates in non revertible data', async () => {
      const badTx = mockTx();
      badTx.data.endNonRevertibleData.newNullifiers[1] = badTx.data.endNonRevertibleData.newNullifiers[0];
      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });

    it('rejects duplicates in revertible data', async () => {
      const badTx = mockTx();
      badTx.data.end.newNullifiers[1] = badTx.data.end.newNullifiers[0];
      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });

    it('rejects duplicates across phases', async () => {
      const badTx = mockTx();
      badTx.data.end.newNullifiers[0] = badTx.data.endNonRevertibleData.newNullifiers[0];
      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });

    it('rejects duplicates across txs', async () => {
      const firstTx = mockTx();
      const secondTx = mockTx();
      secondTx.data.end.newNullifiers[0] = firstTx.data.end.newNullifiers[0];
      await expect(validator.validateTxs([firstTx, secondTx])).resolves.toEqual([[firstTx], [secondTx]]);
    });

    it('rejects duplicates against history', async () => {
      const badTx = mockTx();
      nullifierSource.getNullifierIndex.mockReturnValueOnce(Promise.resolve(1n));
      await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
    });
  });

  // get unique txs that are also stable across test runs
  let txSeed = 1;
  /** Creates a mock tx for the current chain */
  function mockTx() {
    const tx = baseMockTx(txSeed++, false);
    tx.data.constants.txContext.chainId = globalVariables.chainId;
    tx.data.constants.txContext.version = globalVariables.version;

    return tx;
  }
});
