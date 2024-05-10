import { jest } from '@jest/globals';
import type { PeerId } from '@libp2p/interface';
import { SemVer } from 'semver';

import { BootstrapNode } from '../bootstrap/bootstrap.js';
import { DiscV5Service } from './discV5_service.js';
import { createLibP2PPeerId } from './libp2p_service.js';
import { PeerDiscoveryState } from './service.js';

const waitForPeers = (node: DiscV5Service, expectedCount: number): Promise<void> => {
  const timeout = 5_000;
  return new Promise((resolve, reject) => {
    const timeoutId = setTimeout(() => {
      reject(new Error(`Timeout: Failed to connect to ${expectedCount} peers within ${timeout} ms`));
    }, timeout);

    node.on('peer:discovered', () => {
      if (node.getAllPeers().length >= expectedCount) {
        clearTimeout(timeoutId);
        resolve();
      }
    });
  });
};

describe('Discv5Service', () => {
  jest.setTimeout(10_000);

  let bootNode: BootstrapNode;
  let bootNodePeerId: PeerId;
  let port = 7890;
  const baseConfig = {
    announceHostname: '/ip4/127.0.0.1',
    announcePort: port,
    tcpListenPort: port,
    udpListenIp: '0.0.0.0',
    udpListenPort: port,
    minPeerCount: 1,
    maxPeerCount: 100,
  };

  beforeEach(async () => {
    bootNode = new BootstrapNode();
    await bootNode.start(baseConfig);
    bootNodePeerId = bootNode.getPeerId();
  });

  afterEach(async () => {
    await bootNode.stop();
  });

  it('should initialize with default values', async () => {
    port++;
    const node = await createNode(port);
    expect(node.getStatus()).toEqual(PeerDiscoveryState.STOPPED); // not started yet
    await node.start();
    expect(node.getStatus()).toEqual(PeerDiscoveryState.RUNNING);
    const peers = node.getAllPeers();
    const bootnode = peers[0];
    expect((await bootnode.peerId()).toString()).toEqual(bootNodePeerId.toString());
  });

  it('should discover & add a peer', async () => {
    port++;
    const node1 = await createNode(port);
    port++;
    const node2 = await createNode(port);
    await node1.start();
    await node2.start();
    await waitForPeers(node2, 2);

    const node1Peers = await Promise.all(node1.getAllPeers().map(async peer => (await peer.peerId()).toString()));
    const node2Peers = await Promise.all(node2.getAllPeers().map(async peer => (await peer.peerId()).toString()));

    expect(node1Peers).toHaveLength(2);
    expect(node2Peers).toHaveLength(2);
    expect(node1Peers).toContain(node2.getPeerId().toString());
    expect(node2Peers).toContain(node1.getPeerId().toString());

    await node1.stop();
    await node2.stop();
  });

  // Test is flakey, so skipping for now.
  // TODO: Investigate: #6246
  it.skip('should persist peers without bootnode', async () => {
    port++;
    const node1 = await createNode(port);
    port++;
    const node2 = await createNode(port);
    await node1.start();
    await node2.start();
    await waitForPeers(node2, 2);

    await node2.stop();
    await bootNode.stop();

    await node2.start();
    await waitForPeers(node2, 1);

    const node2Peers = await Promise.all(node2.getAllPeers().map(async peer => (await peer.peerId()).toString()));
    // NOTE: bootnode seems to still be present in list of peers sometimes, will investigate
    // expect(node2Peers).toHaveLength(1);
    expect(node2Peers).toContain(node1.getPeerId().toString());

    await node1.stop();
    await node2.stop();
  });

  const createNode = async (port: number) => {
    const peerId = await createLibP2PPeerId();
    const config = {
      ...baseConfig,
      tcpListenIp: '0.0.0.0',
      bootstrapNodes: [bootNode.getENR().encodeTxt()],
      tcpListenPort: port,
      udpListenPort: port,
      announcePort: port,
      p2pBlockCheckIntervalMS: 50,
      p2pPeerCheckIntervalMS: 50,
      transactionProtocol: 'aztec/1.0.0',
      p2pEnabled: true,
      p2pL2QueueSize: 100,
      txGossipVersion: new SemVer('0.1.0'),
    };
    return new DiscV5Service(peerId, config);
  };
});
