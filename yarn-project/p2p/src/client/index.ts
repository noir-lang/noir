import { L2BlockSource } from '@aztec/types';

import { LibP2PService, P2PClient, P2PConfig, TxPool } from '../index.js';
import { DummyP2PService } from '../service/dummy_service.js';

export * from './p2p_client.js';

export const createP2PClient = async (config: P2PConfig, txPool: TxPool, l2BlockSource: L2BlockSource) => {
  const p2pService = config.p2pEnabled ? await LibP2PService.new(config, txPool) : new DummyP2PService();
  return new P2PClient(l2BlockSource, txPool, p2pService);
};
