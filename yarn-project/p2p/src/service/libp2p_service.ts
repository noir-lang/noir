import { type Tx, type TxHash } from '@aztec/circuit-types';
import { SerialQueue } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';
import { type AztecKVStore } from '@aztec/kv-store';
import { AztecLmdbStore } from '@aztec/kv-store/lmdb';

import { ENR } from '@chainsafe/enr';
import { type GossipsubEvents, gossipsub } from '@chainsafe/libp2p-gossipsub';
import { noise } from '@chainsafe/libp2p-noise';
import { yamux } from '@chainsafe/libp2p-yamux';
import { identify } from '@libp2p/identify';
import type { IncomingStreamData, PeerId, PubSub, Stream } from '@libp2p/interface';
import '@libp2p/kad-dht';
import { mplex } from '@libp2p/mplex';
import { peerIdFromString } from '@libp2p/peer-id';
import { createFromJSON, createSecp256k1PeerId } from '@libp2p/peer-id-factory';
import { tcp } from '@libp2p/tcp';
import { pipe } from 'it-pipe';
import { type Libp2p, createLibp2p } from 'libp2p';

import { type P2PConfig } from '../config.js';
import { type TxPool } from '../tx_pool/index.js';
import { AztecDatastore } from './data_store.js';
import { KnownTxLookup } from './known_txs.js';
import { PeerManager } from './peer_manager.js';
import { AztecPeerDb, type AztecPeerStore } from './peer_store.js';
import type { P2PService, PeerDiscoveryService } from './service.js';
import { AztecTxMessageCreator, fromTxMessage } from './tx_messages.js';

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
  private knownTxLookup: KnownTxLookup = new KnownTxLookup();
  private messageCreator: AztecTxMessageCreator;
  private peerManager: PeerManager;
  constructor(
    private config: P2PConfig,
    private node: PubSubLibp2p,
    private peerDiscoveryService: PeerDiscoveryService,
    private peerStore: AztecPeerStore,
    private protocolId: string,
    private txPool: TxPool,
    private bootstrapPeerIds: PeerId[] = [],
    private logger = createDebugLogger('aztec:libp2p_service'),
  ) {
    this.messageCreator = new AztecTxMessageCreator(config.txGossipVersion);
    this.peerManager = new PeerManager(node, peerDiscoveryService, config, logger);
  }

  /**
   * Starts the LibP2P service.
   * @returns An empty promise.
   */
  public async start() {
    if (this.node.status === 'started') {
      throw new Error('P2P service already started');
    }
    const { enableNat, tcpListenIp, tcpListenPort, announceHostname, announcePort } = this.config;
    this.logger.info(`Starting P2P node on ${tcpListenIp}:${tcpListenPort}`);
    if (announceHostname) {
      this.logger.info(`Announcing at ${announceHostname}/tcp/${announcePort ?? tcpListenPort}`);
    }
    if (enableNat) {
      this.logger.info(`Enabling NAT in libp2p module`);
    }

    // handle discovered peers from external discovery service
    this.peerDiscoveryService.on('peer:discovered', async (enr: ENR) => {
      await this.addPeer(enr);
    });

    this.node.addEventListener('peer:connect', async evt => {
      const peerId = evt.detail;
      await this.handleNewConnection(peerId as PeerId);
    });

    this.node.addEventListener('peer:disconnect', async evt => {
      const peerId = evt.detail;
      if (this.isBootstrapPeer(peerId)) {
        this.logger.verbose(`Disconnect from bootstrap peer ${peerId.toString()}`);
      } else {
        this.logger.verbose(`Disconnected from transaction peer ${peerId.toString()}`);
        await this.peerManager.updateDiscoveryService();
      }
    });

    this.jobQueue.start();
    await this.peerDiscoveryService.start();
    await this.node.start();
    await this.node.handle(this.protocolId, (incoming: IncomingStreamData) =>
      this.jobQueue.put(() => Promise.resolve(this.handleProtocolDial(incoming))),
    );
    this.logger.info(`Started P2P client with Peer ID ${this.node.peerId.toString()}`);

    // Subscribe to standard topics by default
    this.subscribeToTopic(this.messageCreator.getTopic());

    // add gossipsub listener
    this.node.services.pubsub.addEventListener('gossipsub:message', async e => {
      const { msg } = e.detail;
      this.logger.debug(`Received PUBSUB message.`);

      await this.handleNewGossipMessage(msg.topic, msg.data);
    });
  }

  /**
   * Stops the LibP2P service.
   * @returns An empty promise.
   */
  public async stop() {
    this.logger.debug('Stopping job queue...');
    await this.jobQueue.end();
    this.logger.debug('Stopping LibP2P...');
    await this.node.stop();
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
    store: AztecKVStore,
  ) {
    const {
      tcpListenIp,
      tcpListenPort,
      minPeerCount,
      maxPeerCount,
      dataDirectory,
      transactionProtocol: protocolId,
    } = config;
    const bindAddrTcp = `/ip4/${tcpListenIp}/tcp/${tcpListenPort}`;

    // The autonat service seems quite problematic in that using it seems to cause a lot of attempts
    // to dial ephemeral ports. I suspect that it works better if you can get the uPNPnat service to
    // work as then you would have a permanent port to be dialled.
    // Alas, I struggled to get this to work reliably either. I find there is a race between the
    // service that reads our listener addresses and the uPnP service.
    // The result being the uPnP service can't find an address to use for the port forward.
    // Need to investigate further.
    // if (enableNat) {
    //   services.autoNAT = autoNATService({
    //     protocolPrefix: 'aztec',
    //   });
    //   services.uPnPNAT = uPnPNATService();
    // }

    const datastore = new AztecDatastore(AztecLmdbStore.open(dataDirectory));

    const node = await createLibp2p({
      start: false,
      peerId,
      addresses: {
        listen: [bindAddrTcp],
      },
      transports: [
        tcp({
          maxConnections: config.maxPeerCount,
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

    // Create an LMDB peer store
    const peerDb = new AztecPeerDb(store);

    // extract bootstrap node peer IDs
    let bootstrapPeerIds: PeerId[] = [];
    if (config.bootstrapNodes.length) {
      bootstrapPeerIds = await Promise.all(
        config.bootstrapNodes.map(bootnodeEnr => ENR.decodeTxt(bootnodeEnr).peerId()),
      );
    }

    return new LibP2PService(config, node, peerDiscoveryService, peerDb, protocolId, txPool, bootstrapPeerIds);
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
  private async handleNewGossipMessage(topic: string, data: Uint8Array) {
    if (topic !== this.messageCreator.getTopic()) {
      // Invalid TX Topic, ignore
      return;
    }

    const tx = fromTxMessage(Buffer.from(data));
    await this.processTxFromPeer(tx);
  }

  /**
   * Propagates the provided transaction to peers.
   * @param tx - The transaction to propagate.
   */
  public propagateTx(tx: Tx): void {
    void this.jobQueue.put(() => Promise.resolve(this.sendTxToPeers(tx)));
  }

  /**
   * Handles the settling of a new batch of transactions.
   * @param txHashes - The hashes of the newly settled transactions.
   */
  public settledTxs(txHashes: TxHash[]): void {
    this.knownTxLookup.handleSettledTxs(txHashes.map(x => x.toString()));
  }

  private async addPeer(enr: ENR) {
    const peerMultiAddr = await enr.getFullMultiaddr('tcp');
    if (!peerMultiAddr) {
      // No TCP address, can't connect
      return;
    }
    const peerIdStr = peerMultiAddr.getPeerId();

    if (!peerIdStr) {
      this.logger.debug(`Peer ID not found in discovered node's multiaddr: ${peerMultiAddr}`);
      return;
    }

    // check if peer is already known
    const peerId = peerIdFromString(peerIdStr);
    const hasPeer = await this.node.peerStore.has(peerId);

    // add to peer store if not already known
    if (!hasPeer) {
      this.logger.info(`Discovered peer ${peerIdStr}. Adding to libp2p peer list`);
      let stream: Stream | undefined;
      try {
        stream = await this.node.dialProtocol(peerMultiAddr, this.protocolId);

        // dial successful, add to DB as well
        if (!this.peerStore.getPeer(peerIdStr)) {
          await this.peerStore.addPeer(peerIdStr, enr);
        }
      } catch (err) {
        this.logger.error(`Failed to dial peer ${peerIdStr}`, err);
      } finally {
        if (stream) {
          await stream.close();
        }
      }
    }
  }

  private async handleProtocolDial(incomingStreamData: IncomingStreamData) {
    try {
      const { message, peer } = await this.consumeInboundStream(incomingStreamData);
      if (!message.length) {
        this.logger.verbose(`Ignoring 0 byte message from peer${peer.toString()}`);
      }
      // await this.processTransactionMessage(message, peer);
    } catch (err) {
      this.logger.error(
        `Failed to handle received message from peer ${incomingStreamData.connection.remotePeer.toString()}`,
        err,
      );
    }
  }

  private async consumeInboundStream(incomingStreamData: IncomingStreamData) {
    let buffer = Buffer.alloc(0);
    await pipe(incomingStreamData.stream, async source => {
      for await (const msg of source) {
        const payload = msg.subarray();
        buffer = Buffer.concat([buffer, Buffer.from(payload)]);
      }
    });
    await incomingStreamData.stream.close();
    return { message: buffer, peer: incomingStreamData.connection.remotePeer };
  }

  private async handleNewConnection(peerId: PeerId) {
    if (this.isBootstrapPeer(peerId)) {
      this.logger.verbose(`Connected to bootstrap peer ${peerId.toString()}`);
    } else {
      this.logger.verbose(`Connected to transaction peer ${peerId.toString()}`);
      await this.peerManager.updateDiscoveryService();
    }
  }

  private async processTxFromPeer(tx: Tx): Promise<void> {
    const txHash = tx.getTxHash();
    const txHashString = txHash.toString();
    this.logger.debug(`Received tx ${txHashString} from external peer.`);
    await this.txPool.addTxs([tx]);
  }

  private async sendTxToPeers(tx: Tx) {
    const { data: txData } = this.messageCreator.createTxMessage(tx);
    this.logger.debug(`Sending tx ${tx.getTxHash().toString()} to peers`);
    const recipientsNum = await this.publishToTopic(this.messageCreator.getTopic(), txData);
    this.logger.debug(`Sent tx ${tx.getTxHash().toString()} to ${recipientsNum} peers`);
  }

  private isBootstrapPeer(peer: PeerId) {
    return this.bootstrapPeerIds.some(bootstrapPeer => bootstrapPeer.equals(peer));
  }
}
