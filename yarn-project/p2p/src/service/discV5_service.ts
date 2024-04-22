import { createDebugLogger } from '@aztec/foundation/log';
import { RunningPromise } from '@aztec/foundation/running-promise';

import { Discv5, type Discv5EventEmitter } from '@chainsafe/discv5';
import { type ENR, SignableENR } from '@chainsafe/enr';
import type { PeerId } from '@libp2p/interface';
import { multiaddr } from '@multiformats/multiaddr';
import EventEmitter from 'events';

import type { P2PConfig } from '../config.js';
import type { PeerDiscoveryService } from './service.js';

export enum PeerDiscoveryState {
  RUNNING = 'running',
  STOPPED = 'stopped',
}

/**
 * Peer discovery service using Discv5.
 */
export class DiscV5Service extends EventEmitter implements PeerDiscoveryService {
  /** The Discv5 instance */
  private discv5: Discv5;

  /** This instance's ENR */
  private enr: SignableENR;

  /** The interval for checking for new peers */
  private discoveryInterval: NodeJS.Timeout | null = null;

  private runningPromise: RunningPromise;

  private currentState = PeerDiscoveryState.STOPPED;

  constructor(private peerId: PeerId, config: P2PConfig, private logger = createDebugLogger('aztec:discv5_service')) {
    super();
    const { announceHostname, tcpListenPort, udpListenIp, udpListenPort, bootstrapNodes } = config;
    // create ENR from PeerId
    this.enr = SignableENR.createFromPeerId(peerId);

    const multiAddrUdp = multiaddr(`${announceHostname}/udp/${udpListenPort}/p2p/${peerId.toString()}`);
    const multiAddrTcp = multiaddr(`${announceHostname}/tcp/${tcpListenPort}/p2p/${peerId.toString()}`);

    const listenMultiAddrUdp = multiaddr(`/ip4/${udpListenIp}/udp/${udpListenPort}`);

    // set location multiaddr in ENR record
    this.enr.setLocationMultiaddr(multiAddrUdp);
    this.enr.setLocationMultiaddr(multiAddrTcp);

    this.discv5 = Discv5.create({
      enr: this.enr,
      peerId,
      bindAddrs: { ip4: listenMultiAddrUdp },
      config: {
        lookupTimeout: 2000,
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

    // Add bootnode ENR if provided
    if (bootstrapNodes?.length) {
      this.logger.info(`Adding bootstrap ENRs: ${bootstrapNodes.join(', ')}`);
      try {
        bootstrapNodes.forEach(enr => {
          this.discv5.addEnr(enr);
        });
      } catch (e) {
        this.logger.error(`Error adding bootnode ENRs: ${e}`);
      }
    }

    this.runningPromise = new RunningPromise(async () => {
      await this.discv5.findRandomNode();
    }, config.p2pPeerCheckIntervalMS);
  }

  public async start(): Promise<void> {
    if (this.currentState === PeerDiscoveryState.RUNNING) {
      throw new Error('DiscV5Service already started');
    }
    this.logger.info('Starting DiscV5');
    await this.discv5.start();
    this.logger.info('DiscV5 started');
    this.currentState = PeerDiscoveryState.RUNNING;
    this.runningPromise.start();
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

  public async stop(): Promise<void> {
    await this.runningPromise.stop();
    await this.discv5.stop();
    this.currentState = PeerDiscoveryState.STOPPED;
  }

  private onDiscovered(enr: ENR) {
    this.emit('peer:discovered', enr);
  }
}
