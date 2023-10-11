import { P2P } from '@aztec/p2p';
import { ContractDataSource, L1ToL2MessageSource, L2BlockSource } from '@aztec/types';
import { WorldStateSynchronizer } from '@aztec/world-state';

import { SoloBlockBuilder } from '../block_builder/solo_block_builder.js';
import { SequencerClientConfig } from '../config.js';
import { getGlobalVariableBuilder } from '../global_variable_builder/index.js';
import { Sequencer, SequencerConfig, getL1Publisher, getVerificationKeys } from '../index.js';
import { EmptyRollupProver } from '../prover/empty.js';
import { PublicProcessorFactory } from '../sequencer/public_processor.js';
import { WasmRollupCircuitSimulator } from '../simulator/rollup.js';

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
   * @returns A new running instance.
   */
  public static async new(
    config: SequencerClientConfig,
    p2pClient: P2P,
    worldStateSynchronizer: WorldStateSynchronizer,
    contractDataSource: ContractDataSource,
    l2BlockSource: L2BlockSource,
    l1ToL2MessageSource: L1ToL2MessageSource,
  ) {
    const publisher = getL1Publisher(config);
    const globalsBuilder = getGlobalVariableBuilder(config);
    const merkleTreeDb = worldStateSynchronizer.getLatest();

    const blockBuilder = new SoloBlockBuilder(
      merkleTreeDb,
      getVerificationKeys(),
      new WasmRollupCircuitSimulator(),
      new EmptyRollupProver(),
    );

    const publicProcessorFactory = new PublicProcessorFactory(merkleTreeDb, contractDataSource, l1ToL2MessageSource);

    const sequencer = new Sequencer(
      publisher,
      globalsBuilder,
      p2pClient,
      worldStateSynchronizer,
      blockBuilder,
      l2BlockSource,
      l1ToL2MessageSource,
      contractDataSource,
      publicProcessorFactory,
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
}
