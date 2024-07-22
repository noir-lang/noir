import { type L2BlockSource } from '@aztec/circuit-types';
import { type AztecKVStore } from '@aztec/kv-store';

import { P2PClient } from '../client/p2p_client.js';
import { type P2PConfig } from '../config.js';
import { DiscV5Service } from '../service/discV5_service.js';
import { DummyP2PService } from '../service/dummy_service.js';
import { LibP2PService, createLibP2PPeerId } from '../service/index.js';
import { type TxPool } from '../tx_pool/index.js';
import { getPublicIp, splitAddressPort } from '../util.js';

export * from './p2p_client.js';

export const createP2PClient = async (
  config: P2PConfig,
  store: AztecKVStore,
  txPool: TxPool,
  l2BlockSource: L2BlockSource,
) => {
  let p2pService;

  if (config.p2pEnabled) {
    // If announceTcpAddress or announceUdpAddress are not provided, query for public IP if config allows
    const {
      tcpAnnounceAddress: configTcpAnnounceAddress,
      udpAnnounceAddress: configUdpAnnounceAddress,
      queryForIp,
    } = config;

    // create variable for re-use if needed
    let publicIp;

    // check if no announce IP was provided
    const splitTcpAnnounceAddress = splitAddressPort(configTcpAnnounceAddress || '', true);
    if (splitTcpAnnounceAddress.length == 2 && splitTcpAnnounceAddress[0] === '') {
      if (queryForIp) {
        publicIp = await getPublicIp();
        const tcpAnnounceAddress = `${publicIp}:${splitTcpAnnounceAddress[1]}`;
        config.tcpAnnounceAddress = tcpAnnounceAddress;
      } else {
        throw new Error(
          `Invalid announceTcpAddress provided: ${configTcpAnnounceAddress}. Expected format: <addr>:<port>`,
        );
      }
    }

    const splitUdpAnnounceAddress = splitAddressPort(configUdpAnnounceAddress || '', true);
    if (splitUdpAnnounceAddress.length == 2 && splitUdpAnnounceAddress[0] === '') {
      // If announceUdpAddress is not provided, use announceTcpAddress
      if (!queryForIp && config.tcpAnnounceAddress) {
        config.udpAnnounceAddress = config.tcpAnnounceAddress;
      } else if (queryForIp) {
        const udpPublicIp = publicIp || (await getPublicIp());
        const udpAnnounceAddress = `${udpPublicIp}:${splitUdpAnnounceAddress[1]}`;
        config.udpAnnounceAddress = udpAnnounceAddress;
      }
    }

    // Create peer discovery service
    const peerId = await createLibP2PPeerId(config.peerIdPrivateKey);
    const discoveryService = new DiscV5Service(peerId, config);
    p2pService = await LibP2PService.new(config, discoveryService, peerId, txPool, store);
  } else {
    p2pService = new DummyP2PService();
  }
  return new P2PClient(store, l2BlockSource, txPool, p2pService);
};
