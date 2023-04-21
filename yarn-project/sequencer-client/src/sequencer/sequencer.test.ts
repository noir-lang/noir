import { makeEmptyProof } from '@aztec/circuits.js';
import { P2P, P2PClientState } from '@aztec/p2p';
import { L2Block, PrivateTx, Tx, UnverifiedData } from '@aztec/types';
import { MerkleTreeId, MerkleTreeOperations, WorldStateRunningState, WorldStateSynchroniser } from '@aztec/world-state';
import { MockProxy, mock } from 'jest-mock-extended';
import times from 'lodash.times';
import { BlockBuilder } from '../block_builder/index.js';
import { L1Publisher, makeEmptyPrivateTx, makePrivateTx } from '../index.js';
import { makeProcessedTx } from './processed_tx.js';
import { PublicProcessor } from './public_processor.js';
import { Sequencer } from './sequencer.js';

describe('sequencer', () => {
  let publisher: MockProxy<L1Publisher>;
  let p2p: MockProxy<P2P>;
  let worldState: MockProxy<WorldStateSynchroniser>;
  let blockBuilder: MockProxy<BlockBuilder>;
  let merkleTreeOps: MockProxy<MerkleTreeOperations>;
  let publicProcessor: MockProxy<PublicProcessor>;

  let lastBlockNumber: number;

  let sequencer: TestSubject;

  beforeEach(() => {
    lastBlockNumber = 0;

    publisher = mock<L1Publisher>();
    merkleTreeOps = mock<MerkleTreeOperations>();
    blockBuilder = mock<BlockBuilder>();

    p2p = mock<P2P>({
      getStatus: () => Promise.resolve({ state: P2PClientState.IDLE, syncedToL2Block: lastBlockNumber }),
    });

    worldState = mock<WorldStateSynchroniser>({
      getLatest: () => merkleTreeOps,
      status: () => Promise.resolve({ state: WorldStateRunningState.IDLE, syncedToL2Block: lastBlockNumber }),
    });

    publicProcessor = mock<PublicProcessor>({
      process: async txs => [await Promise.all(txs.map(tx => makeProcessedTx(tx as PrivateTx))), []],
    });

    sequencer = new TestSubject(publisher, p2p, worldState, blockBuilder, publicProcessor);
  });

  it('builds a block out of a single tx', async () => {
    const tx = makePrivateTx();
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce([tx]);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    publisher.processUnverifiedData.mockResolvedValueOnce(true);

    await sequencer.initialSync();
    await sequencer.work();

    const expectedTxHashes = await Tx.getHashes([tx, ...times(3, makeEmptyPrivateTx)]);
    const expectedUnverifiedData = tx.unverifiedData;

    expect(blockBuilder.buildL2Block).toHaveBeenCalledWith(
      lastBlockNumber + 1,
      expectedTxHashes.map(hash => expect.objectContaining({ hash })),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
    expect(publisher.processUnverifiedData).toHaveBeenCalledWith(lastBlockNumber + 1, expectedUnverifiedData);
  });

  it('builds a block out of several txs rejecting double spends', async () => {
    const txs = [makePrivateTx(0x10000), makePrivateTx(0x20000), makePrivateTx(0x30000)];
    const doubleSpendTx = txs[1];
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce(txs);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    publisher.processUnverifiedData.mockResolvedValueOnce(true);

    // We make a nullifier from tx1 a part of the nullifier tree, so it gets rejected as double spend
    const doubleSpendNullifier = doubleSpendTx.data.end.newNullifiers[0].toBuffer();
    merkleTreeOps.findLeafIndex.mockImplementation((treeId: MerkleTreeId, value: Buffer) => {
      return Promise.resolve(
        treeId === MerkleTreeId.NULLIFIER_TREE && value.equals(doubleSpendNullifier) ? 1n : undefined,
      );
    });

    await sequencer.initialSync();
    await sequencer.work();

    const expectedTxHashes = await Tx.getHashes([txs[0], txs[2], makeEmptyPrivateTx(), makeEmptyPrivateTx()]);
    const expectedUnverifiedData = new UnverifiedData([
      ...txs[0].unverifiedData.dataChunks,
      ...txs[2].unverifiedData.dataChunks,
    ]);

    expect(blockBuilder.buildL2Block).toHaveBeenCalledWith(
      lastBlockNumber + 1,
      expectedTxHashes.map(hash => expect.objectContaining({ hash })),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
    expect(publisher.processUnverifiedData).toHaveBeenCalledWith(lastBlockNumber + 1, expectedUnverifiedData);
    expect(p2p.deleteTxs).toHaveBeenCalledWith([await doubleSpendTx.getTxHash()]);
  });
});

class TestSubject extends Sequencer {
  public work() {
    return super.work();
  }

  public initialSync(): Promise<void> {
    return super.initialSync();
  }
}
