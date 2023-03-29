import { P2P } from '@aztec/p2p';
import { WorldStateSynchroniser } from '@aztec/world-state';
import { getL1Publisher, L1Publisher, Sequencer, SequencerClientConfig } from '../index.js';

/**
 * Encapsulates the full sequencer and publisher.
 */
export class SequencerClient {
  constructor(private publisher: L1Publisher, private sequencer: Sequencer) {}

  public static async new(
    config: SequencerClientConfig,
    p2pClient: P2P,
    worldStateSynchroniser: WorldStateSynchroniser,
  ) {
    const publisher = getL1Publisher(config);
    const sequencer = new Sequencer(publisher, p2pClient, worldStateSynchroniser, config);
    await sequencer.start();
    return new SequencerClient(publisher, sequencer);
  }

  public async stop() {
    await this.sequencer.stop();
  }
}
