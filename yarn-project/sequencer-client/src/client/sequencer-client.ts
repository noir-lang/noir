import { P2P } from '@aztec/p2p';
import { WorldStateSynchroniser } from '@aztec/world-state';

import { ContractDataSource } from '@aztec/types';
import { SoloBlockBuilder } from '../block_builder/solo_block_builder.js';
import { SequencerClientConfig } from '../config.js';
import { getL1Publisher, getVerificationKeys, Sequencer } from '../index.js';
import { EmptyPublicProver, EmptyRollupProver } from '../prover/empty.js';
import { PublicProcessor } from '../sequencer/public_processor.js';
import { FakePublicCircuitSimulator } from '../simulator/fake_public.js';
import { WasmPublicKernelCircuitSimulator } from '../simulator/public_kernel.js';
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
   * @param worldStateSynchroniser - Provides access to world state.
   * @param contractDataSource - Provides access to contract bytecode for public executions.
   * @returns A new running instance.
   */
  public static async new(
    config: SequencerClientConfig,
    p2pClient: P2P,
    worldStateSynchroniser: WorldStateSynchroniser,
    contractDataSource: ContractDataSource,
  ) {
    const publisher = getL1Publisher(config);
    const merkleTreeDb = worldStateSynchroniser.getLatest();

    const blockBuilder = new SoloBlockBuilder(
      merkleTreeDb,
      getVerificationKeys(),
      await WasmRollupCircuitSimulator.new(),
      new EmptyRollupProver(),
    );

    const publicProcessor = new PublicProcessor(
      merkleTreeDb,
      new FakePublicCircuitSimulator(merkleTreeDb, contractDataSource),
      new WasmPublicKernelCircuitSimulator(),
      new EmptyPublicProver(),
      contractDataSource,
    );

    const sequencer = new Sequencer(
      publisher,
      p2pClient,
      worldStateSynchroniser,
      blockBuilder,
      publicProcessor,
      config,
    );

    await sequencer.start();
    return new SequencerClient(sequencer);
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
