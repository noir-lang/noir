import { type L1ToL2MessageSource, type L2BlockSource } from '@aztec/circuit-types';
import { type BlockProver } from '@aztec/circuit-types/interfaces';
import { type P2P } from '@aztec/p2p';
import { type SimulationProvider } from '@aztec/simulator';
import { type ContractDataSource } from '@aztec/types/contracts';
import { type WorldStateSynchronizer } from '@aztec/world-state';

import { type SequencerClientConfig } from '../config.js';
import { getGlobalVariableBuilder } from '../global_variable_builder/index.js';
import { getL1Publisher } from '../publisher/index.js';
import { Sequencer, type SequencerConfig } from '../sequencer/index.js';
import { PublicProcessorFactory } from '../sequencer/public_processor.js';
import { TxValidatorFactory } from '../sequencer/tx_validator_factory.js';

/**
 * Encapsulates the full sequencer and publisher.
 */
export class SequencerClient {
  constructor(private sequencer: Sequencer) {}

  /**
   * Initializes and starts a new instance.
   * @param config - Configuration for the sequencer, publisher, and L1 tx sender.
   * @param p2pClient - P2P client that provides the txs to be sequenced.
   * @param worldStateSynchronizer - Provides access to world state.
   * @param contractDataSource - Provides access to contract bytecode for public executions.
   * @param l2BlockSource - Provides information about the previously published blocks.
   * @param l1ToL2MessageSource - Provides access to L1 to L2 messages.
   * @param prover - An instance of a block prover
   * @param simulationProvider - An instance of a simulation provider
   * @returns A new running instance.
   */
  public static async new(
    config: SequencerClientConfig,
    p2pClient: P2P,
    worldStateSynchronizer: WorldStateSynchronizer,
    contractDataSource: ContractDataSource,
    l2BlockSource: L2BlockSource,
    l1ToL2MessageSource: L1ToL2MessageSource,
    prover: BlockProver,
    simulationProvider: SimulationProvider,
  ) {
    const publisher = getL1Publisher(config);
    const globalsBuilder = getGlobalVariableBuilder(config);
    const merkleTreeDb = worldStateSynchronizer.getLatest();

    const publicProcessorFactory = new PublicProcessorFactory(merkleTreeDb, contractDataSource, simulationProvider);

    const sequencer = new Sequencer(
      publisher,
      globalsBuilder,
      p2pClient,
      worldStateSynchronizer,
      prover,
      l2BlockSource,
      l1ToL2MessageSource,
      publicProcessorFactory,
      new TxValidatorFactory(merkleTreeDb, contractDataSource, config.l1Contracts.gasPortalAddress),
      config,
    );

    await sequencer.start();
    return new SequencerClient(sequencer);
  }

  /**
   * Updates sequencer config.
   * @param config - New parameters.
   */
  public updateSequencerConfig(config: SequencerConfig) {
    this.sequencer.updateConfig(config);
  }

  /**
   * Stops the sequencer from processing new txs.
   */
  public async stop() {
    await this.sequencer.stop();
  }

  /**
   * Restarts the sequencer after being stopped.
   */
  public restart() {
    this.sequencer.restart();
  }

  get coinbase() {
    return this.sequencer.coinbase;
  }

  get feeRecipient() {
    return this.sequencer.feeRecipient;
  }
}
