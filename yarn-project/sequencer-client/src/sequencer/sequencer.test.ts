import {
  ExtendedContractData,
  L1ToL2MessageSource,
  L2Block,
  L2BlockSource,
  MerkleTreeId,
  Tx,
  TxHash,
  mockTx,
} from '@aztec/circuit-types';
import {
  AztecAddress,
  EthAddress,
  Fr,
  GlobalVariables,
  Header,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
  makeEmptyProof,
} from '@aztec/circuits.js';
import { times } from '@aztec/foundation/collection';
import { P2P, P2PClientState } from '@aztec/p2p';
import { MerkleTreeOperations, WorldStateRunningState, WorldStateSynchronizer } from '@aztec/world-state';

import { MockProxy, mock, mockFn } from 'jest-mock-extended';

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

  const chainId = new Fr(12345);
  const version = Fr.ZERO;
  const coinbase = EthAddress.random();
  const feeRecipient = AztecAddress.random();

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
      makeEmptyProcessedTx: () => makeEmptyProcessedTx(Header.empty(), chainId, version),
    });

    publicProcessorFactory = mock<PublicProcessorFactory>({
      create: (_a, _b_) => Promise.resolve(publicProcessor),
    });

    l2BlockSource = mock<L2BlockSource>({
      getBlockNumber: mockFn().mockResolvedValue(lastBlockNumber),
    });

    l1ToL2MessageSource = mock<L1ToL2MessageSource>({
      getPendingL1ToL2Messages: () => Promise.resolve(Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(Fr.ZERO)),
      getBlockNumber: () => Promise.resolve(lastBlockNumber),
    });

    sequencer = new TestSubject(
      publisher,
      globalVariableBuilder,
      p2p,
      worldState,
      blockBuilder,
      l2BlockSource,
      l1ToL2MessageSource,
      publicProcessorFactory,
    );
  });

  it('builds a block out of a single tx', async () => {
    const tx = mockTx();
    tx.data.constants.txContext.chainId = chainId;
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce([tx]);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
    );

    await sequencer.initialSync();
    await sequencer.work();

    const expectedTxHashes = [...(await Tx.getHashes([tx])), ...times(1, () => TxHash.ZERO)];

    expect(blockBuilder.buildL2Block).toHaveBeenCalledWith(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
      expectedTxHashes.map(hash => expect.objectContaining({ hash })),
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
  });

  it('builds a block out of several txs rejecting double spends', async () => {
    const txs = [mockTx(0x10000), mockTx(0x20000), mockTx(0x30000)];
    txs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId;
    });
    const doubleSpendTx = txs[1];
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce(txs);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
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

    const expectedTxHashes = await Tx.getHashes([txs[0], txs[2]]);

    expect(blockBuilder.buildL2Block).toHaveBeenCalledWith(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
      expectedTxHashes.map(hash => expect.objectContaining({ hash })),
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
    expect(p2p.deleteTxs).toHaveBeenCalledWith([await doubleSpendTx.getTxHash()]);
  });

  it('builds a block out of several txs rejecting incorrect chain ids', async () => {
    const txs = [mockTx(0x10000), mockTx(0x20000), mockTx(0x30000)];
    txs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId;
    });
    const invalidChainTx = txs[1];
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce(txs);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
    );

    // We make the chain id on the invalid tx not equal to the configured chain id
    invalidChainTx.data.constants.txContext.chainId = new Fr(1n + chainId.value);

    await sequencer.initialSync();
    await sequencer.work();

    const expectedTxHashes = await Tx.getHashes([txs[0], txs[2]]);

    expect(blockBuilder.buildL2Block).toHaveBeenCalledWith(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
      expectedTxHashes.map(hash => expect.objectContaining({ hash })),
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
    expect(p2p.deleteTxs).toHaveBeenCalledWith([await invalidChainTx.getTxHash()]);
  });

  it('aborts building a block if the chain moves underneath it', async () => {
    const tx = mockTx();
    tx.data.constants.txContext.chainId = chainId;
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce([tx]);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
    );

    await sequencer.initialSync();

    l2BlockSource.getBlockNumber
      // let it work for a bit
      .mockResolvedValueOnce(lastBlockNumber)
      .mockResolvedValueOnce(lastBlockNumber)
      .mockResolvedValueOnce(lastBlockNumber)
      // then tell it to abort
      .mockResolvedValue(lastBlockNumber + 1);

    await sequencer.work();

    expect(publisher.processL2Block).not.toHaveBeenCalled();
  });

  it('publishes contract data', async () => {
    const txWithContract = mockTx(0x10000);
    (txWithContract.newContracts as Array<ExtendedContractData>) = [ExtendedContractData.random()];
    txWithContract.data.constants.txContext.chainId = chainId;

    const txWithEmptyContract = mockTx(0x20000);
    (txWithEmptyContract.newContracts as Array<ExtendedContractData>) = [ExtendedContractData.empty()];
    txWithEmptyContract.data.constants.txContext.chainId = chainId;

    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();

    p2p.getTxs.mockResolvedValueOnce([txWithContract, txWithEmptyContract]);
    blockBuilder.buildL2Block.mockResolvedValueOnce([block, proof]);
    publisher.processL2Block.mockResolvedValueOnce(true);
    publisher.processNewContractData.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
    );

    await sequencer.initialSync();
    await sequencer.work();

    // check that the block was built with both transactions
    expect(blockBuilder.buildL2Block).toHaveBeenCalledWith(
      expect.anything(),
      expect.arrayContaining([
        expect.objectContaining({ hash: await txWithContract.getTxHash() }),
        expect.objectContaining({ hash: await txWithEmptyContract.getTxHash() }),
      ]),
      expect.any(Array),
    );

    // check that the empty contract did not get published
    expect(publisher.processNewContractData).toHaveBeenCalledWith(block.number, block.getCalldataHash(), [
      txWithContract.newContracts[0],
    ]);
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
