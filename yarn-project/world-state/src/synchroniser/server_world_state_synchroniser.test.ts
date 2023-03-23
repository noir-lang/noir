import { ServerWorldStateSynchroniser } from './server_world_state_synchroniser.js';
import { L2BlockSource, L2BlockSourceSyncStatus, L2Block } from '@aztec/archiver';
import { WorldStateRunningState } from './world_state_synchroniser.js';
import { Pedersen, MerkleTreeDb, MerkleTreeId, SiblingPath, StandardMerkleTree } from '@aztec/merkle-tree';
import { sleep } from '@aztec/foundation';
import { jest } from '@jest/globals';

/**
 * Generic mock implementation.
 */
type Mockify<T> = {
  [P in keyof T]: jest.Mock;
};

const syncStatus = {
  syncedToBlock: 0,
  latestBlock: 0,
} as L2BlockSourceSyncStatus;

const LATEST_BLOCK_NUMBER = 5;
const getLatestBlockNumber = () => LATEST_BLOCK_NUMBER;
let nextBlocks: L2Block[] = [];
const consumeNextBlocks = () => {
  const blocks = nextBlocks;
  nextBlocks = [];
  return Promise.resolve(blocks);
};

const getMockBlock = (blockNumber: number, newContractsCommitments?: Buffer[]) => {
  const block = {
    number: blockNumber,
    newContracts: newContractsCommitments ?? [Buffer.alloc(32, 0)],
  } as L2Block;
  return block;
};

const createSynchroniser = (merkleTreeDb: any, rollupSource: any) =>
  new ServerWorldStateSynchroniser(merkleTreeDb as MerkleTreeDb, rollupSource as L2BlockSource, 1, 100);

describe('server_world_state_synchroniser', () => {
  const pedersen: Pedersen = new Pedersen();
  const rollupSource: Mockify<L2BlockSource> = {
    getSyncStatus: jest.fn().mockImplementation(() => Promise.resolve(syncStatus)),
    getLatestBlockNum: jest.fn().mockImplementation(getLatestBlockNumber),
    getL2Blocks: jest.fn().mockImplementation(consumeNextBlocks),
  } as any;

  const merkleTreeDb: Mockify<MerkleTreeDb> = {
    getTreeInfo: jest
      .fn()
      .mockImplementation(() =>
        Promise.resolve({ treeId: MerkleTreeId.CONTRACT_TREE, root: Buffer.alloc(32, 0), size: 0n }),
      ),
    appendLeaves: jest.fn().mockImplementation(() => Promise.resolve()),
    getSiblingPath: jest.fn().mockImplementation(() => {
      return Promise.resolve(SiblingPath.ZERO(32, StandardMerkleTree.ZERO_ELEMENT, pedersen));
    }),
    commit: jest.fn().mockImplementation(() => Promise.resolve()),
    rollback: jest.fn().mockImplementation(() => Promise.resolve()),
  } as any;

  it('can be constructed', () => {
    expect(() => createSynchroniser(merkleTreeDb, rollupSource)).not.toThrow();
  });

  it('updates sync progress', async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource);

    // test initial state
    let status = await server.status();
    expect(status.syncedToL2Block).toEqual(-1);
    expect(status.state).toEqual(WorldStateRunningState.IDLE);

    // create an initial block
    let currentBlockNumber = -1;
    nextBlocks = [getMockBlock(currentBlockNumber + 1)];

    // start the sync process but don't await
    server.start(0).catch(() => console.log('Sync not completed!!'));

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

    // check the status agian, should be fully synced
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.RUNNING);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);

    // stop the synchroniser
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);
  });

  it('enables blocking until synced', async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource);
    let currentBlockNumber = -1;

    const newBlocks = async () => {
      while (currentBlockNumber <= LATEST_BLOCK_NUMBER) {
        await sleep(100);
        nextBlocks = [...nextBlocks, getMockBlock(++currentBlockNumber)];
      }
    };

    // kick off the background queueing of blocks
    const newBlockPromise = newBlocks();

    // kick off the synching
    const syncPromise = server.start(0);

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
    const server = createSynchroniser(merkleTreeDb, rollupSource);
    let currentBlockNumber = -1;

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
    await server.start(0);

    // call start again, should get back the same promise
    await server.start(0);

    // wait until the block production has finished
    await newBlockPromise;

    await server.stop();
  });

  it('immediately syncs if no new blocks', async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource);

    // kick off the synching
    const syncPromise = server.start(5);

    // it should already be synced, no need to push new blocks
    await syncPromise;

    const status = await server.status();
    expect(status.state).toBe(WorldStateRunningState.RUNNING);
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER);
    await server.stop();
  });

  it("can't be started if already stopped", async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource);

    // kick off the synching
    const syncPromise = server.start(5);
    await syncPromise;
    await server.stop();

    await expect(server.start()).rejects.toThrow();
  });

  it('updates the contract tree', async () => {
    merkleTreeDb.appendLeaves.mockReset();
    const server = createSynchroniser(merkleTreeDb, rollupSource);
    const totalBlocks = LATEST_BLOCK_NUMBER + 1;
    nextBlocks = Array(totalBlocks)
      .fill(0)
      .map((_, index) => getMockBlock(index, [Buffer.alloc(32, index)]));
    // sync the server
    await server.start(0);
    expect(merkleTreeDb.appendLeaves).toHaveBeenCalledTimes(totalBlocks);
    for (let i = 0; i < totalBlocks; i++) {
      expect(merkleTreeDb.appendLeaves.mock.calls[i][0]).toEqual(MerkleTreeId.CONTRACT_TREE);
      expect(merkleTreeDb.appendLeaves.mock.calls[i][1]).toEqual([Buffer.alloc(32, i)]);
    }
    await server.stop();
  });
});
