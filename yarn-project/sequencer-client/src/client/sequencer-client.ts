import { P2P } from '@aztec/p2p';
import { WorldStateSynchroniser } from '@aztec/world-state';

import { ContractDataSource } from '@aztec/types';
import { CircuitBlockBuilder } from '../block_builder/circuit_block_builder.js';
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

  public static async new(
    config: SequencerClientConfig,
    p2pClient: P2P,
    worldStateSynchroniser: WorldStateSynchroniser,
    contractDataSource: ContractDataSource,
  ) {
    const publisher = getL1Publisher(config);
    const merkleTreeDb = worldStateSynchroniser.getLatest();

    const blockBuilder = new CircuitBlockBuilder(
      merkleTreeDb,
      getVerificationKeys(),
      await WasmRollupCircuitSimulator.new(),
      new EmptyRollupProver(),
    );

    const publicProcessor = new PublicProcessor(
      merkleTreeDb,
      new FakePublicCircuitSimulator(merkleTreeDb),
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

  public async stop() {
    await this.sequencer.stop();
  }

  public restart() {
    this.sequencer.restart();
  }
}
