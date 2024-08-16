import {
  type L1ToL2MessageSource,
  type L2BlockSource,
  type MerkleTreeOperations,
  type ProverClient,
  type TxProvider,
} from '@aztec/circuit-types';
import { type L1Publisher } from '@aztec/sequencer-client';
import { type PublicProcessorFactory, type SimulationProvider } from '@aztec/simulator';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { type ContractDataSource } from '@aztec/types/contracts';
import { WorldStateRunningState, type WorldStateSynchronizer } from '@aztec/world-state';

import { type MockProxy, mock } from 'jest-mock-extended';

import { type BlockProvingJob } from './job/block-proving-job.js';
import { ProverNode } from './prover-node.js';

describe('prover-node', () => {
  let prover: MockProxy<ProverClient>;
  let publisher: MockProxy<L1Publisher>;
  let l2BlockSource: MockProxy<L2BlockSource>;
  let l1ToL2MessageSource: MockProxy<L1ToL2MessageSource>;
  let contractDataSource: MockProxy<ContractDataSource>;
  let worldState: MockProxy<WorldStateSynchronizer>;
  let txProvider: MockProxy<TxProvider>;
  let simulator: MockProxy<SimulationProvider>;
  let telemetryClient: MockProxy<TelemetryClient>;

  let proverNode: TestProverNode;

  // List of all jobs ever created by the test prover node and their dependencies
  let jobs: {
    job: MockProxy<BlockProvingJob>;
    cleanUp: (job: BlockProvingJob) => Promise<void>;
    db: MerkleTreeOperations;
  }[];

  beforeEach(() => {
    prover = mock<ProverClient>();
    publisher = mock<L1Publisher>();
    l2BlockSource = mock<L2BlockSource>();
    l1ToL2MessageSource = mock<L1ToL2MessageSource>();
    contractDataSource = mock<ContractDataSource>();
    worldState = mock<WorldStateSynchronizer>();
    txProvider = mock<TxProvider>();
    simulator = mock<SimulationProvider>();
    telemetryClient = mock<TelemetryClient>();

    // World state returns a new mock db every time it is asked to fork
    worldState.syncImmediateAndFork.mockImplementation(() => Promise.resolve(mock<MerkleTreeOperations>()));

    jobs = [];
    proverNode = new TestProverNode(
      prover,
      publisher,
      l2BlockSource,
      l1ToL2MessageSource,
      contractDataSource,
      worldState,
      txProvider,
      simulator,
      telemetryClient,
      { maxPendingJobs: 3, pollingIntervalMs: 10 },
    );
  });

  afterEach(async () => {
    await proverNode.stop();
  });

  const setBlockNumbers = (blockNumber: number, provenBlockNumber: number) => {
    l2BlockSource.getBlockNumber.mockResolvedValue(blockNumber);
    l2BlockSource.getProvenBlockNumber.mockResolvedValue(provenBlockNumber);
    worldState.status.mockResolvedValue({ syncedToL2Block: provenBlockNumber, state: WorldStateRunningState.RUNNING });
  };

  it('proves pending blocks', async () => {
    setBlockNumbers(5, 3);

    await proverNode.work();
    await proverNode.work();
    await proverNode.work();

    expect(jobs.length).toEqual(2);
    expect(jobs[0].job.run).toHaveBeenCalledWith(4, 4);
    expect(jobs[1].job.run).toHaveBeenCalledWith(5, 5);
  });

  it('stops proving when maximum jobs are reached', async () => {
    setBlockNumbers(10, 3);

    await proverNode.work();
    await proverNode.work();
    await proverNode.work();
    await proverNode.work();

    expect(jobs.length).toEqual(3);
    expect(jobs[0].job.run).toHaveBeenCalledWith(4, 4);
    expect(jobs[1].job.run).toHaveBeenCalledWith(5, 5);
    expect(jobs[2].job.run).toHaveBeenCalledWith(6, 6);
  });

  it('reports on pending jobs', async () => {
    setBlockNumbers(5, 3);

    await proverNode.work();
    await proverNode.work();

    expect(jobs.length).toEqual(2);
    expect(proverNode.getJobs().length).toEqual(2);
    expect(proverNode.getJobs()).toEqual([
      { uuid: '0', status: 'processing' },
      { uuid: '1', status: 'processing' },
    ]);
  });

  it('cleans up jobs when completed', async () => {
    setBlockNumbers(10, 3);

    await proverNode.work();
    await proverNode.work();
    await proverNode.work();
    await proverNode.work();

    expect(jobs.length).toEqual(3);
    expect(jobs[0].job.run).toHaveBeenCalledWith(4, 4);
    expect(jobs[1].job.run).toHaveBeenCalledWith(5, 5);
    expect(jobs[2].job.run).toHaveBeenCalledWith(6, 6);

    expect(proverNode.getJobs().length).toEqual(3);

    // Clean up the first job
    await jobs[0].cleanUp(jobs[0].job);
    expect(proverNode.getJobs().length).toEqual(2);
    expect(jobs[0].db.delete).toHaveBeenCalled();

    // Request another job to run and ensure it gets pushed
    await proverNode.work();
    expect(jobs.length).toEqual(4);
    expect(jobs[3].job.run).toHaveBeenCalledWith(7, 7);
    expect(proverNode.getJobs().length).toEqual(3);
    expect(proverNode.getJobs().map(({ uuid }) => uuid)).toEqual(['1', '2', '3']);
  });

  it('moves forward when proving fails', async () => {
    setBlockNumbers(10, 3);

    // We trigger an error by setting world state past the block that the prover node will try proving
    worldState.status.mockResolvedValue({ syncedToL2Block: 5, state: WorldStateRunningState.RUNNING });

    // These two calls should return in failures
    await proverNode.work();
    await proverNode.work();
    expect(jobs.length).toEqual(0);

    // But now the prover node should move forward
    await proverNode.work();
    expect(jobs.length).toEqual(1);
    expect(jobs[0].job.run).toHaveBeenCalledWith(6, 6);
  });

  class TestProverNode extends ProverNode {
    protected override doCreateBlockProvingJob(
      db: MerkleTreeOperations,
      _publicProcessorFactory: PublicProcessorFactory,
      cleanUp: (job: BlockProvingJob) => Promise<void>,
    ): BlockProvingJob {
      const job = mock<BlockProvingJob>({ getState: () => 'processing' });
      job.getId.mockReturnValue(jobs.length.toString());
      jobs.push({ job, cleanUp, db });
      return job;
    }

    public override work() {
      return super.work();
    }
  }
});
