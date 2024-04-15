import { type AnyTx, mockTx, mockTxForRollup } from '@aztec/circuit-types';
import { Fr, type GlobalVariables, MaxBlockNumber } from '@aztec/circuits.js';
import { makeGlobalVariables } from '@aztec/circuits.js/testing';

import { MetadataTxValidator } from './metadata_validator.js';

describe('MetadataTxValidator', () => {
  let globalVariables: GlobalVariables;
  let validator: MetadataTxValidator<AnyTx>;

  beforeEach(() => {
    globalVariables = makeGlobalVariables(1, 42);
    validator = new MetadataTxValidator(globalVariables);
  });

  it('allows only transactions for the right chain', async () => {
    const goodTxs = [mockTx(1), mockTxForRollup(2)];
    const badTxs = [mockTx(3), mockTxForRollup(4)];

    goodTxs.forEach(tx => {
      tx.data.constants.txContext.chainId = globalVariables.chainId;
    });

    badTxs.forEach(tx => {
      tx.data.constants.txContext.chainId = globalVariables.chainId.add(new Fr(1));
    });

    await expect(validator.validateTxs([...goodTxs, ...badTxs])).resolves.toEqual([goodTxs, badTxs]);
  });

  it.each([42, 43])('allows txs with valid max block number', async maxBlockNumber => {
    const goodTx = mockTxForRollup(1);
    goodTx.data.constants.txContext.chainId = globalVariables.chainId;
    goodTx.data.forRollup!.rollupValidationRequests.maxBlockNumber = new MaxBlockNumber(true, new Fr(maxBlockNumber));

    await expect(validator.validateTxs([goodTx])).resolves.toEqual([[goodTx], []]);
  });

  it('allows txs with unset max block number', async () => {
    const goodTx = mockTxForRollup(1);
    goodTx.data.constants.txContext.chainId = globalVariables.chainId;
    goodTx.data.forRollup!.rollupValidationRequests.maxBlockNumber = new MaxBlockNumber(false, Fr.ZERO);

    await expect(validator.validateTxs([goodTx])).resolves.toEqual([[goodTx], []]);
  });

  it('rejects txs with lower max block number', async () => {
    const badTx = mockTxForRollup(1);
    badTx.data.constants.txContext.chainId = globalVariables.chainId;
    badTx.data.forRollup!.rollupValidationRequests.maxBlockNumber = new MaxBlockNumber(
      true,
      globalVariables.blockNumber.sub(new Fr(1)),
    );
    await expect(validator.validateTxs([badTx])).resolves.toEqual([[], [badTx]]);
  });
});
