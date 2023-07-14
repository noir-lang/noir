import { L2BlockSource, mockTx } from '@aztec/types';

import { expect, jest } from '@jest/globals';

import { P2PService } from '../index.js';
import { TxPool } from '../tx_pool/index.js';
import { MockBlockSource } from './mocks.js';
import { P2PClient } from './p2p_client.js';

/**
 * Mockify helper for testing purposes.
 */
type Mockify<T> = {
  [P in keyof T]: ReturnType<typeof jest.fn>;
};

describe('In-Memory P2P Client', () => {
  let txPool: Mockify<TxPool>;
  let blockSource: L2BlockSource;
  let p2pService: Mockify<P2PService>;

  beforeEach(() => {
    txPool = {
      addTxs: jest.fn(),
      getTxByHash: jest.fn().mockReturnValue(undefined),
      deleteTxs: jest.fn(),
      getAllTxs: jest.fn().mockReturnValue([]),
      getAllTxHashes: jest.fn().mockReturnValue([]),
      hasTx: jest.fn().mockReturnValue(false),
    };

    p2pService = {
      start: jest.fn(),
      stop: jest.fn(),
      propagateTx: jest.fn(),
      settledTxs: jest.fn(),
    };

    blockSource = new MockBlockSource();
  });

  it('can start & stop', async () => {
    const client = new P2PClient(blockSource, txPool, p2pService);
    expect(await client.isReady()).toEqual(false);

    await client.start();
    expect(await client.isReady()).toEqual(true);

    await client.stop();
    expect(await client.isReady()).toEqual(false);
  });

  it('adds txs to pool', async () => {
    const client = new P2PClient(blockSource, txPool, p2pService);
    await client.start();
    const tx1 = mockTx();
    const tx2 = mockTx();
    await client.sendTx(tx1);
    await client.sendTx(tx2);

    expect(txPool.addTxs).toHaveBeenCalledTimes(2);
    await client.stop();
  });

  it('rejects txs after being stopped', async () => {
    const client = new P2PClient(blockSource, txPool, p2pService);
    await client.start();
    const tx1 = mockTx();
    const tx2 = mockTx();
    await client.sendTx(tx1);
    await client.sendTx(tx2);

    expect(txPool.addTxs).toHaveBeenCalledTimes(2);
    await client.stop();
    const tx3 = mockTx();
    await expect(client.sendTx(tx3)).rejects.toThrow();
    expect(txPool.addTxs).toHaveBeenCalledTimes(2);
  });
});
