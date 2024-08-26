import { sleep } from '@aztec/foundation/sleep';

import { noise } from '@chainsafe/libp2p-noise';
import { yamux } from '@chainsafe/libp2p-yamux';
import { bootstrap } from '@libp2p/bootstrap';
import { tcp } from '@libp2p/tcp';
import { type Libp2p, type Libp2pOptions, createLibp2p } from 'libp2p';

import { PING_PROTOCOL } from './interface.js';
import { ReqResp } from './reqresp.js';

/**
 * Creates a libp2p node, pre configured.
 * @param boostrapAddrs - an optional list of bootstrap addresses
 * @returns Lip2p node
 */
async function createLibp2pNode(boostrapAddrs: string[] = []): Promise<Libp2p> {
  const options: Libp2pOptions = {
    addresses: {
      listen: ['/ip4/0.0.0.0/tcp/0'],
    },
    connectionEncryption: [noise()],
    streamMuxers: [yamux()],
    transports: [tcp()],
  };

  if (boostrapAddrs.length > 0) {
    options.peerDiscovery = [
      bootstrap({
        list: boostrapAddrs,
      }),
    ];
  }

  return await createLibp2p(options);
}

/**
 * A p2p / req resp node pairing the req node will always contain the p2p node.
 * they are provided as a pair to allow access the p2p node directly
 */
type ReqRespNode = {
  p2p: Libp2p;
  req: ReqResp;
};

/**
 * @param numberOfNodes - the number of nodes to create
 * @returns An array of the created nodes
 */
const createNodes = async (numberOfNodes: number): Promise<ReqRespNode[]> => {
  return await Promise.all(Array.from({ length: numberOfNodes }, () => createReqResp()));
};

const startNodes = async (nodes: ReqRespNode[]) => {
  for (const node of nodes) {
    await node.req.start();
  }
};

const stopNodes = async (nodes: ReqRespNode[]): Promise<void> => {
  for (const node of nodes) {
    await node.req.stop();
    await node.p2p.stop();
  }
};

// Create a req resp node, exposing the underlying p2p node
const createReqResp = async (): Promise<ReqRespNode> => {
  const p2p = await createLibp2pNode();
  const req = new ReqResp(p2p);
  return {
    p2p,
    req,
  };
};

// Given a node list; hand shake all of the nodes with each other
const connectToPeers = async (nodes: ReqRespNode[]): Promise<void> => {
  for (const node of nodes) {
    for (const otherNode of nodes) {
      if (node === otherNode) {
        continue;
      }
      const addr = otherNode.p2p.getMultiaddrs()[0];
      await node.p2p.dial(addr);
    }
  }
};

// The Req Resp protocol should allow nodes to dial specific peers
// and ask for specific data that they missed via the traditional gossip protocol.
describe('ReqResp', () => {
  it('Should perform a ping request', async () => {
    // Create two nodes
    // They need to discover each other
    const nodes = await createNodes(2);
    const { req: pinger } = nodes[0];

    await startNodes(nodes);

    // connect the nodes
    await connectToPeers(nodes);

    await sleep(500);

    const res = await pinger.sendRequest(PING_PROTOCOL, Buffer.from('ping'));

    await sleep(500);
    expect(res?.toString('utf-8')).toEqual('pong');

    await stopNodes(nodes);
  });

  it('Should handle gracefully if a peer connected peer is offline', async () => {
    const nodes = await createNodes(2);

    const { req: pinger } = nodes[0];
    const { req: ponger } = nodes[1];
    await startNodes(nodes);

    // connect the nodes
    await connectToPeers(nodes);
    await sleep(500);

    void ponger.stop();

    // It should return undefined if it cannot dial the peer
    const res = await pinger.sendRequest(PING_PROTOCOL, Buffer.from('ping'));

    expect(res).toBeUndefined();

    await stopNodes(nodes);
  });

  it('Should request from a later peer if other peers are offline', async () => {
    const nodes = await createNodes(4);

    await startNodes(nodes);
    await sleep(500);
    await connectToPeers(nodes);
    await sleep(500);

    // Stop the second middle two nodes
    void nodes[1].req.stop();
    void nodes[2].req.stop();

    // send from the first node
    const res = await nodes[0].req.sendRequest(PING_PROTOCOL, Buffer.from('ping'));

    expect(res?.toString('utf-8')).toEqual('pong');

    await stopNodes(nodes);
  });
});
