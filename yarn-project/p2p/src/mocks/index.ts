import { noise } from '@chainsafe/libp2p-noise';
import { yamux } from '@chainsafe/libp2p-yamux';
import { bootstrap } from '@libp2p/bootstrap';
import { tcp } from '@libp2p/tcp';
import { type Libp2p, type Libp2pOptions, createLibp2p } from 'libp2p';

import { pingHandler, statusHandler } from '../service/reqresp/handlers.js';
import {
  PING_PROTOCOL,
  type ReqRespSubProtocolHandlers,
  STATUS_PROTOCOL,
  TX_REQ_PROTOCOL,
} from '../service/reqresp/interface.js';
import { ReqResp } from '../service/reqresp/reqresp.js';

/**
 * Creates a libp2p node, pre configured.
 * @param boostrapAddrs - an optional list of bootstrap addresses
 * @returns Lip2p node
 */
export async function createLibp2pNode(boostrapAddrs: string[] = []): Promise<Libp2p> {
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
export type ReqRespNode = {
  p2p: Libp2p;
  req: ReqResp;
};

// Mock sub protocol handlers
export const MOCK_SUB_PROTOCOL_HANDLERS: ReqRespSubProtocolHandlers = {
  [PING_PROTOCOL]: pingHandler,
  [STATUS_PROTOCOL]: statusHandler,
  [TX_REQ_PROTOCOL]: (_msg: any) => Promise.resolve(Uint8Array.from(Buffer.from('tx'))),
};

/**
 * @param numberOfNodes - the number of nodes to create
 * @returns An array of the created nodes
 */
export const createNodes = async (numberOfNodes: number): Promise<ReqRespNode[]> => {
  return await Promise.all(Array.from({ length: numberOfNodes }, () => createReqResp()));
};

// TODO: think about where else this can go
export const startNodes = async (nodes: ReqRespNode[], subProtocolHandlers = MOCK_SUB_PROTOCOL_HANDLERS) => {
  for (const node of nodes) {
    await node.req.start(subProtocolHandlers);
  }
};

export const stopNodes = async (nodes: ReqRespNode[]): Promise<void> => {
  for (const node of nodes) {
    await node.req.stop();
    await node.p2p.stop();
  }
};

// Create a req resp node, exposing the underlying p2p node
export const createReqResp = async (): Promise<ReqRespNode> => {
  const p2p = await createLibp2pNode();
  const req = new ReqResp(p2p);
  return {
    p2p,
    req,
  };
};

// Given a node list; hand shake all of the nodes with each other
export const connectToPeers = async (nodes: ReqRespNode[]): Promise<void> => {
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
