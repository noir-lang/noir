import { L2Block, L2BlockSource, MerkleTreeId } from '@aztec/circuit-types';
import { Fr } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { sleep } from '@aztec/foundation/sleep';
import { AztecKVStore, AztecLmdbStore } from '@aztec/kv-store';
import { INITIAL_LEAF, Pedersen } from '@aztec/merkle-tree';
import { SiblingPath } from '@aztec/types/membership';

import { jest } from '@jest/globals';
import { mock } from 'jest-mock-extended';

import { MerkleTreeDb, MerkleTrees, WorldStateConfig } from '../index.js';
import { ServerWorldStateSynchronizer } from './server_world_state_synchronizer.js';
import { WorldStateRunningState } from './world_state_synchronizer.js';

const LATEST_BLOCK_NUMBER = 5;
const getLatestBlockNumber = () => Promise.resolve(LATEST_BLOCK_NUMBER);
let nextBlocks: L2Block[] = [];
const consumeNextBlocks = () => {
  const blocks = nextBlocks;
  nextBlocks = [];
  return Promise.resolve(blocks);
};

const getMockBlock = (blockNumber: number, newContractsCommitments?: Buffer[]) => {
  const block = L2Block.random(blockNumber);

  if (newContractsCommitments) {
    block.newContracts = newContractsCommitments.map(x => Fr.fromBuffer(x));
  }

  return block;
};

const log = createDebugLogger('aztec:server_world_state_synchronizer_test');

describe('server_world_state_synchronizer', () => {
  let db: AztecKVStore;
  const rollupSource = mock<L2BlockSource>({
    getBlockNumber: jest.fn(getLatestBlockNumber),
    getBlocks: jest.fn(consumeNextBlocks),
  });

  const merkleTreeDb = mock<MerkleTreeDb>({
    getTreeInfo: jest.fn(() =>
      Promise.resolve({ depth: 8, treeId: MerkleTreeId.CONTRACT_TREE, root: Buffer.alloc(32, 0), size: 0n }),
    ),
    getSiblingPath: jest.fn(() => {
      const pedersen: Pedersen = new Pedersen();
      return Promise.resolve(SiblingPath.ZERO(32, INITIAL_LEAF, pedersen) as SiblingPath<number>);
    }),
    handleL2Block: jest.fn(() => Promise.resolve({ isBlockOurs: false })),
  });

  const performInitialSync = async (server: ServerWorldStateSynchronizer) => {
    // test initial state
    let status = await server.status();
    expect(status.syncedToL2Block).toEqual(0);
    expect(status.state).toEqual(WorldStateRunningState.IDLE);

    // create the initial blocks
    nextBlocks = Array(LATEST_BLOCK_NUMBER)
      .fill(0)
      .map((_, index: number) => getMockBlock(index + 1));

    // start the sync process and await it
    await server.start().catch(err => log.error('Sync not completed: ', err));

    status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER);
  };

  const performSubsequentSync = async (server: ServerWorldStateSynchronizer, count: number) => {
    // test initial state
    let status = await server.status();
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);
    expect(status.state).toEqual(WorldStateRunningState.IDLE);

    // create the initial blocks
    nextBlocks = Array(count)
      .fill(0)
      .map((_, index: number) => getMockBlock(LATEST_BLOCK_NUMBER + index + 1));

    rollupSource.getBlockNumber.mockResolvedValueOnce(LATEST_BLOCK_NUMBER + count);

    // start the sync process and await it
    await server.start().catch(err => log.error('Sync not completed: ', err));

    status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER + count);
  };

  const createSynchronizer = (blockCheckInterval = 100) => {
    const worldStateConfig: WorldStateConfig = {
      worldStateBlockCheckIntervalMS: blockCheckInterval,
      l2QueueSize: 1000,
    };

    return new ServerWorldStateSynchronizer(
      db,
      merkleTreeDb as any as MerkleTrees,
      rollupSource as L2BlockSource,
      worldStateConfig,
    );
  };

  beforeEach(async () => {
    db = await AztecLmdbStore.openTmp();
  });

  it('can be constructed', () => {
    expect(createSynchronizer()).toBeTruthy();
  });

  it('updates sync progress', async () => {
    const server = createSynchronizer();

    // test initial state
    let status = await server.status();
    expect(status.syncedToL2Block).toEqual(0);
    expect(status.state).toEqual(WorldStateRunningState.IDLE);

    // create an initial block
    let currentBlockNumber = 0;
    nextBlocks = [getMockBlock(currentBlockNumber + 1)];

    // start the sync process but don't await
    server.start().catch(err => log.error('Sync not completed: ', err));

    // now setup a loop to monitor the sync progress and push new blocks in
    while (currentBlockNumber <= LATEST_BLOCK_NUMBER) {
      status = await server.status();
      expect(
        status.syncedToL2Block >= currentBlockNumber || status.syncedToL2Block <= currentBlockNumber + 1,
      ).toBeTruthy();
      if (status.syncedToL2Block === LATEST_BLOCK_NUMBER) {
        break;
      }
      expect(
        status.state >= WorldStateRunningState.IDLE || status.state <= WorldStateRunningState.SYNCHING,
      ).toBeTruthy();
      if (status.syncedToL2Block === currentBlockNumber) {
        await sleep(100);
        continue;
      }
      currentBlockNumber++;
      nextBlocks = [getMockBlock(currentBlockNumber + 1)];
    }

    // check the status again, should be fully synced
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.RUNNING);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);

    // stop the synchronizer
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);
  });

  it('enables blocking until synced', async () => {
    const server = createSynchronizer();
    let currentBlockNumber = 0;

    const newBlocks = async () => {
      while (currentBlockNumber <= LATEST_BLOCK_NUMBER) {
        await sleep(100);
        nextBlocks = [...nextBlocks, getMockBlock(++currentBlockNumber)];
      }
    };

    // kick off the background queueing of blocks
    const newBlockPromise = newBlocks();

    // kick off the synching
    const syncPromise = server.start();

    // await the synching
    await syncPromise;

    await newBlockPromise;

    let status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.RUNNING);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);
    await server.stop();
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);
  });

  it('handles multiple calls to start', async () => {
    const server = createSynchronizer();
    let currentBlockNumber = 0;

    const newBlocks = async () => {
      while (currentBlockNumber < LATEST_BLOCK_NUMBER) {
        await sleep(100);
        const newBlock = getMockBlock(++currentBlockNumber);
        nextBlocks = [...nextBlocks, newBlock];
      }
    };

    // kick off the background queueing of blocks
    const newBlockPromise = newBlocks();

    // kick off the synching
    await server.start();

    // call start again, should get back the same promise
    await server.start();

    // wait until the block production has finished
    await newBlockPromise;

    await server.stop();
  });

  it('immediately syncs if no new blocks', async () => {
    const server = createSynchronizer();
    rollupSource.getBlockNumber.mockImplementationOnce(() => {
      return Promise.resolve(0);
    });

    // kick off the synching
    const syncPromise = server.start();

    // it should already be synced, no need to push new blocks
    await syncPromise;

    const status = await server.status();
    expect(status.state).toBe(WorldStateRunningState.RUNNING);
    expect(status.syncedToL2Block).toBe(0);
    await server.stop();
  });

  it("can't be started if already stopped", async () => {
    const server = createSynchronizer();
    rollupSource.getBlockNumber.mockImplementationOnce(() => {
      return Promise.resolve(0);
    });

    // kick off the synching
    const syncPromise = server.start();
    await syncPromise;
    await server.stop();

    await expect(server.start()).rejects.toThrow();
  });

  it('adds the received L2 blocks', async () => {
    merkleTreeDb.handleL2Block.mockClear();
    const server = createSynchronizer();
    const totalBlocks = LATEST_BLOCK_NUMBER + 1;
    nextBlocks = Array(totalBlocks)
      .fill(0)
      .map((_, index) => getMockBlock(index, [Buffer.alloc(32, index)]));
    // sync the server
    await server.start();

    expect(merkleTreeDb.handleL2Block).toHaveBeenCalledTimes(totalBlocks);
    await server.stop();
  });

  it('can immediately sync to latest', async () => {
    const server = createSynchronizer(10000);

    await performInitialSync(server);

    // the server should now be asleep for a long time
    // we will add a new block and force an immediate sync
    nextBlocks = [getMockBlock(LATEST_BLOCK_NUMBER + 1)];
    await server.syncImmediate();

    let status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER + 1);

    nextBlocks = [getMockBlock(LATEST_BLOCK_NUMBER + 2), getMockBlock(LATEST_BLOCK_NUMBER + 3)];
    await server.syncImmediate();

    status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER + 3);

    // stop the synchronizer
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER + 3);
  });

  it('can immediately sync to a minimum block number', async () => {
    const server = createSynchronizer(10000);

    await performInitialSync(server);

    // the server should now be asleep for a long time
    // we will add 20 blocks and force a sync to at least LATEST + 5
    nextBlocks = Array(20)
      .fill(0)
      .map((_, index: number) => getMockBlock(index + 1 + LATEST_BLOCK_NUMBER));
    await server.syncImmediate(LATEST_BLOCK_NUMBER + 5);

    // we should have synced all of the blocks
    let status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER + 20);

    // stop the synchronizer
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER + 20);
  });

  it('can immediately sync to a minimum block in the past', async () => {
    const server = createSynchronizer(10000);

    await performInitialSync(server);
    // syncing to a block in the past should succeed
    await server.syncImmediate(LATEST_BLOCK_NUMBER - 1);
    // syncing to the current block should succeed
    await server.syncImmediate(LATEST_BLOCK_NUMBER);

    // we should have synced all of the blocks
    let status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER);

    // stop the synchronizer
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);
  });

  it('throws if you try to sync to an unavailable block', async () => {
    const server = createSynchronizer();

    await performInitialSync(server);

    // the server should now be asleep for a long time
    // we will add 2 blocks and force a sync to at least LATEST + 5
    nextBlocks = Array(2)
      .fill(0)
      .map((_, index: number) => getMockBlock(index + 1 + LATEST_BLOCK_NUMBER));
    await expect(server.syncImmediate(LATEST_BLOCK_NUMBER + 5)).rejects.toThrow(
      `Unable to sync to block number ${LATEST_BLOCK_NUMBER + 5}, currently synced to block ${LATEST_BLOCK_NUMBER + 2}`,
    );

    let status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER + 2);

    // stop the synchronizer
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER + 2);
  });

  it('throws if you try to immediate sync when not running', async () => {
    const server = createSynchronizer(10000);

    // test initial state
    const status = await server.status();
    expect(status.syncedToL2Block).toEqual(0);
    expect(status.state).toEqual(WorldStateRunningState.IDLE);

    // create an initial block
    nextBlocks = Array(LATEST_BLOCK_NUMBER)
      .fill(0)
      .map((_, index: number) => getMockBlock(index + 1));

    await expect(server.syncImmediate()).rejects.toThrow(`World State is not running, unable to perform sync`);
  });

  it('restores the last synced block', async () => {
    const initialServer = createSynchronizer(10000);

    await performInitialSync(initialServer);
    await initialServer.stop();

    const server = createSynchronizer(10000);
    const status = await server.status();
    expect(status).toEqual({
      state: WorldStateRunningState.IDLE,
      syncedToL2Block: LATEST_BLOCK_NUMBER,
    });
  });

  it('starts syncing from the last block', async () => {
    const initialServer = createSynchronizer(10000);

    await performInitialSync(initialServer);
    await initialServer.stop();

    const server = createSynchronizer(10000);
    await performSubsequentSync(server, 2);
    await server.stop();
  });
});
