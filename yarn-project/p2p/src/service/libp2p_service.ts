import { type Tx, type TxHash } from '@aztec/circuit-types';
import { SerialQueue } from '@aztec/foundation/fifo';
import { createDebugLogger } from '@aztec/foundation/log';

import { noise } from '@chainsafe/libp2p-noise';
import { yamux } from '@chainsafe/libp2p-yamux';
import { bootstrap } from '@libp2p/bootstrap';
import type { ServiceMap } from '@libp2p/interface-libp2p';
import { type PeerId } from '@libp2p/interface-peer-id';
import { type IncomingStreamData } from '@libp2p/interface/stream-handler';
import { type DualKadDHT, kadDHT } from '@libp2p/kad-dht';
import { mplex } from '@libp2p/mplex';
import { createFromJSON, createSecp256k1PeerId, exportToProtobuf } from '@libp2p/peer-id-factory';
import { tcp } from '@libp2p/tcp';
import { pipe } from 'it-pipe';
import { type Libp2p, type Libp2pOptions, type ServiceFactoryMap, createLibp2p } from 'libp2p';
import { identifyService } from 'libp2p/identify';

import { type P2PConfig } from '../config.js';
import { type TxPool } from '../tx_pool/index.js';
import { KnownTxLookup } from './known_txs.js';
import { type P2PService } from './service.js';
import {
  Messages,
  createGetTransactionsRequestMessage,
  createTransactionHashesMessage,
  createTransactionsMessage,
  decodeGetTransactionsRequestMessage,
  decodeTransactionHashesMessage,
  decodeTransactionsMessage,
  getEncodedMessage,
} from './tx_messages.js';

/**
 * Create a libp2p peer ID from the private key if provided, otherwise creates a new random ID.
 * @param privateKey - Optional peer ID private key as hex string
 * @returns The peer ID.
 */
export async function createLibP2PPeerId(privateKey?: string) {
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
 * Exports a given peer id to a string representation.
 * @param peerId - The peerId instance to be converted.
 * @returns The peer id as a string.
 */
export function exportLibP2PPeerIdToString(peerId: PeerId) {
  return Buffer.from(exportToProtobuf(peerId)).toString('hex');
}

/**
 * Lib P2P implementation of the P2PService interface.
 */
export class LibP2PService implements P2PService {
  private jobQueue: SerialQueue = new SerialQueue();
  private knownTxLookup: KnownTxLookup = new KnownTxLookup();
  constructor(
    private config: P2PConfig,
    private node: Libp2p,
    private protocolId: string,
    private txPool: TxPool,
    private logger = createDebugLogger('aztec:libp2p_service'),
  ) {}

  /**
   * Starts the LibP2P service.
   * @returns An empty promise.
   */
  public async start() {
    if (this.node.isStarted()) {
      throw new Error('P2P service already started');
    }
    const { enableNat, tcpListenIp, tcpListenPort, announceHostname, announcePort } = this.config;
    this.logger(`Starting P2P node on ${tcpListenIp}:${tcpListenPort}`);
    if (announceHostname) {
      this.logger(`Announcing at ${announceHostname}/tcp/${announcePort ?? tcpListenPort}`);
    }
    if (enableNat) {
      this.logger(`Enabling NAT in libp2p module`);
    }

    this.node.addEventListener('peer:discovery', evt => {
      const peerId = evt.detail.id;
      if (this.isBootstrapPeer(peerId)) {
        this.logger(`Discovered bootstrap peer ${peerId.toString()}`);
      }
    });

    this.node.addEventListener('peer:connect', evt => {
      const peerId = evt.detail;
      this.handleNewConnection(peerId);
    });

    this.node.addEventListener('peer:disconnect', evt => {
      const peerId = evt.detail;
      if (this.isBootstrapPeer(peerId)) {
        this.logger(`Disconnect from bootstrap peer ${peerId.toString()}`);
      } else {
        this.logger(`Disconnected from transaction peer ${peerId.toString()}`);
      }
    });

    this.jobQueue.start();
    await this.node.start();
    await this.node.handle(this.protocolId, (incoming: IncomingStreamData) =>
      this.jobQueue.put(() => Promise.resolve(this.handleProtocolDial(incoming))),
    );
    const dht = this.node.services['kadDHT'] as DualKadDHT;
    this.logger(`Started P2P client as ${await dht.getMode()} with Peer ID ${this.node.peerId.toString()}`);
  }

  /**
   * Stops the LibP2P service.
   * @returns An empty promise.
   */
  public async stop() {
    this.logger('Stopping job queue...');
    await this.jobQueue.end();
    this.logger('Stopping LibP2P...');
    await this.node.stop();
    this.logger('LibP2P service stopped');
  }

  /**
   * Creates an instance of the LibP2P service.
   * @param config - The configuration to use when creating the service.
   * @param txPool - The transaction pool to be accessed by the service.
   * @returns The new service.
   */
  public static async new(config: P2PConfig, txPool: TxPool) {
    const {
      tcpListenIp,
      tcpListenPort,
      announceHostname,
      announcePort,
      clientKADRouting,
      minPeerCount,
      maxPeerCount,
      peerIdPrivateKey,
    } = config;
    const peerId = await createLibP2PPeerId(peerIdPrivateKey);

    const opts: Libp2pOptions<ServiceMap> = {
      start: false,
      peerId,
      addresses: {
        listen: [`/ip4/${tcpListenIp}/tcp/${tcpListenPort}`],
        announce: announceHostname ? [`${announceHostname}/tcp/${announcePort ?? tcpListenPort}`] : [],
      },
      transports: [tcp()],
      streamMuxers: [yamux(), mplex()],
      connectionEncryption: [noise()],
      connectionManager: {
        minConnections: minPeerCount,
        maxConnections: maxPeerCount,
      },
      peerDiscovery: [
        bootstrap({
          list: config.bootstrapNodes,
        }),
      ],
    };

    const services: ServiceFactoryMap = {
      identify: identifyService({
        protocolPrefix: 'aztec',
      }),
      kadDHT: kadDHT({
        protocolPrefix: 'aztec',
        clientMode: clientKADRouting,
      }),
    };

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

    const node = await createLibp2p({
      ...opts,
      services,
    });
    const protocolId = config.transactionProtocol;
    return new LibP2PService(config, node, protocolId, txPool);
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

  private async handleProtocolDial(incomingStreamData: IncomingStreamData) {
    try {
      const { message, peer } = await this.consumeInboundStream(incomingStreamData);
      if (!message.length) {
        this.logger(`Ignoring 0 byte message from peer${peer.toString()}`);
      }
      await this.processMessage(message, peer);
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

  private handleNewConnection(peerId: PeerId) {
    if (this.isBootstrapPeer(peerId)) {
      this.logger(`Connected to bootstrap peer ${peerId.toString()}`);
    } else {
      this.logger(`Connected to transaction peer ${peerId.toString()}`);
      // send the peer our current pooled transaction hashes
      void this.jobQueue.put(async () => {
        await this.sendTxHashesMessageToPeer(peerId);
      });
    }
  }

  private async processMessage(message: Buffer, peerId: PeerId) {
    const type = message.readUInt32BE(0);
    const encodedMessage = getEncodedMessage(message);
    switch (type) {
      case Messages.POOLED_TRANSACTIONS:
        await this.processReceivedTxs(encodedMessage, peerId);
        return;
      case Messages.POOLED_TRANSACTION_HASHES:
        await this.processReceivedTxHashes(encodedMessage, peerId);
        return;
      case Messages.GET_TRANSACTIONS:
        await this.processReceivedGetTransactionsRequest(encodedMessage, peerId);
        return;
    }
    throw new Error(`Unknown message type ${type}`);
  }

  private async processReceivedTxHashes(encodedMessage: Buffer, peerId: PeerId) {
    try {
      const txHashes = decodeTransactionHashesMessage(encodedMessage);
      this.logger(`Received tx hash messages from ${peerId.toString()}`);
      // we send a message requesting the transactions that we don't have from the set of received hashes
      const requiredHashes = txHashes.filter(hash => !this.txPool.hasTx(hash));
      if (!requiredHashes.length) {
        return;
      }
      await this.sendGetTransactionsMessageToPeer(txHashes, peerId);
    } catch (err) {
      this.logger.error(`Failed to process received tx hashes`, err);
    }
  }

  private async processReceivedGetTransactionsRequest(encodedMessage: Buffer, peerId: PeerId) {
    try {
      this.logger(`Received get txs messages from ${peerId.toString()}`);
      // get the transactions in the list that we have and return them
      const removeUndefined = <S>(value: S | undefined): value is S => value != undefined;
      const txHashes = decodeGetTransactionsRequestMessage(encodedMessage);
      const txs = txHashes.map(x => this.txPool.getTxByHash(x)).filter(removeUndefined);
      if (!txs.length) {
        return;
      }
      await this.sendTransactionsMessageToPeer(txs, peerId);
    } catch (err) {
      this.logger.error(`Failed to process get txs request`, err);
    }
  }

  private async processReceivedTxs(encodedMessage: Buffer, peerId: PeerId) {
    try {
      const txs = decodeTransactionsMessage(encodedMessage);
      // Could optimize here and process all txs at once
      // Propagation would need to filter and send custom tx set per peer
      for (const tx of txs) {
        await this.processTxFromPeer(tx, peerId);
      }
    } catch (err) {
      this.logger.error(`Failed to process pooled transactions message`, err);
    }
  }

  private async processTxFromPeer(tx: Tx, peerId: PeerId): Promise<void> {
    const txHash = tx.getTxHash();
    const txHashString = txHash.toString();
    this.knownTxLookup.addPeerForTx(peerId, txHashString);
    this.logger(`Received tx ${txHashString} from peer ${peerId.toString()}`);
    await this.txPool.addTxs([tx]);
    this.propagateTx(tx);
  }

  private async sendTxToPeers(tx: Tx) {
    const txs = createTransactionsMessage([tx]);
    const payload = new Uint8Array(txs);
    const peers = this.getTxPeers();
    const txHash = tx.getTxHash();
    const txHashString = txHash.toString();
    for (const peer of peers) {
      try {
        if (this.knownTxLookup.hasPeerSeenTx(peer, txHashString)) {
          this.logger(`Not sending tx ${txHashString} to peer ${peer.toString()} as they have already seen it`);
          continue;
        }
        this.logger(`Sending tx ${txHashString} to peer ${peer.toString()}`);
        await this.sendRawMessageToPeer(payload, peer);
        this.knownTxLookup.addPeerForTx(peer, txHashString);
      } catch (err) {
        this.logger.error(`Failed to send txs to peer ${peer.toString()}`, err);
        continue;
      }
    }
  }

  private async sendTxHashesMessageToPeer(peer: PeerId) {
    try {
      const hashes = this.txPool.getAllTxHashes();
      if (!hashes.length) {
        return;
      }
      const message = createTransactionHashesMessage(hashes);
      await this.sendRawMessageToPeer(new Uint8Array(message), peer);
    } catch (err) {
      this.logger.error(`Failed to send tx hashes to peer ${peer.toString()}`, err);
    }
  }

  private async sendGetTransactionsMessageToPeer(hashes: TxHash[], peer: PeerId) {
    try {
      const message = createGetTransactionsRequestMessage(hashes);
      await this.sendRawMessageToPeer(new Uint8Array(message), peer);
    } catch (err) {
      this.logger.error(`Failed to send tx request to peer ${peer.toString()}`, err);
    }
  }

  private async sendTransactionsMessageToPeer(txs: Tx[], peer: PeerId) {
    // don't filter out any transactions based on what we think the peer has seen,
    // we have been explicitly asked for these transactions
    const message = createTransactionsMessage(txs);
    await this.sendRawMessageToPeer(message, peer);
    for (const tx of txs) {
      const hash = tx.getTxHash();
      this.knownTxLookup.addPeerForTx(peer, hash.toString());
    }
  }

  private async sendRawMessageToPeer(message: Uint8Array, peer: PeerId) {
    const stream = await this.node.dialProtocol(peer, this.protocolId);
    await pipe([message], stream);
    await stream.close();
  }

  private getTxPeers() {
    return this.node.getPeers().filter(peer => !this.isBootstrapPeer(peer));
  }

  private isBootstrapPeer(peer: PeerId) {
    return this.config.bootstrapNodes.findIndex(bootstrap => bootstrap.includes(peer.toString())) != -1;
  }
}
