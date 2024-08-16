/**
 * Test fixtures and utilities to set up and run a test using multiple validators
 */
import { type AztecNodeConfig, AztecNodeService } from '@aztec/aztec-node';
import { type SentTx, createDebugLogger } from '@aztec/aztec.js';
import { type AztecAddress } from '@aztec/circuits.js';
import { type BootnodeConfig, BootstrapNode, createLibP2PPeerId } from '@aztec/p2p';
import { type PXEService } from '@aztec/pxe';
import { NoopTelemetryClient } from '@aztec/telemetry-client/noop';

import { generatePrivateKey } from 'viem/accounts';

import { getPrivateKeyFromIndex } from './utils.js';

export interface NodeContext {
  node: AztecNodeService;
  pxeService: PXEService;
  txs: SentTx[];
  account: AztecAddress;
}

export function generatePeerIdPrivateKeys(numberOfPeers: number): string[] {
  const peerIdPrivateKeys = [];
  for (let i = 0; i < numberOfPeers; i++) {
    // magic number is multiaddr prefix: https://multiformats.io/multiaddr/
    peerIdPrivateKeys.push('08021220' + generatePrivateKey().substr(2, 66));
  }
  return peerIdPrivateKeys;
}

export async function createNodes(
  config: AztecNodeConfig,
  peerIdPrivateKeys: string[],
  bootstrapNodeEnr: string,
  numNodes: number,
  bootNodePort: number,
  activateValidators: boolean = false,
): Promise<AztecNodeService[]> {
  const nodes = [];
  for (let i = 0; i < numNodes; i++) {
    const node = await createNode(
      config,
      peerIdPrivateKeys[i],
      i + 1 + bootNodePort,
      bootstrapNodeEnr,
      i,
      activateValidators,
    );
    nodes.push(node);
  }
  return nodes;
}

// creates a P2P enabled instance of Aztec Node Service
export async function createNode(
  config: AztecNodeConfig,
  peerIdPrivateKey: string,
  tcpListenPort: number,
  bootstrapNode: string | undefined,
  publisherAddressIndex: number,
  activateValidators: boolean = false,
  dataDirectory?: string,
) {
  // We use different L1 publisher accounts in order to avoid duplicate tx nonces. We start from
  // publisherAddressIndex + 1 because index 0 was already used during test environment setup.
  const publisherPrivKey = getPrivateKeyFromIndex(publisherAddressIndex + 1);
  config.publisherPrivateKey = `0x${publisherPrivKey!.toString('hex')}`;

  if (activateValidators) {
    const validatorPrivKey = getPrivateKeyFromIndex(1 + publisherAddressIndex);
    config.validatorPrivateKey = `0x${validatorPrivKey!.toString('hex')}`;
    config.disableValidator = false;
  }

  const newConfig: AztecNodeConfig = {
    ...config,
    peerIdPrivateKey: peerIdPrivateKey,
    udpListenAddress: `0.0.0.0:${tcpListenPort}`,
    tcpListenAddress: `0.0.0.0:${tcpListenPort}`,
    tcpAnnounceAddress: `127.0.0.1:${tcpListenPort}`,
    udpAnnounceAddress: `127.0.0.1:${tcpListenPort}`,
    minTxsPerBlock: config.minTxsPerBlock,
    maxTxsPerBlock: config.maxTxsPerBlock,
    p2pEnabled: true,
    blockCheckIntervalMS: 1000,
    l2QueueSize: 1,
    transactionProtocol: '',
    dataDirectory,
    bootstrapNodes: bootstrapNode ? [bootstrapNode] : [],
  };
  return await AztecNodeService.createAndSync(
    newConfig,
    new NoopTelemetryClient(),
    createDebugLogger(`aztec:node-${tcpListenPort}`),
  );
}

export async function createBootstrapNode(port: number) {
  const peerId = await createLibP2PPeerId();
  const bootstrapNode = new BootstrapNode();
  const config: BootnodeConfig = {
    udpListenAddress: `0.0.0.0:${port}`,
    udpAnnounceAddress: `127.0.0.1:${port}`,
    peerIdPrivateKey: Buffer.from(peerId.privateKey!).toString('hex'),
    minPeerCount: 10,
    maxPeerCount: 100,
  };
  await bootstrapNode.start(config);

  return bootstrapNode;
}
