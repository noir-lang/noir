import {
  type L1ToL2MessageSource,
  L2Block,
  type L2BlockSource,
  MerkleTreeId,
  PROVING_STATUS,
  type ProverClient,
  type ProvingSuccess,
  type ProvingTicket,
  makeEmptyProcessedTx,
  makeProcessedTx,
  mockTxForRollup,
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
import { makeProof } from '@aztec/circuits.js/testing';
import { type P2P, P2PClientState } from '@aztec/p2p';
import { type ContractDataSource } from '@aztec/types/contracts';
import { type MerkleTreeOperations, WorldStateRunningState, type WorldStateSynchronizer } from '@aztec/world-state';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { type GlobalVariableBuilder } from '../global_variable_builder/global_builder.js';
import { type L1Publisher } from '../index.js';
import { type PublicProcessor, type PublicProcessorFactory } from './public_processor.js';
import { Sequencer } from './sequencer.js';
import { TxValidatorFactory } from './tx_validator_factory.js';

describe('sequencer', () => {
  let publisher: MockProxy<L1Publisher>;
  let globalVariableBuilder: MockProxy<GlobalVariableBuilder>;
  let p2p: MockProxy<P2P>;
  let worldState: MockProxy<WorldStateSynchronizer>;
  let proverClient: MockProxy<ProverClient>;
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
    proverClient = mock<ProverClient>();

    p2p = mock<P2P>({
      getStatus: () => Promise.resolve({ state: P2PClientState.IDLE, syncedToL2Block: lastBlockNumber }),
    });

    worldState = mock<WorldStateSynchronizer>({
      getLatest: () => merkleTreeOps,
      status: () => Promise.resolve({ state: WorldStateRunningState.IDLE, syncedToL2Block: lastBlockNumber }),
    });

    publicProcessor = mock<PublicProcessor>({
      process: async txs => [
        await Promise.all(txs.map(tx => makeProcessedTx(tx, tx.data.toKernelCircuitPublicInputs(), makeProof()))),
        [],
        [],
      ],
      makeEmptyProcessedTx: () => makeEmptyProcessedTx(Header.empty(), chainId, version),
    });

    publicProcessorFactory = mock<PublicProcessorFactory>({
      create: (_a, _b_) => Promise.resolve(publicProcessor),
    });

    l2BlockSource = mock<L2BlockSource>({
      getBlockNumber: mockFn().mockResolvedValue(lastBlockNumber),
    });

    l1ToL2MessageSource = mock<L1ToL2MessageSource>({
      getL1ToL2Messages: () => Promise.resolve(Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(Fr.ZERO)),
      getBlockNumber: () => Promise.resolve(lastBlockNumber),
    });

    // all txs use the same allowed FPC class
    const fpcClassId = Fr.random();
    const contractSource = mock<ContractDataSource>({
      getContractClass: mockFn().mockResolvedValue(fpcClassId),
    });

    sequencer = new TestSubject(
      publisher,
      globalVariableBuilder,
      p2p,
      worldState,
      proverClient,
      l2BlockSource,
      l1ToL2MessageSource,
      publicProcessorFactory,
      new TxValidatorFactory(merkleTreeOps, contractSource, EthAddress.random()),
      {
        allowedFeePaymentContractClasses: [fpcClassId],
      },
    );
  });

  it('builds a block out of a single tx', async () => {
    const tx = mockTxForRollup();
    tx.data.constants.txContext.chainId = chainId;
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
      proof,
      block,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockResolvedValueOnce([tx]);
    proverClient.startNewBlock.mockResolvedValueOnce(ticket);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
    );

    await sequencer.initialSync();
    await sequencer.work();

    expect(proverClient.startNewBlock).toHaveBeenCalledWith(
      1,
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
      publicProcessor.makeEmptyProcessedTx(),
    );
    expect(proverClient.addNewTx).toHaveBeenCalledWith(expect.objectContaining({ hash: tx.getTxHash() }));
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
  });

  it('builds a block out of several txs rejecting double spends', async () => {
    const txs = [mockTxForRollup(0x10000), mockTxForRollup(0x20000), mockTxForRollup(0x30000)];
    txs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId;
    });
    const doubleSpendTx = txs[1];
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
      proof,
      block,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockResolvedValueOnce(txs);
    proverClient.startNewBlock.mockResolvedValueOnce(ticket);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
    );

    // We make a nullifier from tx1 a part of the nullifier tree, so it gets rejected as double spend
    const doubleSpendNullifier = doubleSpendTx.data.forRollup!.end.newNullifiers[0].value.toBuffer();
    merkleTreeOps.findLeafIndex.mockImplementation((treeId: MerkleTreeId, value: any) => {
      return Promise.resolve(
        treeId === MerkleTreeId.NULLIFIER_TREE && value.equals(doubleSpendNullifier) ? 1n : undefined,
      );
    });

    await sequencer.initialSync();
    await sequencer.work();

    expect(proverClient.startNewBlock).toHaveBeenCalledWith(
      2,
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
      publicProcessor.makeEmptyProcessedTx(),
    );
    expect(proverClient.addNewTx).toHaveBeenCalledWith(expect.objectContaining({ hash: txs[0].getTxHash() }));
    expect(proverClient.addNewTx).toHaveBeenCalledWith(expect.objectContaining({ hash: txs[2].getTxHash() }));
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
    expect(p2p.deleteTxs).toHaveBeenCalledWith([doubleSpendTx.getTxHash()]);
  });

  it('builds a block out of several txs rejecting incorrect chain ids', async () => {
    const txs = [mockTxForRollup(0x10000), mockTxForRollup(0x20000), mockTxForRollup(0x30000)];
    txs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId;
    });
    const invalidChainTx = txs[1];
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
      proof,
      block,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockResolvedValueOnce(txs);
    proverClient.startNewBlock.mockResolvedValueOnce(ticket);
    publisher.processL2Block.mockResolvedValueOnce(true);
    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
    );

    // We make the chain id on the invalid tx not equal to the configured chain id
    invalidChainTx.data.constants.txContext.chainId = new Fr(1n + chainId.value);

    await sequencer.initialSync();
    await sequencer.work();

    expect(proverClient.startNewBlock).toHaveBeenCalledWith(
      2,
      new GlobalVariables(chainId, version, new Fr(lastBlockNumber + 1), Fr.ZERO, coinbase, feeRecipient),
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
      publicProcessor.makeEmptyProcessedTx(),
    );
    expect(proverClient.addNewTx).toHaveBeenCalledWith(expect.objectContaining({ hash: txs[0].getTxHash() }));
    expect(proverClient.addNewTx).toHaveBeenCalledWith(expect.objectContaining({ hash: txs[2].getTxHash() }));
    expect(publisher.processL2Block).toHaveBeenCalledWith(block);
    expect(p2p.deleteTxs).toHaveBeenCalledWith([invalidChainTx.getTxHash()]);
  });

  it('aborts building a block if the chain moves underneath it', async () => {
    const tx = mockTxForRollup();
    tx.data.constants.txContext.chainId = chainId;
    const block = L2Block.random(lastBlockNumber + 1);
    const proof = makeEmptyProof();
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
      proof,
      block,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockResolvedValueOnce([tx]);
    proverClient.startNewBlock.mockResolvedValueOnce(ticket);
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
});

class TestSubject extends Sequencer {
  public work() {
    return super.work();
  }

  public initialSync(): Promise<void> {
    return super.initialSync();
  }
}
