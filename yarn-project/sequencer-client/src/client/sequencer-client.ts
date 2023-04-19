import { P2P } from '@aztec/p2p';
import { WorldStateSynchroniser } from '@aztec/world-state';

import { CircuitBlockBuilder } from '../block_builder/circuit_block_builder.js';
import { SequencerClientConfig } from '../config.js';
import { getL1Publisher, getVerificationKeys, Sequencer } from '../index.js';
import { EmptyPublicProver, EmptyRollupProver } from '../prover/empty.js';
import { MockPublicProcessor } from '../sequencer/public_processor.js';
import { FakePublicCircuitSimulator } from '../simulator/fake_public.js';
import { MockPublicKernelCircuitSimulator } from '../simulator/mock_public_kernel.js';
import { WasmCircuitSimulator } from '../simulator/wasm.js';

/**
 * Encapsulates the full sequencer and publisher.
 */
export class SequencerClient {
  constructor(private sequencer: Sequencer) {}

  public static async new(
    config: SequencerClientConfig,
    p2pClient: P2P,
    worldStateSynchroniser: WorldStateSynchroniser,
  ) {
    const publisher = getL1Publisher(config);
    const merkleTreeDb = worldStateSynchroniser.getLatest();

    const blockBuilder = new CircuitBlockBuilder(
      merkleTreeDb,
      getVerificationKeys(),
      await WasmCircuitSimulator.new(),
      new EmptyRollupProver(),
    );

    // TODO: Swap with actual processor once the integration is good to go
    const publicProcessor = new MockPublicProcessor(
      merkleTreeDb,
      new FakePublicCircuitSimulator(merkleTreeDb),
      new MockPublicKernelCircuitSimulator(),
      new EmptyPublicProver(),
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
