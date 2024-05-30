import { mockTx, mockTxForRollup } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';

import { MetadataTxValidator } from './tx_metadata_validator.js';

describe('MetadataTxValidator', () => {
  let chainId: Fr;
  let validator: MetadataTxValidator;

  beforeEach(() => {
    chainId = new Fr(123);
    validator = new MetadataTxValidator(chainId);
  });

  it('allows only transactions for the right chain', async () => {
    const goodTxs = [mockTx(1), mockTxForRollup(2)];
    const badTxs = [mockTx(3), mockTxForRollup(4)];

    goodTxs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId;
    });

    badTxs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId.add(new Fr(1));
    });

    await expect(validator.validateTxs([...goodTxs, ...badTxs])).resolves.toEqual([goodTxs, badTxs]);
  });
});
