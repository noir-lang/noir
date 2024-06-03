import { type L2BlockSource } from '@aztec/circuit-types';
import { type AztecKVStore } from '@aztec/kv-store';

import { P2PClient } from '../client/p2p_client.js';
import { type P2PConfig } from '../config.js';
import { DiscV5Service } from '../service/discV5_service.js';
import { DummyP2PService, DummyPeerDiscoveryService } from '../service/dummy_service.js';
import { LibP2PService, createLibP2PPeerId } from '../service/index.js';
import { getPublicIp } from '../service/ip_query.js';
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
    // If announceTcpHostname or announceUdpHostname are not provided, query for public IP if config allows
    const {
      announceTcpHostname: configAnnounceTcpHostname,
      announceUdpHostname: configAnnounceUdpHostname,
      queryForIp,
    } = config;
    if (!configAnnounceTcpHostname) {
      if (!queryForIp) {
        const publicIp = await getPublicIp();
        const announceHostname = `/ip4/${publicIp}`;
        config.announceTcpHostname = announceHostname;
      } else {
        throw new Error('No announceTcpHostname provided');
      }
    }

    if (!configAnnounceUdpHostname) {
      // If announceUdpHostname is not provided, use announceTcpHostname
      if (!queryForIp && config.announceTcpHostname) {
        config.announceUdpHostname = config.announceTcpHostname;
      } else if (queryForIp) {
        const publicIp = await getPublicIp();
        const announceHostname = `/ip4/${publicIp}`;
        config.announceUdpHostname = announceHostname;
      }
    }

    // Create peer discovery service
    const peerId = await createLibP2PPeerId(config.peerIdPrivateKey);
    discv5Service = new DiscV5Service(peerId, config);
    p2pService = await LibP2PService.new(config, discv5Service, peerId, txPool, store);
  } else {
    p2pService = new DummyP2PService();
    discv5Service = new DummyPeerDiscoveryService();
  }
  return new P2PClient(store, l2BlockSource, txPool, p2pService);
};
