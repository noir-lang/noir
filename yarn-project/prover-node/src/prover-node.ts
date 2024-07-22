import { type L2BlockSource, type ProverClient, type TxProvider } from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';
import { type L1Publisher } from '@aztec/sequencer-client';
import { type PublicProcessorFactory } from '@aztec/simulator';

import { BlockProvingJob } from './job/block-proving-job.js';

/**
 * An Aztec Prover Node is a standalone process that monitors the unfinalised chain on L1 for unproven blocks,
 * fetches their txs from a tx source in the p2p network or an external node, re-executes their public functions,
 * creates a rollup proof, and submits it to L1.
 */
export class ProverNode {
  private log = createDebugLogger('aztec:prover-node');
  private runningPromise: RunningPromise | undefined;

  constructor(
    private prover: ProverClient,
    private publicProcessorFactory: PublicProcessorFactory,
    private publisher: L1Publisher,
    private l2BlockSource: L2BlockSource,
    private txProvider: TxProvider,
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
  }

  /**
   * Single iteration of recurring work. This method is called periodically by the running promise.
   * Checks whether there are new blocks to prove, proves them, and submits them.
   * Only proves one block per job and one job at a time (for now).
   */
  protected async work() {
    if (this.options.disableAutomaticProving) {
      return;
    }

    const [latestBlockNumber, latestProvenBlockNumber] = await Promise.all([
      this.l2BlockSource.getBlockNumber(),
      this.l2BlockSource.getProvenBlockNumber(),
    ]);

    if (latestProvenBlockNumber >= latestBlockNumber) {
      this.log.debug(`No new blocks to prove`, { latestBlockNumber, latestProvenBlockNumber });
      return;
    }

    const fromBlock = latestProvenBlockNumber + 1;
    const toBlock = fromBlock; // We only prove one block at a time for now
    await this.prove(fromBlock, toBlock);
  }

  /**
   * Creates a proof for a block range. Returns once the proof has been submitted to L1.
   */
  public prove(fromBlock: number, toBlock: number) {
    return this.createProvingJob().run(fromBlock, toBlock);
  }

  /**
   * Starts a proving process and returns immediately.
   */
  public startProof(fromBlock: number, toBlock: number) {
    void this.createProvingJob().run(fromBlock, toBlock);
    return Promise.resolve();
  }

  /**
   * Returns the prover instance.
   */
  public getProver() {
    return this.prover;
  }

  private createProvingJob() {
    return new BlockProvingJob(
      this.prover,
      this.publicProcessorFactory,
      this.publisher,
      this.l2BlockSource,
      this.txProvider,
    );
  }
}
