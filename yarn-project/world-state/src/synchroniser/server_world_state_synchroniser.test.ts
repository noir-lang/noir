import {
  AppendOnlyTreeSnapshot,
  CircuitsWasm,
  Fr,
  GlobalVariables,
  MAX_NEW_COMMITMENTS_PER_TX,
  MAX_NEW_CONTRACTS_PER_TX,
  MAX_NEW_L2_TO_L1_MSGS_PER_CALL,
  MAX_NEW_NULLIFIERS_PER_TX,
  MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
} from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { sleep } from '@aztec/foundation/sleep';
import { INITIAL_LEAF, Pedersen } from '@aztec/merkle-tree';
import {
  ContractData,
  L2Block,
  L2BlockL2Logs,
  L2BlockSource,
  MerkleTreeId,
  PublicDataWrite,
  SiblingPath,
} from '@aztec/types';

import { jest } from '@jest/globals';
import times from 'lodash.times';

import { MerkleTreeDb, MerkleTrees, WorldStateConfig } from '../index.js';
import { ServerWorldStateSynchroniser } from './server_world_state_synchroniser.js';
import { WorldStateRunningState } from './world_state_synchroniser.js';

/**
 * Generic mock implementation.
 */
type Mockify<T> = {
  [P in keyof T]: jest.Mock;
};

const LATEST_BLOCK_NUMBER = 5;
const getLatestBlockNumber = () => LATEST_BLOCK_NUMBER;
let nextBlocks: L2Block[] = [];
const consumeNextBlocks = () => {
  const blocks = nextBlocks;
  nextBlocks = [];
  return Promise.resolve(blocks);
};

const getMockTreeSnapshot = () => {
  return new AppendOnlyTreeSnapshot(Fr.random(), 16);
};

const getMockContractData = () => {
  return ContractData.random();
};

const getMockGlobalVariables = () => {
  return GlobalVariables.from({
    chainId: Fr.random(),
    version: Fr.random(),
    blockNumber: Fr.random(),
    timestamp: Fr.random(),
  });
};

const getMockL1ToL2MessagesData = () => {
  return new Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).map(() => Fr.random());
};

const getMockBlock = (blockNumber: number, newContractsCommitments?: Buffer[]) => {
  const newEncryptedLogs = L2BlockL2Logs.random(1, 2, 3);
  const block = L2Block.fromFields({
    number: blockNumber,
    globalVariables: getMockGlobalVariables(),
    startPrivateDataTreeSnapshot: getMockTreeSnapshot(),
    startNullifierTreeSnapshot: getMockTreeSnapshot(),
    startContractTreeSnapshot: getMockTreeSnapshot(),
    startPublicDataTreeRoot: Fr.random(),
    startL1ToL2MessageTreeSnapshot: getMockTreeSnapshot(),
    startHistoricBlocksTreeSnapshot: getMockTreeSnapshot(),
    endPrivateDataTreeSnapshot: getMockTreeSnapshot(),
    endNullifierTreeSnapshot: getMockTreeSnapshot(),
    endContractTreeSnapshot: getMockTreeSnapshot(),
    endPublicDataTreeRoot: Fr.random(),
    endL1ToL2MessageTreeSnapshot: getMockTreeSnapshot(),
    endHistoricBlocksTreeSnapshot: getMockTreeSnapshot(),
    newCommitments: times(MAX_NEW_COMMITMENTS_PER_TX, Fr.random),
    newNullifiers: times(MAX_NEW_NULLIFIERS_PER_TX, Fr.random),
    newContracts: newContractsCommitments?.map(x => Fr.fromBuffer(x)) ?? [Fr.random()],
    newContractData: times(MAX_NEW_CONTRACTS_PER_TX, getMockContractData),
    newPublicDataWrites: times(MAX_PUBLIC_DATA_UPDATE_REQUESTS_PER_TX, PublicDataWrite.random),
    newL1ToL2Messages: getMockL1ToL2MessagesData(),
    newL2ToL1Msgs: times(MAX_NEW_L2_TO_L1_MSGS_PER_CALL, Fr.random),
    newEncryptedLogs,
  });
  return block;
};

const createSynchroniser = (merkleTreeDb: any, rollupSource: any, blockCheckInterval = 100) => {
  const worldStateConfig: WorldStateConfig = {
    worldStateBlockCheckIntervalMS: blockCheckInterval,
    l2QueueSize: 1000,
  };
  return new ServerWorldStateSynchroniser(merkleTreeDb as MerkleTrees, rollupSource as L2BlockSource, worldStateConfig);
};

const log = createDebugLogger('aztec:server_world_state_synchroniser_test');

describe('server_world_state_synchroniser', () => {
  const rollupSource: Mockify<Pick<L2BlockSource, 'getBlockHeight' | 'getL2Blocks'>> = {
    getBlockHeight: jest.fn().mockImplementation(getLatestBlockNumber),
    getL2Blocks: jest.fn().mockImplementation(consumeNextBlocks),
  };

  const merkleTreeDb: Mockify<MerkleTreeDb> = {
    getTreeInfo: jest
      .fn()
      .mockImplementation(() =>
        Promise.resolve({ treeId: MerkleTreeId.CONTRACT_TREE, root: Buffer.alloc(32, 0), size: 0n }),
      ),
    appendLeaves: jest.fn().mockImplementation(() => Promise.resolve()),
    updateLeaf: jest.fn().mockImplementation(() => Promise.resolve()),
    getSiblingPath: jest.fn().mockImplementation(() => {
      return async () => {
        const wasm = await CircuitsWasm.get();
        const pedersen: Pedersen = new Pedersen(wasm);
        SiblingPath.ZERO(32, INITIAL_LEAF, pedersen);
      }; //Promise.resolve();
    }),
    updateHistoricBlocksTree: jest.fn().mockImplementation(() => Promise.resolve()),
    commit: jest.fn().mockImplementation(() => Promise.resolve()),
    rollback: jest.fn().mockImplementation(() => Promise.resolve()),
    handleL2Block: jest.fn().mockImplementation(() => Promise.resolve()),
    stop: jest.fn().mockImplementation(() => Promise.resolve()),
  } as any;

  const performInitialSync = async (server: ServerWorldStateSynchroniser) => {
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

  it('can be constructed', () => {
    expect(() => createSynchroniser(merkleTreeDb, rollupSource)).not.toThrow();
  });

  it('updates sync progress', async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource);

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
    const server = createSynchroniser(merkleTreeDb, rollupSource);
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
    const server = createSynchroniser(merkleTreeDb, rollupSource);
    rollupSource.getBlockHeight.mockImplementationOnce(() => {
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
    const server = createSynchroniser(merkleTreeDb, rollupSource);
    rollupSource.getBlockHeight.mockImplementationOnce(() => {
      return Promise.resolve(0);
    });

    // kick off the synching
    const syncPromise = server.start();
    await syncPromise;
    await server.stop();

    await expect(server.start()).rejects.toThrow();
  });

  it('adds the received L2 blocks', async () => {
    merkleTreeDb.handleL2Block.mockReset();
    const server = createSynchroniser(merkleTreeDb, rollupSource);
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
    const server = createSynchroniser(merkleTreeDb, rollupSource, 10000);

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

    // stop the synchroniser
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER + 3);
  });

  it('can immediately sync to a minimum block number', async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource, 10000);

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

    // stop the synchroniser
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER + 20);
  });

  it('can immediately sync to a minimum block in the past', async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource, 10000);

    await performInitialSync(server);
    // syncing to a block in the past should succeed
    await server.syncImmediate(LATEST_BLOCK_NUMBER - 1);
    // syncing to the current block should succeed
    await server.syncImmediate(LATEST_BLOCK_NUMBER);

    // we should have synced all of the blocks
    let status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER);

    // stop the synchroniser
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER);
  });

  it('throws if you try to sync to an unavailable block', async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource, 10000);

    await performInitialSync(server);

    // the server should now be asleep for a long time
    // we will add 2 blocks and force a sync to at least LATEST + 5
    nextBlocks = Array(2)
      .fill(0)
      .map((_, index: number) => getMockBlock(index + 1 + LATEST_BLOCK_NUMBER));
    await expect(server.syncImmediate(LATEST_BLOCK_NUMBER + 5)).rejects.toThrow(
      `Unable to sync to block height ${LATEST_BLOCK_NUMBER + 5}, currently synced to block ${LATEST_BLOCK_NUMBER + 2}`,
    );

    let status = await server.status();
    expect(status.syncedToL2Block).toBe(LATEST_BLOCK_NUMBER + 2);

    // stop the synchroniser
    await server.stop();

    // check the final status
    status = await server.status();
    expect(status.state).toEqual(WorldStateRunningState.STOPPED);
    expect(status.syncedToL2Block).toEqual(LATEST_BLOCK_NUMBER + 2);
  });

  it('throws if you try to immediate sync when not running', async () => {
    const server = createSynchroniser(merkleTreeDb, rollupSource, 10000);

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
});
