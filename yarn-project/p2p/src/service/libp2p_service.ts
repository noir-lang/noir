import {
  BlockAttestation,
  BlockProposal,
  type Gossipable,
  type RawGossipMessage,
  TopicType,
  TopicTypeMap,
  Tx,
} from '@aztec/circuit-types';
import { createDebugLogger } from '@aztec/foundation/log';
import { SerialQueue } from '@aztec/foundation/queue';
import { RunningPromise } from '@aztec/foundation/running-promise';
import type { AztecKVStore } from '@aztec/kv-store';

import { type GossipsubEvents, gossipsub } from '@chainsafe/libp2p-gossipsub';
import { noise } from '@chainsafe/libp2p-noise';
import { yamux } from '@chainsafe/libp2p-yamux';
import { identify } from '@libp2p/identify';
import type { PeerId, PubSub } from '@libp2p/interface';
import '@libp2p/kad-dht';
import { mplex } from '@libp2p/mplex';
import { createFromJSON, createSecp256k1PeerId } from '@libp2p/peer-id-factory';
import { tcp } from '@libp2p/tcp';
import { type Libp2p, createLibp2p } from 'libp2p';

import { type AttestationPool } from '../attestation_pool/attestation_pool.js';
import { type P2PConfig } from '../config.js';
import { type TxPool } from '../tx_pool/index.js';
import { convertToMultiaddr } from '../util.js';
import { AztecDatastore } from './data_store.js';
import { PeerManager } from './peer_manager.js';
import type { P2PService, PeerDiscoveryService } from './service.js';

export interface PubSubLibp2p extends Libp2p {
  services: {
    pubsub: PubSub<GossipsubEvents>;
  };
}
/**
 * Create a libp2p peer ID from the private key if provided, otherwise creates a new random ID.
 * @param privateKey - Optional peer ID private key as hex string
 * @returns The peer ID.
 */
export async function createLibP2PPeerId(privateKey?: string): Promise<PeerId> {
  if (!privateKey?.length) {
    return await createSecp256k1PeerId();
  }
  const base64 = Buffer.from(privateKey, 'hex').toString('base64');
  return await createFromJSON({
    id: '',
    privKey: base64,
  });
}

/**
 * Lib P2P implementation of the P2PService interface.
 */
export class LibP2PService implements P2PService {
  private jobQueue: SerialQueue = new SerialQueue();
  private peerManager: PeerManager;
  private discoveryRunningPromise?: RunningPromise;

  private blockReceivedCallback: (block: BlockProposal) => Promise<BlockAttestation | undefined>;

  constructor(
    private config: P2PConfig,
    private node: PubSubLibp2p,
    private peerDiscoveryService: PeerDiscoveryService,
    private txPool: TxPool,
    private attestationPool: AttestationPool,
    private logger = createDebugLogger('aztec:libp2p_service'),
  ) {
    this.peerManager = new PeerManager(node, peerDiscoveryService, config, logger);

    this.blockReceivedCallback = (block: BlockProposal): Promise<BlockAttestation | undefined> => {
      this.logger.verbose(
        `[WARNING] handler not yet registered: Block received callback not set. Received block ${block.p2pMessageIdentifier()} from peer.`,
      );
      return Promise.resolve(undefined);
    };
  }

  /**
   * Starts the LibP2P service.
   * @returns An empty promise.
   */
  public async start() {
    // Check if service is already started
    if (this.node.status === 'started') {
      throw new Error('P2P service already started');
    }

    // Log listen & announce addresses
    const { tcpListenAddress, tcpAnnounceAddress } = this.config;
    this.logger.info(`Starting P2P node on ${tcpListenAddress}`);
    if (!tcpAnnounceAddress) {
      throw new Error('Announce address not provided.');
    }
    const announceTcpMultiaddr = convertToMultiaddr(tcpAnnounceAddress, 'tcp');
    this.logger.info(`Announcing at ${announceTcpMultiaddr}`);

    // Start job queue, peer discovery service and libp2p node
    this.jobQueue.start();
    await this.peerDiscoveryService.start();
    await this.node.start();
    this.logger.info(`Started P2P client with Peer ID ${this.node.peerId.toString()}`);

    // Subscribe to standard GossipSub topics by default
    for (const topic in TopicType) {
      this.subscribeToTopic(TopicTypeMap[topic].p2pTopic);
    }

    // add GossipSub listener
    this.node.services.pubsub.addEventListener('gossipsub:message', async e => {
      const { msg } = e.detail;
      this.logger.debug(`Received PUBSUB message.`);

      await this.jobQueue.put(() => this.handleNewGossipMessage(msg));
    });

    // Start running promise for peer discovery
    this.discoveryRunningPromise = new RunningPromise(() => {
      this.peerManager.discover();
    }, this.config.peerCheckIntervalMS);
    this.discoveryRunningPromise.start();
  }

  /**
   * Stops the LibP2P service.
   * @returns An empty promise.
   */
  public async stop() {
    this.logger.debug('Stopping job queue...');
    await this.jobQueue.end();
    this.logger.debug('Stopping running promise...');
    await this.discoveryRunningPromise?.stop();
    this.logger.debug('Stopping peer discovery service...');
    await this.peerDiscoveryService.stop();
    this.logger.debug('Stopping LibP2P...');
    await this.stopLibP2P();
    this.logger.info('LibP2P service stopped');
  }

  /**
   * Creates an instance of the LibP2P service.
   * @param config - The configuration to use when creating the service.
   * @param txPool - The transaction pool to be accessed by the service.
   * @returns The new service.
   */
  public static async new(
    config: P2PConfig,
    peerDiscoveryService: PeerDiscoveryService,
    peerId: PeerId,
    txPool: TxPool,
    attestationPool: AttestationPool,
    store: AztecKVStore,
  ) {
    const { tcpListenAddress, tcpAnnounceAddress, minPeerCount, maxPeerCount } = config;
    const bindAddrTcp = convertToMultiaddr(tcpListenAddress, 'tcp');
    // We know tcpAnnounceAddress cannot be null here because we set it or throw when setting up the service.
    const announceAddrTcp = convertToMultiaddr(tcpAnnounceAddress!, 'tcp');

    const datastore = new AztecDatastore(store);

    const node = await createLibp2p({
      start: false,
      peerId,
      addresses: {
        listen: [bindAddrTcp],
        announce: [announceAddrTcp],
      },
      transports: [
        tcp({
          maxConnections: config.maxPeerCount,
          // socket option: the maximum length of the queue of pending connections
          // https://nodejs.org/dist/latest-v18.x/docs/api/net.html#serverlisten
          // it's not safe if we increase this number
          backlog: 5,
          closeServerOnMaxConnections: {
            closeAbove: maxPeerCount ?? Infinity,
            listenBelow: maxPeerCount ?? Infinity,
          },
        }),
      ],
      datastore,
      streamMuxers: [yamux(), mplex()],
      connectionEncryption: [noise()],
      connectionManager: {
        minConnections: minPeerCount,
        maxConnections: maxPeerCount,
      },
      services: {
        identify: identify({
          protocolPrefix: 'aztec',
        }),
        pubsub: gossipsub({
          allowPublishToZeroTopicPeers: true,
          D: 6,
          Dlo: 4,
          Dhi: 12,
          heartbeatInterval: 1_000,
          mcacheLength: 5,
          mcacheGossip: 3,
        }),
      },
    });

    return new LibP2PService(config, node, peerDiscoveryService, txPool, attestationPool);
  }

  public registerBlockReceivedCallback(callback: (block: BlockProposal) => Promise<BlockAttestation | undefined>) {
    this.blockReceivedCallback = callback;
    this.logger.verbose('Block received callback registered');
  }

  /**
   * Subscribes to a topic.
   * @param topic - The topic to subscribe to.
   */
  private subscribeToTopic(topic: string) {
    if (!this.node.services.pubsub) {
      throw new Error('Pubsub service not available.');
    }
    void this.node.services.pubsub.subscribe(topic);
  }

  /**
   * Publishes data to a topic.
   * @param topic - The topic to publish to.
   * @param data - The data to publish.
   * @returns The number of recipients the data was sent to.
   */
  private async publishToTopic(topic: string, data: Uint8Array) {
    if (!this.node.services.pubsub) {
      throw new Error('Pubsub service not available.');
    }
    const result = await this.node.services.pubsub.publish(topic, data);

    return result.recipients.length;
  }

  /**
   * Handles a new gossip message that was received by the client.
   * @param topic - The message's topic.
   * @param data - The message data
   */
  private async handleNewGossipMessage(message: RawGossipMessage) {
    if (message.topic === Tx.p2pTopic) {
      const tx = Tx.fromBuffer(Buffer.from(message.data));
      await this.processTxFromPeer(tx);
    }
    if (message.topic === BlockAttestation.p2pTopic) {
      const attestation = BlockAttestation.fromBuffer(Buffer.from(message.data));
      await this.processAttestationFromPeer(attestation);
    }
    if (message.topic == BlockProposal.p2pTopic) {
      const block = BlockProposal.fromBuffer(Buffer.from(message.data));
      await this.processBlockFromPeer(block);
    }

    return;
  }

  /**Process Attestation From Peer
   * When a proposal is received from a peer, we add it to the attestation pool, so it can be accessed by other services.
   *
   * @param attestation - The attestation to process.
   */
  private async processAttestationFromPeer(attestation: BlockAttestation): Promise<void> {
    this.logger.verbose(`Received attestation ${attestation.p2pMessageIdentifier()} from external peer.`);
    await this.attestationPool.addAttestations([attestation]);
  }

  /**Process block from peer
   *
   * Pass the received block to the validator client
   *
   * @param block - The block to process.
   */
  // REVIEW: callback pattern https://github.com/AztecProtocol/aztec-packages/issues/7963
  private async processBlockFromPeer(block: BlockProposal): Promise<void> {
    this.logger.verbose(`Received block ${block.p2pMessageIdentifier()} from external peer.`);
    const attestation = await this.blockReceivedCallback(block);

    // TODO: fix up this pattern - the abstraction is not nice
    // The attestation can be undefined if no handler is registered / the validator deems the block invalid
    if (attestation != undefined) {
      this.propagate(attestation);
    }
  }

  /**
   * Propagates provided message to peers.
   * @param message - The message to propagate.
   */
  public propagate<T extends Gossipable>(message: T): void {
    void this.jobQueue.put(() => Promise.resolve(this.sendToPeers(message)));
  }

  private async processTxFromPeer(tx: Tx): Promise<void> {
    const txHash = tx.getTxHash();
    const txHashString = txHash.toString();
    this.logger.verbose(`Received tx ${txHashString} from external peer.`);
    await this.txPool.addTxs([tx]);
  }

  private async sendToPeers<T extends Gossipable>(message: T) {
    const parent = message.constructor as typeof Gossipable;

    const identifier = message.p2pMessageIdentifier().toString();
    this.logger.verbose(`Sending message ${identifier} to peers`);

    const recipientsNum = await this.publishToTopic(parent.p2pTopic, message.toBuffer());
    this.logger.verbose(`Sent tx ${identifier} to ${recipientsNum} peers`);
  }

  // Libp2p seems to hang sometimes if new peers are initiating connections.
  private async stopLibP2P() {
    const TIMEOUT_MS = 5000; // 5 seconds timeout
    const timeout = new Promise((resolve, reject) => {
      setTimeout(() => reject(new Error('Timeout during libp2p.stop()')), TIMEOUT_MS);
    });
    try {
      await Promise.race([this.node.stop(), timeout]);
      this.logger.debug('Libp2p stopped');
    } catch (error) {
      this.logger.error('Error during stop or timeout:', error);
    }
  }
}
