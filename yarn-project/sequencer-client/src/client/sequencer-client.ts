import { P2P } from '@aztec/p2p';
import { WorldStateSynchroniser } from '@aztec/world-state';
import { CircuitBlockBuilder } from '../block_builder/circuit_block_builder.js';
import { getL1Publisher, getVerificationKeys, Sequencer, SequencerClientConfig } from '../index.js';
import { EmptyProver } from '../prover/empty.js';
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
    const blockBuilder = new CircuitBlockBuilder(
      worldStateSynchroniser.getLatest(),
      getVerificationKeys(),
      await WasmCircuitSimulator.new(),
      new EmptyProver(),
    );
    const sequencer = new Sequencer(publisher, p2pClient, worldStateSynchroniser, blockBuilder, config);
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
