import { createDebugLogger } from '@aztec/foundation/log';
import { sleep } from '@aztec/foundation/sleep';

import { Discv5, type Discv5EventEmitter } from '@chainsafe/discv5';
import { ENR, SignableENR } from '@chainsafe/enr';
import type { PeerId } from '@libp2p/interface';
import { multiaddr } from '@multiformats/multiaddr';
import EventEmitter from 'events';

import type { P2PConfig } from '../config.js';
import { convertToMultiaddr } from '../util.js';
import { type PeerDiscoveryService, PeerDiscoveryState } from './service.js';

export const AZTEC_ENR_KEY = 'aztec_network';

const delayBeforeStart = 2000; // 2sec

export enum AztecENR {
  devnet = 0x01,
  testnet = 0x02,
  mainnet = 0x03,
}

// TODO: Make this an env var
export const AZTEC_NET = AztecENR.devnet;

/**
 * Peer discovery service using Discv5.
 */
export class DiscV5Service extends EventEmitter implements PeerDiscoveryService {
  /** The Discv5 instance */
  private discv5: Discv5;

  /** This instance's ENR */
  private enr: SignableENR;

  private currentState = PeerDiscoveryState.STOPPED;

  private bootstrapNodes: string[];
  private bootstrapNodePeerIds: PeerId[] = [];

  private startTime = 0;

  constructor(private peerId: PeerId, config: P2PConfig, private logger = createDebugLogger('aztec:discv5_service')) {
    super();
    const { tcpAnnounceAddress, udpAnnounceAddress, udpListenAddress, bootstrapNodes } = config;
    this.bootstrapNodes = bootstrapNodes;
    // create ENR from PeerId
    this.enr = SignableENR.createFromPeerId(peerId);
    // Add aztec identification to ENR
    this.enr.set(AZTEC_ENR_KEY, Uint8Array.from([AZTEC_NET]));

    if (!tcpAnnounceAddress) {
      throw new Error('You need to provide at least a TCP announce address.');
    }

    const multiAddrTcp = multiaddr(`${convertToMultiaddr(tcpAnnounceAddress, 'tcp')}/p2p/${peerId.toString()}`);
    // if no udp announce address is provided, use the tcp announce address
    const multiAddrUdp = multiaddr(
      `${convertToMultiaddr(udpAnnounceAddress || tcpAnnounceAddress, 'udp')}/p2p/${peerId.toString()}`,
    );

    const listenMultiAddrUdp = multiaddr(convertToMultiaddr(udpListenAddress, 'udp'));

    // set location multiaddr in ENR record
    this.enr.setLocationMultiaddr(multiAddrUdp);
    this.enr.setLocationMultiaddr(multiAddrTcp);

    this.discv5 = Discv5.create({
      enr: this.enr,
      peerId,
      bindAddrs: { ip4: listenMultiAddrUdp },
      config: {
        lookupTimeout: 2000,
        allowUnverifiedSessions: true,
      },
    });

    this.logger.info(`ENR NodeId: ${this.enr.nodeId}`);
    this.logger.info(`ENR UDP: ${multiAddrUdp.toString()}`);

    (this.discv5 as Discv5EventEmitter).on('discovered', (enr: ENR) => this.onDiscovered(enr));
    (this.discv5 as Discv5EventEmitter).on('enrAdded', async (enr: ENR) => {
      const multiAddrTcp = await enr.getFullMultiaddr('tcp');
      const multiAddrUdp = await enr.getFullMultiaddr('udp');
      this.logger.debug(`ENR multiaddr: ${multiAddrTcp?.toString()}, ${multiAddrUdp?.toString()}`);
    });
  }

  public async start(): Promise<void> {
    // Do this conversion once since it involves an async function call
    this.bootstrapNodePeerIds = await Promise.all(this.bootstrapNodes.map(enr => ENR.decodeTxt(enr).peerId()));
    if (this.currentState === PeerDiscoveryState.RUNNING) {
      throw new Error('DiscV5Service already started');
    }
    this.logger.info('Starting DiscV5');
    await this.discv5.start();
    this.startTime = Date.now();

    this.logger.info('DiscV5 started');
    this.currentState = PeerDiscoveryState.RUNNING;

    // Add bootnode ENR if provided
    if (this.bootstrapNodes?.length) {
      this.logger.info(`Adding bootstrap ENRs: ${this.bootstrapNodes.join(', ')}`);
      try {
        this.bootstrapNodes.forEach(enr => {
          this.discv5.addEnr(enr);
        });
      } catch (e) {
        this.logger.error(`Error adding bootnode ENRs: ${e}`);
      }
    }
  }

  public async runRandomNodesQuery(): Promise<void> {
    if (this.currentState !== PeerDiscoveryState.RUNNING) {
      throw new Error('DiscV5Service not running');
    }

    // First, wait some time before starting the peer discovery
    // reference: https://github.com/ChainSafe/lodestar/issues/3423
    const msSinceStart = Date.now() - this.startTime;
    if (Date.now() - this.startTime <= delayBeforeStart) {
      await sleep(delayBeforeStart - msSinceStart);
    }

    try {
      await this.discv5.findRandomNode();
    } catch (err) {
      this.logger.error(`Error running discV5 random node query: ${err}`);
    }
  }

  public getAllPeers(): ENR[] {
    return this.discv5.kadValues();
  }

  public getEnr(): ENR {
    return this.enr.toENR();
  }

  public getPeerId(): PeerId {
    return this.peerId;
  }

  public getStatus(): PeerDiscoveryState {
    return this.currentState;
  }

  public isBootstrapPeer(peerId: PeerId): boolean {
    return this.bootstrapNodePeerIds.some(node => node.equals(peerId));
  }

  public async stop(): Promise<void> {
    await this.discv5.stop();
    this.currentState = PeerDiscoveryState.STOPPED;
  }

  private onDiscovered(enr: ENR) {
    // check the peer is an aztec peer
    const value = enr.kvs.get(AZTEC_ENR_KEY);
    if (value) {
      const network = value[0];
      // check if the peer is on the same network
      if (network === AZTEC_NET) {
        this.emit('peer:discovered', enr);
      }
    }
  }
}
