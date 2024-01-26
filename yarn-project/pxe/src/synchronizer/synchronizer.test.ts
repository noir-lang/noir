import { AztecNode, INITIAL_L2_BLOCK_NUM, L2Block, MerkleTreeId } from '@aztec/circuit-types';
import { BlockHeader, CompleteAddress, Fr, GrumpkinScalar } from '@aztec/circuits.js';
import { Grumpkin } from '@aztec/circuits.js/barretenberg';
import { SerialQueue } from '@aztec/foundation/fifo';
import { TestKeyStore } from '@aztec/key-store';
import { AztecLmdbStore } from '@aztec/kv-store';

import { MockProxy, mock } from 'jest-mock-extended';
import omit from 'lodash.omit';

import { PxeDatabase } from '../database/index.js';
import { KVPxeDatabase } from '../database/kv_pxe_database.js';
import { Synchronizer } from './synchronizer.js';

describe('Synchronizer', () => {
  let aztecNode: MockProxy<AztecNode>;
  let database: PxeDatabase;
  let synchronizer: TestSynchronizer;
  let roots: Record<MerkleTreeId, Fr>;
  let blockHeader: BlockHeader;
  let jobQueue: SerialQueue;

  beforeEach(async () => {
    blockHeader = BlockHeader.random();
    roots = {
      [MerkleTreeId.CONTRACT_TREE]: blockHeader.contractTreeRoot,
      [MerkleTreeId.NOTE_HASH_TREE]: blockHeader.noteHashTreeRoot,
      [MerkleTreeId.NULLIFIER_TREE]: blockHeader.nullifierTreeRoot,
      [MerkleTreeId.PUBLIC_DATA_TREE]: blockHeader.publicDataTreeRoot,
      [MerkleTreeId.L1_TO_L2_MESSAGE_TREE]: blockHeader.l1ToL2MessageTreeRoot,
      [MerkleTreeId.ARCHIVE]: blockHeader.archiveRoot,
    };

    aztecNode = mock<AztecNode>();
    database = new KVPxeDatabase(await AztecLmdbStore.openTmp());
    jobQueue = new SerialQueue();
    synchronizer = new TestSynchronizer(aztecNode, database, jobQueue);
  });

  it('sets tree roots from aztec node on initial sync', async () => {
    aztecNode.getBlockNumber.mockResolvedValue(3);
    aztecNode.getBlockHeader.mockResolvedValue(blockHeader);

    await synchronizer.initialSync();

    expect(database.getTreeRoots()).toEqual(roots);
  });

  it('sets tree roots from latest block', async () => {
    const block = L2Block.random(1, 4);
    aztecNode.getBlocks.mockResolvedValue([L2Block.fromFields(omit(block, 'newEncryptedLogs', 'newUnencryptedLogs'))]);
    aztecNode.getLogs.mockResolvedValueOnce([block.newEncryptedLogs!]).mockResolvedValue([block.newUnencryptedLogs!]);

    await synchronizer.work();

    const roots = database.getTreeRoots();
    expect(roots[MerkleTreeId.CONTRACT_TREE]).toEqual(block.header.state.partial.contractTree.root);
  });

  it('overrides tree roots from initial sync once current block number is larger', async () => {
    // Initial sync is done on block with height 3
    aztecNode.getBlockNumber.mockResolvedValue(3);
    aztecNode.getBlockHeader.mockResolvedValue(blockHeader);

    await synchronizer.initialSync();
    const roots0 = database.getTreeRoots();
    expect(roots0[MerkleTreeId.CONTRACT_TREE]).toEqual(roots[MerkleTreeId.CONTRACT_TREE]);

    // We then process block with height 1, this should not change tree roots
    const block1 = L2Block.random(1, 4);
    aztecNode.getBlocks.mockResolvedValueOnce([
      L2Block.fromFields(omit(block1, 'newEncryptedLogs', 'newUnencryptedLogs')),
    ]);
    aztecNode.getLogs.mockResolvedValue([block1.newEncryptedLogs!]).mockResolvedValue([block1.newUnencryptedLogs!]);

    await synchronizer.work();
    const roots1 = database.getTreeRoots();
    expect(roots1[MerkleTreeId.CONTRACT_TREE]).toEqual(roots[MerkleTreeId.CONTRACT_TREE]);
    expect(roots1[MerkleTreeId.CONTRACT_TREE]).not.toEqual(block1.header.state.partial.contractTree.root);

    // But they should change when we process block with height 5
    const block5 = L2Block.random(5, 4);
    aztecNode.getBlocks.mockResolvedValueOnce([
      L2Block.fromFields(omit(block5, 'newEncryptedLogs', 'newUnencryptedLogs')),
    ]);

    await synchronizer.work();
    const roots5 = database.getTreeRoots();
    expect(roots5[MerkleTreeId.CONTRACT_TREE]).not.toEqual(roots[MerkleTreeId.CONTRACT_TREE]);
    expect(roots5[MerkleTreeId.CONTRACT_TREE]).toEqual(block5.header.state.partial.contractTree.root);
  });

  it('note processor successfully catches up', async () => {
    const blocks = [L2Block.random(1, 4), L2Block.random(2, 4)];

    aztecNode.getBlocks
      // called by synchronizer.work
      .mockResolvedValueOnce([L2Block.fromFields(omit(blocks[0], 'newEncryptedLogs', 'newUnencryptedLogs'))])
      .mockResolvedValueOnce([L2Block.fromFields(omit(blocks[1], 'newEncryptedLogs', 'newUnencryptedLogs'))])
      // called by synchronizer.workNoteProcessorCatchUp
      .mockResolvedValueOnce([L2Block.fromFields(omit(blocks[0], 'newEncryptedLogs', 'newUnencryptedLogs'))])
      .mockResolvedValueOnce([L2Block.fromFields(omit(blocks[1], 'newEncryptedLogs', 'newUnencryptedLogs'))]);

    aztecNode.getLogs
      // called by synchronizer.work
      .mockResolvedValueOnce([blocks[0].newEncryptedLogs!])
      .mockResolvedValueOnce([blocks[0].newUnencryptedLogs!])
      .mockResolvedValueOnce([blocks[1].newEncryptedLogs!])
      .mockResolvedValueOnce([blocks[1].newUnencryptedLogs!])
      // called by synchronizer.workNoteProcessorCatchUp
      .mockResolvedValueOnce([blocks[0].newEncryptedLogs!])
      .mockResolvedValueOnce([blocks[1].newEncryptedLogs!]);

    aztecNode.getBlockNumber.mockResolvedValue(INITIAL_L2_BLOCK_NUM + 1);

    // Sync the synchronizer so that note processor has something to catch up to
    // There are two blocks, and we have a limit of 1 block per work call
    await synchronizer.work(1);
    expect(await synchronizer.isGlobalStateSynchronized()).toBe(false);
    await synchronizer.work(1);
    expect(await synchronizer.isGlobalStateSynchronized()).toBe(true);

    // Manually adding account to database so that we can call synchronizer.isAccountStateSynchronized
    const keyStore = new TestKeyStore(new Grumpkin(), await AztecLmdbStore.openTmp());
    const addAddress = async (startingBlockNum: number) => {
      const privateKey = GrumpkinScalar.random();
      await keyStore.addAccount(privateKey);
      const completeAddress = CompleteAddress.fromPrivateKeyAndPartialAddress(privateKey, Fr.random());
      await database.addCompleteAddress(completeAddress);
      synchronizer.addAccount(completeAddress.publicKey, keyStore, startingBlockNum);
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
  public work(limit = 1) {
    return super.work(limit);
  }

  public initialSync(): Promise<void> {
    return super.initialSync();
  }

  public workNoteProcessorCatchUp(limit = 1): Promise<boolean> {
    return super.workNoteProcessorCatchUp(limit);
  }
}
