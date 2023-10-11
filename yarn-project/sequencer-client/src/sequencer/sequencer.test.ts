import {
  Fr,
  GlobalVariables,
  HistoricBlockData,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { P2P, P2PClientState } from '@aztec/p2p';
import {
  ContractDataSource,
  L1ToL2MessageSource,
  L2Block,
  L2BlockSource,
  MerkleTreeId,
  Tx,
  TxHash,
  mockTx,
} from '@aztec/types';
import { MerkleTreeOperations, WorldStateRunningState, WorldStateSynchronizer } from '@aztec/world-state';

import { MockProxy, mock } from 'jest-mock-extended';
import times from 'lodash.times';

import { BlockBuilder } from '../block_builder/index.js';
import { GlobalVariableBuilder } from '../global_variable_builder/global_builder.js';
import { L1Publisher } from '../index.js';
import { makeEmptyProcessedTx, makeProcessedTx } from './processed_tx.js';
import { PublicProcessor, PublicProcessorFactory } from './public_processor.js';
import { Sequencer } from './sequencer.js';

describe('sequencer', () => {
  let publisher: MockProxy<L1Publisher>;
  let globalVariableBuilder: MockProxy<GlobalVariableBuilder>;
  let p2p: MockProxy<P2P>;
  let worldState: MockProxy<WorldStateSynchronizer>;
  let blockBuilder: MockProxy<BlockBuilder>;
  let merkleTreeOps: MockProxy<MerkleTreeOperations>;
  let publicProcessor: MockProxy<PublicProcessor>;
  let l2BlockSource: MockProxy<L2BlockSource>;
  let l1ToL2MessageSource: MockProxy<L1ToL2MessageSource>;
  let publicProcessorFactory: MockProxy<PublicProcessorFactory>;

  let lastBlockNumber: number;

  let sequencer: TestSubject;

  const chainId = Fr.ZERO;
  const version = Fr.ZERO;

  beforeEach(() => {
    lastBlockNumber = 0;

    publisher = mock<L1Publisher>();
    globalVariableBuilder = mock<GlobalVariableBuilder>();
    merkleTreeOps = mock<MerkleTreeOperations>();
    blockBuilder = mock<BlockBuilder>();

    p2p = mock<P2P>({
      getStatus: () => Promise.resolve({ state: P2PClientState.IDLE, syncedToL2Block: lastBlockNumber }),
    });

    worldState = mock<WorldStateSynchronizer>({
      getLatest: () => merkleTreeOps,
      status: () => Promise.resolve({ state: WorldStateRunningState.IDLE, syncedToL2Block: lastBlockNumber }),
    });

    publicProcessor = mock<PublicProcessor>({
      process: async txs => [await Promise.all(txs.map(tx => makeProcessedTx(tx))), []],
      makeEmptyProcessedTx: () => makeEmptyProcessedTx(HistoricBlockData.empty(), chainId, version),
    });

    publicProcessorFactory = mock<PublicProcessorFactory>({
      create: (_a, _b_) => Promise.resolve(publicProcessor),
    });

    l2BlockSource = mock<L2BlockSource>({
      getBlockNumber: () => Promise.resolve(lastBlockNumber),
    });

    l1ToL2MessageSource = mock<L1ToL2MessageSource>({
      getPendingL1ToL2Messages: () => Promise.resolve(Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(Fr.ZERO)),
      getBlockNumber: () => Promise.resolve(lastBlockNumber),
    });

    const contractDataSource = mock<ContractDataSource>({});

    sequencer = new TestSubject(
      publisher,
      globalVariableBuilder,
      p2p,
      worldState,
      blockBuilder,
      l2BlockSource,
      l1ToL2MessageSource,
      contractDataSource,
      publicProcessorFactory,
    );
  });

  it('builds a block out of a single tx', async () => {
    const tx = mockTx();
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce([tx]);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO),
    );

    await sequencer.initialSync();
    await sequencer.work();

    const expectedTxHashes = [...(await Tx.getHashes([tx])), ...times(3, () => TxHash.ZERO)];

    expect(blockBuilder.buildL2Block).toHaveBeenCalledWith(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO),
      expectedTxHashes.map(hash => expect.objectContaining({ hash })),
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
  });

  it('builds a block out of several txs rejecting double spends', async () => {
    const txs = [mockTx(0x10000), mockTx(0x20000), mockTx(0x30000)];
    const doubleSpendTx = txs[1];
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce(txs);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO),
    );

    // We make a nullifier from tx1 a part of the nullifier tree, so it gets rejected as double spend
    const doubleSpendNullifier = doubleSpendTx.data.end.newNullifiers[0].toBuffer();
    merkleTreeOps.findLeafIndex.mockImplementation((treeId: MerkleTreeId, value: Buffer) => {
      return Promise.resolve(
        treeId === MerkleTreeId.NULLIFIER_TREE && value.equals(doubleSpendNullifier) ? 1n : undefined,
      );
    });

    await sequencer.initialSync();
    await sequencer.work();

    const expectedTxHashes = [...(await Tx.getHashes([txs[0], txs[2]])), TxHash.ZERO, TxHash.ZERO];

    expect(blockBuilder.buildL2Block).toHaveBeenCalledWith(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO),
      expectedTxHashes.map(hash => expect.objectContaining({ hash })),
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
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
