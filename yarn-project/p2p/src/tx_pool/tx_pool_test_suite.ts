import { mockTx } from '@aztec/circuit-types';

import { type TxPool } from './tx_pool.js';

/**
 * Tests a TxPool implementation.
 * @param getTxPool - Gets a fresh TxPool
 */
export function describeTxPool(getTxPool: () => TxPool) {
  let pool: TxPool;

  beforeEach(() => {
    pool = getTxPool();
  });

  it('Adds txs to the pool as pending', async () => {
    const tx1 = mockTx();

    await pool.addTxs([tx1]);
    const poolTx = pool.getTxByHash(tx1.getTxHash());
    expect(poolTx!.getTxHash()).toEqual(tx1.getTxHash());
    expect(pool.getTxStatus(tx1.getTxHash())).toEqual('pending');
    expect(pool.getPendingTxHashes()).toEqual([tx1.getTxHash()]);
  });

  it('Removes txs from the pool', async () => {
    const tx1 = mockTx();

    await pool.addTxs([tx1]);
    await pool.deleteTxs([tx1.getTxHash()]);

    expect(pool.getTxByHash(tx1.getTxHash())).toBeFalsy();
    expect(pool.getTxStatus(tx1.getTxHash())).toBeUndefined();
  });

  it('Marks txs as mined', async () => {
    const tx1 = mockTx(1);
    const tx2 = mockTx(2);

    await pool.addTxs([tx1, tx2]);
    await pool.markAsMined([tx1.getTxHash()]);

    expect(pool.getTxByHash(tx1.getTxHash())).toEqual(tx1);
    expect(pool.getTxStatus(tx1.getTxHash())).toEqual('mined');
    expect(pool.getMinedTxHashes()).toEqual([tx1.getTxHash()]);
    expect(pool.getPendingTxHashes()).toEqual([tx2.getTxHash()]);
  });

  it('Returns all transactions in the pool', async () => {
    const tx1 = mockTx(1);
    const tx2 = mockTx(2);
    const tx3 = mockTx(3);

    await pool.addTxs([tx1, tx2, tx3]);

    const poolTxs = pool.getAllTxs();
    expect(poolTxs).toHaveLength(3);
    expect(poolTxs).toEqual(expect.arrayContaining([tx1, tx2, tx3]));
  });

  it('Returns all txHashes in the pool', async () => {
    const tx1 = mockTx(1);
    const tx2 = mockTx(2);
    const tx3 = mockTx(3);

    await pool.addTxs([tx1, tx2, tx3]);

    const poolTxHashes = pool.getAllTxHashes();
    expect(poolTxHashes).toHaveLength(3);
    expect(poolTxHashes).toEqual(expect.arrayContaining([tx1.getTxHash(), tx2.getTxHash(), tx3.getTxHash()]));
  });
}
