import { mockTx } from '@aztec/circuit-types';
import { retryUntil } from '@aztec/foundation/retry';
import { type AztecKVStore } from '@aztec/kv-store';
import { openTmpStore } from '@aztec/kv-store/utils';

import { expect, jest } from '@jest/globals';

import { type P2PService } from '../index.js';
import { type TxPool } from '../tx_pool/index.js';
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
  let blockSource: MockBlockSource;
  let p2pService: Mockify<P2PService>;
  let kvStore: AztecKVStore;
  let client: P2PClient;

  beforeEach(() => {
    txPool = {
      addTxs: jest.fn(),
      getTxByHash: jest.fn().mockReturnValue(undefined),
      deleteTxs: jest.fn(),
      getAllTxs: jest.fn().mockReturnValue([]),
      getAllTxHashes: jest.fn().mockReturnValue([]),
      getMinedTxHashes: jest.fn().mockReturnValue([]),
      getPendingTxHashes: jest.fn().mockReturnValue([]),
      getTxStatus: jest.fn().mockReturnValue(undefined),
      markAsMined: jest.fn(),
    };

    p2pService = {
      start: jest.fn(),
      stop: jest.fn(),
      propagateTx: jest.fn(),
    };

    blockSource = new MockBlockSource();

    kvStore = openTmpStore();
    client = new P2PClient(kvStore, blockSource, txPool, p2pService, 0);
  });

  const advanceToProvenBlock = async (provenBlockNum: number) => {
    blockSource.setProvenBlockNumber(provenBlockNum);
    await retryUntil(() => Promise.resolve(client.getSyncedProvenBlockNum() >= provenBlockNum), 'synced', 10, 0.1);
  };

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

    const client2 = new P2PClient(kvStore, blockSource, txPool, p2pService, 0);
    expect(client2.getSyncedLatestBlockNum()).toEqual(client.getSyncedLatestBlockNum());
  });

  it('deletes txs once block is proven', async () => {
    blockSource.setProvenBlockNumber(0);
    await client.start();
    expect(txPool.deleteTxs).not.toHaveBeenCalled();

    await advanceToProvenBlock(5);
    expect(txPool.deleteTxs).toHaveBeenCalledTimes(5);
    await client.stop();
  });

  it('deletes txs after waiting the set number of blocks', async () => {
    client = new P2PClient(kvStore, blockSource, txPool, p2pService, 10);
    blockSource.setProvenBlockNumber(0);
    await client.start();
    expect(txPool.deleteTxs).not.toHaveBeenCalled();

    await advanceToProvenBlock(5);
    expect(txPool.deleteTxs).not.toHaveBeenCalled();

    await advanceToProvenBlock(12);
    expect(txPool.deleteTxs).toHaveBeenCalledTimes(2);

    await advanceToProvenBlock(20);
    expect(txPool.deleteTxs).toHaveBeenCalledTimes(10);
    await client.stop();
  });
});
