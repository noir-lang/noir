import { type L1ToL2MessageSource, type L2BlockSource, type ProverClient, type TxProvider } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { type L1Publisher } from '@aztec/sequencer-client';
import { PublicProcessorFactory, type SimulationProvider } from '@aztec/simulator';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { type WorldStateSynchronizer } from '@aztec/world-state';

import { type ContractDataSource } from '../../types/src/contracts/contract_data_source.js';
import { BlockProvingJob } from './job/block-proving-job.js';

/**
 * An Aztec Prover Node is a standalone process that monitors the unfinalised chain on L1 for unproven blocks,
 * fetches their txs from a tx source in the p2p network or an external node, re-executes their public functions,
 * creates a rollup proof, and submits it to L1.
 */
export class ProverNode {
  private log = createDebugLogger('aztec:prover-node');
  private runningPromise: RunningPromise | undefined;
  private latestBlockWeAreProving: number | undefined;

  constructor(
    private prover: ProverClient,
    private publisher: L1Publisher,
    private l2BlockSource: L2BlockSource,
    private l1ToL2MessageSource: L1ToL2MessageSource,
    private contractDataSource: ContractDataSource,
    private worldState: WorldStateSynchronizer,
    private txProvider: TxProvider,
    private simulator: SimulationProvider,
    private telemetryClient: TelemetryClient,
    private options: { pollingIntervalMs: number; disableAutomaticProving: boolean } = {
      pollingIntervalMs: 1_000,
      disableAutomaticProving: false,
    },
  ) {}

  /**
   * Starts the prover node so it periodically checks for unproven blocks in the unfinalised chain from L1 and proves them.
   * This may change once we implement a prover coordination mechanism.
   */
  start() {
    this.runningPromise = new RunningPromise(this.work.bind(this), this.options.pollingIntervalMs);
    this.runningPromise.start();
    this.log.info('Started ProverNode');
  }

  /**
   * Stops the prover node and all its dependencies.
   */
  async stop() {
    this.log.info('Stopping ProverNode');
    await this.runningPromise?.stop();
    await this.prover.stop();
    await this.l2BlockSource.stop();
    this.publisher.interrupt();
    this.log.info('Stopped ProverNode');
    // TODO(palla/prover-node): Keep a reference to all ongoing ProvingJobs and stop them.
  }

  /**
   * Single iteration of recurring work. This method is called periodically by the running promise.
   * Checks whether there are new blocks to prove, proves them, and submits them.
   */
  protected async work() {
    try {
      if (this.options.disableAutomaticProving) {
        return;
      }

      const [latestBlockNumber, latestProvenBlockNumber] = await Promise.all([
        this.l2BlockSource.getBlockNumber(),
        this.l2BlockSource.getProvenBlockNumber(),
      ]);

      // Consider both the latest block we are proving and the last block proven on the chain
      const latestBlockBeingProven = this.latestBlockWeAreProving ?? 0;
      const latestProven = Math.max(latestBlockBeingProven, latestProvenBlockNumber);
      if (latestProven >= latestBlockNumber) {
        this.log.debug(`No new blocks to prove`, {
          latestBlockNumber,
          latestProvenBlockNumber,
          latestBlockBeingProven,
        });
        return;
      }

      const fromBlock = latestProven + 1;
      const toBlock = fromBlock; // We only prove one block at a time for now

      await this.startProof(fromBlock, toBlock);
      this.latestBlockWeAreProving = toBlock;
    } catch (err) {
      this.log.error(`Error in prover node work`, err);
    }
  }

  /**
   * Creates a proof for a block range. Returns once the proof has been submitted to L1.
   */
  public async prove(fromBlock: number, toBlock: number) {
    const job = await this.createProvingJob(fromBlock);
    return job.run(fromBlock, toBlock);
  }

  /**
   * Starts a proving process and returns immediately.
   */
  public async startProof(fromBlock: number, toBlock: number) {
    const job = await this.createProvingJob(fromBlock);
    void job.run(fromBlock, toBlock);
  }

  /**
   * Returns the prover instance.
   */
  public getProver() {
    return this.prover;
  }

  private async createProvingJob(fromBlock: number) {
    if ((await this.worldState.status()).syncedToL2Block >= fromBlock) {
      throw new Error(`Cannot create proving job for block ${fromBlock} as it is behind the current world state`);
    }

    // Fast forward world state to right before the target block and get a fork
    const db = await this.worldState.syncImmediateAndFork(fromBlock - 1, true);

    // Create a processor using the forked world state
    const publicProcessorFactory = new PublicProcessorFactory(
      db,
      this.contractDataSource,
      this.simulator,
      this.telemetryClient,
    );

    return new BlockProvingJob(
      this.prover.createBlockProver(db),
      publicProcessorFactory,
      this.publisher,
      this.l2BlockSource,
      this.l1ToL2MessageSource,
      this.txProvider,
      () => db.delete(),
    );
  }
}
