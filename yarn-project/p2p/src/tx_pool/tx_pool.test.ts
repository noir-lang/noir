import { MockTx } from '../client/mocks.js';
import { InMemoryTxPool } from './index.js';

describe('In-Memory TX pool', () => {
  it('Adds txs to the pool', () => {
    const pool = new InMemoryTxPool();
    const tx1 = MockTx();

    pool.addTxs([tx1]);
    const poolTx = pool.getTx(tx1.txId);
    expect(poolTx?.txId.toString('hex')).toEqual(tx1.txId.toString('hex'));
  });

  it('Removes txs from the pool', () => {
    const pool = new InMemoryTxPool();
    const tx1 = MockTx();

    pool.addTxs([tx1]);
    pool.deleteTxs([tx1.txId]);

    const poolTx = pool.getTx(tx1.txId);
    expect(poolTx).toBeFalsy();
  });
});
