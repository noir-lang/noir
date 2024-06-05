import { type AztecNode, L2Block } from '@aztec/circuit-types';
import { Fr, type Header, INITIAL_L2_BLOCK_NUM } from '@aztec/circuits.js';
import { makeHeader } from '@aztec/circuits.js/testing';
import { randomInt } from '@aztec/foundation/crypto';
import { SerialQueue } from '@aztec/foundation/fifo';
import { KeyStore } from '@aztec/key-store';
import { openTmpStore } from '@aztec/kv-store/utils';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type PxeDatabase } from '../database/index.js';
import { KVPxeDatabase } from '../database/kv_pxe_database.js';
import { Synchronizer } from './synchronizer.js';

describe('Synchronizer', () => {
  let aztecNode: MockProxy<AztecNode>;
  let database: PxeDatabase;
  let synchronizer: TestSynchronizer;
  let jobQueue: SerialQueue;
  const initialSyncBlockNumber = 3;
  let headerBlock3: Header;

  beforeEach(() => {
    headerBlock3 = makeHeader(randomInt(1000), initialSyncBlockNumber);

    aztecNode = mock<AztecNode>();
    database = new KVPxeDatabase(openTmpStore());
    jobQueue = new SerialQueue();
    synchronizer = new TestSynchronizer(aztecNode, database, jobQueue);
  });

  it('sets header from aztec node on initial sync', async () => {
    aztecNode.getBlockNumber.mockResolvedValue(initialSyncBlockNumber);
    aztecNode.getHeader.mockResolvedValue(headerBlock3);

    await synchronizer.initialSync();

    expect(database.getHeader()).toEqual(headerBlock3);
  });

  it('sets header from latest block', async () => {
    const block = L2Block.random(1, 4);
    aztecNode.getLogs.mockResolvedValueOnce([block.body.encryptedLogs]).mockResolvedValue([block.body.unencryptedLogs]);
    aztecNode.getBlocks.mockResolvedValue([block]);

    await synchronizer.work();

    const obtainedHeader = database.getHeader();
    expect(obtainedHeader).toEqual(block.header);
  });

  it('overrides header from initial sync once current block number is larger', async () => {
    // Initial sync is done on block with height 3
    aztecNode.getBlockNumber.mockResolvedValue(initialSyncBlockNumber);
    aztecNode.getHeader.mockResolvedValue(headerBlock3);

    await synchronizer.initialSync();
    const header0 = database.getHeader();
    expect(header0).toEqual(headerBlock3);

    // We then process block with height 1, this should not change the header
    const block1 = L2Block.random(1, 4);

    aztecNode.getLogs
      .mockResolvedValueOnce([block1.body.encryptedLogs])
      .mockResolvedValue([block1.body.unencryptedLogs]);

    aztecNode.getBlocks.mockResolvedValue([block1]);

    await synchronizer.work();
    const header1 = database.getHeader();
    expect(header1).toEqual(headerBlock3);
    expect(header1).not.toEqual(block1.header);

    // But they should change when we process block with height 5
    const block5 = L2Block.random(5, 4);

    aztecNode.getBlocks.mockResolvedValue([block5]);

    await synchronizer.work();
    const header5 = database.getHeader();
    expect(header5).not.toEqual(headerBlock3);
    expect(header5).toEqual(block5.header);
  });

  it('note processor successfully catches up', async () => {
    const blocks = [L2Block.random(1, 4), L2Block.random(2, 4)];

    aztecNode.getLogs
      // called by synchronizer.work
      .mockResolvedValueOnce([blocks[0].body.encryptedLogs])
      .mockResolvedValueOnce([blocks[0].body.unencryptedLogs])
      .mockResolvedValueOnce([blocks[1].body.encryptedLogs])
      .mockResolvedValueOnce([blocks[1].body.encryptedLogs])
      // called by synchronizer.workNoteProcessorCatchUp
      .mockResolvedValueOnce([blocks[0].body.encryptedLogs])
      .mockResolvedValueOnce([blocks[1].body.encryptedLogs]);

    aztecNode.getBlocks
      // called by synchronizer.work, we are testing fromFields in this first call
      .mockResolvedValueOnce([
        L2Block.fromFields({
          archive: blocks[0].archive,
          header: blocks[0].header,
          body: blocks[0].body,
        }),
      ])
      .mockResolvedValueOnce([
        L2Block.fromFields({
          archive: blocks[1].archive,
          header: blocks[1].header,
          body: blocks[1].body,
        }),
      ])
      // called by synchronizer.workNoteProcessorCatchUp
      .mockResolvedValueOnce([blocks[0]])
      .mockResolvedValueOnce([blocks[1]]);

    aztecNode.getBlockNumber.mockResolvedValue(INITIAL_L2_BLOCK_NUM + 1);

    // Sync the synchronizer so that note processor has something to catch up to
    // There are two blocks, and we have a limit of 1 block per work call
    await synchronizer.work(1);
    expect(await synchronizer.isGlobalStateSynchronized()).toBe(false);
    await synchronizer.work(1);
    expect(await synchronizer.isGlobalStateSynchronized()).toBe(true);

    // Manually adding account to database so that we can call synchronizer.isAccountStateSynchronized
    const keyStore = new KeyStore(openTmpStore());
    const addAddress = async (startingBlockNum: number) => {
      const secretKey = Fr.random();
      const partialAddress = Fr.random();
      const completeAddress = await keyStore.addAccount(secretKey, partialAddress);
      await database.addCompleteAddress(completeAddress);
      await synchronizer.addAccount(completeAddress.address, keyStore, startingBlockNum);
      return completeAddress;
    };

    const [completeAddressA, completeAddressB, completeAddressC] = await Promise.all([
      addAddress(INITIAL_L2_BLOCK_NUM),
      addAddress(INITIAL_L2_BLOCK_NUM),
      addAddress(INITIAL_L2_BLOCK_NUM + 1),
    ]);

    await synchronizer.workNoteProcessorCatchUp();

    expect(await synchronizer.isAccountStateSynchronized(completeAddressA.address)).toBe(false);
    expect(await synchronizer.isAccountStateSynchronized(completeAddressB.address)).toBe(false);
    expect(await synchronizer.isAccountStateSynchronized(completeAddressC.address)).toBe(false);

    await synchronizer.workNoteProcessorCatchUp();

    expect(await synchronizer.isAccountStateSynchronized(completeAddressA.address)).toBe(true);
    expect(await synchronizer.isAccountStateSynchronized(completeAddressB.address)).toBe(true);
    expect(await synchronizer.isAccountStateSynchronized(completeAddressC.address)).toBe(true);
  });
});

class TestSynchronizer extends Synchronizer {
  public override work(limit = 1) {
    return super.work(limit);
  }

  public override initialSync(): Promise<void> {
    return super.initialSync();
  }

  public override workNoteProcessorCatchUp(limit = 1): Promise<boolean> {
    return super.workNoteProcessorCatchUp(limit);
  }
}
