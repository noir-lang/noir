import { createDebugLogger } from '@aztec/foundation/log';

import { type ENR } from '@chainsafe/enr';
import { type PeerId } from '@libp2p/interface';
import { type Multiaddr } from '@multiformats/multiaddr';
import { type Libp2p } from 'libp2p';

import { type P2PConfig } from '../config.js';
import { type PeerDiscoveryService } from './service.js';

const MAX_DIAL_ATTEMPTS = 3;
const MAX_CACHED_PEERS = 100;

type CachedPeer = {
  peerId: PeerId;
  enr: ENR;
  multiaddrTcp: Multiaddr;
  dialAttempts: number;
};

export class PeerManager {
  private cachedPeers: Map<string, CachedPeer> = new Map();
  constructor(
    private libP2PNode: Libp2p,
    private peerDiscoveryService: PeerDiscoveryService,
    private config: P2PConfig,
    private logger = createDebugLogger('aztec:p2p:peer_manager'),
  ) {
    // Handle new established connections
    this.libP2PNode.addEventListener('peer:connect', evt => {
      const peerId = evt.detail;
      if (this.peerDiscoveryService.isBootstrapPeer(peerId)) {
        this.logger.debug(`Connected to bootstrap peer ${peerId.toString()}`);
      } else {
        this.logger.debug(`Connected to transaction peer ${peerId.toString()}`);
      }
    });

    // Handle lost connections
    this.libP2PNode.addEventListener('peer:disconnect', evt => {
      const peerId = evt.detail;
      if (this.peerDiscoveryService.isBootstrapPeer(peerId)) {
        this.logger.debug(`Disconnected from bootstrap peer ${peerId.toString()}`);
      } else {
        this.logger.debug(`Disconnected from transaction peer ${peerId.toString()}`);
      }
    });

    // Handle Discovered peers
    this.peerDiscoveryService.on('peer:discovered', async (enr: ENR) => {
      await this.handleDiscoveredPeer(enr);
    });
  }

  /**
   * Discovers peers.
   */
  public discover() {
    // Get current connections
    const connections = this.libP2PNode.getConnections();

    // Calculate how many connections we're looking to make
    const peersToConnect = this.config.maxPeerCount - connections.length;

    this.logger.debug(
      `Connections: ${connections.length}, Peers to connect: ${peersToConnect}, maxPeerCount: ${this.config.maxPeerCount}, cachedPeers: ${this.cachedPeers.size}`,
    );

    // Exit if no peers to connect
    if (peersToConnect <= 0) {
      return;
    }

    const cachedPeersToDial: CachedPeer[] = [];

    const pendingDials = new Set(
      this.libP2PNode
        .getDialQueue()
        .map(pendingDial => pendingDial.peerId?.toString())
        .filter(Boolean) as string[],
    );

    for (const [id, peerData] of this.cachedPeers.entries()) {
      // if already dialling or connected to, remove from cache
      if (pendingDials.has(id) || connections.some(conn => conn.remotePeer.equals(peerData.peerId))) {
        this.cachedPeers.delete(id);
      } else {
        // cachedPeersToDial.set(id, enr);
        cachedPeersToDial.push(peerData);
      }
    }

    // reverse to dial older entries first
    cachedPeersToDial.reverse();

    for (const peer of cachedPeersToDial) {
      this.cachedPeers.delete(peer.peerId.toString());
      void this.dialPeer(peer);
    }

    // if we need more peers, start randomNodesQuery
    if (peersToConnect > 0) {
      this.logger.debug('Running random nodes query');
      void this.peerDiscoveryService.runRandomNodesQuery();
    }
  }

  /**
   *  Handles a discovered peer.
   * @param enr - The discovered peer's ENR.
   */
  private async handleDiscoveredPeer(enr: ENR) {
    // TODO: Will be handling peer scoring here

    // check if peer is already connected
    const [peerId, multiaddrTcp] = await Promise.all([enr.peerId(), enr.getFullMultiaddr('tcp')]);

    this.logger.debug(`Handling discovered peer ${peerId.toString()}, ${multiaddrTcp?.toString()}`);

    // throw if no tcp addr in multiaddr
    if (!multiaddrTcp) {
      this.logger.debug(`No TCP address in discovered node's multiaddr: ${enr.toString()}`);
      return;
    }
    const connections = this.libP2PNode.getConnections();
    if (connections.some(conn => conn.remotePeer.equals(peerId))) {
      this.logger.debug(`Already connected to peer ${peerId.toString()}`);
      return;
    }

    // check if peer is already in cache
    const id = peerId.toString();
    if (this.cachedPeers.has(id)) {
      this.logger.debug(`Already in cache ${id}`);
      return;
    }

    // create cached peer object
    const cachedPeer: CachedPeer = {
      peerId,
      enr,
      multiaddrTcp,
      dialAttempts: 0,
    };

    // Determine if we should dial immediately or not
    if (this.shouldDialPeer()) {
      this.logger.debug(`Dialing peer ${id}`);
      void this.dialPeer(cachedPeer);
    } else {
      this.logger.debug(`Caching peer ${id}`);
      this.cachedPeers.set(id, cachedPeer);
      // Prune set of cached peers
      this.pruneCachedPeers();
    }
  }

  async dialPeer(peer: CachedPeer) {
    const id = peer.peerId.toString();
    await this.libP2PNode.peerStore.merge(peer.peerId, { multiaddrs: [peer.multiaddrTcp] });

    this.logger.debug(`Dialing peer ${id}`);
    try {
      await this.libP2PNode.dial(peer.multiaddrTcp);
    } catch {
      this.logger.debug(`Failed to dial peer ${id}`);
      peer.dialAttempts++;
      if (peer.dialAttempts < MAX_DIAL_ATTEMPTS) {
        this.cachedPeers.set(id, peer);
      } else {
        this.cachedPeers.delete(id);
      }
    }
  }

  private shouldDialPeer(): boolean {
    const connections = this.libP2PNode.getConnections().length;
    this.logger.debug(`Connections: ${connections}, maxPeerCount: ${this.config.maxPeerCount}`);
    if (connections >= this.config.maxPeerCount) {
      this.logger.debug('Not dialing peer, maxPeerCount reached');
      return false;
    }
    return true;
  }

  private pruneCachedPeers() {
    let peersToDelete = this.cachedPeers.size - MAX_CACHED_PEERS;
    if (peersToDelete <= 0) {
      return;
    }

    // Remove the oldest peers
    for (const key of this.cachedPeers.keys()) {
      this.cachedPeers.delete(key);
      peersToDelete--;
      if (peersToDelete <= 0) {
        break;
      }
    }
  }
}
