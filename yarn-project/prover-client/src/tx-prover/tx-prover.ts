import { ProcessedTx } from '@aztec/circuit-types';
import { ProverClient, ProvingTicket } from '@aztec/circuit-types/interfaces';
import { Fr, GlobalVariables } from '@aztec/circuits.js';
import { SimulationProvider } from '@aztec/simulator';
import { WorldStateSynchronizer } from '@aztec/world-state';

import { ProverConfig } from '../config.js';
import { VerificationKeys, getVerificationKeys } from '../mocks/verification_keys.js';
import { ProvingOrchestrator } from '../orchestrator/orchestrator.js';
import { EmptyRollupProver } from '../prover/empty.js';

/**
 * A prover accepting individual transaction requests
 */
export class TxProver implements ProverClient {
  private orchestrator: ProvingOrchestrator;
  constructor(
    worldStateSynchronizer: WorldStateSynchronizer,
    simulationProvider: SimulationProvider,
    protected vks: VerificationKeys,
  ) {
    this.orchestrator = new ProvingOrchestrator(
      worldStateSynchronizer.getLatest(),
      simulationProvider,
      getVerificationKeys(),
      new EmptyRollupProver(),
    );
  }

  /**
   * Starts the prover instance
   */
  public start() {
    this.orchestrator.start();
    return Promise.resolve();
  }

  /**
   * Stops the prover instance
   */
  public async stop() {
    await this.orchestrator.stop();
  }

  /**
   *
   * @param config - The prover configuration.
   * @param worldStateSynchronizer - An instance of the world state
   * @returns An instance of the prover, constructed and started.
   */
  public static async new(
    config: ProverConfig,
    worldStateSynchronizer: WorldStateSynchronizer,
    simulationProvider: SimulationProvider,
  ) {
    const prover = new TxProver(worldStateSynchronizer, simulationProvider, getVerificationKeys());
    await prover.start();
    return prover;
  }

  public startNewBlock(
    numTxs: number,
    globalVariables: GlobalVariables,
    newL1ToL2Messages: Fr[],
    emptyTx: ProcessedTx,
  ): Promise<ProvingTicket> {
    return this.orchestrator.startNewBlock(numTxs, globalVariables, newL1ToL2Messages, emptyTx);
  }

  public addNewTx(tx: ProcessedTx): Promise<void> {
    return this.orchestrator.addNewTx(tx);
  }
}
