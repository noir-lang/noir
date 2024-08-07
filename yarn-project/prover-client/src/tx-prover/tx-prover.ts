import { BBNativeRollupProver, TestCircuitProver } from '@aztec/bb-prover';
import {
  type BlockProver,
  type ProverClient,
  type ProvingJobSource,
  type ServerCircuitProver,
} from '@aztec/circuit-types/interfaces';
import { Fr } from '@aztec/circuits.js';
import { NativeACVMSimulator } from '@aztec/simulator';
import { type TelemetryClient } from '@aztec/telemetry-client';
import { type MerkleTreeOperations } from '@aztec/world-state';

import { type ProverClientConfig } from '../config.js';
import { ProvingOrchestrator } from '../orchestrator/orchestrator.js';
import { MemoryProvingQueue } from '../prover-agent/memory-proving-queue.js';
import { ProverAgent } from '../prover-agent/prover-agent.js';

/**
 * A prover factory.
 * TODO(palla/prover-node): Rename this class
 */
export class TxProver implements ProverClient {
  private queue: MemoryProvingQueue;
  private running = false;

  private constructor(
    private config: ProverClientConfig,
    private telemetry: TelemetryClient,
    private agent?: ProverAgent,
  ) {
    // TODO(palla/prover-node): Cache the paddingTx here, and not in each proving orchestrator,
    // so it can be reused across multiple ones and not recomputed every time.
    this.queue = new MemoryProvingQueue(telemetry, config.proverJobTimeoutMs, config.proverJobPollIntervalMs);
  }

  public createBlockProver(db: MerkleTreeOperations): BlockProver {
    return new ProvingOrchestrator(db, this.queue, this.telemetry, this.config.proverId);
  }

  public getProverId(): Fr {
    return this.config.proverId ?? Fr.ZERO;
  }

  async updateProverConfig(config: Partial<ProverClientConfig>): Promise<void> {
    const newConfig = { ...this.config, ...config };

    if (newConfig.realProofs !== this.config.realProofs && this.agent) {
      const circuitProver = await TxProver.buildCircuitProver(newConfig, this.telemetry);
      this.agent.setCircuitProver(circuitProver);
    }

    if (this.config.proverAgentConcurrency !== newConfig.proverAgentConcurrency) {
      this.agent?.setMaxConcurrency(newConfig.proverAgentConcurrency);
    }

    if (!this.config.realProofs && newConfig.realProofs) {
      // TODO(palla/prover-node): Reset padding tx here once we cache it at this class
    }

    this.config = newConfig;
  }

  /**
   * Starts the prover instance
   */
  public start() {
    if (this.running) {
      return Promise.resolve();
    }

    this.running = true;
    this.queue.start();
    this.agent?.start(this.queue);
    return Promise.resolve();
  }

  /**
   * Stops the prover instance
   */
  public async stop() {
    if (!this.running) {
      return;
    }
    this.running = false;

    // TODO(palla/prover-node): Keep a reference to all proving orchestrators that are alive and stop them?
    await this.agent?.stop();
    await this.queue.stop();
  }

  /**
   * Creates a new prover client and starts it
   * @param config - The prover configuration.
   * @param vks - The verification keys for the prover
   * @param worldStateSynchronizer - An instance of the world state
   * @returns An instance of the prover, constructed and started.
   */
  public static async new(config: ProverClientConfig, telemetry: TelemetryClient) {
    const agent = config.proverAgentEnabled
      ? new ProverAgent(
          await TxProver.buildCircuitProver(config, telemetry),
          config.proverAgentConcurrency,
          config.proverAgentPollInterval,
        )
      : undefined;

    const prover = new TxProver(config, telemetry, agent);
    await prover.start();
    return prover;
  }

  private static async buildCircuitProver(
    config: ProverClientConfig,
    telemetry: TelemetryClient,
  ): Promise<ServerCircuitProver> {
    if (config.realProofs) {
      return await BBNativeRollupProver.new(config, telemetry);
    }

    const simulationProvider = config.acvmBinaryPath
      ? new NativeACVMSimulator(config.acvmWorkingDirectory, config.acvmBinaryPath)
      : undefined;

    return new TestCircuitProver(telemetry, simulationProvider);
  }

  public getProvingJobSource(): ProvingJobSource {
    return this.queue;
  }
}
