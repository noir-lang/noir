import { type L2BlockSource } from '@aztec/circuit-types';
import { type AztecKVStore } from '@aztec/kv-store';

import { P2PClient } from '../client/p2p_client.js';
import { type P2PConfig } from '../config.js';
import { DiscV5Service } from '../service/discV5_service.js';
import { DummyP2PService, DummyPeerDiscoveryService } from '../service/dummy_service.js';
import { LibP2PService, createLibP2PPeerId } from '../service/index.js';
import { type TxPool } from '../tx_pool/index.js';

export * from './p2p_client.js';

export const createP2PClient = async (
  store: AztecKVStore,
  config: P2PConfig,
  txPool: TxPool,
  l2BlockSource: L2BlockSource,
) => {
  let discv5Service;
  let p2pService;
  if (config.p2pEnabled) {
    // Create peer discovery service]
    const peerId = await createLibP2PPeerId(config.peerIdPrivateKey);
    discv5Service = new DiscV5Service(peerId, config);
    p2pService = await LibP2PService.new(config, discv5Service, peerId, txPool, store);
  } else {
    p2pService = new DummyP2PService();
    discv5Service = new DummyPeerDiscoveryService();
  }
  return new P2PClient(store, l2BlockSource, txPool, p2pService);
};
