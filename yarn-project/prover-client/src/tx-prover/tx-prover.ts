import { type ProcessedTx } from '@aztec/circuit-types';
import {
  type BlockResult,
  type ProverClient,
  type ProvingJobSource,
  type ProvingTicket,
} from '@aztec/circuit-types/interfaces';
import { type Fr, type GlobalVariables } from '@aztec/circuits.js';
import { type SimulationProvider } from '@aztec/simulator';
import { type WorldStateSynchronizer } from '@aztec/world-state';

import { type ProverConfig } from '../config.js';
import { type VerificationKeys, getVerificationKeys } from '../mocks/verification_keys.js';
import { ProvingOrchestrator } from '../orchestrator/orchestrator.js';
import { MemoryProvingQueue } from '../prover-pool/memory-proving-queue.js';
import { ProverPool } from '../prover-pool/prover-pool.js';

/**
 * A prover accepting individual transaction requests
 */
export class TxProver implements ProverClient {
  private orchestrator: ProvingOrchestrator;
  private queue = new MemoryProvingQueue();

  constructor(
    private worldStateSynchronizer: WorldStateSynchronizer,
    protected vks: VerificationKeys,
    private proverPool?: ProverPool,
  ) {
    this.orchestrator = new ProvingOrchestrator(worldStateSynchronizer.getLatest(), this.queue);
  }

  /**
   * Starts the prover instance
   */
  public async start() {
    await this.proverPool?.start(this.queue);
  }

  /**
   * Stops the prover instance
   */
  public async stop() {
    await this.proverPool?.stop();
  }

  /**
   *
   * @param config - The prover configuration.
   * @param worldStateSynchronizer - An instance of the world state
   * @returns An instance of the prover, constructed and started.
   */
  public static async new(
    config: ProverConfig,
    simulationProvider: SimulationProvider,
    worldStateSynchronizer: WorldStateSynchronizer,
  ) {
    let pool: ProverPool | undefined;
    if (config.proverAgents === 0) {
      pool = undefined;
    } else if (config.realProofs) {
      if (
        !config.acvmBinaryPath ||
        !config.acvmWorkingDirectory ||
        !config.bbBinaryPath ||
        !config.bbWorkingDirectory
      ) {
        throw new Error();
      }

      pool = ProverPool.nativePool(config, config.proverAgents, 10);
    } else {
      pool = ProverPool.testPool(simulationProvider, config.proverAgents, 10);
    }

    const prover = new TxProver(worldStateSynchronizer, getVerificationKeys(), pool);
    await prover.start();
    return prover;
  }

  /**
   * Cancels any block that is currently being built and prepares for a new one to be built
   * @param numTxs - The complete size of the block, must be a power of 2
   * @param globalVariables - The global variables for this block
   * @param l1ToL2Messages - The set of L1 to L2 messages to be included in this block
   * @param emptyTx - An instance of an empty transaction to be used in this block
   */
  public async startNewBlock(
    numTxs: number,
    globalVariables: GlobalVariables,
    newL1ToL2Messages: Fr[],
    emptyTx: ProcessedTx,
  ): Promise<ProvingTicket> {
    const previousBlockNumber = globalVariables.blockNumber.toNumber() - 1;
    await this.worldStateSynchronizer.syncImmediate(previousBlockNumber);
    return this.orchestrator.startNewBlock(numTxs, globalVariables, newL1ToL2Messages, emptyTx);
  }

  /**
   * Add a processed transaction to the current block
   * @param tx - The transaction to be added
   */
  public addNewTx(tx: ProcessedTx): Promise<void> {
    return this.orchestrator.addNewTx(tx);
  }

  /**
   * Cancels the block currently being proven. Proofs already bring built may continue but further proofs should not be started.
   */
  public cancelBlock(): void {
    this.orchestrator.cancelBlock();
  }

  /**
   * Performs the final archive tree insertion for this block and returns the L2Block and Proof instances
   */
  public finaliseBlock(): Promise<BlockResult> {
    return this.orchestrator.finaliseBlock();
  }

  /**
   * Mark the block as having all the transactions it is going to contain.
   * Will pad the block to it's complete size with empty transactions and prove all the way to the root rollup.
   */
  public setBlockCompleted(): Promise<void> {
    return this.orchestrator.setBlockCompleted();
  }

  getProvingJobSource(): ProvingJobSource {
    return this.queue;
  }
}
