import { L2BlockSource, mockTx } from '@aztec/circuit-types';
import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';

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
  let kvStore: AztecKVStore;
  let client: P2PClient;

  beforeEach(async () => {
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

    kvStore = await AztecLmdbStore.openTmp();
    client = new P2PClient(kvStore, blockSource, txPool, p2pService);
  });

  it('can start & stop', async () => {
    expect(await client.isReady()).toEqual(false);

    await client.start();
    expect(await client.isReady()).toEqual(true);

    await client.stop();
    expect(await client.isReady()).toEqual(false);
  });

  it('adds txs to pool', async () => {
    await client.start();
    const tx1 = mockTx();
    const tx2 = mockTx();
    await client.sendTx(tx1);
    await client.sendTx(tx2);

    expect(txPool.addTxs).toHaveBeenCalledTimes(2);
    await client.stop();
  });

  it('rejects txs after being stopped', async () => {
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

  it('republishes previously stored txs on start', async () => {
    const tx1 = mockTx();
    const tx2 = mockTx();
    txPool.getAllTxs.mockReturnValue([tx1, tx2]);

    await client.start();
    expect(p2pService.propagateTx).toHaveBeenCalledTimes(2);
    expect(p2pService.propagateTx).toHaveBeenCalledWith(tx1);
    expect(p2pService.propagateTx).toHaveBeenCalledWith(tx2);
  });

  it('restores the previous block number it was at', async () => {
    await client.start();
    await client.stop();

    const client2 = new P2PClient(kvStore, blockSource, txPool, p2pService);
    expect(client2.getSyncedBlockNum()).toEqual(client.getSyncedBlockNum());
  });
});
