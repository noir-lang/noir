import { MockTx } from '../client/mocks.js';
import { InMemoryTxPool } from './index.js';

describe('In-Memory TX pool', () => {
  it('Adds txs to the pool', () => {
    const pool = new InMemoryTxPool();
    const tx1 = MockTx();

    pool.addTxs([tx1]);
    const poolTx = pool.getTxByHash(tx1.txHash);
    expect(poolTx?.txHash.toString()).toEqual(tx1.txHash.toString());
  });

  it('Removes txs from the pool', () => {
    const pool = new InMemoryTxPool();
    const tx1 = MockTx();

    pool.addTxs([tx1]);
    pool.deleteTxs([tx1.txHash]);

    const poolTx = pool.getTxByHash(tx1.txHash);
    expect(poolTx).toBeFalsy();
  });
});
