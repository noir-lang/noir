import { CircuitsWasm } from '@aztec/circuits.js';
import { P2P } from '@aztec/p2p';
import { WorldStateSynchroniser } from '@aztec/world-state';
import { getL1Publisher, Sequencer, SequencerClientConfig } from '../index.js';

/**
 * Encapsulates the full sequencer and publisher.
 */
export class SequencerClient {
  constructor(private sequencer: Sequencer) {}

  public static async new(
    config: SequencerClientConfig,
    p2pClient: P2P,
    worldStateSynchroniser: WorldStateSynchroniser,
    wasm: CircuitsWasm,
  ) {
    const publisher = getL1Publisher(config);
    const sequencer = new Sequencer(publisher, p2pClient, worldStateSynchroniser, wasm, config);
    await sequencer.start();
    return new SequencerClient(sequencer);
  }

  public async stop() {
    await this.sequencer.stop();
  }
}
