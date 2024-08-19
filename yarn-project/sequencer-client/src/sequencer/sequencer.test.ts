import {
  type BlockSimulator,
  type L1ToL2MessageSource,
  L2Block,
  type L2BlockSource,
  MerkleTreeId,
  PROVING_STATUS,
  type ProvingSuccess,
  type ProvingTicket,
  type Tx,
  type UnencryptedL2Log,
  UnencryptedTxL2Logs,
  makeProcessedTx,
  mockTxForRollup,
} from '@aztec/circuit-types';
import {
  AztecAddress,
  EthAddress,
  Fr,
  GasFees,
  GlobalVariables,
  IS_DEV_NET,
  NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP,
} from '@aztec/circuits.js';
import { times } from '@aztec/foundation/collection';
import { randomBytes } from '@aztec/foundation/crypto';
import { type Writeable } from '@aztec/foundation/types';
import { type P2P, P2PClientState } from '@aztec/p2p';
import { type PublicProcessor, type PublicProcessorFactory } from '@aztec/simulator';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';
import { type ContractDataSource } from '@aztec/types/contracts';
import { type ValidatorClient } from '@aztec/validator-client';
import { type MerkleTreeOperations, WorldStateRunningState, type WorldStateSynchronizer } from '@aztec/world-state';

import { type MockProxy, mock, mockFn } from 'jest-mock-extended';

import { type BlockBuilderFactory } from '../block_builder/index.js';
import { type GlobalVariableBuilder } from '../global_variable_builder/global_builder.js';
import { type L1Publisher } from '../publisher/l1-publisher.js';
import { TxValidatorFactory } from '../tx_validator/tx_validator_factory.js';
import { Sequencer } from './sequencer.js';

describe('sequencer', () => {
  let publisher: MockProxy<L1Publisher>;
  let validatorClient: MockProxy<ValidatorClient>;
  let globalVariableBuilder: MockProxy<GlobalVariableBuilder>;
  let p2p: MockProxy<P2P>;
  let worldState: MockProxy<WorldStateSynchronizer>;
  let blockSimulator: MockProxy<BlockSimulator>;
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
  const gasFees = GasFees.empty();

  // We mock an attestation
  const mockedAttestation = {
    isEmpty: false,
    v: 27,
    r: Fr.random().toString(),
    s: Fr.random().toString(),
  };

  const getAttestations = () => (IS_DEV_NET ? undefined : [mockedAttestation]);

  beforeEach(() => {
    lastBlockNumber = 0;

    publisher = mock<L1Publisher>();

    publisher.isItMyTurnToSubmit.mockResolvedValue(true);

    globalVariableBuilder = mock<GlobalVariableBuilder>();
    merkleTreeOps = mock<MerkleTreeOperations>();
    blockSimulator = mock<BlockSimulator>();

    p2p = mock<P2P>({
      getStatus: () => Promise.resolve({ state: P2PClientState.IDLE, syncedToL2Block: lastBlockNumber }),
    });

    worldState = mock<WorldStateSynchronizer>({
      getLatest: () => merkleTreeOps,
      status: () => Promise.resolve({ state: WorldStateRunningState.IDLE, syncedToL2Block: lastBlockNumber }),
    });

    publicProcessor = mock<PublicProcessor>({
      process: async txs => [
        await Promise.all(txs.map(tx => makeProcessedTx(tx, tx.data.toKernelCircuitPublicInputs(), []))),
        [],
        [],
      ],
    });

    publicProcessorFactory = mock<PublicProcessorFactory>({
      create: (_a, _b) => publicProcessor,
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

    const blockBuilderFactory = mock<BlockBuilderFactory>({
      create: () => blockSimulator,
    });

    sequencer = new TestSubject(
      publisher,
      // TDOO(md): add the relevant methods to the validator client that will prevent it stalling when waiting for attestations
      validatorClient,
      globalVariableBuilder,
      p2p,
      worldState,
      blockBuilderFactory,
      l2BlockSource,
      l1ToL2MessageSource,
      publicProcessorFactory,
      new TxValidatorFactory(merkleTreeOps, contractSource, false),
      new NoopTelemetryClient(),
    );
  });

  it('builds a block out of a single tx', async () => {
    const tx = mockTxForRollup();
    tx.data.constants.txContext.chainId = chainId;
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockReturnValueOnce([tx]);
    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

    await sequencer.initialSync();
    await sequencer.work();

    expect(blockSimulator.startNewBlock).toHaveBeenCalledWith(
      2,
      mockedGlobalVariables,
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledTimes(1);
    expect(publisher.processL2Block).toHaveBeenCalledWith(block, getAttestations());
    expect(blockSimulator.cancelBlock).toHaveBeenCalledTimes(0);
  });

  it('builds a block when it is their turn', async () => {
    const tx = mockTxForRollup();
    tx.data.constants.txContext.chainId = chainId;
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockReturnValueOnce([tx]);
    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

    // Not your turn!
    publisher.isItMyTurnToSubmit.mockClear().mockResolvedValue(false);
    await sequencer.initialSync();
    await sequencer.work();
    expect(blockSimulator.startNewBlock).not.toHaveBeenCalled();

    // Now it is!
    publisher.isItMyTurnToSubmit.mockClear().mockResolvedValue(true);
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledWith(
      2,
      mockedGlobalVariables,
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block, getAttestations());
    expect(blockSimulator.cancelBlock).toHaveBeenCalledTimes(0);
  });

  it('builds a block out of several txs rejecting double spends', async () => {
    const txs = [mockTxForRollup(0x10000), mockTxForRollup(0x20000), mockTxForRollup(0x30000)];
    txs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId;
    });
    const doubleSpendTx = txs[1];
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockReturnValueOnce(txs);
    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

    // We make a nullifier from tx1 a part of the nullifier tree, so it gets rejected as double spend
    const doubleSpendNullifier = doubleSpendTx.data.forRollup!.end.nullifiers[0].toBuffer();
    merkleTreeOps.findLeafIndex.mockImplementation((treeId: MerkleTreeId, value: any) => {
      return Promise.resolve(
        treeId === MerkleTreeId.NULLIFIER_TREE && value.equals(doubleSpendNullifier) ? 1n : undefined,
      );
    });

    await sequencer.initialSync();
    await sequencer.work();

    expect(blockSimulator.startNewBlock).toHaveBeenCalledWith(
      2,
      mockedGlobalVariables,
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block, getAttestations());
    expect(p2p.deleteTxs).toHaveBeenCalledWith([doubleSpendTx.getTxHash()]);
    expect(blockSimulator.cancelBlock).toHaveBeenCalledTimes(0);
  });

  it('builds a block out of several txs rejecting incorrect chain ids', async () => {
    const txs = [mockTxForRollup(0x10000), mockTxForRollup(0x20000), mockTxForRollup(0x30000)];
    txs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId;
    });
    const invalidChainTx = txs[1];
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockReturnValueOnce(txs);
    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

    // We make the chain id on the invalid tx not equal to the configured chain id
    invalidChainTx.data.constants.txContext.chainId = new Fr(1n + chainId.value);

    await sequencer.initialSync();
    await sequencer.work();

    expect(blockSimulator.startNewBlock).toHaveBeenCalledWith(
      2,
      mockedGlobalVariables,
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block, getAttestations());
    expect(p2p.deleteTxs).toHaveBeenCalledWith([invalidChainTx.getTxHash()]);
    expect(blockSimulator.cancelBlock).toHaveBeenCalledTimes(0);
  });

  it('builds a block out of several txs dropping the ones that go over max size', async () => {
    const txs = [mockTxForRollup(0x10000), mockTxForRollup(0x20000), mockTxForRollup(0x30000)];
    txs.forEach(tx => {
      tx.data.constants.txContext.chainId = chainId;
    });
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockReturnValueOnce(txs);
    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

    // We make txs[1] too big to fit
    (txs[1] as Writeable<Tx>).unencryptedLogs = UnencryptedTxL2Logs.random(2, 4);
    (txs[1].unencryptedLogs.functionLogs[0].logs[0] as Writeable<UnencryptedL2Log>).data = randomBytes(1024 * 1022);

    await sequencer.initialSync();
    await sequencer.work();

    expect(blockSimulator.startNewBlock).toHaveBeenCalledWith(
      2,
      mockedGlobalVariables,
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledWith(block, getAttestations());
    expect(blockSimulator.cancelBlock).toHaveBeenCalledTimes(0);
  });

  it('builds a block once it reaches the minimum number of transactions', async () => {
    const txs = times(8, i => {
      const tx = mockTxForRollup(i * 0x10000);
      tx.data.constants.txContext.chainId = chainId;
      return tx;
    });
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

    await sequencer.initialSync();

    sequencer.updateConfig({ minTxsPerBlock: 4 });

    // block is not built with 0 txs
    p2p.getTxs.mockReturnValueOnce([]);
    //p2p.getTxs.mockReturnValueOnce(txs.slice(0, 4));
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledTimes(0);

    // block is not built with 3 txs
    p2p.getTxs.mockReturnValueOnce(txs.slice(0, 3));
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledTimes(0);

    // block is built with 4 txs
    p2p.getTxs.mockReturnValueOnce(txs.slice(0, 4));
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledWith(
      4,
      mockedGlobalVariables,
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledTimes(1);
    expect(publisher.processL2Block).toHaveBeenCalledWith(block, getAttestations());
    expect(blockSimulator.cancelBlock).toHaveBeenCalledTimes(0);
  });

  it('builds a block that contains zero real transactions once flushed', async () => {
    const txs = times(8, i => {
      const tx = mockTxForRollup(i * 0x10000);
      tx.data.constants.txContext.chainId = chainId;
      return tx;
    });
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

    await sequencer.initialSync();

    sequencer.updateConfig({ minTxsPerBlock: 4 });

    // block is not built with 0 txs
    p2p.getTxs.mockReturnValueOnce([]);
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledTimes(0);

    // block is not built with 3 txs
    p2p.getTxs.mockReturnValueOnce(txs.slice(0, 3));
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledTimes(0);

    // flush the sequencer and it should build a block
    sequencer.flush();

    // block is built with 0 txs
    p2p.getTxs.mockReturnValueOnce([]);
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledTimes(1);
    expect(blockSimulator.startNewBlock).toHaveBeenCalledWith(
      2,
      mockedGlobalVariables,
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledTimes(1);
    expect(publisher.processL2Block).toHaveBeenCalledWith(block, getAttestations());
    expect(blockSimulator.cancelBlock).toHaveBeenCalledTimes(0);
  });

  it('builds a block that contains less than the minimum number of transactions once flushed', async () => {
    const txs = times(8, i => {
      const tx = mockTxForRollup(i * 0x10000);
      tx.data.constants.txContext.chainId = chainId;
      return tx;
    });
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

    await sequencer.initialSync();

    sequencer.updateConfig({ minTxsPerBlock: 4 });

    // block is not built with 0 txs
    p2p.getTxs.mockReturnValueOnce([]);
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledTimes(0);

    // block is not built with 3 txs
    p2p.getTxs.mockReturnValueOnce(txs.slice(0, 3));
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledTimes(0);

    // flush the sequencer and it should build a block
    sequencer.flush();

    // block is built with 3 txs
    p2p.getTxs.mockReturnValueOnce(txs.slice(0, 3));
    await sequencer.work();
    expect(blockSimulator.startNewBlock).toHaveBeenCalledTimes(1);
    expect(blockSimulator.startNewBlock).toHaveBeenCalledWith(
      3,
      mockedGlobalVariables,
      Array(NUMBER_OF_L1_L2_MESSAGES_PER_ROLLUP).fill(new Fr(0n)),
    );
    expect(publisher.processL2Block).toHaveBeenCalledTimes(1);
    expect(publisher.processL2Block).toHaveBeenCalledWith(block, getAttestations());
    expect(blockSimulator.cancelBlock).toHaveBeenCalledTimes(0);
  });

  it('aborts building a block if the chain moves underneath it', async () => {
    const tx = mockTxForRollup();
    tx.data.constants.txContext.chainId = chainId;
    const block = L2Block.random(lastBlockNumber + 1);
    const result: ProvingSuccess = {
      status: PROVING_STATUS.SUCCESS,
    };
    const ticket: ProvingTicket = {
      provingPromise: Promise.resolve(result),
    };

    p2p.getTxs.mockReturnValueOnce([tx]);
    blockSimulator.startNewBlock.mockResolvedValueOnce(ticket);
    blockSimulator.finaliseBlock.mockResolvedValue({ block });
    publisher.processL2Block.mockResolvedValueOnce(true);

    const mockedGlobalVariables = new GlobalVariables(
      chainId,
      version,
      new Fr(lastBlockNumber + 1),
      block.header.globalVariables.slotNumber,
      Fr.ZERO,
      coinbase,
      feeRecipient,
      gasFees,
    );

    globalVariableBuilder.buildGlobalVariables.mockResolvedValueOnce(mockedGlobalVariables);

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
  public override work() {
    return super.work();
  }

  public override initialSync(): Promise<void> {
    return super.initialSync();
  }
}
