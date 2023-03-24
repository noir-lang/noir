import { P2P } from '@aztec/p2p';
import { WorldStateSynchroniser } from '@aztec/world-state';
import { getL2BlockPublisher, L2BlockPublisher, Sequencer, SequencerClientConfig } from '../index.js';

/**
 * Encapsulates the full sequencer and publisher.
 */
export class SequencerClient {
  constructor(private publisher: L2BlockPublisher, private sequencer: Sequencer) {}

  public static async new(
    config: SequencerClientConfig,
    p2pClient: P2P,
    worldStateSynchroniser: WorldStateSynchroniser,
  ) {
    const publisher = getL2BlockPublisher(config);
    const sequencer = new Sequencer(publisher, p2pClient, worldStateSynchroniser, config);
    await sequencer.start();
    return new SequencerClient(publisher, sequencer);
  }

  public async stop() {
    await this.sequencer.stop();
  }
}
