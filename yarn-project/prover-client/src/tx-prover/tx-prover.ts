import { BBCircuitVerifier, type BBProverConfig } from '@aztec/bb-prover';
import { type ProcessedTx } from '@aztec/circuit-types';
import {
  type BlockResult,
  type ProverClient,
  type ProvingJobSource,
  type ProvingTicket,
} from '@aztec/circuit-types/interfaces';
import { type Fr, type GlobalVariables, type VerificationKeys, getMockVerificationKeys } from '@aztec/circuits.js';
import { createDebugLogger } from '@aztec/foundation/log';
import { type SimulationProvider } from '@aztec/simulator';
import { type WorldStateSynchronizer } from '@aztec/world-state';

import { type ProverClientConfig } from '../config.js';
import { ProvingOrchestrator } from '../orchestrator/orchestrator.js';
import { MemoryProvingQueue } from '../prover-pool/memory-proving-queue.js';
import { ProverPool } from '../prover-pool/prover-pool.js';

const logger = createDebugLogger('aztec:tx-prover');

const PRIVATE_KERNEL = 'PrivateKernelTailArtifact';
const PRIVATE_KERNEL_TO_PUBLIC = 'PrivateKernelTailToPublicArtifact';

async function retrieveRealPrivateKernelVerificationKeys(config: BBProverConfig) {
  logger.info(`Retrieving private kernel verification keys`);
  const bbVerifier = await BBCircuitVerifier.new(config, [PRIVATE_KERNEL, PRIVATE_KERNEL_TO_PUBLIC]);
  const vks: VerificationKeys = {
    privateKernelCircuit: await bbVerifier.getVerificationKeyData(PRIVATE_KERNEL),
    privateKernelToPublicCircuit: await bbVerifier.getVerificationKeyData(PRIVATE_KERNEL_TO_PUBLIC),
  };
  return vks;
}

/**
 * A prover accepting individual transaction requests
 */
export class TxProver implements ProverClient {
  private orchestrator: ProvingOrchestrator;
  private queue = new MemoryProvingQueue();

  constructor(
    private config: ProverClientConfig,
    private worldStateSynchronizer: WorldStateSynchronizer,
    protected vks: VerificationKeys,
    private proverPool?: ProverPool,
  ) {
    logger.info(`BB ${config.bbBinaryPath}, directory: ${config.bbWorkingDirectory}`);
    this.orchestrator = new ProvingOrchestrator(worldStateSynchronizer.getLatest(), this.queue);
  }

  async updateProverConfig(config: Partial<ProverClientConfig>): Promise<void> {
    if (typeof config.proverAgents === 'number') {
      await this.proverPool?.rescale(config.proverAgents);
    }
    if (typeof config.realProofs === 'boolean' && config.realProofs) {
      this.vks = await retrieveRealPrivateKernelVerificationKeys(this.config);
    }
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
    config: ProverClientConfig,
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

      pool = ProverPool.nativePool(config, config.proverAgents, config.proverAgentPollInterval);
    } else {
      pool = ProverPool.testPool(simulationProvider, config.proverAgents, config.proverAgentPollInterval);
    }

    const vks = config.realProofs ? await retrieveRealPrivateKernelVerificationKeys(config) : getMockVerificationKeys();

    const prover = new TxProver(config, worldStateSynchronizer, vks, pool);
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
    return this.orchestrator.startNewBlock(numTxs, globalVariables, newL1ToL2Messages, emptyTx, this.vks);
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
