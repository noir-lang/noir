import { createDebugLogger } from '@aztec/foundation/log';

import { type Libp2p } from 'libp2p';

import { type P2PConfig } from '../config.js';
import { type PeerDiscoveryService, PeerDiscoveryState } from './service.js';

export class PeerManager {
  constructor(
    private libP2PNode: Libp2p,
    private discV5Node: PeerDiscoveryService,
    private config: P2PConfig,
    private logger = createDebugLogger('aztec:p2p:peer_manager'),
  ) {}

  async updateDiscoveryService() {
    const peerCount = this.libP2PNode.getPeers().length;
    if (peerCount >= this.config.maxPeerCount && this.discV5Node.getStatus() === PeerDiscoveryState.RUNNING) {
      this.logger.debug('Max peer count reached, stopping discovery service');
      await this.discV5Node.stop();
    } else if (peerCount <= this.config.minPeerCount && this.discV5Node.getStatus() === PeerDiscoveryState.STOPPED) {
      this.logger.debug('Min peer count reached, starting discovery service');
      await this.discV5Node.start();
    }
  }
}
