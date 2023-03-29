import { expect, jest } from '@jest/globals';
import { L2BlockSource } from '@aztec/l2-block';

import { P2PClient } from './p2p_client.js';
import { TxPool } from '../tx_pool/index.js';
import { MockBlockSource } from './mocks.js';
import { MockTx } from './mocks.js';

/**
 * Mockify helper for testing purposes.
 */
type Mockify<T> = {
  [P in keyof T]: ReturnType<typeof jest.fn>;
};

describe('In-Memory P2P Client', () => {
  let txPool: Mockify<TxPool>;
  let blockSource: L2BlockSource;

  beforeEach(() => {
    txPool = {
      addTxs: jest.fn(),
      getTxByHash: jest.fn().mockReturnValue(undefined),
      deleteTxs: jest.fn(),
      getAllTxs: jest.fn().mockReturnValue([]),
    };

    blockSource = new MockBlockSource();
  });

  it('can start & stop', async () => {
    const client = new P2PClient(blockSource, txPool);
    expect(await client.isReady()).toEqual(false);

    await client.start();
    expect(await client.isReady()).toEqual(true);

    await client.stop();
    expect(await client.isReady()).toEqual(false);
  });

  it('adds txs to pool', async () => {
    const client = new P2PClient(blockSource, txPool);
    await client.start();
    const tx1 = MockTx();
    const tx2 = MockTx();
    await client.sendTx(tx1);
    await client.sendTx(tx2);

    expect(txPool.addTxs).toHaveBeenCalledTimes(2);
    await client.stop();
  });

  it('rejects txs after being stopped', async () => {
    const client = new P2PClient(blockSource, txPool);
    await client.start();
    const tx1 = MockTx();
    const tx2 = MockTx();
    await client.sendTx(tx1);
    await client.sendTx(tx2);

    expect(txPool.addTxs).toHaveBeenCalledTimes(2);
    await client.stop();
    const tx3 = MockTx();
    await expect(client.sendTx(tx3)).rejects.toThrow();
    expect(txPool.addTxs).toHaveBeenCalledTimes(2);
  });
});
